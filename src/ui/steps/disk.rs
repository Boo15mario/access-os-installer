use crate::app::state::{DriveOption, SharedState};
use crate::backend;
use crate::ui::common::a11y::{apply_button_role, set_accessible_description, set_accessible_label};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, ComboBoxText, Label, Stack};
use std::rc::Rc;

pub fn build_disk_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 1: Select Target Drive")
        .margin_bottom(24)
        .build();

    let drive_combo = ComboBoxText::new();
    drive_combo.set_focusable(true);
    set_accessible_label(&drive_combo, "Target Internal Drive");
    set_accessible_description(
        &drive_combo,
        "Select the internal drive that will be erased and used for installation.",
    );
    let warning = Label::builder()
        .label("Warning: Installing will erase all data on the selected disk.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let next_btn = Button::builder().label("Next: Disk Setup").build();
    next_btn.set_sensitive(false);
    apply_button_role(&next_btn);

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
                drive_combo.append_text(&format!(
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
        let next_btn = next_btn.clone();
        drive_combo.connect_changed(move |combo| {
            let has_selected_drive = combo.active().is_some();
            next_btn.set_sensitive(has_selected_drive);
        });
    }

    {
        let drive_combo = drive_combo.clone();
        let drive_options = drive_options.clone();
        let status_label = status_label.clone();
        let state = state.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let Some(selected_index) = drive_combo.active() else {
                status_label.set_label("Select a target drive.");
                return;
            };

            let Some(option) = drive_options.get(selected_index as usize) else {
                status_label.set_label("Invalid drive selection.");
                return;
            };

            let mut app_state = state.borrow_mut();
            app_state.drive = option.path.clone();
            app_state.selected_disk_gib = Some(option.disk_gib);
            app_state.resolved_layout = None;
            stack.set_visible_child_name("disk_setup");
        });
    }

    vbox.append(&title);
    let drive_label = Label::new(Some("_Target Internal Drive"));
    drive_label.set_use_underline(true);
    drive_label.set_mnemonic_widget(Some(&drive_combo));
    vbox.append(&drive_label);
    vbox.append(&drive_combo);
    vbox.append(&warning);
    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox
}
