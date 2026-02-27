use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BlockDevice {
    pub name: String,
    pub size: String,
    pub model: Option<String>,
    pub tran: Option<String>,
    #[serde(rename = "type")]
    pub device_type: String,
}

#[derive(Deserialize, Debug)]
struct LsblkOutput {
    pub blockdevices: Vec<BlockDevice>,
}

pub fn get_block_devices() -> Result<Vec<BlockDevice>, String> {
    let output = Command::new("lsblk")
        .args(&["-J", "-o", "NAME,SIZE,MODEL,TRAN,TYPE"])
        .output()
        .map_err(|e| format!("Failed to execute lsblk: {}", e))?;

    let decoded: LsblkOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk JSON: {}", e))?;

    Ok(decoded.blockdevices.into_iter()
        .filter(|d| d.device_type == "disk")
        .collect())
}

pub struct PartitionPlan {
    pub drive: String,
    pub efi_gb: u64,
    pub swap_gb: u64,
    pub fs_type: String,
}

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

pub fn execute_partitioning(drive: &str, swap_gb: u64, fs_type: &str) -> Result<(), String> {
    // 1. Wipe with sgdisk --zap-all
    let output = Command::new("sgdisk")
        .args(&["--zap-all", drive])
        .output()
        .map_err(|e| format!("Failed to execute sgdisk --zap-all: {}", e))?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // 2. Partition 1 (1G, EF00) - EFI
    // 3. Partition 2 (swap_gb, 8200) - Swap
    // 4. Partition 3 (Remainder, 8300) - Root
    let swap_end = format!("+{}G", swap_gb);
    let output = Command::new("sgdisk")
        .args(&[
            "-n", "1:0:+1G", "-t", "1:ef00", "-c", "1:boot",
            "-n", "2:0", &swap_end, "-t", "2:8200", "-c", "2:swap",
            "-n", "3:0:0", "-t", "3:8300", "-c", "3:root",
            drive
        ])
        .output()
        .map_err(|e| format!("Failed to execute sgdisk to create partitions: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Determine partition names (assuming standard /dev/sdX -> /dev/sdX1)
    let p1 = format!("{}1", drive);
    let p2 = format!("{}2", drive);
    let p3 = format!("{}3", drive);

    // 4. Format partitions
    // EFI
    let _ = Command::new("mkfs.fat").args(&["-F", "32", &p1]).output();
    // Swap
    let _ = Command::new("mkswap").args(&[&p2]).output();
    // Root
    if fs_type == "xfs" {
        let _ = Command::new("mkfs.xfs").args(&["-f", &p3]).output();
    } else {
        let _ = Command::new("mkfs.ext4").args(&["-F", &p3]).output();
    }

    Ok(())
}
