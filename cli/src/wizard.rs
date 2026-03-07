use installer_core::backend::config_engine::{DesktopEnv, KernelVariant};
use installer_core::backend::disk_manager::{self};
use installer_core::backend::storage_plan::{
    self, HomeLocation, HomeMode, ManualCreatePartition, ManualPartitionRole,
    ResolvedInstallLayout, SetupMode, StorageSelection, SwapMode,
};
use installer_core::backend::{network, preflight};
use installer_core::constants::{KEYMAPS, LOCALES, MIRROR_REGIONS, TIMEZONES};
use installer_core::services::{mirror, mount, power};

use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

pub struct Wizard {
    dry_run: bool,
    step: Step,
    state: State,
    cached_layout: Option<ResolvedInstallLayout>,
    cached_disk_gib: Option<u64>,
    network_status: NetworkStatus,
    cleared_screen: bool,
    done: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Welcome,
    WiFiSetup,
    InstallOptions,
    DiskSelection,
    DiskSetup,
    Regional,
    UserSettings,
    Review,
    Install,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NetworkStatus {
    Checking,
    Connected,
    NotConnected,
}

struct State {
    desktop_env: Option<DesktopEnv>,
    kernel: KernelVariant,
    nvidia: bool,
    removable_media: bool,

    mirror_region: String,
    timezone: String,
    locale: String,
    keymap: String,

    hostname: String,
    username: String,
    password: String,

