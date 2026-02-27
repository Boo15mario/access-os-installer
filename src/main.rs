mod backend;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation, Align, Stack, StackTransitionType, DropDown, StringList};

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

    // STEP 3: Host Selection & Metadata
    let step3 = build_step3(&stack);
    stack.add_titled(&step3, Some("host"), "Host Selection");

    // STEP 4: Installation Progress
    let step4 = build_step4(&stack);
    stack.add_titled(&step4, Some("install"), "Installing");

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
    vbox.append(&drive_label_placeholder());
    vbox.append(&drive_entry);
    vbox.append(&next_btn);
    vbox
}

fn drive_label_placeholder() -> Label {
    Label::builder().label("Target Drive (e.g. /dev/sda)").halign(Align::Start).build()
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
    let next_btn = Button::builder().label("Next: Scan Configuration").build();
    let back_btn = Button::builder().label("Back").build();

    let stack_clone = stack.clone();
    next_btn.connect_clicked(move |_| {
        // Here we would normally trigger backend::config_engine::clone_repo_to_temp(repo_entry.text())
        // And then update Step 3 hosts list.
        stack_clone.set_visible_child_name("host");
    });

    back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("disk");
    });

    vbox.append(&title);
    vbox.append(&repo_entry);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}

fn build_step3(stack: &Stack) -> Box {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Step 3: Choose Host & Settings")
        .margin_bottom(24)
        .build();

    let host_label = Label::builder().label("Select Host Template").halign(Align::Start).build();
    let host_list = StringList::new(&["hp-boo", "boo76", "boo15mario", "boo15mario-main"]);
    let host_dropdown = DropDown::builder().model(&host_list).build();

    let user_label = Label::builder().label("Username").halign(Align::Start).build();
    let user_entry = Entry::builder().placeholder_text("Username").build();

    let pass_label = Label::builder().label("Password").halign(Align::Start).build();
    let pass_entry = PasswordEntry_placeholder();

    let next_btn = Button::builder().label("Next: Confirm and Install").build();
    let back_btn = Button::builder().label("Back").build();

    let stack_clone = stack.clone();
    next_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("install");
    });

    back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("repo");
    });

    vbox.append(&title);
    vbox.append(&host_label);
    vbox.append(&host_dropdown);
    vbox.append(&user_label);
    vbox.append(&user_entry);
    vbox.append(&pass_label);
    vbox.append(&pass_entry);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}

fn PasswordEntry_placeholder() -> gtk4::PasswordEntry {
    gtk4::PasswordEntry::builder().placeholder_text("Password").build()
}

fn build_step4(stack: &Stack) -> Box {
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Step 4: Installation Progress")
        .margin_bottom(24)
        .build();

    let progress_label = Label::builder()
        .label("Ready to install access-OS...")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let start_btn = Button::builder().label("Start Installation").build();
    let back_btn = Button::builder().label("Back").build();

    let stack_clone = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("host");
    });

    vbox.append(&title);
    vbox.append(&progress_label);
    vbox.append(&start_btn);
    vbox.append(&back_btn);
    vbox
}
