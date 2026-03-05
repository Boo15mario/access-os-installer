use installer_core::backend::config_engine::{DesktopEnv, KernelVariant};
use installer_core::backend::disk_manager::{self};
use installer_core::backend::storage_plan::{
    self, HomeLocation, HomeMode, ResolvedInstallLayout, SetupMode, StorageSelection, SwapMode,
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
    done: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Welcome,
    InstallOptions,
    DiskSelection,
    DiskSetup,
    Regional,
    UserSettings,
    Review,
    Install,
    Complete,
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
                    format_efi: true,
                    format_root: true,
                    format_home: true,
                    format_swap: true,
                    removable_media: false,
                },
            },
            cached_layout: None,
            cached_disk_gib: None,
            done: false,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        while !self.done {
            match self.step {
                Step::Welcome => self.step_welcome()?,
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
        println!();
        println!("Access OS Installer (CLI)");
        println!("Type `next` to begin, or `quit` to exit.");
        println!("If you need Wi-Fi, type `wifi`.");
        println!("Commands always available: next, back, help, quit");
        println!();

        loop {
            let input = prompt("> ")?;
            match input.as_str() {
                "help" => {
                    println!("This is a line-based installer. Type `next` to move forward, `back` to return to the previous screen.");
                }
                "wifi" => {
                    if let Err(err) = wifi_flow() {
                        println!("ERROR: {}", err);
                    }
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
                    if !network::check_connectivity() {
                        println!("ERROR: No internet connectivity detected.");
                        continue;
                    }
                    self.step = Step::InstallOptions;
                    return Ok(());
                }
                other => {
                    println!("Unknown command: {}", other);
                }
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
            if self.state.storage.home_mode == HomeMode::Separate
                && self.state.storage.setup_mode == SetupMode::Automatic
            {
                println!(
                    "6) Home disk (auto): {}",
                    self.state
                        .storage
                        .home_disk
                        .as_deref()
                        .unwrap_or("(not selected)")
                );
            }
            if self.state.storage.setup_mode == SetupMode::Manual {
                println!("6) Manual partitions + format flags");
            }
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
                "6" => {
                    if self.state.storage.setup_mode == SetupMode::Manual {
                        self.edit_manual_partitions()?;
                    } else if self.state.storage.home_mode == HomeMode::Separate {
                        self.select_home_disk_auto()?;
                    } else {
                        println!("Nothing to edit for option 6 in the current configuration.");
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

        println!("1/7 Partitioning and mounting...");
        mount::prepare_install_targets(&layout)?;

        println!("2/7 Staging system config...");
        installer_core::backend::install_worker::stage_system_config_repo(
            installer_core::constants::DOTFILES_REPO_URL,
        )?;
        installer_core::backend::install_worker::overlay_staged_config_to_target()?;

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
        installer_core::backend::install_worker::run_pacstrap(&config)?;

        println!("5/7 Setting up swap (if configured)...");
        installer_core::backend::disk_manager::setup_swap_file(&layout)?;

        println!("6/7 Generating fstab...");
        installer_core::backend::install_worker::generate_fstab()?;

        println!("7/7 Configuring system...");
        installer_core::backend::install_worker::configure_system(&config, &root_partition)?;

        if de == DesktopEnv::Gnome {
            println!("Extra: configuring GNOME (non-fatal)...");
            if let Err(err) = installer_core::backend::install_worker::configure_gnome(&config.username) {
                println!("WARN: GNOME config failed (non-fatal): {}", err);
            }
        }

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
                    self.state.storage.manual_home_partition = None;
                    self.state.storage.home_disk = None;
                    return Ok(());
                }
                "2" => {
                    self.state.storage.home_mode = HomeMode::Separate;
                    return Ok(());
                }
                _ => println!("Invalid input."),
            }
        }
    }

    fn select_home_disk_auto(&mut self) -> Result<(), String> {
        let devices = disk_manager::get_internal_block_devices()
            .map_err(|e| format!("Failed to list internal drives: {}", e))?;
        println!();
        println!("Select home disk (automatic mode)");
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
                    self.state.storage.home_location = HomeLocation::OtherDisk;
                    self.state.storage.home_disk = Some(format!("/dev/{}", devices[choice - 1].name));
                    return Ok(());
                }
            }
        }
    }

    fn edit_manual_partitions(&mut self) -> Result<(), String> {
        loop {
            println!();
            println!("Manual partitions");
            println!(
                "1) EFI partition: {}",
                self.state
                    .storage
                    .manual_efi_partition
                    .as_deref()
                    .unwrap_or("(not set)")
            );
            println!(
                "2) Root partition: {}",
                self.state
                    .storage
                    .manual_root_partition
                    .as_deref()
                    .unwrap_or("(not set)")
            );
            if self.state.storage.home_mode == HomeMode::Separate {
                println!(
                    "3) /home partition: {}",
                    self.state
                        .storage
                        .manual_home_partition
                        .as_deref()
                        .unwrap_or("(not set)")
                );
            }
            if self.state.storage.swap_mode == SwapMode::Partition {
                println!(
                    "4) swap partition: {}",
                    self.state
                        .storage
                        .manual_swap_partition
                        .as_deref()
                        .unwrap_or("(not set)")
                );
            }
            println!("5) Format EFI: {}", yes_no(self.state.storage.format_efi));
            println!("6) Format root: {}", yes_no(self.state.storage.format_root));
            if self.state.storage.home_mode == HomeMode::Separate {
                println!("7) Format /home: {}", yes_no(self.state.storage.format_home));
            }
            if self.state.storage.swap_mode == SwapMode::Partition {
                println!("8) Format swap: {}", yes_no(self.state.storage.format_swap));
            }
            println!();
            println!("Type a number to edit/toggle, or `back` to return.");

            match prompt("> ")?.as_str() {
                "back" => return Ok(()),
                "1" => self.state.storage.manual_efi_partition = pick_partition_path("EFI")?,
                "2" => self.state.storage.manual_root_partition = pick_partition_path("root")?,
                "3" => {
                    if self.state.storage.home_mode == HomeMode::Separate {
                        self.state.storage.manual_home_partition = pick_partition_path("/home")?;
                    } else {
                        println!("Home is not set to separate.");
                    }
                }
                "4" => {
                    if self.state.storage.swap_mode == SwapMode::Partition {
                        self.state.storage.manual_swap_partition = pick_partition_path("swap")?;
                    } else {
                        println!("Swap mode is not a partition.");
                    }
                }
                "5" => self.state.storage.format_efi = !self.state.storage.format_efi,
                "6" => self.state.storage.format_root = !self.state.storage.format_root,
                "7" => {
                    if self.state.storage.home_mode == HomeMode::Separate {
                        self.state.storage.format_home = !self.state.storage.format_home
                    }
                }
                "8" => {
                    if self.state.storage.swap_mode == SwapMode::Partition {
                        self.state.storage.format_swap = !self.state.storage.format_swap
                    }
                }
                other => println!("Unknown input: {}", other),
            }
        }
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

fn pick_partition_path(label: &str) -> Result<Option<String>, String> {
    let partitions = disk_manager::get_partition_devices()?;
    if partitions.is_empty() {
        println!("No partitions detected.");
        return Ok(None);
    }

    println!();
    println!("Select partition for {}", label);
    for (idx, part) in partitions.iter().enumerate() {
        println!(
            "{}) {} ({}) {}",
            idx + 1,
            part.path,
            disk_manager::human_gib_label(part.size_bytes),
            part.fstype.as_deref().unwrap_or("")
        );
    }
    println!("Type a number, or `back` to cancel.");
    loop {
        match prompt("> ")?.as_str() {
            "back" => return Ok(None),
            other => {
                let choice = other.parse::<usize>().ok().unwrap_or(0);
                if choice == 0 || choice > partitions.len() {
                    println!("Invalid selection.");
                    continue;
                }
                return Ok(Some(partitions[choice - 1].path.clone()));
            }
        }
    }
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

fn wifi_flow() -> Result<(), String> {
    println!();
    println!("Wi-Fi");
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
    println!("Connected.");
    Ok(())
}
