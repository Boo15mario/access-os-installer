use crate::backend::config_engine::{DesktopEnv, KernelVariant};
use crate::backend::preflight::CheckResult;
use crate::backend::storage_plan::{HomeLocation, HomeMode, ResolvedInstallLayout, SetupMode, SwapMode};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct DriveOption {
    pub path: String,
    pub disk_gib: u64,
}

pub struct AppState {
    pub drive: String,
    pub selected_disk_gib: Option<u64>,
    pub swap_gb: u64,
    pub swap_mode: SwapMode,
    pub swap_file_mb: u64,
    pub fs_type: String,
    pub setup_mode: SetupMode,
    pub home_mode: HomeMode,
    pub home_location: HomeLocation,
    pub home_disk: String,
    pub manual_efi_partition: String,
    pub manual_root_partition: String,
    pub manual_home_partition: String,
    pub manual_swap_partition: String,
    pub format_efi: bool,
    pub format_root: bool,
    pub format_home: bool,
    pub format_swap: bool,
    pub removable_media: bool,
    pub desktop_env: Option<DesktopEnv>,
    pub kernel: KernelVariant,
    pub nvidia: bool,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub timezone: String,
    pub locale: String,
    pub keymap: String,
    pub mirror_region: String,
    pub preflight_results: Vec<CheckResult>,
    pub resolved_layout: Option<ResolvedInstallLayout>,
}

pub type SharedState = Rc<RefCell<AppState>>;
