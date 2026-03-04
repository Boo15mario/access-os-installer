use std::process::Command;

pub fn reboot_system() -> Result<(), String> {
    let output = Command::new("reboot")
        .output()
        .map_err(|e| format!("Failed to execute reboot: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("Reboot command failed with status {}", output.status))
        } else {
            Err(stderr)
        }
    }
}

pub fn shutdown_system() -> Result<(), String> {
    let output = Command::new("shutdown")
        .arg("now")
        .output()
        .map_err(|e| format!("Failed to execute shutdown: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!(
                "Shutdown command failed with status {}",
                output.status
            ))
        } else {
            Err(stderr)
        }
    }
}
