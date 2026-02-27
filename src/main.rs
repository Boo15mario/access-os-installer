mod backend;

use backend::network;
use backend::preflight::{self, CheckResult, CheckStatus, PreflightContext};
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box, Button, CheckButton, DropDown, Entry, Label,
    Orientation, PasswordEntry, Stack, StackTransitionType, StringList, StringObject,
};
use std::cell::RefCell;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use sysinfo::System;

const APP_ID: &str = "org.accessos.Installer";
const DRIVE_CONFIRMATION_TEXT: &str = "ERASE";

#[derive(Clone)]
struct DriveOption {
    path: String,
    disk_gib: u64,
}

struct AppState {
    drive: String,
    selected_disk_gib: Option<u64>,
    repo_url: String,
    temp_repo_path: String,
    selected_host: String,
    username: String,
    password: String,
    timezone: String,
    locale: String,
    keymap: String,
    preflight_results: Vec<CheckResult>,
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn padded_box(spacing: i32, margin: i32) -> Box {
    Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(spacing)
        .margin_top(margin)
        .margin_bottom(margin)
        .margin_start(margin)
        .margin_end(margin)
        .build()
}

fn system_ram_gib() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.total_memory() / (1024 * 1024 * 1024)
}

fn preflight_context_from_state(state: &AppState) -> PreflightContext {
    PreflightContext {
        is_uefi: Path::new("/sys/firmware/efi").exists(),
        has_disk: !state.drive.is_empty(),
        online: network::check_connectivity(),
        ram_gib: system_ram_gib(),
        disk_gib: state.selected_disk_gib,
    }
}

fn format_check_group(results: &[CheckResult], hard_only: bool) -> String {
    let mut lines = Vec::new();

    for check in results.iter().filter(|result| result.is_hard == hard_only) {
        lines.push(format!(
            "[{}] {}: {}",
            check.status.label(),
            check.label,
            check.message
        ));
    }

    if lines.is_empty() {
        "No checks in this group.".to_string()
    } else {
        lines.join("\n")
    }
}

fn format_review_summary(state: &AppState) -> String {
    let target_disk = if state.drive.is_empty() {
        "(not selected)".to_string()
    } else if let Some(gib) = state.selected_disk_gib {
        format!("{} ({} GiB)", state.drive, gib)
    } else {
        state.drive.clone()
    };

    let repo = if state.repo_url.is_empty() {
        "(not set)"
    } else {
        &state.repo_url
    };
    let host = if state.selected_host.is_empty() {
        "(not selected)"
    } else {
        &state.selected_host
    };
    let username = if state.username.is_empty() {
        "(not set)"
    } else {
        &state.username
    };

    format!(
        "Target disk: {}\nRepository: {}\nHost: {}\nUsername: {}\nTimezone: {}\nLocale: {}",
        target_disk, repo, host, username, state.timezone, state.locale
    )
}

fn format_warning_lines(results: &[CheckResult]) -> String {
    let warnings: Vec<String> = results
        .iter()
        .filter(|result| result.status == CheckStatus::Warn)
        .map(|result| format!("- {}", result.message))
        .collect();

    if warnings.is_empty() {
        "No warnings detected.".to_string()
    } else {
        warnings.join("\n")
    }
}

