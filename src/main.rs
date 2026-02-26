use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation, PasswordEntry, Align};
use std::process::Command;

const APP_ID: &str = "org.accessos.Installer";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("access-OS Installer")
        .default_width(600)
        .default_height(500)
        .build();

    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Welcome to the access-OS Installer")
        .css_classes(["title-1"])
        .margin_bottom(24)
        .build();

    let drive_label = Label::builder().label("Target Drive (e.g. /dev/sda)").halign(Align::Start).build();
    let drive_entry = Entry::builder().placeholder_text("/dev/sda").build();

    let user_label = Label::builder().label("New Username").halign(Align::Start).build();
    let user_entry = Entry::builder().placeholder_text("Username").build();

    let pass_label = Label::builder().label("New Password").halign(Align::Start).build();
    let pass_entry = PasswordEntry::builder().placeholder_text("Password").build();

    let host_label = Label::builder().label("Hostname").halign(Align::Start).build();
    let host_entry = Entry::builder().placeholder_text("access-os").build();

    let install_btn = Button::builder()
        .label("Install access-OS")
        .margin_top(24)
        .build();

    let status_label = Label::builder()
        .label("")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let drive_entry_clone = drive_entry.clone();
    let user_entry_clone = user_entry.clone();
    let host_entry_clone = host_entry.clone();
    let status_label_clone = status_label.clone();

    install_btn.connect_clicked(move |_| {
        let drive = drive_entry_clone.text().to_string();
        let user = user_entry_clone.text().to_string();
        let host = host_entry_clone.text().to_string();
        
        status_label_clone.set_label(&format!("Starting installation on {} for user {}...", drive, user));
        
        // This is where the actual NixOS install logic would go.
        // For a full installer, this would:
        // 1. Partition the drive using `parted` or `sfdisk`
        // 2. Format the partitions (mkfs.fat, mkfs.ext4)
        // 3. Mount to /mnt
        // 4. nixos-generate-config --root /mnt
        // 5. Inject user/hostname into the config
        // 6. nixos-install --no-root-passwd --flake ...
        
        println!("Install triggered. Drive: {}, User: {}, Host: {}", drive, user, host);
    });

    vbox.append(&title);
    vbox.append(&drive_label);
    vbox.append(&drive_entry);
    vbox.append(&user_label);
    vbox.append(&user_entry);
    vbox.append(&pass_label);
    vbox.append(&pass_entry);
    vbox.append(&host_label);
    vbox.append(&host_entry);
    vbox.append(&install_btn);
    vbox.append(&status_label);

    window.set_child(Some(&vbox));
    window.present();
}
