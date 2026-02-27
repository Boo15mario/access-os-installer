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

pub struct HostSettings {
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub keymap: Option<String>,
}

pub fn check_settings(host_path: &str) -> HostSettings {
    let mut settings = HostSettings {
        timezone: None,
        locale: None,
        keymap: None,
    };

    let host_config_path = Path::new(host_path).join("configuration.nix");
    let content = match fs::read_to_string(host_config_path) {
        Ok(c) => c,
        Err(_) => return settings,
    };

    for line in content.lines() {
        let line = line.trim();
        if line.contains("time.timeZone") {
            settings.timezone = extract_value(line);
        } else if line.contains("i18n.defaultLocale") {
            settings.locale = extract_value(line);
        } else if line.contains("console.keyMap") {
            settings.keymap = extract_value(line);
        }
    }

    settings
}

fn extract_value(line: &str) -> Option<String> {
    // Basic logic to extract a string value between quotes
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() < 2 {
        return None;
    }
    let val_part = parts[1].trim();
    let val_part = val_part.trim_end_matches(';');
    let start = val_part.find('"')?;
    let end = val_part.rfind('"')?;
    if start < end {
        Some(val_part[start + 1..end].to_string())
    } else {
        None
    }
}
