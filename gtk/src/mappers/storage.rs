use crate::app::state::AppState;
use crate::backend::network;
use crate::backend::preflight::{CheckResult, CheckStatus, PreflightContext};
use crate::backend::storage_plan::{HomeLocation, HomeMode, SetupMode, StorageSelection, SwapMode};
use std::path::Path;
use sysinfo::System;

const DOTFILES_REPO_URL: &str = "https://github.com/Boo15mario/access-os-config";

fn system_ram_gib() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.total_memory() / (1024 * 1024 * 1024)
}

pub fn preflight_context_from_state(state: &AppState) -> PreflightContext {
    PreflightContext {
        is_uefi: Path::new("/sys/firmware/efi").exists(),
        has_disk: !state.drive.is_empty(),
        online: network::check_connectivity(),
        ram_gib: system_ram_gib(),
        disk_gib: state.selected_disk_gib,
    }
}

pub fn format_check_group(results: &[CheckResult], hard_only: bool) -> String {
    let mut lines = Vec::new();

    for check in results.iter().filter(|result| result.is_hard == hard_only) {
        lines.push(format!(
            "[{}] {}: {}",
            check.status.label(),
            check.label,
            check.message
        ));
    }

    if lines.is_empty() {
        "No checks in this group.".to_string()
    } else {
        lines.join("\n")
    }
}

pub fn format_review_summary(state: &AppState) -> String {
    let target_disk = if state.drive.is_empty() {
        "(not selected)".to_string()
    } else if let Some(gib) = state.selected_disk_gib {
        format!("{} ({} GiB)", state.drive, gib)
    } else {
        state.drive.clone()
    };

    let de_label = match &state.desktop_env {
        Some(de) => de.label(),
        None => "(not selected)",
    };
    let hostname = if state.hostname.is_empty() {
        "(not set)"
    } else {
        &state.hostname
    };
    let username = if state.username.is_empty() {
        "(not set)"
    } else {
        &state.username
    };
    let nvidia_label = if state.nvidia { "Yes" } else { "No" };
    let removable_label = if state.removable_media { "Yes" } else { "No" };
    let setup_mode_label = match state.setup_mode {
        SetupMode::Automatic => "Automatic",
        SetupMode::Manual => "Manual",
    };
    let swap_label = match state.swap_mode {
        SwapMode::Partition => format!("Swap partition ({} GiB)", state.swap_gb),
        SwapMode::File => format!("Swap file ({} MB)", state.swap_file_mb),
    };
    let home_label = match state.home_mode {
        HomeMode::OnRoot => "Home on root filesystem".to_string(),
        HomeMode::Separate => match state.home_location {
            HomeLocation::SameDisk => "Separate /home on same disk".to_string(),
            HomeLocation::OtherDisk => {
                if state.home_disk.is_empty() {
                    "Separate /home on another disk (not selected)".to_string()
                } else {
                    format!("Separate /home on another disk ({})", state.home_disk)
                }
            }
        },
    };
    format!(
        "Target disk: {}\nDisk setup: {}\nHome mode: {}\nSwap: {}\nRoot filesystem: {}\nRemovable media: {}\nMirror region: {}\nKernel: {}\nDesktop environment: {}\nNvidia drivers: {}\nHostname: {}\nUsername: {}\nTimezone: {}\nLocale: {}\nKeymap: {}\nDotfiles: {}",
        target_disk,
        setup_mode_label,
        home_label,
        swap_label,
        state.fs_type,
        removable_label,
        state.mirror_region,
        state.kernel.label(),
        de_label,
        nvidia_label,
        hostname,
        username,
        state.timezone,
        state.locale,
        state.keymap,
        DOTFILES_REPO_URL
    )
}

pub fn storage_selection_from_state(state: &AppState) -> StorageSelection {
    StorageSelection {
        install_disk: state.drive.clone(),
        setup_mode: state.setup_mode.clone(),
        fs_type: state.fs_type.clone(),
        swap_mode: state.swap_mode.clone(),
        swap_size_gib: state.swap_gb,
        swap_file_size_mb: Some(state.swap_file_mb),
        home_mode: state.home_mode.clone(),
        home_location: state.home_location.clone(),
        home_disk: if state.home_disk.is_empty() {
            None
        } else {
            Some(state.home_disk.clone())
        },
        manual_efi_partition: if state.manual_efi_partition.is_empty() {
            None
        } else {
            Some(state.manual_efi_partition.clone())
        },
        manual_root_partition: if state.manual_root_partition.is_empty() {
            None
        } else {
            Some(state.manual_root_partition.clone())
        },
        manual_home_partition: if state.manual_home_partition.is_empty() {
            None
        } else {
            Some(state.manual_home_partition.clone())
        },
        manual_swap_partition: if state.manual_swap_partition.is_empty() {
            None
        } else {
            Some(state.manual_swap_partition.clone())
        },
        format_efi: state.format_efi,
        format_root: state.format_root,
        format_home: state.format_home,
        format_swap: state.format_swap,
        removable_media: state.removable_media,
    }
}

pub fn format_warning_lines(results: &[CheckResult]) -> String {
    let warnings: Vec<String> = results
        .iter()
        .filter(|result| result.status == CheckStatus::Warn)
        .map(|result| format!("- {}", result.message))
        .collect();

    if warnings.is_empty() {
        "No warnings detected.".to_string()
    } else {
        warnings.join("\n")
    }
}
