mod backend;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation, Align, Stack, StackTransitionType, DropDown, StringList, PasswordEntry, StringObject};
use std::rc::Rc;
use std::cell::RefCell;
use std::process::Command;
use backend::network;

const APP_ID: &str = "org.accessos.Installer";
const DRIVE_CONFIRMATION_TEXT: &str = "ERASE";

struct AppState {
    drive: String,
    repo_url: String,
    temp_repo_path: String,
    selected_host: String,
    username: String,
    password: String,
    timezone: String,
    locale: String,
    keymap: String,
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
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
        repo_url: String::new(),
        temp_repo_path: String::new(),
        selected_host: String::new(),
        username: String::new(),
        password: String::new(),
        timezone: "America/Chicago".to_string(), // Default
        locale: "en_US.UTF-8".to_string(),
        keymap: "us".to_string(),
    }));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("access-OS Installer")
        .default_width(600)
        .default_height(500)
        .build();

    let stack = Stack::builder().transition_type(StackTransitionType::SlideLeftRight).transition_duration(500).build();
    let host_list = StringList::new(&[]);

    let step_welcome = build_welcome_step(&stack);
    stack.add_titled(&step_welcome, Some("welcome"), "Welcome");

    let wifi_ssid_list = StringList::new(&[]);
    let step_wifi = build_wifi_step(&stack, wifi_ssid_list.clone());
    stack.add_titled(&step_wifi, Some("wifi"), "Wi-Fi Setup");

    let step1 = build_step1(&stack, state.clone());
    stack.add_titled(&step1, Some("disk"), "Disk Selection");

    let step2 = build_step2(&stack, state.clone(), host_list.clone());
    stack.add_titled(&step2, Some("repo"), "Repo Selection");

    let step3 = build_step3(&stack, state.clone(), host_list.clone());
    stack.add_titled(&step3, Some("host"), "Host Selection");

    let step4 = build_step4(&stack, state.clone());
    stack.add_titled(&step4, Some("install"), "Installing");

    let step5 = build_step5(&window);
    stack.add_titled(&step5, Some("complete"), "Complete");

    window.set_child(Some(&stack));
    stack.set_visible_child_name("welcome");

    window.fullscreen();
    window.present();
}

