use serde::Deserialize;
use std::process::Command;

use super::storage_plan::{AutoPartitionPlan, FilesystemType, ResolvedInstallLayout, SwapAction};

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
pub struct BlockDevice {
    pub name: String,
    #[serde(rename = "size")]
    pub size_bytes: u64,
    pub model: Option<String>,
    pub tran: Option<String>,
    pub rm: Option<u8>,
    #[serde(rename = "type")]
    pub device_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionInfo {
    pub path: String,
    pub parent_disk: String,
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

    let decoded: LsblkOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk JSON: {}", e))?;

    Ok(decoded.blockdevices.into_iter()
        .filter(|d| d.device_type == "disk")
        .collect())
}

pub fn get_partition_devices() -> Result<Vec<PartitionInfo>, String> {
    let output = Command::new("lsblk")
        .args(["-b", "-r", "-o", "NAME,TYPE,PKNAME,SIZE,FSTYPE"])
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
        let Some(size_raw) = fields.next() else {
            continue;
        };
        let fstype = fields.next().map(|value| value.to_string());

        if device_type != "part" {
            continue;
        }

        let size_bytes = size_raw.parse::<u64>().unwrap_or(0);
        partitions.push(PartitionInfo {
            path: format!("/dev/{}", name),
            parent_disk: format!("/dev/{}", parent),
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

fn is_internal_device(device: &BlockDevice) -> bool {
    if device.device_type != "disk" {
        return false;
    }

    if device.rm == Some(1) {
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

pub fn execute_layout(layout: &ResolvedInstallLayout) -> Result<(), String> {
    run_optional_command("swapoff", &["/mnt/swapfile"], "Swapoff /mnt/swapfile");
    run_optional_command("umount", &["/mnt/home"], "Unmount /mnt/home");
    run_optional_command("umount", &["/mnt/boot"], "Unmount /mnt/boot");
    run_optional_command("umount", &["/mnt"], "Unmount /mnt");

    for plan in &layout.auto_partition {
        apply_auto_partition(plan)?;
    }

    for action in &layout.format_actions {
        format_partition(&action.device, &action.fs)?;
    }

    if let Some(SwapAction::Partition { device }) = &layout.swap_action {
        let already_activated = layout
            .format_actions
            .iter()
            .any(|action| action.device == *device && action.fs == FilesystemType::Swap);
        if !already_activated {
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
        ensure_mount_target(&mount.target)?;
        run_command(
            "mount",
            &[&mount.device, &mount.target],
            &format!("Failed to mount {} at {}", mount.device, mount.target),
        )?;
    }

    Ok(())
}

pub fn setup_swap_file(layout: &ResolvedInstallLayout) -> Result<(), String> {
    let Some(SwapAction::File { path, size_mb }) = &layout.swap_action else {
        return Ok(());
    };

    run_optional_command("swapoff", &[path], &format!("Swapoff {}", path));
    run_optional_command("rm", &["-f", path], &format!("Remove old {}", path));

    let length = format!("{}M", size_mb);
    run_command(
        "fallocate",
        &["-l", &length, path],
        &format!("Failed to allocate swap file {}", path),
    )?;
    run_command(
        "chmod",
        &["600", path],
        &format!("Failed to chmod swap file {}", path),
    )?;
    run_command("mkswap", &[path], &format!("Failed to initialize {}", path))?;
    run_command("swapon", &[path], &format!("Failed to activate {}", path))
}

#[cfg(test)]
mod tests {
    use super::{
        BlockDevice, get_internal_block_devices, is_internal_device, partition_device_path,
    };

    fn fixture_device(name: &str, device_type: &str, tran: Option<&str>, rm: Option<u8>) -> BlockDevice {
        BlockDevice {
            name: name.to_string(),
            size_bytes: 100 * 1024 * 1024 * 1024,
            model: Some("Fixture".to_string()),
            tran: tran.map(|value| value.to_string()),
            rm,
            device_type: device_type.to_string(),
        }
    }

    #[test]
    fn internal_filter_excludes_usb_or_removable_disks() {
        let sata = fixture_device("sda", "disk", Some("sata"), Some(0));
        let usb = fixture_device("sdb", "disk", Some("usb"), Some(1));
        let partition = fixture_device("sda1", "part", Some("sata"), Some(0));

        assert!(is_internal_device(&sata));
        assert!(!is_internal_device(&usb));
        assert!(!is_internal_device(&partition));
    }

    #[test]
    fn internal_filter_allows_unknown_transport_non_removable_disks() {
        let nvme = fixture_device("nvme0n1", "disk", None, Some(0));
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
}
