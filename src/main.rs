mod backend;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation, PasswordEntry, Align};

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
        .margin_bottom(24)
        .build();

    let suggested_swap = backend::get_suggested_swap_gb();

    let drive_label = Label::builder().label("Target Drive (e.g. /dev/sda)").halign(Align::Start).build();
    let drive_entry = Entry::builder().placeholder_text("/dev/sda").build();

    let user_label = Label::builder().label("New Username").halign(Align::Start).build();
    let user_entry = Entry::builder().placeholder_text("Username").build();

    let pass_label = Label::builder().label("New Password").halign(Align::Start).build();
    let pass_entry = PasswordEntry::builder().placeholder_text("Password").build();

    let host_label = Label::builder().label("Hostname").halign(Align::Start).build();
    let host_entry = Entry::builder().placeholder_text("access-os").build();

    let swap_label = Label::builder()
        .label(&format!("Swap Size (GB, suggested: {})"), suggested_swap)
        .halign(Align::Start)
        .build();
    let swap_entry = Entry::builder().text(&suggested_swap.to_string()).build();

    let install_btn = Button::builder()
        .label("Install access-OS")
        .margin_top(24)
        .build();

    let status_label = Label::builder()
        .label("")
        .halign(Align::Start)
        .wrap(true)
        .build();

    vbox.append(&title);
    vbox.append(&drive_label);
    vbox.append(&drive_entry);
    vbox.append(&user_label);
    vbox.append(&user_entry);
    vbox.append(&pass_label);
    vbox.append(&pass_entry);
    vbox.append(&host_label);
    vbox.append(&host_entry);
    vbox.append(&swap_label);
    vbox.append(&swap_entry);
    vbox.append(&install_btn);
    vbox.append(&status_label);

    window.set_child(Some(&vbox));
    window.present();
}
