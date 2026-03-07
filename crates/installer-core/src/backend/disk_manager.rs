use serde::Deserialize;
use std::process::Command;

use super::emit_progress;
use super::storage_plan::{
    AutoPartitionPlan, FilesystemType, ManualCreatePartition, ResolvedInstallLayout, SwapAction,
};

fn run_command(program: &str, args: &[&str], context: &str) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{}: {}", context, e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("{}: command exited with {}", context, output.status))
        } else {
            Err(format!("{}: {}", context, stderr))
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RmFlag {
    Bool(bool),
    Int(u8),
}

impl RmFlag {
    fn is_removable(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Int(value) => *value != 0,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BlockDevice {
    pub name: String,
    #[serde(rename = "size")]
    pub size_bytes: u64,
    pub model: Option<String>,
    pub tran: Option<String>,
    pub rm: Option<RmFlag>,
    #[serde(rename = "type")]
    pub device_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionInfo {
    pub path: String,
    pub parent_disk: String,
    pub partition_number: u8,
    pub size_bytes: u64,
    pub fstype: Option<String>,
}

#[derive(Deserialize, Debug)]
struct LsblkOutput {
    pub blockdevices: Vec<BlockDevice>,
}

pub fn get_block_devices() -> Result<Vec<BlockDevice>, String> {
    let output = Command::new("lsblk")
        .args(&["-b", "-J", "-o", "NAME,SIZE,MODEL,TRAN,TYPE,RM"])
        .output()
        .map_err(|e| format!("Failed to execute lsblk: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            return Err(format!("lsblk exited with {}", output.status));
        }
        return Err(format!("lsblk: {}", stderr));
    }

    let decoded: LsblkOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk JSON: {}", e))?;

    Ok(decoded.blockdevices.into_iter()
        .filter(|d| d.device_type == "disk")
        .collect())
}

pub fn get_partition_devices() -> Result<Vec<PartitionInfo>, String> {
    let output = Command::new("lsblk")
        .args(["-b", "-r", "-o", "NAME,TYPE,PKNAME,PARTN,SIZE,FSTYPE"])
        .output()
        .map_err(|e| format!("Failed to execute lsblk for partitions: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "lsblk partition query failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut partitions = Vec::new();
    for line in stdout.lines() {
        let mut fields = line.split_whitespace();
        let Some(name) = fields.next() else {
            continue;
        };
        let Some(device_type) = fields.next() else {
            continue;
        };
        let Some(parent) = fields.next() else {
            continue;
        };
        let Some(partition_number_raw) = fields.next() else {
            continue;
        };
        let Some(size_raw) = fields.next() else {
            continue;
        };
        let fstype = fields.next().map(|value| value.to_string());

        if device_type != "part" {
            continue;
        }

        let partition_number = partition_number_raw.parse::<u8>().unwrap_or(0);
        let size_bytes = size_raw.parse::<u64>().unwrap_or(0);
        partitions.push(PartitionInfo {
            path: format!("/dev/{}", name),
            parent_disk: format!("/dev/{}", parent),
            partition_number,
            size_bytes,
            fstype,
        });
    }

    Ok(partitions)
}

#[allow(dead_code)]
pub fn get_partitions_for_disk(disk_path: &str) -> Result<Vec<PartitionInfo>, String> {
    Ok(get_partition_devices()?
        .into_iter()
        .filter(|partition| partition.parent_disk == disk_path)
        .collect())
}

pub fn get_partitions_for_managed_disks(allowed_disks: &[String]) -> Result<Vec<PartitionInfo>, String> {
    Ok(get_partition_devices()?
        .into_iter()
        .filter(|partition| allowed_disks.iter().any(|disk| disk == &partition.parent_disk))
        .collect())
}

fn is_internal_device(device: &BlockDevice) -> bool {
    if device.device_type != "disk" {
        return false;
    }

    if device.rm.as_ref().is_some_and(|rm| rm.is_removable()) {
        return false;
    }

    !matches!(device.tran.as_deref(), Some("usb"))
}

pub fn get_internal_block_devices() -> Result<Vec<BlockDevice>, String> {
    Ok(get_block_devices()?
        .into_iter()
        .filter(is_internal_device)
        .collect())
}

pub fn bytes_to_gib(bytes: u64) -> u64 {
    bytes / (1024 * 1024 * 1024)
}

pub fn human_gib_label(bytes: u64) -> String {
    format!("{} GiB", bytes_to_gib(bytes))
}

#[allow(dead_code)]
pub fn partition_device_path(drive: &str, partition: u8) -> String {
    let suffix = partition.to_string();
    if drive.chars().last().is_some_and(|c| c.is_ascii_digit()) {
        format!("{}p{}", drive, suffix)
    } else {
        format!("{}{}", drive, suffix)
    }
}

pub fn partition_belongs_to_disk(partition_path: &str, disk_path: &str) -> bool {
    partition_path.starts_with(disk_path)
}

pub fn partition_belongs_to_managed_disks(partition_path: &str, allowed_disks: &[String]) -> bool {
    allowed_disks
        .iter()
        .any(|disk| partition_belongs_to_disk(partition_path, disk))
}

pub fn next_available_partition_number(
    disk: &str,
    existing: &[PartitionInfo],
    pending_creates: &[ManualCreatePartition],
    pending_deletes: &[String],
) -> Result<u8, String> {
    let mut used = Vec::new();
    for partition in existing.iter().filter(|partition| partition.parent_disk == disk) {
        if pending_deletes.iter().any(|path| path == &partition.path) {
            continue;
        }
        if partition.partition_number > 0 {
            used.push(partition.partition_number);
        }
    }
    for action in pending_creates.iter().filter(|action| action.disk == disk) {
        used.push(action.partition_number);
    }
    used.sort_unstable();
    used.dedup();

    for number in 1..=127 {
        if !used.contains(&number) {
            return Ok(number);
        }
    }

    Err(format!("No free GPT partition numbers remain on {}", disk))
}

pub fn create_manual_partition(action: &ManualCreatePartition) -> Result<(), String> {
    let number = action.partition_number.to_string();
    let size = if action.use_remaining {
        "0".to_string()
    } else {
        format!("+{}G", action.size_gib.unwrap_or(0))
    };
    let type_code = format!("{}:{}", action.partition_number, action.role.gpt_type());
    let label = format!("{}:{}", action.partition_number, action.role.label().to_lowercase());

    run_command(
        "sgdisk",
        &[
            "-n",
            &format!("{}:0:{}", number, size),
            "-t",
            &type_code,
            "-c",
            &label,
            &action.disk,
        ],
        &format!(
            "Failed to create {} partition on {}",
            action.role.label(),
            action.disk
        ),
    )
}

pub fn delete_partition(partition_path: &str, allowed_disks: &[String]) -> Result<(), String> {
    if !partition_belongs_to_managed_disks(partition_path, allowed_disks) {
        return Err(format!(
            "Partition {} is outside the selected install/home disks.",
            partition_path
        ));
    }

    let partition = get_partition_devices()?
        .into_iter()
        .find(|candidate| candidate.path == partition_path)
        .ok_or_else(|| format!("Partition {} was not found.", partition_path))?;

    run_command(
        "sgdisk",
        &[
            "--delete",
            &partition.partition_number.to_string(),
            &partition.parent_disk,
        ],
        &format!("Failed to delete {}", partition_path),
    )
}

#[allow(dead_code)]
pub struct PartitionPlan {
    pub drive: String,
    pub efi_gb: u64,
    pub swap_gb: u64,
    pub fs_type: String,
}

#[allow(dead_code)]
impl PartitionPlan {
    pub fn new(drive: String, swap_gb: u64, fs_type: String) -> Self {
        Self {
            drive,
            efi_gb: 1,
            swap_gb,
            fs_type,
        }
    }
}

#[allow(dead_code)]
pub fn execute_partitioning(drive: &str, swap_gb: u64, fs_type: &str) -> Result<(), String> {
    if fs_type != "xfs" && fs_type != "ext4" {
        return Err(format!(
            "Unsupported filesystem '{}'. Supported values: xfs, ext4.",
            fs_type
        ));
    }

    // 1. Wipe with sgdisk --zap-all
    run_command("sgdisk", &["--zap-all", drive], "Failed to wipe disk table")?;

    // 2. Partition 1 (1G, EF00) - EFI
    // 3. Partition 2 (swap_gb, 8200) - Swap
    // 4. Partition 3 (Remainder, 8300) - Root
    let swap_end = format!("+{}G", swap_gb);
    let partition_args = vec![
        "-n",
        "1:0:+1G",
        "-t",
        "1:ef00",
        "-c",
        "1:boot",
        "-n",
        "2:0",
        swap_end.as_str(),
        "-t",
        "2:8200",
        "-c",
        "2:swap",
        "-n",
        "3:0:0",
        "-t",
        "3:8300",
        "-c",
        "3:root",
        drive,
    ];
    run_command(
        "sgdisk",
        &partition_args,
        "Failed to create GPT partitions with sgdisk",
    )?;

    let p1 = partition_device_path(drive, 1);
    let p2 = partition_device_path(drive, 2);
    let p3 = partition_device_path(drive, 3);

    // 4. Format partitions
    // EFI
    run_command("mkfs.fat", &["-F", "32", &p1], "Failed to format EFI partition")?;
    // Swap
    run_command("mkswap", &[&p2], "Failed to initialize swap partition")?;
    run_command("swapon", &[&p2], "Failed to activate swap partition")?;
    // Root
    if fs_type == "xfs" {
        run_command("mkfs.xfs", &["-f", &p3], "Failed to format root partition as XFS")?;
    } else {
        run_command(
            "mkfs.ext4",
            &["-F", &p3],
            "Failed to format root partition as EXT4",
        )?;
    }

    Ok(())
}

fn run_optional_command(program: &str, args: &[&str], context: &str) {
    let _ = run_command(program, args, context);
}

fn apply_auto_partition(plan: &AutoPartitionPlan) -> Result<(), String> {
    run_command(
        "sgdisk",
        &["--zap-all", &plan.disk],
        &format!("Failed to wipe disk table on {}", plan.disk),
    )?;

    let mut args: Vec<String> = Vec::new();
    for partition in &plan.partitions {
        let end = partition
            .size_gib
            .map(|size| format!("+{}G", size))
            .unwrap_or_else(|| "0".to_string());
        args.push("-n".to_string());
        args.push(format!("{}:0:{}", partition.index, end));
        args.push("-t".to_string());
        args.push(format!("{}:{}", partition.index, partition.partition_type));
        args.push("-c".to_string());
        args.push(format!("{}:{}", partition.index, partition.label));
    }
    args.push(plan.disk.clone());

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command(
        "sgdisk",
        &arg_refs,
        &format!("Failed to partition {}", plan.disk),
    )
}

fn format_partition(device: &str, fs: &FilesystemType) -> Result<(), String> {
    match fs {
        FilesystemType::Fat32 => run_command(
            "mkfs.fat",
            &["-F", "32", device],
            &format!("Failed to format {} as FAT32", device),
        ),
        FilesystemType::Ext4 => run_command(
            "mkfs.ext4",
            &["-F", device],
            &format!("Failed to format {} as EXT4", device),
        ),
        FilesystemType::Xfs => run_command(
            "mkfs.xfs",
            &["-f", device],
            &format!("Failed to format {} as XFS", device),
        ),
        FilesystemType::Swap => {
            run_command(
                "mkswap",
                &[device],
                &format!("Failed to create swap signature on {}", device),
            )?;
            run_command(
                "swapon",
                &[device],
                &format!("Failed to activate swap on {}", device),
            )
        }
    }
}

fn ensure_mount_target(target: &str) -> Result<(), String> {
    if target == "/mnt" {
        return run_command("mkdir", &["-p", "/mnt"], "Failed to create /mnt");
    }
    run_command(
        "mkdir",
        &["-p", target],
        &format!("Failed to create mount target {}", target),
    )
}

pub fn execute_layout(
    layout: &ResolvedInstallLayout,
    progress: Option<&super::ProgressCallback>,
) -> Result<(), String> {
    emit_progress(progress, "Clearing previous mounts and swap state");
    run_optional_command("swapoff", &["/mnt/swapfile"], "Swapoff /mnt/swapfile");
    run_optional_command("umount", &["/mnt/home"], "Unmount /mnt/home");
    run_optional_command("umount", &["/mnt/boot"], "Unmount /mnt/boot");
    run_optional_command("umount", &["/mnt"], "Unmount /mnt");

    for plan in &layout.auto_partition {
        emit_progress(progress, &format!("Partitioning disk {}", plan.disk));
        apply_auto_partition(plan)?;
    }

    for partition in &layout.partitions_to_delete {
        emit_progress(progress, &format!("Deleting partition {}", partition));
        delete_partition(partition, &layout.managed_disks)?;
    }
    for action in &layout.manual_create_actions {
        emit_progress(
            progress,
            &format!("Creating {} partition on {}", action.role.label(), action.disk),
        );
        create_manual_partition(action)?;
    }

    for action in &layout.format_actions {
        emit_progress(
            progress,
            &format!("Formatting {} as {}", action.device, action.fs.label()),
        );
        format_partition(&action.device, &action.fs)?;
    }

    if let Some(SwapAction::Partition { device }) = &layout.swap_action {
        let already_activated = layout
            .format_actions
            .iter()
            .any(|action| action.device == *device && action.fs == FilesystemType::Swap);
        if !already_activated {
            emit_progress(progress, &format!("Activating swap on {}", device));
            run_command(
                "swapon",
                &[device],
                &format!("Failed to activate existing swap partition {}", device),
            )?;
        }
    }

    let mut mounts = layout.mount_actions.clone();
    mounts.sort_by_key(|mount| mount.target.len());
    for mount in mounts {
        emit_progress(progress, &format!("Mounting {} at {}", mount.device, mount.target));
        ensure_mount_target(&mount.target)?;
        run_command(
            "mount",
            &[&mount.device, &mount.target],
            &format!("Failed to mount {} at {}", mount.device, mount.target),
        )?;
    }

    Ok(())
}

pub fn setup_swap_file(
    layout: &ResolvedInstallLayout,
    progress: Option<&super::ProgressCallback>,
) -> Result<(), String> {
    let Some(SwapAction::File { path, size_mb }) = &layout.swap_action else {
        return Ok(());
    };

    emit_progress(progress, &format!("Preparing swap file {}", path));
    run_optional_command("swapoff", &[path], &format!("Swapoff {}", path));
    run_optional_command("rm", &["-f", path], &format!("Remove old {}", path));

    let length = format!("{}M", size_mb);
    emit_progress(progress, &format!("Allocating {} MB swap file", size_mb));
    run_command(
        "fallocate",
        &["-l", &length, path],
        &format!("Failed to allocate swap file {}", path),
    )?;
    emit_progress(progress, &format!("Setting permissions on {}", path));
    run_command(
        "chmod",
        &["600", path],
        &format!("Failed to chmod swap file {}", path),
    )?;
    emit_progress(progress, &format!("Activating swap file {}", path));
    run_command("mkswap", &[path], &format!("Failed to initialize {}", path))?;
    run_command("swapon", &[path], &format!("Failed to activate {}", path))
}

#[cfg(test)]
mod tests {
    use super::{
        BlockDevice, ManualCreatePartition, PartitionInfo, RmFlag, get_internal_block_devices,
        get_partitions_for_managed_disks, is_internal_device, next_available_partition_number,
        partition_belongs_to_managed_disks, partition_device_path,
    };
    use crate::backend::storage_plan::ManualPartitionRole;

    fn fixture_device(
        name: &str,
        device_type: &str,
        tran: Option<&str>,
        rm: Option<RmFlag>,
    ) -> BlockDevice {
        BlockDevice {
            name: name.to_string(),
            size_bytes: 100 * 1024 * 1024 * 1024,
            model: Some("Fixture".to_string()),
            tran: tran.map(|value| value.to_string()),
            rm,
            device_type: device_type.to_string(),
        }
    }

    fn fixture_partition(path: &str, parent_disk: &str, partition_number: u8) -> PartitionInfo {
        PartitionInfo {
            path: path.to_string(),
            parent_disk: parent_disk.to_string(),
            partition_number,
            size_bytes: 10 * 1024 * 1024 * 1024,
            fstype: None,
        }
    }

    #[test]
    fn internal_filter_excludes_usb_or_removable_disks() {
        let sata = fixture_device("sda", "disk", Some("sata"), Some(RmFlag::Int(0)));
        let usb = fixture_device("sdb", "disk", Some("usb"), Some(RmFlag::Int(1)));
        let partition = fixture_device("sda1", "part", Some("sata"), Some(RmFlag::Int(0)));

        assert!(is_internal_device(&sata));
        assert!(!is_internal_device(&usb));
        assert!(!is_internal_device(&partition));
    }

    #[test]
    fn internal_filter_allows_unknown_transport_non_removable_disks() {
        let nvme = fixture_device("nvme0n1", "disk", None, Some(RmFlag::Int(0)));
        let unknown_rm = fixture_device("vda", "disk", None, None);

        assert!(is_internal_device(&nvme));
        assert!(is_internal_device(&unknown_rm));
    }

    #[test]
    fn internal_device_query_function_is_available() {
        let _ = get_internal_block_devices;
    }

    #[test]
    fn partition_device_path_supports_nvme_and_sd_drives() {
        assert_eq!(partition_device_path("/dev/sda", 1), "/dev/sda1");
        assert_eq!(partition_device_path("/dev/nvme0n1", 1), "/dev/nvme0n1p1");
    }

    #[test]
    fn bytes_to_gib_uses_binary_units() {
        assert_eq!(super::bytes_to_gib(128 * 1024 * 1024 * 1024), 128);
    }

    #[test]
    fn partition_path_supports_selected_partition_refs() {
        assert_eq!(partition_device_path("/dev/nvme0n1", 4), "/dev/nvme0n1p4");
        assert_eq!(partition_device_path("/dev/sda", 2), "/dev/sda2");
    }

    #[test]
    fn managed_disk_filter_function_is_available() {
        let _ = get_partitions_for_managed_disks;
    }

    #[test]
    fn next_partition_number_reuses_deleted_slot() {
        let existing = vec![
            fixture_partition("/dev/nvme0n1p1", "/dev/nvme0n1", 1),
            fixture_partition("/dev/nvme0n1p2", "/dev/nvme0n1", 2),
            fixture_partition("/dev/nvme0n1p3", "/dev/nvme0n1", 3),
        ];
        let pending_creates = vec![ManualCreatePartition {
            disk: "/dev/nvme0n1".to_string(),
            partition_number: 4,
            role: ManualPartitionRole::Root,
            size_gib: None,
            use_remaining: true,
            path: "/dev/nvme0n1p4".to_string(),
        }];
        let pending_deletes = vec!["/dev/nvme0n1p2".to_string()];

        let next = next_available_partition_number(
            "/dev/nvme0n1",
            &existing,
            &pending_creates,
            &pending_deletes,
        )
        .unwrap();

        assert_eq!(next, 2);
    }

    #[test]
    fn managed_disk_check_handles_nvme_and_sata() {
        let allowed = vec!["/dev/nvme0n1".to_string(), "/dev/sda".to_string()];
        assert!(partition_belongs_to_managed_disks("/dev/nvme0n1p4", &allowed));
        assert!(partition_belongs_to_managed_disks("/dev/sda2", &allowed));
        assert!(!partition_belongs_to_managed_disks("/dev/sdb1", &allowed));
    }
}