    storage: StorageSelection,
}

impl Wizard {
    pub fn new(dry_run: bool) -> Self {
        let suggested_swap = installer_core::backend::get_suggested_swap_gb();
        Self {
            dry_run,
            step: Step::Welcome,
            state: State {
                desktop_env: None,
                kernel: KernelVariant::Standard,
                nvidia: false,
                removable_media: false,
                mirror_region: "Worldwide".to_string(),
                timezone: "America/Chicago".to_string(),
                locale: "en_US.UTF-8".to_string(),
                keymap: "us".to_string(),
                hostname: String::new(),
                username: String::new(),
                password: String::new(),
                storage: StorageSelection {
                    install_disk: String::new(),
                    setup_mode: SetupMode::Automatic,
                    fs_type: "xfs".to_string(),
                    swap_mode: SwapMode::Partition,
                    swap_size_gib: suggested_swap,
                    swap_file_size_mb: Some(suggested_swap * 1024),
                    home_mode: HomeMode::OnRoot,
                    home_location: HomeLocation::SameDisk,
                    home_disk: None,
                    manual_efi_partition: None,
                    manual_root_partition: None,
                    manual_home_partition: None,
                    manual_swap_partition: None,
                    manual_create_actions: Vec::new(),
                    manual_delete_partitions: Vec::new(),
                    format_efi: true,
                    format_root: true,
                    format_home: true,
                    format_swap: true,
                    removable_media: false,
                },
            },
            cached_layout: None,
            cached_disk_gib: None,
            network_status: NetworkStatus::Checking,
            cleared_screen: false,
            done: false,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        while !self.done {
            match self.step {
                Step::Welcome => self.step_welcome()?,
                Step::WiFiSetup => self.step_wifi_setup()?,
                Step::InstallOptions => self.step_install_options()?,
                Step::DiskSelection => self.step_disk_selection()?,
                Step::DiskSetup => self.step_disk_setup()?,
                Step::Regional => self.step_regional()?,
                Step::UserSettings => self.step_user_settings()?,
                Step::Review => self.step_review()?,
                Step::Install => self.step_install()?,
                Step::Complete => self.step_complete()?,
            }
        }
        Ok(())
    }

    fn step_welcome(&mut self) -> Result<(), String> {
        self.clear_screen_once()?;
        self.refresh_network_status();

        println!();
        println!("Access OS Installer (CLI) v{}", env!("CARGO_PKG_VERSION"));
        println!("Type `next` to begin, or `quit` to exit.");
        println!("Network: {}", self.network_status_label());
        println!("Commands always available: next, back, help, quit");
        println!();

        loop {
            let input = prompt("> ")?;
            match input.as_str() {
                "help" => {
                    println!(
                        "This is a line-based installer. Type `next` to move forward. If internet is unavailable, `next` opens Wi-Fi setup."
                    );
                }
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "back" => {
                    println!("Already at the first step.");
                }
                "next" => {
                    if !is_root() {
                        println!("ERROR: You must run this installer as root.");
                        continue;
                    }
                    self.refresh_network_status();
                    if self.network_status == NetworkStatus::Connected {
                        self.step = Step::InstallOptions;
                    } else {
                        self.step = Step::WiFiSetup;
                    }
                    return Ok(());
                }
                other => {
                    println!("Unknown command: {}", other);
                }
            }
        }
    }

    fn step_wifi_setup(&mut self) -> Result<(), String> {
        loop {
            self.refresh_network_status();
            println!();
            println!("Wi-Fi Setup");
            println!("Network: {}", self.network_status_label());
            println!("1) Scan and connect to Wi-Fi");
            println!("2) Re-check internet status");
            println!();
            println!("Type a number to act, or `next` / `back`.");

            match prompt("> ")?.as_str() {
                "help" => {
                    println!("Connect to Wi-Fi here. Once internet is connected, type `next` to continue to install options.");
                }
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "back" => {
                    self.step = Step::Welcome;
                    return Ok(());
                }
                "next" => {
                    self.refresh_network_status();
                    if self.network_status != NetworkStatus::Connected {
                        println!("ERROR: Internet is still not connected.");
                        continue;
                    }
                    self.step = Step::InstallOptions;
                    return Ok(());
                }
                "1" => {
                    if let Err(err) = self.wifi_connect_flow() {
                        println!("ERROR: {}", err);
                    }
                }
                "2" => self.refresh_network_status(),
                other => println!("Unknown input: {}", other),
            }
        }
    }

    fn step_install_options(&mut self) -> Result<(), String> {
        loop {
            println!();
            println!("Install Options");
            println!("1) Desktop environment: {}", desktop_env_label(self.state.desktop_env.as_ref()));
            println!("2) Kernel: {}", self.state.kernel.label());
            println!("3) Nvidia drivers: {}", yes_no(self.state.nvidia));
            println!("4) Removable media install: {}", yes_no(self.state.removable_media));
            println!();
            println!("Type a number to edit, or `next` / `back`.");

            match prompt("> ")?.as_str() {
                "help" => {
                    println!("Set your desktop environment and kernel. These choices affect which packages are installed.");
                }
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "back" => {
                    self.step = Step::Welcome;
                    return Ok(());
                }
                "next" => {
                    if self.state.desktop_env.is_none() {
                        println!("ERROR: Select a desktop environment.");
                        continue;
                    }
                    self.step = Step::DiskSelection;
                    return Ok(());
                }
                "1" => self.select_desktop_env()?,
                "2" => self.select_kernel()?,
                "3" => self.state.nvidia = !self.state.nvidia,
                "4" => {
                    self.state.removable_media = !self.state.removable_media;
                    self.state.storage.removable_media = self.state.removable_media;
                }
                other => {
                    println!("Unknown input: {}", other);
                }
            }
        }
    }

    fn step_disk_selection(&mut self) -> Result<(), String> {
        let devices = match disk_manager::get_internal_block_devices() {
            Ok(devices) => devices,
            Err(err) => {
                println!("ERROR: Failed to read internal drives: {}", err);
                println!("Type `back` to return or `quit` to exit.");
                loop {
                    match prompt("> ")?.as_str() {
                        "back" => {
                            self.step = Step::InstallOptions;
                            return Ok(());
                        }
                        "quit" | "exit" => {
                            self.done = true;
                            return Ok(());
                        }
                        _ => println!("Type `back` or `quit`."),
                    }
                }
            }
        };

        if devices.is_empty() {
            return Err("No internal disks detected. Add a disk and retry.".to_string());
        }

        loop {
            println!();
            println!("Disk Selection");
            println!("WARNING: Installing will erase data on the selected disk.");
            println!();
            for (idx, device) in devices.iter().enumerate() {
                let path = format!("/dev/{}", device.name);
                let size_label = disk_manager::human_gib_label(device.size_bytes);
                let model = device
                    .model
                    .as_deref()
                    .unwrap_or("Unknown model")
                    .trim();
                let transport = device.tran.as_deref().unwrap_or("internal");
                println!("{}) {} | {} | {} | {}", idx + 1, path, size_label, model, transport);
            }
            println!();
            println!("Selected: {}", if self.state.storage.install_disk.is_empty() { "(none)" } else { &self.state.storage.install_disk });
            println!("Type a number to select, or `next` / `back`.");

            let input = prompt("> ")?;
            match input.as_str() {
                "help" => println!("Pick the internal disk where Access OS will be installed."),
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "back" => {
                    self.step = Step::InstallOptions;
                    return Ok(());
                }
                "next" => {
                    if self.state.storage.install_disk.is_empty() {
                        println!("ERROR: Select a disk.");
                        continue;
                    }
                    self.step = Step::DiskSetup;
                    return Ok(());
                }
                other => {
                    if let Ok(choice) = other.parse::<usize>() {
                        if choice == 0 || choice > devices.len() {
                            println!("ERROR: Invalid selection.");
                            continue;
                        }
                        let device = &devices[choice - 1];
                        self.state.storage.install_disk = format!("/dev/{}", device.name);
                        self.cached_disk_gib = Some(disk_manager::bytes_to_gib(device.size_bytes));
                        self.cached_layout = None;
                    } else {
                        println!("Unknown input: {}", other);
                    }
                }
            }
        }
    }

    fn step_disk_setup(&mut self) -> Result<(), String> {
        loop {
            println!();
            println!("Disk Setup");
            println!("1) Setup mode: {:?}", self.state.storage.setup_mode);
            println!("2) Root filesystem: {}", self.state.storage.fs_type);
            println!("3) Swap mode: {:?}", self.state.storage.swap_mode);
            match self.state.storage.swap_mode {
                SwapMode::Partition => println!("4) Swap size (GiB): {}", self.state.storage.swap_size_gib),
                SwapMode::File => println!(
                    "4) Swap file size (MB): {}",
                    self.state.storage.swap_file_size_mb.unwrap_or(self.state.storage.swap_size_gib * 1024)
                ),
            }
            println!("5) Home mode: {:?}", self.state.storage.home_mode);
            println!("6) Home location: {}", self.home_location_label());
            println!(
                "7) Home disk: {}",
                if self.state.storage.home_mode == HomeMode::Separate
                    && self.state.storage.home_location == HomeLocation::OtherDisk
                {
                    self.state
                        .storage
                        .home_disk
                        .as_deref()
                        .unwrap_or("(not selected)")
                } else {
                    "n/a"
                }
            );
            println!(
                "8) Manual partition manager: {}",
                if self.state.storage.setup_mode == SetupMode::Manual {
                    "open"
                } else {
                    "n/a"
                }
            );
            println!();
            println!("Type a number to edit, or `next` / `back`.");

            match prompt("> ")?.as_str() {
                "help" => {
                    println!("Configure automatic or manual partitioning. `next` validates the setup and computes the destructive plan.");
                }
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "back" => {
                    self.step = Step::DiskSelection;
                    return Ok(());
                }
                "next" => {
                    let layout = match storage_plan::resolve_layout(&self.state.storage) {
                        Ok(layout) => layout,
                        Err(err) => {
                            println!("ERROR: {}", err);
                            continue;
                        }
                    };
                    self.cached_layout = Some(layout);
                    self.step = Step::Regional;
                    return Ok(());
                }
                "1" => self.select_setup_mode()?,
                "2" => self.select_root_fs()?,
                "3" => self.select_swap_mode()?,
                "4" => self.edit_swap_size()?,
                "5" => self.select_home_mode()?,
                "6" => self.select_home_location()?,
                "7" => self.select_home_disk()?,
                "8" => {
                    if self.state.storage.setup_mode == SetupMode::Manual {
                        self.edit_manual_partitions()?;
                    } else {
                        println!("Manual partition manager is only available in manual setup mode.");
                    }
                }
                other => println!("Unknown input: {}", other),
            }
        }
    }

    fn step_regional(&mut self) -> Result<(), String> {
        loop {
            println!();
            println!("Regional Settings");
            println!("1) Mirror region: {}", self.state.mirror_region);
            println!("2) Timezone: {}", self.state.timezone);
            println!("3) Locale: {}", self.state.locale);
            println!("4) Keymap: {}", self.state.keymap);
            println!();
            println!("Type a number to edit, or `next` / `back`.");

            match prompt("> ")?.as_str() {
                "help" => println!("These settings control mirrors, timezone, locale, and console keymap."),
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "back" => {
                    self.step = Step::DiskSetup;
                    return Ok(());
                }
                "next" => {
                    self.step = Step::UserSettings;
                    return Ok(());
                }
                "1" => {
                    if let Some(value) = self.pick_from_list("Mirror region", MIRROR_REGIONS)? {
                        self.state.mirror_region = value;
                    }
                }
                "2" => {
                    if let Some(value) = self.pick_from_list("Timezone", TIMEZONES)? {
                        self.state.timezone = value;
                    }
                }
                "3" => {
                    if let Some(value) = self.pick_from_list("Locale", LOCALES)? {
                        self.state.locale = value;
                    }
                }
                "4" => {
                    if let Some(value) = self.pick_from_list("Keymap", KEYMAPS)? {
                        self.state.keymap = value;
                    }
                }
                other => println!("Unknown input: {}", other),
            }
        }
    }

    fn step_user_settings(&mut self) -> Result<(), String> {
        loop {
            println!();
            println!("User Settings");
            println!("1) Hostname: {}", empty_as(&self.state.hostname, "(not set)"));
            println!("2) Username: {}", empty_as(&self.state.username, "(not set)"));
            println!(
                "3) Password: {}",
                if self.state.password.is_empty() { "(not set)" } else { "(set)" }
            );
            println!();
            println!("Type a number to edit, or `next` / `back`.");

            match prompt("> ")?.as_str() {
                "help" => println!("A user account will be created with sudo access (wheel)."),
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "back" => {
                    self.step = Step::Regional;
                    return Ok(());
                }
                "next" => {
                    if self.state.hostname.trim().is_empty() {
                        println!("ERROR: Hostname is required.");
                        continue;
                    }
                    if self.state.username.trim().is_empty() {
                        println!("ERROR: Username is required.");
                        continue;
                    }
                    if self.state.password.is_empty() {
                        println!("ERROR: Password is required.");
                        continue;
                    }
                    self.step = Step::Review;
                    return Ok(());
                }
                "1" => {
                    let value = prompt("Hostname: ")?;
                    self.state.hostname = value.split_whitespace().next().unwrap_or("").to_string();
                }
                "2" => {
                    let value = prompt("Username: ")?;
                    self.state.username = value
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                }
                "3" => {
                    let p1 = prompt_password("Password: ")?;
                    let p2 = prompt_password("Retype password: ")?;
                    if p1.is_empty() {
                        println!("ERROR: Password cannot be empty.");
                        continue;
                    }
                    if p1 != p2 {
                        println!("ERROR: Passwords do not match.");
                        continue;
                    }
                    self.state.password = p1;
                }
                other => println!("Unknown input: {}", other),
            }
        }
    }

    fn step_review(&mut self) -> Result<(), String> {
        let Some(de) = self.state.desktop_env.clone() else {
            return Err("Desktop environment missing in state.".to_string());
        };

        let layout = storage_plan::resolve_layout(&self.state.storage)?;
        self.cached_layout = Some(layout.clone());

        println!();
        println!("Review");
        println!();
        println!("Target disk: {}", self.state.storage.install_disk);
        if let Some(gib) = self.cached_disk_gib {
            println!("Disk size: {} GiB", gib);
        }
        println!("Disk setup: {:?} ({})", self.state.storage.setup_mode, self.state.storage.fs_type);
        println!("Mirror region: {}", self.state.mirror_region);
        println!("Kernel: {}", self.state.kernel.label());
        println!("Desktop environment: {}", de.label());
        println!("Nvidia drivers: {}", yes_no(self.state.nvidia));
        println!("Removable media: {}", yes_no(self.state.removable_media));
        println!("Hostname: {}", self.state.hostname);
        println!("Username: {}", self.state.username);
        println!("Timezone: {}", self.state.timezone);
        println!("Locale: {}", self.state.locale);
        println!("Keymap: {}", self.state.keymap);
        println!();
        println!("Destructive plan:");
        println!("{}", storage_plan::format_destructive_plan(&layout));
        println!();

        let checks = preflight::evaluate_checks(&preflight::PreflightContext {
            is_uefi: Path::new("/sys/firmware/efi").exists(),
            has_disk: !self.state.storage.install_disk.is_empty(),
            online: network::check_connectivity(),
            ram_gib: system_ram_gib(),
            disk_gib: self.cached_disk_gib,
        });
        if preflight::has_hard_fail(&checks) {
            println!("Preflight: HARD FAIL");
            for check in checks.iter().filter(|c| c.is_hard) {
                println!("[{}] {}: {}", check.status.label(), check.label, check.message);
            }
            println!();
            println!("Fix hard failures before installing. Type `back` to review settings.");
        } else {
            let warns: Vec<_> = checks.iter().filter(|c| c.status == preflight::CheckStatus::Warn).collect();
            if !warns.is_empty() {
                println!("Preflight warnings:");
                for warn in warns {
                    println!("- {}", warn.message);
                }
                println!();
            }
        }

        println!("Type `install` to proceed, `back` to make changes, or `quit` to exit.");
        if self.dry_run {
            println!("NOTE: --dry-run enabled; no disk or install actions will be performed.");
        }

        loop {
            match prompt("> ")?.as_str() {
                "help" => println!("`install` starts the installation. `back` returns to the previous screen."),
                "back" => {
                    self.step = Step::UserSettings;
                    return Ok(());
                }
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "install" => {
                    if self.dry_run {
                        self.step = Step::Complete;
                        return Ok(());
                    }
                    self.step = Step::Install;
                    return Ok(());
                }
                "next" => println!("Type `install` to start, not `next`."),
                other => println!("Unknown input: {}", other),
            }
        }
    }

    fn step_install(&mut self) -> Result<(), String> {
        let Some(de) = self.state.desktop_env.clone() else {
            return Err("Desktop environment missing in state.".to_string());
        };
        let layout = self
            .cached_layout
            .clone()
            .ok_or("Internal error: missing cached layout. Return to Review.")?;
        let root_partition = layout.root_partition.clone();

        println!();
        println!("Installing...");
        let progress = |message: &str| println!("  {}", message);

        println!("1/7 Partitioning and mounting...");
        mount::prepare_install_targets(&layout, Some(&progress))?;

        println!("2/7 Staging system config...");
        installer_core::backend::install_worker::stage_system_config_repo(
            installer_core::constants::DOTFILES_REPO_URL,
            Some(&progress),
        )?;
        installer_core::backend::install_worker::overlay_staged_config_to_target(Some(&progress))?;

        println!("3/7 Applying mirror region ({})...", self.state.mirror_region);
        if let Err(err) = mirror::apply_mirror_region(&self.state.mirror_region) {
            println!("WARN: mirror apply failed (non-fatal): {}", err);
        }

        println!("4/7 Installing base system (pacstrap)...");
        let config = installer_core::backend::install_worker::InstallConfig {
            username: self.state.username.clone(),
            password: self.state.password.clone(),
            hostname: self.state.hostname.clone(),
            timezone: self.state.timezone.clone(),
            locale: self.state.locale.clone(),
            keymap: self.state.keymap.clone(),
            desktop_env: de.clone(),
            kernel: self.state.kernel.clone(),
            nvidia: self.state.nvidia,
            removable_media: self.state.removable_media,
        };
        installer_core::backend::install_worker::run_pacstrap(&config, Some(&progress))?;

        println!("5/7 Setting up swap (if configured)...");
        installer_core::backend::disk_manager::setup_swap_file(&layout, Some(&progress))?;

        println!("6/7 Generating fstab...");
        installer_core::backend::install_worker::generate_fstab(Some(&progress))?;

        println!("7/7 Configuring system...");
        installer_core::backend::install_worker::configure_system(
            &config,
            &root_partition,
            Some(&progress),
        )?;

        // Best-effort: clear password in memory after install.
        self.state.password.clear();

        println!();
        println!("SUCCESS: Installation finished.");
        self.step = Step::Complete;
        Ok(())
    }

    fn step_complete(&mut self) -> Result<(), String> {
        println!();
        println!("Complete");
        println!("Commands: reboot, shutdown, unmount, quit");
        println!();

        loop {
            match prompt("> ")?.as_str() {
                "help" => println!("Use `unmount` if you want to safely remove the install media; then `reboot` or `shutdown`."),
                "quit" | "exit" => {
                    self.done = true;
                    return Ok(());
                }
                "unmount" => match mount::unmount_install_targets() {
                    Ok(()) => println!("Unmounted /mnt targets."),
                    Err(err) => println!("ERROR: {}", err),
                },
                "reboot" => match power::reboot_system() {
                    Ok(()) => {
                        self.done = true;
                        return Ok(());
                    }
                    Err(err) => println!("ERROR: {}", err),
                },
                "shutdown" => match power::shutdown_system() {
                    Ok(()) => {
                        self.done = true;
                        return Ok(());
                    }
                    Err(err) => println!("ERROR: {}", err),
                },
                "back" => println!("No previous step after completion."),
                "next" => println!("No next step after completion."),
                other => println!("Unknown input: {}", other),
            }
        }
    }

    fn select_desktop_env(&mut self) -> Result<(), String> {
        let all = DesktopEnv::all();
        println!();
        println!("Desktop Environment");
        let mut mapping = Vec::new();
        for env in all {
            if !env.is_available() {
                continue;
            }
            mapping.push(env.clone());
        }
        for (idx, env) in mapping.iter().enumerate() {
            println!("{}) {} - {}", idx + 1, env.label(), env.description());
        }
        println!("Type a number, or `back` to cancel.");
        loop {
            let input = prompt("> ")?;
            match input.as_str() {
                "back" => return Ok(()),
                other => {
                    let choice = other.parse::<usize>().ok();
                    let Some(choice) = choice else {
                        println!("Invalid input.");
                        continue;
                    };
                    if choice == 0 || choice > mapping.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    self.state.desktop_env = Some(mapping[choice - 1].clone());
                    return Ok(());
                }
            }
        }
    }

    fn select_kernel(&mut self) -> Result<(), String> {
        println!();
        println!("Kernel");
        let all = KernelVariant::all();
        for (idx, kernel) in all.iter().enumerate() {
            println!("{}) {} - {}", idx + 1, kernel.label(), kernel.description());
        }
        println!("Type a number, or `back` to cancel.");
        loop {
            let input = prompt("> ")?;
            match input.as_str() {
                "back" => return Ok(()),
                other => {
                    let choice = other.parse::<usize>().ok();
                    let Some(choice) = choice else {
                        println!("Invalid input.");
                        continue;
                    };
                    if choice == 0 || choice > all.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    self.state.kernel = all[choice - 1].clone();
                    return Ok(());
                }
            }
        }
    }

    fn select_setup_mode(&mut self) -> Result<(), String> {
        println!();
        println!("Setup mode");
        println!("1) Automatic");
        println!("2) Manual");
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "1" => {
                    self.state.storage.setup_mode = SetupMode::Automatic;
                    return Ok(());
                }
                "2" => {
                    self.state.storage.setup_mode = SetupMode::Manual;
                    return Ok(());
                }
                _ => println!("Invalid input."),
            }
        }
    }

