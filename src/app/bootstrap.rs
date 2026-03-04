use crate::app::constants::APP_ID;
use crate::app::state::{AppState, SharedState};
use crate::backend;
use crate::backend::config_engine::KernelVariant;
use crate::backend::storage_plan::{HomeLocation, HomeMode, SetupMode, SwapMode};
use crate::ui::steps;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Stack, StackTransitionType, Widget};
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

pub fn run() {
    prepare_accessibility_environment();
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let state = initial_state();

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
    if is_orca_running() || is_screen_reader_setting_enabled() {
        // Animations can interfere with focus/events for screen readers.
        stack.set_transition_type(StackTransitionType::None);
        stack.set_transition_duration(0);
    }

    let step_welcome = steps::build_welcome_step(&stack);
    stack.add_titled(&step_welcome, Some("welcome"), "Welcome");

    let step_wifi = steps::build_wifi_step(&stack);
    stack.add_titled(&step_wifi, Some("wifi"), "Wi-Fi Setup");

    let step_disk = steps::build_disk_step(&stack, state.clone());
    stack.add_titled(&step_disk, Some("disk"), "Disk Selection");

    let step_disk_setup = steps::build_disk_setup_step(&stack, state.clone());
    stack.add_titled(&step_disk_setup, Some("disk_setup"), "Disk Setup");

    let step_de = steps::build_de_step(&stack, state.clone());
    stack.add_titled(&step_de, Some("desktop_env"), "Desktop Environment");

    let step_mirror = steps::build_mirror_step(&stack, state.clone());
    stack.add_titled(&step_mirror, Some("mirror"), "Mirror Region");

    let step_settings = steps::build_settings_step(&stack, state.clone());
    stack.add_titled(&step_settings, Some("settings"), "User Settings");

    let (step_preflight, refresh_preflight) = steps::build_preflight_step(&stack, state.clone());
    stack.add_titled(&step_preflight, Some("preflight"), "Preflight");

    let (step_review, refresh_review) = steps::build_review_step(&stack, state.clone());
    stack.add_titled(&step_review, Some("review"), "Review");

    let step_install = steps::build_install_step(&stack, state.clone());
    stack.add_titled(&step_install, Some("install"), "Installing");

    let step_complete = steps::build_complete_step(&window, state.clone());
    stack.add_titled(&step_complete, Some("complete"), "Complete");

    {
        let refresh_preflight = refresh_preflight.clone();
        let refresh_review = refresh_review.clone();
        let stack_for_focus = stack.clone();
        stack.connect_visible_child_name_notify(move |stack| {
            if let Some(name) = stack.visible_child_name() {
                let step_name = name.to_string();
                match name.as_str() {
                    "preflight" => refresh_preflight(),
                    "review" => refresh_review(),
                    _ => {}
                }
                maybe_announce_step(&step_name);
            }
            schedule_focus_visible_step(&stack_for_focus);
        });
    }

    window.set_child(Some(&stack));
    stack.set_visible_child_name("welcome");
    window.fullscreen();
    window.present();
    schedule_focus_visible_step(&stack);
    maybe_announce_step("welcome");
    schedule_startup_sound();
}

fn initial_state() -> SharedState {
    let suggested_swap = backend::get_suggested_swap_gb();

    Rc::new(RefCell::new(AppState {
        drive: String::new(),
        selected_disk_gib: None,
        swap_gb: suggested_swap,
        swap_mode: SwapMode::Partition,
        swap_file_mb: suggested_swap * 1024,
        fs_type: "xfs".to_string(),
        setup_mode: SetupMode::Automatic,
        home_mode: HomeMode::OnRoot,
        home_location: HomeLocation::SameDisk,
        home_disk: String::new(),
        manual_efi_partition: String::new(),
        manual_root_partition: String::new(),
        manual_home_partition: String::new(),
        manual_swap_partition: String::new(),
        format_efi: true,
        format_root: true,
        format_home: true,
        format_swap: true,
        removable_media: false,
        desktop_env: None,
        kernel: KernelVariant::Standard,
        nvidia: false,
        hostname: String::new(),
        username: String::new(),
        password: String::new(),
        timezone: "America/Chicago".to_string(),
        locale: "en_US.UTF-8".to_string(),
        keymap: "us".to_string(),
        mirror_region: "Worldwide".to_string(),
        preflight_results: Vec::new(),
        resolved_layout: None,
    }))
}