fn unmount_install_targets() -> Result<(), String> {
    let mut errors = Vec::new();

    for mount_point in ["/mnt/boot", "/mnt"] {
        let output = Command::new("umount")
            .arg(mount_point)
            .output()
            .map_err(|e| format!("Failed to execute umount for {}: {}", mount_point, e))?;

        if output.status.success() {
            continue;
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("not mounted") || stderr.contains("no mount point specified") {
            continue;
        }

        let detail = if stderr.is_empty() {
            format!("umount returned status {}", output.status)
        } else {
            stderr
        };
        errors.push(format!("{}: {}", mount_point, detail));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn reboot_system() -> Result<(), String> {
    let output = Command::new("reboot")
        .output()
        .map_err(|e| format!("Failed to execute reboot: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("Reboot command failed with status {}", output.status))
        } else {
            Err(stderr)
        }
    }
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(AppState {
        drive: String::new(),
        selected_disk_gib: None,
        repo_url: String::new(),
        temp_repo_path: String::new(),
        selected_host: String::new(),
        username: String::new(),
        password: String::new(),
        timezone: "America/Chicago".to_string(),
        locale: "en_US.UTF-8".to_string(),
        keymap: "us".to_string(),
        preflight_results: Vec::new(),
    }));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("access-OS Installer")
        .default_width(600)
        .default_height(500)
        .build();

    let stack = Stack::builder()
        .transition_type(StackTransitionType::SlideLeftRight)
        .transition_duration(500)
        .build();
    let host_list = StringList::new(&[]);

    let step_welcome = build_welcome_step(&stack);
    stack.add_titled(&step_welcome, Some("welcome"), "Welcome");

    let wifi_ssid_list = StringList::new(&[]);
    let step_wifi = build_wifi_step(&stack, wifi_ssid_list);
    stack.add_titled(&step_wifi, Some("wifi"), "Wi-Fi Setup");

    let step_disk = build_step1(&stack, state.clone());
    stack.add_titled(&step_disk, Some("disk"), "Disk Selection");

    let step_repo = build_step2(&stack, state.clone(), host_list.clone());
    stack.add_titled(&step_repo, Some("repo"), "Repo Selection");

    let step_host = build_step3(&stack, state.clone(), host_list);
    stack.add_titled(&step_host, Some("host"), "Host Selection");

    let (step_preflight, refresh_preflight) = build_preflight_step(&stack, state.clone());
    stack.add_titled(&step_preflight, Some("preflight"), "Preflight");

    let (step_review, refresh_review) = build_review_step(&stack, state.clone());
    stack.add_titled(&step_review, Some("review"), "Review");

    let step_install = build_step4(&stack, state.clone());
    stack.add_titled(&step_install, Some("install"), "Installing");

    let step_complete = build_step5(&window);
    stack.add_titled(&step_complete, Some("complete"), "Complete");

    {
        let refresh_preflight = refresh_preflight.clone();
        let refresh_review = refresh_review.clone();
        stack.connect_visible_child_name_notify(move |stack| {
            if let Some(name) = stack.visible_child_name() {
                match name.as_str() {
                    "preflight" => refresh_preflight(),
                    "review" => refresh_review(),
                    _ => {}
                }
            }
        });
    }

    window.set_child(Some(&stack));
    stack.set_visible_child_name("welcome");
    window.fullscreen();
    window.present();
}

fn build_welcome_step(stack: &Stack) -> Box {
    let vbox = padded_box(16, 48);
    vbox.set_halign(Align::Center);
    vbox.set_valign(Align::Center);

    let title = Label::builder()
        .label("Welcome to access-OS Installer")
        .margin_bottom(12)
        .build();
    title.set_markup("<span font='28' weight='bold'>Welcome to access-OS Installer</span>");

    let subtitle = Label::builder()
        .label("This installer will guide you through setting up access-OS on your machine.")
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .margin_bottom(24)
        .build();

    let start_btn = Button::builder().label("Get Started").margin_top(16).build();

    {
        let stack = stack.clone();
        start_btn.connect_clicked(move |_| {
            if network::check_connectivity() {
                stack.set_visible_child_name("disk");
            } else {
                stack.set_visible_child_name("wifi");
            }
        });
    }

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&start_btn);
    vbox
}

fn build_wifi_step(stack: &Stack, ssid_list: StringList) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder().label("Wi-Fi Setup").margin_bottom(8).build();
    let subtitle = Label::builder()
        .label("No internet connection detected. Connect to Wi-Fi to continue.")
        .wrap(true)
        .margin_bottom(16)
        .build();

    for ssid in network::scan_wifi() {
        ssid_list.append(&ssid);
    }

    let ssid_dropdown = DropDown::builder().model(&ssid_list).build();
    let pass_entry = PasswordEntry::builder()
        .placeholder_text("Wi-Fi Password")
        .show_peek_icon(true)
        .build();
    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let connect_btn = Button::builder().label("Connect").build();
    let refresh_btn = Button::builder().label("Refresh Networks").build();
    let skip_btn = Button::builder().label("Skip (already connected)").build();

    {
        let ssid_dropdown = ssid_dropdown.clone();
        let pass_entry = pass_entry.clone();
        let status_label = status_label.clone();
        let stack = stack.clone();
        connect_btn.connect_clicked(move |_| {
            let ssid = match ssid_dropdown.selected_item() {
                Some(obj) => obj.downcast::<StringObject>().unwrap().string().to_string(),
                None => {
                    status_label.set_label("Please select a network.");
                    return;
                }
            };

            let password = pass_entry.text().to_string();
            status_label.set_label("Connecting...");
            match network::connect_wifi(&ssid, &password) {
                Ok(_) => {
                    if network::check_connectivity() {
                        stack.set_visible_child_name("disk");
                    } else {
                        status_label
                            .set_label("Connected to Wi-Fi but no internet. Check password or network.");
                    }
                }
                Err(e) => status_label.set_label(&format!("Failed: {}", e)),
            }
        });
    }

    {
        let ssid_list = ssid_list.clone();
        let status_label = status_label.clone();
        refresh_btn.connect_clicked(move |_| {
            status_label.set_label("Scanning...");
            while ssid_list.n_items() > 0 {
                ssid_list.remove(0);
            }
            for ssid in network::scan_wifi() {
                ssid_list.append(&ssid);
            }
            status_label.set_label("Scan complete.");
        });
    }

    {
        let stack = stack.clone();
        skip_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("disk");
        });
    }

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&Label::new(Some("Available Networks")));
    vbox.append(&ssid_dropdown);
    vbox.append(&Label::new(Some("Password")));
    vbox.append(&pass_entry);
    vbox.append(&status_label);
    vbox.append(&connect_btn);
    vbox.append(&refresh_btn);
    vbox.append(&skip_btn);
    vbox
}

