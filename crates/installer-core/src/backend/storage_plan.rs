#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetupMode {
    Automatic,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwapMode {
    Partition,
    File,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HomeMode {
    OnRoot,
    Separate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HomeLocation {
    SameDisk,
    OtherDisk,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilesystemType {
    Fat32,
    Ext4,
    Xfs,
    Swap,
}

impl FilesystemType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fat32 => "vfat",
            Self::Ext4 => "ext4",
            Self::Xfs => "xfs",
            Self::Swap => "swap",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionSpec {
    pub index: u8,
    pub size_gib: Option<u64>,
    pub partition_type: &'static str,
    pub label: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoPartitionPlan {
    pub disk: String,
    pub partitions: Vec<PartitionSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatAction {
    pub device: String,
    pub fs: FilesystemType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountAction {
    pub device: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwapAction {
    Partition { device: String },
    File { path: String, size_mb: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedInstallLayout {
    pub setup_mode: SetupMode,
    pub fs_type: FilesystemType,
    pub root_partition: String,
    pub efi_partition: String,
    pub home_partition: Option<String>,
    pub auto_partition: Vec<AutoPartitionPlan>,
    pub format_actions: Vec<FormatAction>,
    pub mount_actions: Vec<MountAction>,
    pub swap_action: Option<SwapAction>,
    pub disks_to_wipe: Vec<String>,
    pub partitions_to_create: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StorageSelection {
    pub install_disk: String,
    pub setup_mode: SetupMode,
    pub fs_type: String,
    pub swap_mode: SwapMode,
    pub swap_size_gib: u64,
    pub swap_file_size_mb: Option<u64>,
    pub home_mode: HomeMode,
    pub home_location: HomeLocation,
    pub home_disk: Option<String>,
    pub manual_efi_partition: Option<String>,
    pub manual_root_partition: Option<String>,
    pub manual_home_partition: Option<String>,
    pub manual_swap_partition: Option<String>,
    pub format_efi: bool,
    pub format_root: bool,
    pub format_home: bool,
    pub format_swap: bool,
    pub removable_media: bool,
}

fn parse_root_fs(fs_type: &str) -> Result<FilesystemType, String> {
    match fs_type {
        "xfs" => Ok(FilesystemType::Xfs),
        "ext4" => Ok(FilesystemType::Ext4),
        other => Err(format!(
            "Unsupported filesystem '{}'. Supported values: xfs, ext4.",
            other
        )),
    }
}

fn partition_device_path(drive: &str, partition: u8) -> String {
    let suffix = partition.to_string();
    if drive.chars().last().is_some_and(|c| c.is_ascii_digit()) {
        format!("{}p{}", drive, suffix)
    } else {
        format!("{}{}", drive, suffix)
    }
}

fn validate_common(selection: &StorageSelection) -> Result<(), String> {
    if selection.install_disk.trim().is_empty() {
        return Err("No install disk selected.".to_string());
    }

    if selection.swap_mode == SwapMode::File {
        let size = selection
            .swap_file_size_mb
            .ok_or("Swap file mode requires a size in MB.")?;
        if size < 512 {
            return Err("Swap file size must be at least 512 MB.".to_string());
        }
    }

    if selection.setup_mode == SetupMode::Automatic
        && selection.home_mode == HomeMode::Separate
        && selection.home_location == HomeLocation::OtherDisk
    {
        let home_disk = selection
            .home_disk
            .as_ref()
            .ok_or("Select a second disk for /home.")?;
        if home_disk == &selection.install_disk {
            return Err("Home disk must be different from install disk.".to_string());
        }
    }

    Ok(())
}

pub fn resolve_layout(selection: &StorageSelection) -> Result<ResolvedInstallLayout, String> {
    validate_common(selection)?;
    let root_fs = parse_root_fs(&selection.fs_type)?;

    match selection.setup_mode {
        SetupMode::Automatic => resolve_automatic(selection, root_fs),
        SetupMode::Manual => resolve_manual(selection, root_fs),
    }
}

fn resolve_automatic(
    selection: &StorageSelection,
    root_fs: FilesystemType,
) -> Result<ResolvedInstallLayout, String> {
    if selection.home_mode == HomeMode::Separate
        && selection.home_location == HomeLocation::SameDisk
    {
        return Err(
            "Automatic setup does not support a separate /home on the same disk. Use Manual mode or choose another disk.".to_string(),
        );
    }

    let install_disk = selection.install_disk.clone();
    let mut install_parts = vec![PartitionSpec {
        index: 1,
        size_gib: Some(1),
        partition_type: "ef00",
        label: "boot",
    }];

    let mut swap_action = None;
    let (root_part_index, root_start_index) = if selection.swap_mode == SwapMode::Partition {
        install_parts.push(PartitionSpec {
            index: 2,
            size_gib: Some(selection.swap_size_gib),
            partition_type: "8200",
            label: "swap",
        });
        swap_action = Some(SwapAction::Partition {
            device: partition_device_path(&install_disk, 2),
        });
        (3_u8, 3_u8)
    } else {
        (2_u8, 2_u8)
    };

    install_parts.push(PartitionSpec {
        index: root_start_index,
        size_gib: None,
        partition_type: "8300",
        label: "root",
    });

    let efi_partition = partition_device_path(&install_disk, 1);
    let root_partition = partition_device_path(&install_disk, root_part_index);
    let mut home_partition = None;
    let mut auto_partition = vec![AutoPartitionPlan {
        disk: install_disk.clone(),
        partitions: install_parts.clone(),
    }];
    let mut disks_to_wipe = vec![install_disk.clone()];
    let mut partitions_to_create = vec![
        format!("{}: EFI 1 GiB", install_disk),
        format!("{}: root ({})", install_disk, root_fs.label()),
    ];

    if selection.home_mode == HomeMode::Separate
        && selection.home_location == HomeLocation::OtherDisk
    {
        let home_disk = selection
            .home_disk
            .as_ref()
            .ok_or("Select a second disk for /home.")?
            .clone();
        auto_partition.push(AutoPartitionPlan {
            disk: home_disk.clone(),
            partitions: vec![PartitionSpec {
                index: 1,
                size_gib: None,
                partition_type: "8300",
                label: "home",
            }],
        });
        disks_to_wipe.push(home_disk.clone());
        home_partition = Some(partition_device_path(&home_disk, 1));
        partitions_to_create.push(format!("{}: home (ext4)", home_disk));
    }

    if selection.swap_mode == SwapMode::Partition {
        partitions_to_create.push(format!("{}: swap {} GiB", install_disk, selection.swap_size_gib));
    } else {
        swap_action = Some(SwapAction::File {
            path: "/mnt/swapfile".to_string(),
            size_mb: selection.swap_file_size_mb.unwrap_or(selection.swap_size_gib * 1024),
        });
    }

    let mut format_actions = vec![
        FormatAction {
            device: efi_partition.clone(),
            fs: FilesystemType::Fat32,
        },
        FormatAction {
            device: root_partition.clone(),
            fs: root_fs.clone(),
        },
    ];
    if let Some(home) = &home_partition {
        format_actions.push(FormatAction {
            device: home.clone(),
            fs: FilesystemType::Ext4,
        });
    }
    if selection.swap_mode == SwapMode::Partition {
        format_actions.push(FormatAction {
            device: partition_device_path(&install_disk, 2),
            fs: FilesystemType::Swap,
        });
    }

    let mut mount_actions = vec![
        MountAction {
            device: root_partition.clone(),
            target: "/mnt".to_string(),
        },
        MountAction {
            device: efi_partition.clone(),
            target: "/mnt/boot".to_string(),
        },
    ];
    if let Some(home) = &home_partition {
        mount_actions.push(MountAction {
            device: home.clone(),
            target: "/mnt/home".to_string(),
        });
    }

    Ok(ResolvedInstallLayout {
        setup_mode: SetupMode::Automatic,
        fs_type: root_fs,
        root_partition,
        efi_partition,
        home_partition,
        auto_partition,
        format_actions,
        mount_actions,
        swap_action,
        disks_to_wipe,
        partitions_to_create,
    })
}

fn resolve_manual(
    selection: &StorageSelection,
    root_fs: FilesystemType,
) -> Result<ResolvedInstallLayout, String> {
    let efi_partition = selection
        .manual_efi_partition
        .as_ref()
        .ok_or("Select an EFI partition for manual setup.")?
        .clone();
    let root_partition = selection
        .manual_root_partition
        .as_ref()
        .ok_or("Select a root partition for manual setup.")?
        .clone();

    if efi_partition == root_partition {
        return Err("EFI and root partitions must be different.".to_string());
    }

    let home_partition = if selection.home_mode == HomeMode::Separate {
        let home = selection
            .manual_home_partition
            .as_ref()
            .ok_or("Select a /home partition for manual setup.")?
            .clone();
        if home == efi_partition || home == root_partition {
            return Err("/home partition must differ from EFI and root.".to_string());
        }
        Some(home)
    } else {
        None
    };

    let mut format_actions = Vec::new();
    if selection.format_efi {
        format_actions.push(FormatAction {
            device: efi_partition.clone(),
            fs: FilesystemType::Fat32,
        });
    }
    if selection.format_root {
        format_actions.push(FormatAction {
            device: root_partition.clone(),
            fs: root_fs.clone(),
        });
    }
    if selection.format_home {
        if let Some(home) = &home_partition {
            format_actions.push(FormatAction {
                device: home.clone(),
                fs: FilesystemType::Ext4,
            });
        }
    }

    let swap_action = match selection.swap_mode {
        SwapMode::Partition => {
            let swap_partition = selection
                .manual_swap_partition
                .as_ref()
                .ok_or("Select a swap partition for manual setup.")?
                .clone();
            if swap_partition == efi_partition || swap_partition == root_partition {
                return Err("Swap partition must differ from EFI and root.".to_string());
            }
            if let Some(home) = &home_partition {
                if swap_partition == *home {
                    return Err("Swap partition must differ from /home.".to_string());
                }
            }
            if selection.format_swap {
                format_actions.push(FormatAction {
                    device: swap_partition.clone(),
                    fs: FilesystemType::Swap,
                });
            }
            Some(SwapAction::Partition {
                device: swap_partition,
            })
        }
        SwapMode::File => Some(SwapAction::File {
            path: "/mnt/swapfile".to_string(),
            size_mb: selection.swap_file_size_mb.unwrap_or(selection.swap_size_gib * 1024),
        }),
    };

    let mut mount_actions = vec![
        MountAction {
            device: root_partition.clone(),
            target: "/mnt".to_string(),
        },
        MountAction {
            device: efi_partition.clone(),
            target: "/mnt/boot".to_string(),
        },
    ];
    if let Some(home) = &home_partition {
        mount_actions.push(MountAction {
            device: home.clone(),
            target: "/mnt/home".to_string(),
        });
    }

    Ok(ResolvedInstallLayout {
        setup_mode: SetupMode::Manual,
        fs_type: root_fs,
        root_partition,
        efi_partition,
        home_partition,
        auto_partition: Vec::new(),
        format_actions,
        mount_actions,
        swap_action,
        disks_to_wipe: Vec::new(),
        partitions_to_create: Vec::new(),
    })
}

pub fn format_destructive_plan(layout: &ResolvedInstallLayout) -> String {
    let mut lines = Vec::new();

    lines.push("Disks to wipe:".to_string());
    if layout.disks_to_wipe.is_empty() {
        lines.push("- none".to_string());
    } else {
        for disk in &layout.disks_to_wipe {
            lines.push(format!("- {}", disk));
        }
    }

    lines.push("Partitions to create:".to_string());
    if layout.partitions_to_create.is_empty() {
        lines.push("- none".to_string());
    } else {
        for item in &layout.partitions_to_create {
            lines.push(format!("- {}", item));
        }
    }

    lines.push("Partitions to format:".to_string());
    if layout.format_actions.is_empty() {
        lines.push("- none".to_string());
    } else {
        for action in &layout.format_actions {
            lines.push(format!("- {} as {}", action.device, action.fs.label()));
        }
    }

    lines.push("Mount plan:".to_string());
    for mount in &layout.mount_actions {
        lines.push(format!("- {} -> {}", mount.device, mount.target));
    }

    if let Some(swap) = &layout.swap_action {
        match swap {
            SwapAction::Partition { device } => {
                lines.push(format!("Swap: partition {}", device));
            }
            SwapAction::File { path, size_mb } => {
                lines.push(format!("Swap: file {} ({} MB)", path, size_mb));
            }
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{
        FilesystemType, HomeLocation, HomeMode, SetupMode, StorageSelection, SwapMode,
        format_destructive_plan, resolve_layout,
    };

    fn base_selection() -> StorageSelection {
        StorageSelection {
            install_disk: "/dev/nvme0n1".to_string(),
            setup_mode: SetupMode::Automatic,
            fs_type: "xfs".to_string(),
            swap_mode: SwapMode::Partition,
            swap_size_gib: 8,
            swap_file_size_mb: Some(8192),
            home_mode: HomeMode::OnRoot,
            home_location: HomeLocation::SameDisk,
            home_disk: None,
            manual_efi_partition: None,
            manual_root_partition: None,
            manual_home_partition: None,
            manual_swap_partition: None,
            format_efi: true,
            format_root: true,
            format_home: true,
            format_swap: true,
            removable_media: false,
        }
    }

    #[test]
    fn auto_home_other_disk_requires_distinct_disk() {
        let mut selection = base_selection();
        selection.home_mode = HomeMode::Separate;
        selection.home_location = HomeLocation::OtherDisk;
        selection.home_disk = Some("/dev/nvme0n1".to_string());

        let err = resolve_layout(&selection).unwrap_err();
        assert!(err.contains("different from install disk"));
    }

    #[test]
    fn auto_with_other_disk_home_emits_two_disk_actions() {
        let mut selection = base_selection();
        selection.swap_mode = SwapMode::File;
        selection.home_mode = HomeMode::Separate;
        selection.home_location = HomeLocation::OtherDisk;
        selection.home_disk = Some("/dev/sdb".to_string());

        let layout = resolve_layout(&selection).unwrap();
        assert_eq!(layout.auto_partition.len(), 2);
        assert!(layout
            .mount_actions
            .iter()
            .any(|mount| mount.target == "/mnt/home"));
    }

    #[test]
    fn manual_requires_efi_and_root() {
        let mut selection = base_selection();
        selection.setup_mode = SetupMode::Manual;
        selection.swap_mode = SwapMode::File;

        let err = resolve_layout(&selection).unwrap_err();
        assert!(err.contains("EFI partition"));
    }

    #[test]
    fn manual_allows_swap_partition_mode() {
        let mut selection = base_selection();
        selection.setup_mode = SetupMode::Manual;
        selection.swap_mode = SwapMode::Partition;
        selection.manual_efi_partition = Some("/dev/nvme0n1p1".to_string());
        selection.manual_root_partition = Some("/dev/nvme0n1p2".to_string());
        selection.manual_swap_partition = Some("/dev/nvme0n1p3".to_string());

        let layout = resolve_layout(&selection).unwrap();
        assert!(layout
            .format_actions
            .iter()
            .any(|action| action.device == "/dev/nvme0n1p3" && action.fs == FilesystemType::Swap));
    }

    #[test]
    fn manual_swap_partition_must_not_overlap_root() {
        let mut selection = base_selection();
        selection.setup_mode = SetupMode::Manual;
        selection.swap_mode = SwapMode::Partition;
        selection.manual_efi_partition = Some("/dev/nvme0n1p1".to_string());
        selection.manual_root_partition = Some("/dev/nvme0n1p2".to_string());
        selection.manual_swap_partition = Some("/dev/nvme0n1p2".to_string());

        let err = resolve_layout(&selection).unwrap_err();
        assert!(err.contains("Swap partition must differ"));
    }

    #[test]
    fn destructive_summary_lists_wipes_and_formats() {
        let layout = resolve_layout(&base_selection()).unwrap();
        let summary = format_destructive_plan(&layout);
        assert!(summary.contains("Disks to wipe"));
        assert!(summary.contains("Partitions to format"));
        assert!(summary.contains("/mnt"));
    }
}
