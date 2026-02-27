mod backend;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation, Align, Stack, StackTransitionType};

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

    let stack = Stack::builder()
        .transition_type(StackTransitionType::SlideLeftRight)
        .transition_duration(500)
        .build();

    // STEP 1: Disk Selection
    let step1 = build_step1(&stack);
    stack.add_titled(&step1, Some("disk"), "Disk Selection");

    // STEP 2: Repo Selection
    let step2 = build_step2(&stack);
    stack.add_titled(&step2, Some("repo"), "Repo Selection");

    window.set_child(Some(&stack));
    window.present();
}

fn build_step1(stack: &Stack) -> Box {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Step 1: Select Target Drive")
        .margin_bottom(24)
        .build();

    let drive_entry = Entry::builder().placeholder_text("/dev/sda").build();
    let next_btn = Button::builder().label("Next: Select Repository").build();

    let stack_clone = stack.clone();
    next_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("repo");
    });

    vbox.append(&title);
    vbox.append(&drive_entry);
    vbox.append(&next_btn);
    vbox
}

fn build_step2(stack: &Stack) -> Box {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Step 2: Enter Configuration Repository URL")
        .margin_bottom(24)
        .build();

    let repo_entry = Entry::builder().placeholder_text("https://github.com/user/nix-config").build();
    let back_btn = Button::builder().label("Back").build();
    let next_btn = Button::builder().label("Next: Scan Configuration").build();

    let stack_clone = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("disk");
    });

    vbox.append(&title);
    vbox.append(&repo_entry);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