fn schedule_startup_sound() {
    if is_orca_running() || is_screen_reader_setting_enabled() {
        return;
    }

    gtk4::glib::idle_add_local_once(|| {
        std::thread::spawn(|| {
            if let Err(e) = play_startup_sound() {
                eprintln!("Startup sound unavailable: {}", e);
            }
        });
    });
}

fn is_orca_running() -> bool {
    // Orca may appear as "orca" or as a Python command line that includes "orca".
    if command_succeeds("pgrep", &["-x", "orca"]) {
        return true;
    }

    Command::new("pgrep")
        .args(["-f", "(^|/)orca([[:space:]]|$)"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn schedule_focus_visible_step(stack: &Stack) {
    let stack = stack.clone();
    let mut attempts = 0u8;
    gtk4::glib::timeout_add_local(Duration::from_millis(60), move || {
        attempts = attempts.saturating_add(1);
        if let Some(step) = stack.visible_child() {
            if focus_first_interactive(&step) {
                return gtk4::glib::ControlFlow::Break;
            }
        }
        if attempts >= 20 {
            gtk4::glib::ControlFlow::Break
        } else {
            gtk4::glib::ControlFlow::Continue
        }
    });
}

fn focus_first_interactive(widget: &Widget) -> bool {
    if widget.is_visible() && widget.is_sensitive() && widget.is_focusable() {
        if widget.grab_focus() {
            return true;
        }
    }

    let mut child = widget.first_child();
    while let Some(current) = child {
        if focus_first_interactive(&current) {
            return true;
        }
        child = current.next_sibling();
    }

    false
}

fn maybe_announce_step(step_name: &str) {
    let _ = step_name;
}

fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_screen_reader_setting_enabled() -> bool {
    let output = match Command::new("gsettings")
        .args([
            "get",
            "org.gnome.desktop.a11y.applications",
            "screen-reader-enabled",
        ])
        .output()
    {
        Ok(output) => output,
        Err(_) => return false,
    };

    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout).contains("true")
}

fn prepare_accessibility_environment() {
    if std::env::var_os("GTK_A11Y").is_none() {
        std::env::set_var("GTK_A11Y", "atspi");
    }
    std::env::remove_var("NO_AT_BRIDGE");
}

fn play_startup_sound() -> Result<(), String> {
    if let Some(sound_file) = find_startup_sound_file() {
        let sound_file_arg = sound_file.to_string_lossy().to_string();
        if run_sound_command("paplay", &[&sound_file_arg]).is_ok()
            || run_sound_command("pw-play", &[&sound_file_arg]).is_ok()
            || run_sound_command("aplay", &[&sound_file_arg]).is_ok()
        {
            return Ok(());
        }
    }

    if run_sound_command("canberra-gtk-play", &["-i", "desktop-login"]).is_ok()
        || run_sound_command("canberra-gtk-play", &["-i", "bell"]).is_ok()
    {
        return Ok(());
    }

    Err("no working playback command found".to_string())
}

fn run_sound_command(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{} failed to execute: {}", program, e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("{} exited with {}", program, output.status))
        } else {
            Err(format!("{}: {}", program, stderr))
        }
    }
}

fn find_startup_sound_file() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("assets/login.wav"));
        candidates.push(cwd.join("assets/login.ogg"));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("assets/login.wav"));
            candidates.push(exe_dir.join("assets/login.ogg"));
            candidates.push(exe_dir.join("../assets/login.wav"));
            candidates.push(exe_dir.join("../assets/login.ogg"));
        }
    }

    candidates.into_iter().find(|path| path.exists())
}