fn build_step1(stack: &Stack, state: Rc<RefCell<AppState>>) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 1: Select Target Drive")
        .margin_bottom(24)
        .build();

    let drive_list = StringList::new(&[]);
    let drive_dropdown = DropDown::builder().model(&drive_list).build();
    let confirm_entry = Entry::builder()
        .placeholder_text("Type ERASE to allow disk wipe")
        .build();
    let warning = Label::builder()
        .label("Warning: Installing will erase all data on the selected disk.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let next_btn = Button::builder().label("Next: Select Repository").build();
    next_btn.set_sensitive(false);

    let mut drive_options_data: Vec<DriveOption> = Vec::new();
    match backend::disk_manager::get_internal_block_devices() {
        Ok(devices) if devices.is_empty() => {
            status_label.set_label(
                "No internal drives detected. Add an internal drive and restart the installer.",
            );
        }
        Ok(devices) => {
            for device in devices {
                let path = format!("/dev/{}", device.name);
                let model = device
                    .model
                    .unwrap_or_else(|| "Unknown model".to_string())
                    .trim()
                    .to_string();
                let transport = device.tran.unwrap_or_else(|| "internal".to_string());
                let size_label = backend::disk_manager::human_gib_label(device.size_bytes);
                let disk_gib = backend::disk_manager::bytes_to_gib(device.size_bytes);
                drive_list.append(&format!(
                    "{} | {} | {} | {}",
                    path, size_label, model, transport
                ));
                drive_options_data.push(DriveOption { path, disk_gib });
            }
            status_label.set_label("Select the internal drive to install access-OS.");
        }
        Err(e) => {
            status_label.set_label(&format!("Failed to read internal drives: {}", e));
        }
    }
    let drive_options = Rc::new(drive_options_data);

    {
        let drive_dropdown = drive_dropdown.clone();
        let next_btn = next_btn.clone();
        confirm_entry.connect_changed(move |entry| {
            let has_selected_drive = drive_dropdown.selected_item().is_some();
            let is_confirmed = entry.text().as_str() == DRIVE_CONFIRMATION_TEXT;
            next_btn.set_sensitive(has_selected_drive && is_confirmed);
        });
    }

    {
        let confirm_entry = confirm_entry.clone();
        let next_btn = next_btn.clone();
        drive_dropdown.connect_selected_notify(move |dropdown| {
            let has_selected_drive = dropdown.selected_item().is_some();
            let is_confirmed = confirm_entry.text().as_str() == DRIVE_CONFIRMATION_TEXT;
            next_btn.set_sensitive(has_selected_drive && is_confirmed);
        });
    }

    {
        let drive_dropdown = drive_dropdown.clone();
        let drive_options = drive_options.clone();
        let status_label = status_label.clone();
        let state = state.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let selected_index = drive_dropdown.selected();
            if selected_index == gtk4::INVALID_LIST_POSITION {
                status_label.set_label("Select a target drive.");
                return;
            }

            let Some(option) = drive_options.get(selected_index as usize) else {
                status_label.set_label("Invalid drive selection.");
                return;
            };

            let mut app_state = state.borrow_mut();
            app_state.drive = option.path.clone();
            app_state.selected_disk_gib = Some(option.disk_gib);
            stack.set_visible_child_name("repo");
        });
    }

    vbox.append(&title);
    vbox.append(&Label::new(Some("Target Internal Drive")));
    vbox.append(&drive_dropdown);
    vbox.append(&warning);
    vbox.append(&Label::new(Some("Destructive Action Confirmation")));
    vbox.append(&confirm_entry);
    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox
}

