use std::collections::HashSet;
use std::process::Command;

/// Returns true if the machine can reach the internet.
pub fn check_connectivity() -> bool {
    Command::new("ping")
        .args(["-c", "1", "-W", "2", "1.1.1.1"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns a deduplicated list of visible SSIDs.
pub fn scan_wifi() -> Vec<String> {
    // Trigger a rescan (ignore errors – device may not be wifi)
    let _ = Command::new("nmcli").args(["dev", "wifi", "rescan"]).output();

    let output = match Command::new("nmcli")
        .args(["-t", "-f", "SSID", "dev", "wifi", "list"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut seen = HashSet::new();
    stdout
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.to_string()))
        .map(str::to_string)
        .collect()
}

/// Attempts to connect to a Wi-Fi network via nmcli.
pub fn connect_wifi(ssid: &str, password: &str) -> Result<(), String> {
    let output = Command::new("nmcli")
        .args(["dev", "wifi", "connect", ssid, "password", password])
        .output()
        .map_err(|e| format!("Failed to run nmcli: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
