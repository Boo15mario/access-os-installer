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