fn build_step2(stack: &Stack, state: Rc<RefCell<AppState>>, host_list: StringList) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 2: Configuration Repository")
        .margin_bottom(24)
        .build();
    let repo_entry = Entry::builder()
        .placeholder_text("https://github.com/user/nix-config")
        .build();
    let next_btn = Button::builder().label("Next: Scan Configuration").build();
    let back_btn = Button::builder().label("Back").build();

    {
        let repo_entry = repo_entry.clone();
        let state = state.clone();
        let host_list = host_list.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let url = repo_entry.text().to_string();
            state.borrow_mut().repo_url = url.clone();
            if let Ok(path) = backend::config_engine::clone_repo_to_temp(&url) {
                state.borrow_mut().temp_repo_path = path.clone();
                let hosts = backend::config_engine::list_hosts(&path);
                while host_list.n_items() > 0 {
                    host_list.remove(0);
                }
                for host in hosts {
                    host_list.append(&host);
                }
                stack.set_visible_child_name("host");
            }
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("disk"));
    }

    vbox.append(&title);
    vbox.append(&repo_entry);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}

fn build_step3(stack: &Stack, state: Rc<RefCell<AppState>>, host_list: StringList) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 3: Host & User Settings")
        .margin_bottom(24)
        .build();
    let host_dropdown = DropDown::builder().model(&host_list).build();
    let user_entry = Entry::builder().placeholder_text("Username").build();
    let pass_entry = PasswordEntry::builder().placeholder_text("Password").build();

    let tz_entry = Entry::builder().text("America/Chicago").build();
    let locale_entry = Entry::builder().text("en_US.UTF-8").build();

    let next_btn = Button::builder().label("Next: Run Preflight Checks").build();
    let back_btn = Button::builder().label("Back").build();

    {
        let state = state.clone();
        let user_entry = user_entry.clone();
        let pass_entry = pass_entry.clone();
        let tz_entry = tz_entry.clone();
        let locale_entry = locale_entry.clone();
        let host_dropdown = host_dropdown.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let mut s = state.borrow_mut();
            s.username = user_entry.text().to_string();
            s.password = pass_entry.text().to_string();
            s.timezone = tz_entry.text().to_string();
            s.locale = locale_entry.text().to_string();
            s.selected_host = match host_dropdown.selected_item() {
                Some(obj) => obj.downcast::<StringObject>().unwrap().string().to_string(),
                None => String::new(),
            };
            stack.set_visible_child_name("preflight");
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("repo"));
    }

    vbox.append(&title);
    vbox.append(&Label::new(Some("Select Host Template")));
    vbox.append(&host_dropdown);
    vbox.append(&Label::new(Some("Username")));
    vbox.append(&user_entry);
    vbox.append(&Label::new(Some("Password")));
    vbox.append(&pass_entry);
    vbox.append(&Label::new(Some("Timezone")));
    vbox.append(&tz_entry);
    vbox.append(&Label::new(Some("Locale")));
    vbox.append(&locale_entry);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}

