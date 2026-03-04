use crate::backend::storage_plan::ResolvedInstallLayout;
use std::process::Command;

pub fn prepare_install_targets(layout: &ResolvedInstallLayout) -> Result<(), String> {
    crate::backend::disk_manager::execute_layout(layout)
}

pub fn unmount_install_targets() -> Result<(), String> {
    let mut errors = Vec::new();
    let _ = Command::new("swapoff").arg("/mnt/swapfile").output();

    for mount_point in ["/mnt/home", "/mnt/boot", "/mnt"] {
        let output = Command::new("umount")
            .arg(mount_point)
            .output()
            .map_err(|e| format!("Failed to execute umount for {}: {}", mount_point, e))?;

        if output.status.success() {
            continue;
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("not mounted") || stderr.contains("no mount point specified") {
            continue;
        }

        let detail = if stderr.is_empty() {
            format!("umount returned status {}", output.status)
        } else {
            stderr
        };
        errors.push(format!("{}: {}", mount_point, detail));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
