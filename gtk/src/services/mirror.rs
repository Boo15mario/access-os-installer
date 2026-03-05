use std::process::Command;

pub fn apply_mirror_region(region: &str) -> Result<(), String> {
    if region == "Worldwide" {
        return Ok(());
    }

    let output = Command::new("reflector")
        .args(["-c", region, "--save", "/etc/pacman.d/mirrorlist"])
        .output()
        .map_err(|e| format!("Failed to execute reflector: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!(
                "reflector failed for region '{}' with status {}",
                region, output.status
            ))
        } else {
            Err(stderr)
        }
    }
}
