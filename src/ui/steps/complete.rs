use crate::app::state::SharedState;
use crate::services::mount::unmount_install_targets;
use crate::services::power::{reboot_system, shutdown_system};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Box, Button, Entry, Label};
use std::process::Command;

pub fn build_complete_step(window: &ApplicationWindow, _state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 9: Installation Complete")
        .margin_bottom(24)
        .build();
    let status_label = Label::builder()
        .label("Installation completed successfully. You can install extra packages, reboot, shut down, or unmount and exit.")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let packages_entry = Entry::builder()
        .placeholder_text("Additional packages (space-separated)")
        .build();
    let install_packages_btn = Button::builder().label("Install Additional Packages").build();
    let reboot_btn = Button::builder().label("Reboot").build();
    let shutdown_btn = Button::builder().label("Shutdown").build();
    let unmount_exit_btn = Button::builder().label("Unmount and Exit").build();
    let force_reboot_btn = Button::builder().label("Force Reboot").build();
    let force_shutdown_btn = Button::builder().label("Force Shutdown").build();
    let force_exit_btn = Button::builder().label("Force Exit").build();
    force_reboot_btn.set_visible(false);
    force_shutdown_btn.set_visible(false);
    force_exit_btn.set_visible(false);

    {
        let status_label = status_label.clone();
        let packages_entry = packages_entry.clone();
        install_packages_btn.connect_clicked(move |_| {
            let packages_raw = packages_entry.text().trim().to_string();
            if packages_raw.is_empty() {
                status_label.set_label("Enter at least one package name.");
                return;
            }

            let packages: Vec<&str> = packages_raw.split_whitespace().collect();
            status_label.set_label("Installing additional packages...");

            let output = Command::new("arch-chroot")
                .arg("/mnt")
                .arg("pacman")
                .args(["-S", "--noconfirm", "--needed"])
                .args(&packages)
                .output();

            match output {
                Ok(result) if result.status.success() => {
                    status_label.set_label("Additional packages installed successfully.");
                }
                Ok(result) => {
                    let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
                    if stderr.is_empty() {
                        status_label.set_label(&format!(
                            "Failed to install packages. pacman exited with {}",
                            result.status
                        ));
                    } else {
                        status_label.set_label(&format!("Failed to install packages: {}", stderr));
                    }
                }
                Err(e) => {
                    status_label.set_label(&format!("Failed to run package install: {}", e));
                }
            }
        });
    }

    {
        let status_label = status_label.clone();
        let force_reboot_btn = force_reboot_btn.clone();
        let force_shutdown_btn = force_shutdown_btn.clone();
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
                        "Unmount failed: {}. Choose Force Reboot, Force Shutdown, or Force Exit.",
                        e
                    ));
                    force_reboot_btn.set_visible(true);
                    force_shutdown_btn.set_visible(true);
                    force_exit_btn.set_visible(true);
                }
            }
        });
    }

    {
        let status_label = status_label.clone();
        let force_reboot_btn = force_reboot_btn.clone();
        let force_shutdown_btn = force_shutdown_btn.clone();
        let force_exit_btn = force_exit_btn.clone();
        let window = window.clone();
        unmount_exit_btn.connect_clicked(move |_| {
            status_label.set_label("Unmounting install targets...");
            match unmount_install_targets() {
                Ok(()) => window.close(),
                Err(e) => {
                    status_label.set_label(&format!(
                        "Unmount failed: {}. Choose Force Reboot, Force Shutdown, or Force Exit.",
                        e
                    ));
                    force_reboot_btn.set_visible(true);
                    force_shutdown_btn.set_visible(true);
                    force_exit_btn.set_visible(true);
                }
            }
        });
    }

    {
        let status_label = status_label.clone();
        let force_reboot_btn = force_reboot_btn.clone();
        let force_shutdown_btn = force_shutdown_btn.clone();
        let force_exit_btn = force_exit_btn.clone();
        shutdown_btn.connect_clicked(move |_| {
            status_label.set_label("Unmounting install targets...");
            match unmount_install_targets() {
                Ok(()) => match shutdown_system() {
                    Ok(()) => status_label.set_label("Shutdown command issued."),
                    Err(e) => status_label.set_label(&format!("Failed to shut down: {}", e)),
                },
                Err(e) => {
                    status_label.set_label(&format!(
                        "Unmount failed: {}. Choose Force Reboot, Force Shutdown, or Force Exit.",
                        e
                    ));
                    force_reboot_btn.set_visible(true);
                    force_shutdown_btn.set_visible(true);
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
        let status_label = status_label.clone();
        force_shutdown_btn.connect_clicked(move |_| match shutdown_system() {
            Ok(()) => status_label.set_label("Force shutdown command issued."),
            Err(e) => status_label.set_label(&format!("Failed to shut down: {}", e)),
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
    vbox.append(&packages_entry);
    vbox.append(&install_packages_btn);
    vbox.append(&reboot_btn);
    vbox.append(&shutdown_btn);
    vbox.append(&unmount_exit_btn);
    vbox.append(&force_reboot_btn);
    vbox.append(&force_shutdown_btn);
    vbox.append(&force_exit_btn);
    vbox
}
