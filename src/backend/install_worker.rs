use std::process::Command;
use std::io::Write;

pub fn start_install(host: &str, user: &str, pass: &str) -> Result<(), String> {
    // 1. nixos-install --flake /mnt/etc/nixos/#host
    let flake_arg = format!("/mnt/etc/nixos/#{}", host);
    let output = Command::new("nixos-install")
        .args(&["--flake", &flake_arg, "--no-root-passwd"])
        .output()
        .map_err(|e| format!("Failed to execute nixos-install: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // 2. chroot /mnt chpasswd <<< "user:pass"
    let mut child = Command::new("chroot")
        .args(&["/mnt", "chpasswd"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn chroot chpasswd: {}", e))?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin for chpasswd")?;
    let input = format!("{}:{}", user, pass);
    stdin.write_all(input.as_bytes()).map_err(|e| format!("Failed to write to chpasswd stdin: {}", e))?;
    drop(stdin);

    let status = child.wait().map_err(|e| format!("Failed to wait for chpasswd: {}", e))?;
    if !status.success() {
        return Err("chpasswd failed to set user password".to_string());
    }

    Ok(())
}
