pub mod disk_manager;
pub mod config_engine;
pub mod install_worker;
pub mod network;
pub mod preflight;
pub mod storage_plan;

pub type ProgressCallback = dyn Fn(&str);

pub fn emit_progress(progress: Option<&ProgressCallback>, message: &str) {
    if let Some(callback) = progress {
        callback(message);
    }
}

use sysinfo::System;

pub fn get_suggested_swap_gb() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_ram_bytes = sys.total_memory();
    let total_ram_gb = total_ram_bytes / (1024 * 1024 * 1024);
    
    if total_ram_gb > 16 {
        total_ram_gb
    } else {
        total_ram_gb * 2
    }
}
