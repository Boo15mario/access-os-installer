use std::process::Command;
use std::fs;
use std::path::Path;

pub fn clone_repo_to_temp(url: &str) -> Result<String, String> {
    let temp_dir = "/tmp/installer-source";
    
    // Clean up if it already exists
    if Path::new(temp_dir).exists() {
        fs::remove_dir_all(temp_dir).map_err(|e| format!("Failed to remove existing temp dir: {}", e))?;
    }

    let output = Command::new("git")
        .args(&["clone", "--depth", "1", url, temp_dir])
        .output()
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(temp_dir.to_string())
}

pub fn list_hosts(path: &str) -> Vec<String> {
    let mut hosts = Vec::new();
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return hosts,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let config_nix = entry_path.join("configuration.nix");
            if config_nix.exists() {
                if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    // Filter out some common folders that aren't hosts
                    if name != ".git" && name != "custom-iso" {
                        hosts.push(name.to_string());
                    }
                }
            }
        }
    }
    hosts
}