    fn select_root_fs(&mut self) -> Result<(), String> {
        println!();
        println!("Root filesystem");
        println!("1) xfs");
        println!("2) ext4");
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "1" => {
                    self.state.storage.fs_type = "xfs".to_string();
                    return Ok(());
                }
                "2" => {
                    self.state.storage.fs_type = "ext4".to_string();
                    return Ok(());
                }
                _ => println!("Invalid input."),
            }
        }
    }

    fn select_swap_mode(&mut self) -> Result<(), String> {
        println!();
        println!("Swap mode");
        println!("1) Swap partition");
        println!("2) Swap file");
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "1" => {
                    self.state.storage.swap_mode = SwapMode::Partition;
                    return Ok(());
                }
                "2" => {
                    self.state.storage.swap_mode = SwapMode::File;
                    self.clear_manual_role_state(ManualPartitionRole::Swap);
                    return Ok(());
                }
                _ => println!("Invalid input."),
            }
        }
    }

    fn edit_swap_size(&mut self) -> Result<(), String> {
        match self.state.storage.swap_mode {
            SwapMode::Partition => {
                let value = prompt("Enter swap size in GiB: ")?;
                let gib = value.parse::<u64>().unwrap_or(0);
                if gib == 0 {
                    println!("ERROR: swap size must be > 0.");
                    return Ok(());
                }
                self.state.storage.swap_size_gib = gib;
                self.state.storage.swap_file_size_mb = Some(gib * 1024);
            }
            SwapMode::File => {
                let value = prompt("Enter swap file size in MB: ")?;
                let mb = value.parse::<u64>().unwrap_or(0);
                if mb < 512 {
                    println!("ERROR: swap file size must be at least 512 MB.");
                    return Ok(());
                }
                self.state.storage.swap_file_size_mb = Some(mb);
            }
        }
        Ok(())
    }

    fn select_home_mode(&mut self) -> Result<(), String> {
        println!();
        println!("Home mode");
        println!("1) Home on root");
        println!("2) Separate /home");
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "1" => {
                    self.state.storage.home_mode = HomeMode::OnRoot;
                    self.state.storage.home_location = HomeLocation::SameDisk;
                    self.state.storage.home_disk = None;
                    self.clear_manual_role_state(ManualPartitionRole::Home);
                    return Ok(());
                }
                "2" => {
                    self.state.storage.home_mode = HomeMode::Separate;
                    self.state.storage.home_location = HomeLocation::SameDisk;
                    return Ok(());
                }
                _ => println!("Invalid input."),
            }
        }
    }

    fn select_home_location(&mut self) -> Result<(), String> {
        if self.state.storage.home_mode != HomeMode::Separate {
            println!("Home is not set to separate.");
            return Ok(());
        }

        println!();
        println!("Home location");
        println!("1) Same disk as root");
        println!("2) Other disk");
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "1" => {
                    self.state.storage.home_location = HomeLocation::SameDisk;
                    self.state.storage.home_disk = None;
                    return Ok(());
                }
                "2" => {
                    self.state.storage.home_location = HomeLocation::OtherDisk;
                    return self.select_home_disk();
                }
                _ => println!("Invalid input."),
            }
        }
    }

    fn select_home_disk(&mut self) -> Result<(), String> {
        if self.state.storage.home_mode != HomeMode::Separate
            || self.state.storage.home_location != HomeLocation::OtherDisk
        {
            println!("Home disk selection is only used for a separate /home on another disk.");
            return Ok(());
        }

        let devices = disk_manager::get_internal_block_devices()
            .map_err(|e| format!("Failed to list internal drives: {}", e))?;
        let devices: Vec<_> = devices
            .into_iter()
            .filter(|device| format!("/dev/{}", device.name) != self.state.storage.install_disk)
            .collect();
        if devices.is_empty() {
            println!("No second internal disk is available for /home.");
            return Ok(());
        }
        println!();
        println!("Select home disk");
        for (idx, device) in devices.iter().enumerate() {
            println!("{}) /dev/{}", idx + 1, device.name);
        }
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                other => {
                    let choice = other.parse::<usize>().ok().unwrap_or(0);
                    if choice == 0 || choice > devices.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    self.state.storage.home_disk = Some(format!("/dev/{}", devices[choice - 1].name));
                    return Ok(());
                }
            }
        }
    }

    fn edit_manual_partitions(&mut self) -> Result<(), String> {
        loop {
            self.normalize_manual_storage_state();
            let partitions = self.manual_partition_entries()?;

            println!();
            println!("Manual partitions");
            println!(
                "Managed disks: {}",
                join_or_none(&storage_plan::managed_disks(&self.state.storage))
            );
            println!("EFI: {}", self.manual_role_label(ManualPartitionRole::Efi));
            println!("root: {}", self.manual_role_label(ManualPartitionRole::Root));
            if self.state.storage.home_mode == HomeMode::Separate {
                println!("home: {}", self.manual_role_label(ManualPartitionRole::Home));
            }
            if self.state.storage.swap_mode == SwapMode::Partition {
                println!("swap: {}", self.manual_role_label(ManualPartitionRole::Swap));
            }
            println!(
                "Format flags: EFI {} | root {}{}{}",
                yes_no(self.state.storage.format_efi),
                yes_no(self.state.storage.format_root),
                if self.state.storage.home_mode == HomeMode::Separate {
                    format!(" | home {}", yes_no(self.state.storage.format_home))
                } else {
                    String::new()
                },
                if self.state.storage.swap_mode == SwapMode::Partition {
                    format!(" | swap {}", yes_no(self.state.storage.format_swap))
                } else {
                    String::new()
                }
            );
            println!();
            println!("Partitions:");
            if partitions.is_empty() {
                println!("- none");
            } else {
                for (idx, part) in partitions.iter().enumerate() {
                    println!(
                        "{}) {} | {} | {} | {}",
                        idx + 1,
                        part.path,
                        part.size_label,
                        empty_as(&part.fstype, "unknown"),
                        part.status_label
                    );
                }
            }
            println!();
            println!("1) Create partition");
            println!("2) Delete partition");
            println!("3) Assign role");
            println!("4) Toggle format flags");
            println!("5) Back");

            match prompt("> ")?.as_str() {
                "back" | "5" => return Ok(()),
                "1" => self.create_manual_partition_flow()?,
                "2" => self.delete_manual_partition_flow()?,
                "3" => self.assign_manual_role_flow()?,
                "4" => self.toggle_manual_format_flags()?,
                "help" => println!("Create, delete, assign, and format installer partitions on the managed disks."),
                other => println!("Unknown input: {}", other),
            }
        }
    }

    fn create_manual_partition_flow(&mut self) -> Result<(), String> {
        let Some(role) = self.pick_manual_role("Create partition", "Choose a role to create.")? else {
            return Ok(());
        };
        let candidate_disks = self.manual_role_target_disks(role);
        if candidate_disks.is_empty() {
            println!("No managed disk is available for {}.", role.label());
            return Ok(());
        }

        let disk = if candidate_disks.len() == 1 {
            candidate_disks[0].clone()
        } else {
            match self.choose_disk_from_list("Target disk", &candidate_disks)? {
                Some(value) => value,
                None => return Ok(()),
            }
        };

        let existing = disk_manager::get_partitions_for_managed_disks(&storage_plan::managed_disks(
            &self.state.storage,
        ))?;
        let partition_number = disk_manager::next_available_partition_number(
            &disk,
            &existing,
            &self.state.storage.manual_create_actions,
            &self.state.storage.manual_delete_partitions,
        )?;
        let Some((size_gib, use_remaining)) = self.prompt_manual_partition_size(role)? else {
            return Ok(());
        };
        let path = disk_manager::partition_device_path(&disk, partition_number);
        let action = ManualCreatePartition {
            disk: disk.clone(),
            partition_number,
            role,
            size_gib,
            use_remaining,
            path: path.clone(),
        };
        self.state.storage.manual_create_actions.push(action);
        self.set_manual_role_assignment(role, Some(path.clone()));

        println!("Planned {} on {} as {}.", path, disk, role.label());
        Ok(())
    }

    fn delete_manual_partition_flow(&mut self) -> Result<(), String> {
        let partitions = self.manual_partition_entries()?;
        if partitions.is_empty() {
            println!("No managed partitions are available to delete.");
            return Ok(());
        }

        println!();
        println!("Delete partition");
        for (idx, part) in partitions.iter().enumerate() {
            println!("{}) {} | {}", idx + 1, part.path, part.status_label);
        }
        println!("Type a number, or `back` to cancel.");

        let target = loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                other => {
                    let choice = other.parse::<usize>().ok().unwrap_or(0);
                    if choice == 0 || choice > partitions.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    break partitions[choice - 1].clone();
                }
            }
        };

        let token = format!("delete {}", target.path);
        println!(
            "Type `{}` to confirm. Assigned roles using this partition will be cleared.",
            token
        );
        if prompt("> ")? != token {
            println!("Delete cancelled.");
            return Ok(());
        }

        if target.pending_create {
            self.state
                .storage
                .manual_create_actions
                .retain(|action| action.path != target.path);
        } else if !self
            .state
            .storage
            .manual_delete_partitions
            .iter()
            .any(|path| path == &target.path)
        {
            self.state
                .storage
                .manual_delete_partitions
                .push(target.path.clone());
        }
        self.clear_assignments_for_path(&target.path);
        storage_plan::clear_deleted_partition_assignments(&mut self.state.storage);
        println!("Marked {} for deletion.", target.path);
        Ok(())
    }

    fn assign_manual_role_flow(&mut self) -> Result<(), String> {
        let Some(role) = self.pick_manual_role("Assign role", "Choose a role to assign.")? else {
            return Ok(());
        };
        let target_disks = self.manual_role_target_disks(role);
        let partitions: Vec<_> = self
            .manual_partition_entries()?
            .into_iter()
            .filter(|partition| target_disks.iter().any(|disk| disk == &partition.parent_disk))
            .collect();

        println!();
        println!("Assign {}", role.label());
        println!("Current: {}", self.manual_role_label(role));
        if partitions.is_empty() {
            println!("No partitions are available for {}.", role.label());
            return Ok(());
        }

        for (idx, part) in partitions.iter().enumerate() {
            println!("{}) {} | {}", idx + 1, part.path, part.status_label);
        }
        println!("Type a number, `clear`, or `back`.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "clear" => {
                    self.set_manual_role_assignment(role, None);
                    return Ok(());
                }
                other => {
                    let choice = other.parse::<usize>().ok().unwrap_or(0);
                    if choice == 0 || choice > partitions.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    self.set_manual_role_assignment(role, Some(partitions[choice - 1].path.clone()));
                    return Ok(());
                }
            }
        }
    }

    fn toggle_manual_format_flags(&mut self) -> Result<(), String> {
        loop {
            println!();
            println!("Format flags");
            println!("1) EFI: {}", yes_no(self.state.storage.format_efi));
            println!("2) root: {}", yes_no(self.state.storage.format_root));
            if self.state.storage.home_mode == HomeMode::Separate {
                println!("3) home: {}", yes_no(self.state.storage.format_home));
            }
            if self.state.storage.swap_mode == SwapMode::Partition {
                println!("4) swap: {}", yes_no(self.state.storage.format_swap));
            }
            println!("Type a number to toggle, or `back`.");

            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "1" => self.state.storage.format_efi = !self.state.storage.format_efi,
                "2" => self.state.storage.format_root = !self.state.storage.format_root,
                "3" => {
                    if self.state.storage.home_mode == HomeMode::Separate {
                        self.state.storage.format_home = !self.state.storage.format_home;
                    } else {
                        println!("Home is not set to separate.");
                    }
                }
                "4" => {
                    if self.state.storage.swap_mode == SwapMode::Partition {
                        self.state.storage.format_swap = !self.state.storage.format_swap;
                    } else {
                        println!("Swap mode is not a partition.");
                    }
                }
                _ => println!("Invalid input."),
            }
        }
    }

    fn pick_manual_role(
        &self,
        title: &str,
        guidance: &str,
    ) -> Result<Option<ManualPartitionRole>, String> {
        let roles = storage_plan::valid_manual_roles(&self.state.storage);
        println!();
        println!("{}", title);
        println!("{}", guidance);
        for (idx, role) in roles.iter().enumerate() {
            println!("{}) {}", idx + 1, role.label());
        }
        println!("Type a number, or `back` to cancel.");

        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(None),
                other => {
                    let choice = other.parse::<usize>().ok().unwrap_or(0);
                    if choice == 0 || choice > roles.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    return Ok(Some(roles[choice - 1]));
                }
            }
        }
    }

    fn choose_disk_from_list(
        &self,
        title: &str,
        disks: &[String],
    ) -> Result<Option<String>, String> {
        println!();
        println!("{}", title);
        for (idx, disk) in disks.iter().enumerate() {
            println!("{}) {}", idx + 1, disk);
        }
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(None),
                other => {
                    let choice = other.parse::<usize>().ok().unwrap_or(0);
                    if choice == 0 || choice > disks.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    return Ok(Some(disks[choice - 1].clone()));
                }
            }
        }
    }

    fn prompt_manual_partition_size(
        &self,
        role: ManualPartitionRole,
    ) -> Result<Option<(Option<u64>, bool)>, String> {
        match role {
            ManualPartitionRole::Root => {
                println!();
                println!("Root size");
                println!("1) Use remaining free space");
                println!("2) Enter size in GiB");
                println!("Type a number, or `back` to cancel.");
                loop {
                    match prompt("> ")?.as_str() {
                        "back" => return Ok(None),
                        "1" => return Ok(Some((None, true))),
                        "2" => {
                            let size = prompt("Size in GiB: ")?;
                            let gib = size.parse::<u64>().unwrap_or(0);
                            if gib == 0 {
                                println!("Size must be greater than zero.");
                                continue;
                            }
                            return Ok(Some((Some(gib), false)));
                        }
                        _ => println!("Invalid input."),
                    }
                }
            }
            _ => loop {
                let size = prompt(&format!("{} size in GiB: ", role.label()))?;
                if size == "back" {
                    return Ok(None);
                }
                let gib = size.parse::<u64>().unwrap_or(0);
                if gib == 0 {
                    println!("Size must be greater than zero.");
                    continue;
                }
                return Ok(Some((Some(gib), false)));
            },
        }
    }

    fn manual_partition_entries(&self) -> Result<Vec<ManualPartitionEntry>, String> {
        let managed_disks = storage_plan::managed_disks(&self.state.storage);
        if managed_disks.is_empty() {
            return Ok(Vec::new());
        }

        let mut entries: Vec<ManualPartitionEntry> = disk_manager::get_partitions_for_managed_disks(
            &managed_disks,
        )?
        .into_iter()
        .filter(|partition| {
            !self
                .state
                .storage
                .manual_delete_partitions
                .iter()
                .any(|path| path == &partition.path)
        })
        .map(|partition| ManualPartitionEntry {
            path: partition.path,
            parent_disk: partition.parent_disk,
            partition_number: partition.partition_number,
            size_label: disk_manager::human_gib_label(partition.size_bytes),
            fstype: partition.fstype.unwrap_or_default(),
            status_label: "existing".to_string(),
            pending_create: false,
        })
        .collect();

        for action in &self.state.storage.manual_create_actions {
            entries.push(ManualPartitionEntry {
                path: action.path.clone(),
                parent_disk: action.disk.clone(),
                partition_number: action.partition_number,
                size_label: if action.use_remaining {
                    "remaining space".to_string()
                } else {
                    format!("{} GiB", action.size_gib.unwrap_or(0))
                },
                fstype: action
                    .role
                    .default_fs(&self.root_fs_type())
                    .label()
                    .to_string(),
                status_label: format!("pending {}", action.role.label()),
                pending_create: true,
            });
        }

        entries.sort_by(|left, right| {
            left.parent_disk
                .cmp(&right.parent_disk)
                .then(left.partition_number.cmp(&right.partition_number))
        });
        Ok(entries)
    }

    fn manual_role_target_disks(&self, role: ManualPartitionRole) -> Vec<String> {
        match role {
            ManualPartitionRole::Home => storage_plan::managed_disks(&self.state.storage),
            _ => {
                if self.state.storage.install_disk.is_empty() {
                    Vec::new()
                } else {
                    vec![self.state.storage.install_disk.clone()]
                }
            }
        }
    }

    fn manual_role_label(&self, role: ManualPartitionRole) -> &str {
        match role {
            ManualPartitionRole::Efi => self
                .state
                .storage
                .manual_efi_partition
                .as_deref()
                .unwrap_or("(not set)"),
            ManualPartitionRole::Root => self
                .state
                .storage
                .manual_root_partition
                .as_deref()
                .unwrap_or("(not set)"),
            ManualPartitionRole::Home => self
                .state
                .storage
                .manual_home_partition
                .as_deref()
                .unwrap_or("(not set)"),
            ManualPartitionRole::Swap => self
                .state
                .storage
                .manual_swap_partition
                .as_deref()
                .unwrap_or("(not set)"),
        }
    }

    fn set_manual_role_assignment(&mut self, role: ManualPartitionRole, path: Option<String>) {
        match role {
            ManualPartitionRole::Efi => self.state.storage.manual_efi_partition = path,
            ManualPartitionRole::Root => self.state.storage.manual_root_partition = path,
            ManualPartitionRole::Home => self.state.storage.manual_home_partition = path,
            ManualPartitionRole::Swap => self.state.storage.manual_swap_partition = path,
        }
    }

    fn clear_manual_role_state(&mut self, role: ManualPartitionRole) {
        self.state
            .storage
            .manual_create_actions
            .retain(|action| action.role != role);
        self.set_manual_role_assignment(role, None);
    }

    fn clear_assignments_for_path(&mut self, path: &str) {
        if self.state.storage.manual_efi_partition.as_deref() == Some(path) {
            self.state.storage.manual_efi_partition = None;
        }
        if self.state.storage.manual_root_partition.as_deref() == Some(path) {
            self.state.storage.manual_root_partition = None;
        }
        if self.state.storage.manual_home_partition.as_deref() == Some(path) {
            self.state.storage.manual_home_partition = None;
        }
        if self.state.storage.manual_swap_partition.as_deref() == Some(path) {
            self.state.storage.manual_swap_partition = None;
        }
    }

    fn normalize_manual_storage_state(&mut self) {
        let valid_roles = storage_plan::valid_manual_roles(&self.state.storage);
        self.state
            .storage
            .manual_create_actions
            .retain(|action| valid_roles.iter().any(|role| role == &action.role));
        if self.state.storage.home_mode != HomeMode::Separate {
            self.state.storage.home_location = HomeLocation::SameDisk;
            self.state.storage.home_disk = None;
            self.state.storage.manual_home_partition = None;
        }
        if self.state.storage.home_location != HomeLocation::OtherDisk {
            self.state.storage.home_disk = None;
        }
        if self.state.storage.swap_mode != SwapMode::Partition {
            self.state.storage.manual_swap_partition = None;
        }
        storage_plan::clear_deleted_partition_assignments(&mut self.state.storage);
    }

    fn home_location_label(&self) -> &str {
        if self.state.storage.home_mode != HomeMode::Separate {
            "n/a"
        } else {
            match self.state.storage.home_location {
                HomeLocation::SameDisk => "same disk",
                HomeLocation::OtherDisk => "other disk",
            }
        }
    }

    fn root_fs_type(&self) -> storage_plan::FilesystemType {
        if self.state.storage.fs_type == "ext4" {
            storage_plan::FilesystemType::Ext4
        } else {
            storage_plan::FilesystemType::Xfs
        }
    }

    fn clear_screen_once(&mut self) -> Result<(), String> {
        if self.cleared_screen {
            return Ok(());
        }
        print!("\x1B[2J\x1B[H");
        io::stdout().flush().map_err(|e| e.to_string())?;
        self.cleared_screen = true;
        Ok(())
    }

    fn refresh_network_status(&mut self) {
        self.network_status = NetworkStatus::Checking;
        self.network_status = if network::check_connectivity() {
            NetworkStatus::Connected
        } else {
            NetworkStatus::NotConnected
        };
    }

    fn network_status_label(&self) -> &'static str {
        match self.network_status {
            NetworkStatus::Checking => "checking...",
            NetworkStatus::Connected => "connected",
            NetworkStatus::NotConnected => "not connected",
        }
    }

    fn wifi_connect_flow(&mut self) -> Result<(), String> {
        println!("Scanning...");
        let ssids = network::scan_wifi();
        if ssids.is_empty() {
            return Err("No Wi-Fi networks found.".to_string());
        }

        for (idx, ssid) in ssids.iter().enumerate() {
            println!("{}) {}", idx + 1, ssid);
        }
        println!("Type a number to connect, or `back` to cancel.");

        let selected = loop {
            let input = prompt("> ")?;
            match input.as_str() {
                "back" => return Ok(()),
                other => {
                    let choice = other.parse::<usize>().ok().unwrap_or(0);
                    if choice == 0 || choice > ssids.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    break ssids[choice - 1].clone();
                }
            }
        };

        println!("Connecting to: {}", selected);
        let password = prompt_password("Wi-Fi password (leave blank for open): ")?;
        network::connect_wifi(&selected, &password)?;
        self.refresh_network_status();
        println!("Connected.");
        Ok(())
    }

    fn pick_from_list(
        &mut self,
        title: &str,
        items: &[&str],
    ) -> Result<Option<String>, String> {
        println!();
        println!("{}", title);
        for (idx, item) in items.iter().enumerate() {
            println!("{}) {}", idx + 1, item);
        }
        println!("Type a number, or `back` to cancel.");
        loop {
            match prompt("> ")?.as_str() {
                "back" => return Ok(None),
                other => {
                    let choice = other.parse::<usize>().ok().unwrap_or(0);
                    if choice == 0 || choice > items.len() {
                        println!("Invalid selection.");
                        continue;
                    }
                    return Ok(Some(items[choice - 1].to_string()));
                }
            }
        }
    }
}

