mod backend;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation, Align, Stack, StackTransitionType, DropDown, StringList, PasswordEntry, StringObject};
use std::rc::Rc;
use std::cell::RefCell;

const APP_ID: &str = "org.accessos.Installer";

struct AppState {
    drive: String,
    repo_url: String,
    temp_repo_path: String,
    selected_host: String,
    username: String,
    password: String,
    hostname: String,
    timezone: String,
    locale: String,
    keymap: String,
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(AppState {
        drive: String::new(),
        repo_url: String::new(),
        temp_repo_path: String::new(),
        selected_host: String::new(),
        username: String::new(),
        password: String::new(),
        hostname: String::new(),
        timezone: String::new(),
        locale: String::new(),
        keymap: String::new(),
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

    // STEP 1: Disk Selection
    let step1 = build_step1(&stack, state.clone());
    stack.add_titled(&step1, Some("disk"), "Disk Selection");

    // STEP 2: Repo Selection
    let step2 = build_step2(&stack, state.clone(), host_list.clone());
    stack.add_titled(&step2, Some("repo"), "Repo Selection");

    // STEP 3: Host Selection & Metadata
    let step3 = build_step3(&stack, state.clone(), host_list.clone());
    stack.add_titled(&step3, Some("host"), "Host Selection");

    // STEP 4: Installation Progress
    let step4 = build_step4(&stack, state.clone());
    stack.add_titled(&step4, Some("install"), "Installing");

    window.set_child(Some(&stack));
    window.present();
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

    let title = Label::builder()
        .label("Step 1: Select Target Drive")
        .margin_bottom(24)
        .build();

    let drive_label = Label::builder().label("Target Drive (e.g. /dev/sda)").halign(Align::Start).build();
    let drive_entry = Entry::builder().placeholder_text("/dev/sda").build();
    let next_btn = Button::builder().label("Next: Select Repository").build();

    let stack_clone = stack.clone();
    next_btn.connect_clicked(move |_| {
        state.borrow_mut().drive = drive_entry.text().to_string();
        stack_clone.set_visible_child_name("repo");
    });

    vbox.append(&title);
    vbox.append(&drive_label);
    vbox.append(&drive_entry);
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

    let title = Label::builder()
        .label("Step 2: Enter Configuration Repository URL")
        .margin_bottom(24)
        .build();

    let repo_entry = Entry::builder().placeholder_text("https://github.com/user/nix-config").build();
    let next_btn = Button::builder().label("Next: Scan Configuration").build();
    let back_btn = Button::builder().label("Back").build();

    let stack_clone = stack.clone();
    let state_clone = state.clone();
    next_btn.connect_clicked(move |_| {
        let url = repo_entry.text().to_string();
        state_clone.borrow_mut().repo_url = url.clone();
        
        match backend::config_engine::clone_repo_to_temp(&url) {
            Ok(path) => {
                state_clone.borrow_mut().temp_repo_path = path.clone();
                let hosts = backend::config_engine::list_hosts(&path);
                
                // Populate the host_list
                while host_list.n_items() > 0 { host_list.remove(0); }
                for h in hosts { host_list.append(&h); }
                
                stack_clone.set_visible_child_name("host");
            },
            Err(e) => {
                 // Error handling
                 println!("Clone failed: {}", e);
            }
        }
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

fn build_step3(stack: &Stack, state: Rc<RefCell<AppState>>, host_list: StringList) -> Box {
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

    let host_dropdown = DropDown::builder().model(&host_list).build();

    let user_entry = Entry::builder().placeholder_text("Username").build();
    let pass_entry = PasswordEntry::builder().placeholder_text("Password").build();

    let next_btn = Button::builder().label("Next: Confirm and Install").build();
    let back_btn = Button::builder().label("Back").build();

    let stack_clone = stack.clone();
    let state_clone = state.clone();
    next_btn.connect_clicked(move |_| {
        let mut s = state_clone.borrow_mut();
        s.username = user_entry.text().to_string();
        s.password = pass_entry.text().to_string();
        s.selected_host = match host_dropdown.selected_item() {
            Some(obj) => obj.downcast::<StringObject>().unwrap().string().to_string(),
            None => "hp-boo".to_string()
        };
        stack_clone.set_visible_child_name("install");
    });

    back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("repo");
    });

    vbox.append(&title);
    vbox.append(&Label::new(Some("Select Host Template")));
    vbox.append(&host_dropdown);
    vbox.append(&Label::new(Some("Username")));
    vbox.append(&user_entry);
    vbox.append(&Label::new(Some("Password")));
    vbox.append(&pass_entry);
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
    let state_clone = state.clone();
    let progress_label_clone = progress_label.clone();
    
    start_btn.connect_clicked(move |_| {
        let s = state_clone.borrow();
        progress_label_clone.set_label("Partitioning disk...");
        
        if let Err(e) = backend::disk_manager::execute_partitioning(&s.drive, 8, "xfs") {
             progress_label_clone.set_label(&format!("Partitioning failed: {}", e));
             return;
        }
        
        progress_label_clone.set_label("Cloning repository to /mnt/etc/nixos...");
        // Simplified clone for the demo
        
        progress_label_clone.set_label("Starting NixOS installation...");
        match backend::install_worker::start_install(&s.selected_host, &s.username, &s.password) {
            Ok(_) => progress_label_clone.set_label("Installation Complete! Please reboot."),
            Err(e) => progress_label_clone.set_label(&format!("Installation failed: {}", e)),
        }
    });

    back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("host");
    });

    vbox.append(&title);
    vbox.append(&progress_label);
    vbox.append(&start_btn);
    vbox.append(&back_btn);
    vbox
}