fn build_preflight_step(stack: &Stack, state: Rc<RefCell<AppState>>) -> (Box, Rc<dyn Fn()>) {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 4: Preflight Checks")
        .margin_bottom(12)
        .build();
    let subtitle = Label::builder()
        .label("Hard blockers must pass before you can continue.")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let hard_checks_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let warning_checks_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let rerun_btn = Button::builder().label("Re-run Checks").build();
    let continue_btn = Button::builder().label("Next: Review & Confirm").build();
    continue_btn.set_sensitive(false);
    let back_btn = Button::builder().label("Back").build();

    let refresh_preflight: Rc<dyn Fn()> = {
        let state = state.clone();
        let hard_checks_label = hard_checks_label.clone();
        let warning_checks_label = warning_checks_label.clone();
        let status_label = status_label.clone();
        let continue_btn = continue_btn.clone();
        Rc::new(move || {
            let context = {
                let app_state = state.borrow();
                preflight_context_from_state(&app_state)
            };

            let results = preflight::evaluate_checks(&context);
            let hard_fail = preflight::has_hard_fail(&results);

            {
                let mut app_state = state.borrow_mut();
                app_state.preflight_results = results.clone();
            }

            hard_checks_label.set_label(&format_check_group(&results, true));
            warning_checks_label.set_label(&format_check_group(&results, false));

            if hard_fail {
                status_label.set_label("Preflight failed. Resolve hard blockers before continuing.");
                continue_btn.set_sensitive(false);
            } else {
                status_label.set_label("Preflight passed. Continue to review and confirm.");
                continue_btn.set_sensitive(true);
            }
        })
    };

    {
        let refresh_preflight = refresh_preflight.clone();
        rerun_btn.connect_clicked(move |_| refresh_preflight());
    }

    {
        let stack = stack.clone();
        continue_btn.connect_clicked(move |_| stack.set_visible_child_name("review"));
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("host"));
    }

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&Label::new(Some("Hard blockers")));
    vbox.append(&hard_checks_label);
    vbox.append(&Label::new(Some("Soft warnings")));
    vbox.append(&warning_checks_label);
    vbox.append(&status_label);
    vbox.append(&rerun_btn);
    vbox.append(&continue_btn);
    vbox.append(&back_btn);

    (vbox, refresh_preflight)
}

fn build_review_step(stack: &Stack, state: Rc<RefCell<AppState>>) -> (Box, Rc<dyn Fn()>) {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 5: Review & Confirm")
        .margin_bottom(12)
        .build();

    let summary_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let warnings_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let acknowledge_warning_btn = CheckButton::builder()
        .label("I understand these warnings and want to continue.")
        .build();
    acknowledge_warning_btn.set_visible(false);

    let continue_btn = Button::builder().label("Next: Start Installation").build();
    let back_btn = Button::builder().label("Back").build();

    {
        let state = state.clone();
        let continue_btn = continue_btn.clone();
        acknowledge_warning_btn.connect_toggled(move |checkbox| {
            let has_warnings = state
                .borrow()
                .preflight_results
                .iter()
                .any(|result| result.status == CheckStatus::Warn);
            continue_btn.set_sensitive(!has_warnings || checkbox.is_active());
        });
    }

    let refresh_review: Rc<dyn Fn()> = {
        let state = state.clone();
        let summary_label = summary_label.clone();
        let warnings_label = warnings_label.clone();
        let acknowledge_warning_btn = acknowledge_warning_btn.clone();
        let continue_btn = continue_btn.clone();
        Rc::new(move || {
            let app_state = state.borrow();
            summary_label.set_label(&format_review_summary(&app_state));
            warnings_label.set_label(&format_warning_lines(&app_state.preflight_results));

            let has_warnings = app_state
                .preflight_results
                .iter()
                .any(|result| result.status == CheckStatus::Warn);
            drop(app_state);

            if has_warnings {
                acknowledge_warning_btn.set_visible(true);
                acknowledge_warning_btn.set_active(false);
                continue_btn.set_sensitive(false);
            } else {
                acknowledge_warning_btn.set_visible(false);
                continue_btn.set_sensitive(true);
            }
        })
    };

    {
        let stack = stack.clone();
        continue_btn.connect_clicked(move |_| stack.set_visible_child_name("install"));
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("preflight"));
    }

    vbox.append(&title);
    vbox.append(&Label::new(Some("Selected Configuration")));
    vbox.append(&summary_label);
    vbox.append(&Label::new(Some("Warnings")));
    vbox.append(&warnings_label);
    vbox.append(&acknowledge_warning_btn);
    vbox.append(&continue_btn);
    vbox.append(&back_btn);

    (vbox, refresh_review)
}