#[derive(Clone)]
struct ManualPartitionEntry {
    path: String,
    parent_disk: String,
    partition_number: u8,
    size_label: String,
    fstype: String,
    status_label: String,
    pending_create: bool,
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

fn prompt(prompt: &str) -> Result<String, String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    Ok(input.trim().to_lowercase())
}

fn prompt_password(prompt: &str) -> Result<String, String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let guard = SttyEchoGuard::disable();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    drop(guard);

    // Ensure we move to the next line after hidden input.
    println!();

    Ok(input.trim_end().to_string())
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "Yes"
    } else {
        "No"
    }
}

fn empty_as<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn desktop_env_label(value: Option<&DesktopEnv>) -> &'static str {
    match value {
        Some(env) => env.label(),
        None => "(not selected)",
    }
}

fn is_root() -> bool {
    Command::new("id")
        .args(["-u"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .is_some_and(|s| s.trim() == "0")
}

fn system_ram_gib() -> u64 {
    // Avoid pulling in extra deps here; use /proc/meminfo.
    let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") else {
        return 0;
    };
    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb = rest.trim().split_whitespace().next().unwrap_or("0");
            let kb = kb.parse::<u64>().unwrap_or(0);
            return kb / 1024 / 1024;
        }
    }
    0
}

struct SttyEchoGuard(bool);

impl SttyEchoGuard {
    fn disable() -> Self {
        let enabled = Command::new("stty")
            .arg("-echo")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        Self(enabled)
    }
}

impl Drop for SttyEchoGuard {
    fn drop(&mut self) {
        if !self.0 {
            return;
        }
        let _ = Command::new("stty").arg("echo").status();
    }
}
