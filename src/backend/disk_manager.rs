use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BlockDevice {
    pub name: String,
    pub size: String,
    pub model: Option<String>,
    pub tran: Option<String>,
    pub rm: Option<u8>,
    #[serde(rename = "type")]
    pub device_type: String,
}

#[derive(Deserialize, Debug)]
struct LsblkOutput {
    pub blockdevices: Vec<BlockDevice>,
}

pub fn get_block_devices() -> Result<Vec<BlockDevice>, String> {
    let output = Command::new("lsblk")
        .args(&["-J", "-o", "NAME,SIZE,MODEL,TRAN,TYPE,RM"])
        .output()
        .map_err(|e| format!("Failed to execute lsblk: {}", e))?;

    let decoded: LsblkOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse lsblk JSON: {}", e))?;

    Ok(decoded.blockdevices.into_iter()
        .filter(|d| d.device_type == "disk")
        .collect())
}

fn is_internal_device(device: &BlockDevice) -> bool {
    if device.device_type != "disk" {
        return false;
    }

    if device.rm == Some(1) {
        return false;
    }

    !matches!(device.tran.as_deref(), Some("usb"))
}

pub fn get_internal_block_devices() -> Result<Vec<BlockDevice>, String> {
    Ok(get_block_devices()?
        .into_iter()
        .filter(is_internal_device)
        .collect())
}

pub fn partition_device_path(drive: &str, partition: u8) -> String {
    let suffix = partition.to_string();
    if drive.chars().last().is_some_and(|c| c.is_ascii_digit()) {
        format!("{}p{}", drive, suffix)
    } else {
        format!("{}{}", drive, suffix)
    }
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

    let p1 = partition_device_path(drive, 1);
    let p2 = partition_device_path(drive, 2);
    let p3 = partition_device_path(drive, 3);

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

#[cfg(test)]
mod tests {
    use super::{BlockDevice, get_internal_block_devices, is_internal_device, partition_device_path};

    fn fixture_device(name: &str, device_type: &str, tran: Option<&str>, rm: Option<u8>) -> BlockDevice {
        BlockDevice {
            name: name.to_string(),
            size: "100G".to_string(),
            model: Some("Fixture".to_string()),
            tran: tran.map(|value| value.to_string()),
            rm,
            device_type: device_type.to_string(),
        }
    }

    #[test]
    fn internal_filter_excludes_usb_or_removable_disks() {
        let sata = fixture_device("sda", "disk", Some("sata"), Some(0));
        let usb = fixture_device("sdb", "disk", Some("usb"), Some(1));
        let partition = fixture_device("sda1", "part", Some("sata"), Some(0));

        assert!(is_internal_device(&sata));
        assert!(!is_internal_device(&usb));
        assert!(!is_internal_device(&partition));
    }

    #[test]
    fn internal_filter_allows_unknown_transport_non_removable_disks() {
        let nvme = fixture_device("nvme0n1", "disk", None, Some(0));
        let unknown_rm = fixture_device("vda", "disk", None, None);

        assert!(is_internal_device(&nvme));
        assert!(is_internal_device(&unknown_rm));
    }

    #[test]
    fn internal_device_query_function_is_available() {
        let _ = get_internal_block_devices;
    }

    #[test]
    fn partition_device_path_supports_nvme_and_sd_drives() {
        assert_eq!(partition_device_path("/dev/sda", 1), "/dev/sda1");
        assert_eq!(partition_device_path("/dev/nvme0n1", 1), "/dev/nvme0n1p1");
    }
}