fn build_step4(stack: &Stack, state: Rc<RefCell<AppState>>) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 6: Installation Progress")
        .margin_bottom(24)
        .build();
    let progress_label = Label::builder()
        .label("Ready to install...")
        .halign(Align::Start)
        .wrap(true)
        .build();
    let start_btn = Button::builder().label("Start Installation").build();

    {
        let stack = stack.clone();
        let progress_label = progress_label.clone();
        let state = state.clone();
        start_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);

            let s = state.borrow();
            progress_label.set_label("Partitioning and mounting...");
            if let Err(e) = backend::disk_manager::execute_partitioning(&s.drive, 8, "xfs") {
                progress_label.set_label(&format!("Error: {}", e));
                btn.set_sensitive(true);
                return;
            }

            let p3 = backend::disk_manager::partition_device_path(&s.drive, 3);
            let _ = Command::new("mount").args([&p3, "/mnt"]).output();
            let p1 = backend::disk_manager::partition_device_path(&s.drive, 1);
            let _ = Command::new("mkdir").args(["-p", "/mnt/boot"]).output();
            let _ = Command::new("mount").args([&p1, "/mnt/boot"]).output();

            progress_label.set_label("Cloning repository to target...");
            let _ = Command::new("mkdir").args(["-p", "/mnt/etc/nixos"]).output();
            let _ = Command::new("git")
                .args(["clone", &s.repo_url, "/mnt/etc/nixos"])
                .output();

            progress_label.set_label("Applying local settings...");
            let host_dir = format!("/mnt/etc/nixos/{}", s.selected_host);
            let settings = backend::config_engine::HostSettings {
                timezone: Some(s.timezone.clone()),
                locale: Some(s.locale.clone()),
                keymap: Some(s.keymap.clone()),
            };
            let _ = backend::config_engine::apply_local_settings(&host_dir, &settings);

            progress_label.set_label("Generating hardware configuration...");
            let _ = Command::new("nixos-generate-config")
                .args(["--root", "/mnt", "--dir", &host_dir])
                .output();

            progress_label.set_label("Running nixos-install...");
            match backend::install_worker::start_install(&s.selected_host, &s.username, &s.password) {
                Ok(_) => {
                    progress_label.set_label("Installation complete. Opening completion options...");
                    stack.set_visible_child_name("complete");
                }
                Err(e) => {
                    progress_label.set_label(&format!("Installation failed: {}", e));
                    btn.set_sensitive(true);
                }
            }
        });
    }

    vbox.append(&title);
    vbox.append(&progress_label);
    vbox.append(&start_btn);
    vbox
}

fn build_step5(window: &ApplicationWindow) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 7: Installation Complete")
        .margin_bottom(24)
        .build();
    let status_label = Label::builder()
        .label("Installation completed successfully. Choose Reboot or Exit Installer.")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let reboot_btn = Button::builder().label("Reboot").build();
    let exit_btn = Button::builder().label("Exit Installer").build();
    let force_reboot_btn = Button::builder().label("Force Reboot").build();
    let force_exit_btn = Button::builder().label("Force Exit").build();
    force_reboot_btn.set_visible(false);
    force_exit_btn.set_visible(false);

    {
        let status_label = status_label.clone();
        let force_reboot_btn = force_reboot_btn.clone();
        let force_exit_btn = force_exit_btn.clone();
        reboot_btn.connect_clicked(move |_| {
            status_label.set_label("Unmounting install targets...");
            match unmount_install_targets() {
                Ok(()) => match reboot_system() {
                    Ok(()) => status_label.set_label("Reboot command issued."),
                    Err(e) => status_label.set_label(&format!("Failed to reboot: {}", e)),
                },
                Err(e) => {
                    status_label.set_label(&format!(
                        "Unmount failed: {}. Choose Force Reboot or Force Exit.",
                        e
                    ));
                    force_reboot_btn.set_visible(true);
                    force_exit_btn.set_visible(true);
                }
            }
        });
    }

    {
        let status_label = status_label.clone();
        let force_reboot_btn = force_reboot_btn.clone();
        let force_exit_btn = force_exit_btn.clone();
        let window = window.clone();
        exit_btn.connect_clicked(move |_| {
            status_label.set_label("Unmounting install targets...");
            match unmount_install_targets() {
                Ok(()) => window.close(),
                Err(e) => {
                    status_label.set_label(&format!(
                        "Unmount failed: {}. Choose Force Reboot or Force Exit.",
                        e
                    ));
                    force_reboot_btn.set_visible(true);
                    force_exit_btn.set_visible(true);
                }
            }
        });
    }

    {
        let status_label = status_label.clone();
        force_reboot_btn.connect_clicked(move |_| match reboot_system() {
            Ok(()) => status_label.set_label("Force reboot command issued."),
            Err(e) => status_label.set_label(&format!("Failed to reboot: {}", e)),
        });
    }

    {
        let window = window.clone();
        force_exit_btn.connect_clicked(move |_| window.close());
    }

    vbox.append(&title);
    vbox.append(&status_label);
    vbox.append(&reboot_btn);
    vbox.append(&exit_btn);
    vbox.append(&force_reboot_btn);
    vbox.append(&force_exit_btn);
    vbox
}