fn build_welcome_step(stack: &Stack) -> Box {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(48)
        .margin_end(48)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let title = Label::builder()
        .label("Welcome to access-OS Installer")
        .margin_bottom(12)
        .build();
    // Make title larger via markup
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
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = Label::builder().label("Wi-Fi Setup").margin_bottom(8).build();
    let subtitle = Label::builder()
        .label("No internet connection detected. Connect to Wi-Fi to continue.")
        .wrap(true)
        .margin_bottom(16)
        .build();

    // Populate SSID list
    let networks = network::scan_wifi();
    for ssid in &networks {
        ssid_list.append(ssid);
    }

    let ssid_dropdown = DropDown::builder().model(&ssid_list).build();
    let pass_entry = PasswordEntry::builder().placeholder_text("Wi-Fi Password").show_peek_icon(true).build();
    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let connect_btn = Button::builder().label("Connect").build();
    let refresh_btn = Button::builder().label("Refresh Networks").build();
    let skip_btn = Button::builder().label("Skip (already connected)").build();

    // Connect button
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
            status_label.set_label("Connecting…");
            match network::connect_wifi(&ssid, &password) {
                Ok(_) => {
                    if network::check_connectivity() {
                        stack.set_visible_child_name("disk");
                    } else {
                        status_label.set_label("Connected to Wi-Fi but no internet. Check password or network.");
                    }
                }
                Err(e) => status_label.set_label(&format!("Failed: {}", e)),
            }
        });
    }

    // Refresh button
    {
        let ssid_list = ssid_list.clone();
        let status_label = status_label.clone();
        refresh_btn.connect_clicked(move |_| {
            status_label.set_label("Scanning…");
            while ssid_list.n_items() > 0 {
                ssid_list.remove(0);
            }
            for ssid in network::scan_wifi() {
                ssid_list.append(&ssid);
            }
            status_label.set_label("Scan complete.");
        });
    }

    // Skip button
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
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = Label::builder().label("Step 1: Select Target Drive").margin_bottom(24).build();

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
    let next_btn = Button::builder()
        .label("Next: Select Repository")
        .build();
    next_btn.set_sensitive(false);

    match backend::disk_manager::get_internal_block_devices() {
        Ok(devices) if devices.is_empty() => {
            status_label.set_label("No internal drives detected. Insert an internal drive and restart the installer.");
        }
        Ok(devices) => {
            for device in devices {
                let path = format!("/dev/{}", device.name);
                let model = device.model.unwrap_or_else(|| "Unknown model".to_string());
                let transport = device.tran.unwrap_or_else(|| "internal".to_string());
                let row = format!("{} | {} | {} | {}", path, device.size, model.trim(), transport);
                drive_list.append(&row);
            }
            status_label.set_label("Select the internal drive to install access-OS.");
        }
        Err(e) => {
            status_label.set_label(&format!("Failed to read internal drives: {}", e));
        }
    }

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
        let status_label = status_label.clone();
        let state = state.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let selected = match drive_dropdown.selected_item() {
                Some(obj) => obj.downcast::<StringObject>().unwrap().string().to_string(),
                None => {
                    status_label.set_label("Select a target drive.");
                    return;
                }
            };

            let drive = selected.split(" | ").next().unwrap_or("").to_string();
            if drive.is_empty() {
                status_label.set_label("Invalid drive selection.");
                return;
            }

            state.borrow_mut().drive = drive;
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
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = Label::builder().label("Step 2: Configuration Repository").margin_bottom(24).build();
    let repo_entry = Entry::builder().placeholder_text("https://github.com/user/nix-config").build();
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
                for h in hosts {
                    host_list.append(&h);
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
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = Label::builder().label("Step 3: Host & User Settings").margin_bottom(24).build();
    let host_dropdown = DropDown::builder().model(&host_list).build();
    let user_entry = Entry::builder().placeholder_text("Username").build();
    let pass_entry = PasswordEntry::builder().placeholder_text("Password").build();
    
    let tz_entry = Entry::builder().text("America/Chicago").build();
    let locale_entry = Entry::builder().text("en_US.UTF-8").build();

    let next_btn = Button::builder().label("Next: Confirm and Install").build();
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
            stack.set_visible_child_name("install");
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

fn build_step4(stack: &Stack, state: Rc<RefCell<AppState>>) -> Box {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = Label::builder().label("Step 4: Installation Progress").margin_bottom(24).build();
    let progress_label = Label::builder().label("Ready to install...").halign(Align::Start).wrap(true).build();
    let start_btn = Button::builder().label("Start Installation").build();

    {
        let stack = stack.clone();
        let progress_label = progress_label.clone();
        start_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
        let s = state.borrow();
        progress_label.set_label("Partitioning and Mounting...");
        if let Err(e) = backend::disk_manager::execute_partitioning(&s.drive, 8, "xfs") {
            progress_label.set_label(&format!("Error: {}", e));
            btn.set_sensitive(true);
            return;
        }

        let p3 = backend::disk_manager::partition_device_path(&s.drive, 3);
        let _ = Command::new("mount").args(&[&p3, "/mnt"]).output();
        let p1 = backend::disk_manager::partition_device_path(&s.drive, 1);
        let _ = Command::new("mkdir").args(&["-p", "/mnt/boot"]).output();
        let _ = Command::new("mount").args(&[&p1, "/mnt/boot"]).output();

        progress_label.set_label("Cloning repository to target...");
        let _ = Command::new("mkdir").args(&["-p", "/mnt/etc/nixos"]).output();
        let _ = Command::new("git").args(&["clone", &s.repo_url, "/mnt/etc/nixos"]).output();

        progress_label.set_label("Applying local settings...");
        let host_dir = format!("/mnt/etc/nixos/{}", s.selected_host);
        let settings = backend::config_engine::HostSettings {
            timezone: Some(s.timezone.clone()),
            locale: Some(s.locale.clone()),
            keymap: Some(s.keymap.clone()),
        };
        let _ = backend::config_engine::apply_local_settings(&host_dir, &settings);

        progress_label.set_label("Generating hardware configuration...");
        let _ = Command::new("nixos-generate-config").args(&["--root", "/mnt", "--dir", &host_dir]).output();

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
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = Label::builder().label("Step 5: Installation Complete").margin_bottom(24).build();
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
        force_exit_btn.connect_clicked(move |_| {
            window.close();
        });
    }

    vbox.append(&title);
    vbox.append(&status_label);
    vbox.append(&reboot_btn);
    vbox.append(&exit_btn);
    vbox.append(&force_reboot_btn);
    vbox.append(&force_exit_btn);
    vbox
}
