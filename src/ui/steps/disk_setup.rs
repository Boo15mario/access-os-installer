use crate::app::state::SharedState;
use crate::backend;
use crate::backend::storage_plan::{resolve_layout, HomeLocation, HomeMode, SetupMode, SwapMode};
use crate::mappers::storage::storage_selection_from_state;
use crate::ui::common::a11y::{
    append_list_row, apply_button_role, apply_textbox_role, build_list_box, build_mnemonic_label,
    select_list_box_index, selected_list_box_index, set_accessible_description,
};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{
    AccessibleRole, Align, Box, Button, CheckButton, Entry, Label, ListBox, ScrolledWindow, Stack,
};
use std::cell::Cell;
use std::rc::Rc;

fn build_list_scroller(list: &ListBox, min_height: i32) -> ScrolledWindow {
    let scroller = ScrolledWindow::builder()
        .child(list)
        .hexpand(true)
        .min_content_height(min_height)
        .build();
    // Keep focus on the actual list, not the scroller container.
    scroller.set_focusable(false);
    scroller
}

fn open_manual_partitions_dialog(
    stack: &Stack,
    state: SharedState,
    needs_home: bool,
    needs_swap_partition: bool,
    on_updated: Rc<dyn Fn()>,
) {
    let snapshot = {
        let s = state.borrow();
        (
            s.manual_efi_partition.clone(),
            s.manual_root_partition.clone(),
            s.manual_home_partition.clone(),
            s.manual_swap_partition.clone(),
        )
    };
    let keep_changes = Rc::new(Cell::new(false));

    let parent_window = stack
        .ancestor(gtk4::Window::static_type())
        .and_then(|w| w.downcast::<gtk4::Window>().ok());
    let dialog = if let Some(parent) = parent_window.as_ref() {
        gtk4::Window::builder()
            .title("Manual Partitioning")
            .modal(true)
            .transient_for(parent)
            .destroy_with_parent(true)
            .default_width(620)
            .default_height(540)
            .build()
    } else {
        gtk4::Window::builder()
            .title("Manual Partitioning")
            .modal(true)
            .default_width(620)
            .default_height(540)
            .build()
    };

    let vbox = padded_box(12, 12);
    let title = Label::builder()
        .label("Manual Partitioning")
        .margin_bottom(8)
        .build();
    let subtitle = Label::builder()
        .label("Assign partitions for EFI, root, /home, and swap.")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let partitions = backend::disk_manager::get_partition_devices().unwrap_or_default();
    let partition_paths: Vec<String> = partitions.iter().map(|part| part.path.clone()).collect();
    let partition_labels: Vec<String> = partitions
        .iter()
        .map(|part| {
            let size = backend::disk_manager::human_gib_label(part.size_bytes);
            let fstype = part.fstype.clone().unwrap_or_else(|| "unknown".to_string());
            format!("{} | {} | {} | {}", part.path, size, part.parent_disk, fstype)
        })
        .collect();
    let partition_paths = Rc::new(partition_paths);

    let partition_list = build_list_box("Available partitions", "");
    for label in &partition_labels {
        append_list_row(&partition_list, label);
    }
    let partition_scroller = ScrolledWindow::builder()
        .child(&partition_list)
        .hexpand(true)
        .vexpand(true)
        .min_content_height(180)
        .build();
    partition_scroller.set_focusable(false);

    let role_field = Box::new(gtk4::Orientation::Vertical, 6);
    let role_efi_btn = CheckButton::builder().label("EFI").build();
    role_efi_btn.set_accessible_role(AccessibleRole::Radio);
    let role_root_btn = CheckButton::builder().label("Root").build();
    role_root_btn.set_group(Some(&role_efi_btn));
    role_root_btn.set_accessible_role(AccessibleRole::Radio);
    let role_home_btn = CheckButton::builder().label("/home").build();
    role_home_btn.set_group(Some(&role_efi_btn));
    role_home_btn.set_accessible_role(AccessibleRole::Radio);
    let role_swap_btn = CheckButton::builder().label("Swap").build();
    role_swap_btn.set_group(Some(&role_efi_btn));
    role_swap_btn.set_accessible_role(AccessibleRole::Radio);

    role_home_btn.set_visible(needs_home);
    role_home_btn.set_sensitive(needs_home);
    role_swap_btn.set_visible(needs_swap_partition);
    role_swap_btn.set_sensitive(needs_swap_partition);

    // Ensure a role is selected.
    role_root_btn.set_active(true);

    let role_group = Box::new(gtk4::Orientation::Vertical, 6);
    role_group.set_accessible_role(AccessibleRole::RadioGroup);
    role_group.append(&role_efi_btn);
    role_group.append(&role_root_btn);
    role_group.append(&role_home_btn);
    role_group.append(&role_swap_btn);
    role_field.append(&build_mnemonic_label("_Assign To", &role_root_btn));
    role_field.append(&role_group);

    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let summary_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let update_summary: Rc<dyn Fn()> = {
        let state = state.clone();
        let summary_label = summary_label.clone();
        Rc::new(move || {
            let s = state.borrow();
            let efi = if s.manual_efi_partition.is_empty() {
                "(not set)"
            } else {
                &s.manual_efi_partition
            };
            let root = if s.manual_root_partition.is_empty() {
                "(not set)"
            } else {
                &s.manual_root_partition
            };
            let home = if s.manual_home_partition.is_empty() {
                "(not set)"
            } else {
                &s.manual_home_partition
            };
            let swap = if s.manual_swap_partition.is_empty() {
                "(not set)"
            } else {
                &s.manual_swap_partition
            };

            let mut lines = vec![format!("EFI: {}", efi), format!("Root: {}", root)];
            if needs_home {
                lines.push(format!("/home: {}", home));
            }
            if needs_swap_partition {
                lines.push(format!("Swap: {}", swap));
            }
            summary_label.set_label(&lines.join("\n"));
        })
    };
    update_summary();

    let assign_btn = Button::builder().label("Assign").build();
    let clear_btn = Button::builder().label("Clear Role").build();
    let done_btn = Button::builder().label("Done").build();
    let cancel_btn = Button::builder().label("Cancel").build();
    apply_button_role(&assign_btn);
    apply_button_role(&clear_btn);
    apply_button_role(&done_btn);
    apply_button_role(&cancel_btn);
    assign_btn.set_sensitive(false);

    {
        let assign_btn = assign_btn.clone();
        partition_list.connect_row_selected(move |_, row| {
            assign_btn.set_sensitive(row.is_some());
        });
    }

    {
        let state = state.clone();
        let status_label = status_label.clone();
        let partition_list = partition_list.clone();
        let partition_paths = partition_paths.clone();
        let role_efi_btn = role_efi_btn.clone();
        let role_root_btn = role_root_btn.clone();
        let role_home_btn = role_home_btn.clone();
        let role_swap_btn = role_swap_btn.clone();
        let update_summary = update_summary.clone();
        let on_updated = on_updated.clone();
        assign_btn.connect_clicked(move |_| {
            let Some(idx) = selected_list_box_index(&partition_list) else {
                status_label.set_label("Select a partition first.");
                return;
            };
            let Some(selected_path) = partition_paths.get(idx).cloned() else {
                status_label.set_label("Invalid partition selection.");
                return;
            };

            let target_role = if role_efi_btn.is_active() {
                "efi"
            } else if role_root_btn.is_active() {
                "root"
            } else if role_home_btn.is_active() {
                "home"
            } else if role_swap_btn.is_active() {
                "swap"
            } else {
                status_label.set_label("Select a role to assign.");
                return;
            };

            if target_role == "home" && !needs_home {
                status_label.set_label("/home is not enabled for this setup.");
                return;
            }
            if target_role == "swap" && !needs_swap_partition {
                status_label.set_label("Swap partition is not enabled (swap file selected).");
                return;
            }

            {
                let mut s = state.borrow_mut();
                let used_elsewhere = [
                    ("efi", &s.manual_efi_partition),
                    ("root", &s.manual_root_partition),
                    ("home", &s.manual_home_partition),
                    ("swap", &s.manual_swap_partition),
                ]
                .iter()
                .any(|(role, value)| *role != target_role && !value.is_empty() && *value == &selected_path);
                if used_elsewhere {
                    status_label.set_label("That partition is already assigned to another role.");
                    return;
                }

                match target_role {
                    "efi" => s.manual_efi_partition = selected_path,
                    "root" => s.manual_root_partition = selected_path,
                    "home" => s.manual_home_partition = selected_path,
                    "swap" => s.manual_swap_partition = selected_path,
                    _ => {}
                }
            }

            status_label.set_label("Assigned.");
            update_summary();
            on_updated();
        });
    }

    {
        let state = state.clone();
        let status_label = status_label.clone();
        let role_efi_btn = role_efi_btn.clone();
        let role_root_btn = role_root_btn.clone();
        let role_home_btn = role_home_btn.clone();
        let role_swap_btn = role_swap_btn.clone();
        let update_summary = update_summary.clone();
        let on_updated = on_updated.clone();
        clear_btn.connect_clicked(move |_| {
            let target_role = if role_efi_btn.is_active() {
                "efi"
            } else if role_root_btn.is_active() {
                "root"
            } else if role_home_btn.is_active() {
                "home"
            } else if role_swap_btn.is_active() {
                "swap"
            } else {
                status_label.set_label("Select a role to clear.");
                return;
            };

            {
                let mut s = state.borrow_mut();
                match target_role {
                    "efi" => s.manual_efi_partition.clear(),
                    "root" => s.manual_root_partition.clear(),
                    "home" => s.manual_home_partition.clear(),
                    "swap" => s.manual_swap_partition.clear(),
                    _ => {}
                }
            }
            status_label.set_label("Cleared.");
            update_summary();
            on_updated();
        });
    }

    {
        let keep_changes = keep_changes.clone();
        let dialog = dialog.clone();
        done_btn.connect_clicked(move |_| {
            keep_changes.set(true);
            dialog.close();
        });
    }

    {
        let dialog = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog.close();
        });
    }

    {
        let state = state.clone();
        let keep_changes = keep_changes.clone();
        let on_updated = on_updated.clone();
        dialog.connect_close_request(move |_| {
            if !keep_changes.get() {
                let mut s = state.borrow_mut();
                s.manual_efi_partition = snapshot.0.clone();
                s.manual_root_partition = snapshot.1.clone();
                s.manual_home_partition = snapshot.2.clone();
                s.manual_swap_partition = snapshot.3.clone();
            }
            on_updated();
            gtk4::glib::Propagation::Proceed
        });
    }

    let buttons_row = Box::new(gtk4::Orientation::Horizontal, 8);
    buttons_row.append(&assign_btn);
    buttons_row.append(&clear_btn);
    buttons_row.append(&cancel_btn);
    buttons_row.append(&done_btn);

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&build_mnemonic_label("_Available Partitions", &partition_list));
    vbox.append(&partition_scroller);
    vbox.append(&role_field);
    vbox.append(&summary_label);
    vbox.append(&status_label);
    vbox.append(&buttons_row);

    dialog.set_child(Some(&vbox));
    dialog.present();

    gtk4::glib::idle_add_local_once(move || {
        let _ = partition_list.grab_focus();
    });
}

pub fn build_disk_setup_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 2: Disk Setup")
        .margin_bottom(16)
        .build();
    let subtitle = Label::builder()
        .label("Configure automatic or manual partition setup. This controls what will be wiped and formatted.")
        .halign(Align::Start)
        .wrap(true)
        .build();

    // Use radio-style CheckButton groups for the main "mode" selectors. This is more direct to
    // navigate than lists for two-choice inputs, and behaves well with Orca.
    let setup_auto_btn = CheckButton::builder().label("Automatic").build();
    let setup_manual_btn = CheckButton::builder().label("Manual").build();
    setup_manual_btn.set_group(Some(&setup_auto_btn));
    setup_auto_btn.set_accessible_role(AccessibleRole::Radio);
    setup_manual_btn.set_accessible_role(AccessibleRole::Radio);
    setup_auto_btn.set_widget_name("a11y-default-focus");

    let home_on_root_btn = CheckButton::builder()
        .label("Home on root filesystem")
        .build();
    let home_separate_btn = CheckButton::builder().label("Separate /home").build();
    home_separate_btn.set_group(Some(&home_on_root_btn));
    home_on_root_btn.set_accessible_role(AccessibleRole::Radio);
    home_separate_btn.set_accessible_role(AccessibleRole::Radio);

    let home_same_disk_btn = CheckButton::builder().label("Same disk").build();
    let home_other_disk_btn = CheckButton::builder().label("Another disk").build();
    home_other_disk_btn.set_group(Some(&home_same_disk_btn));
    home_same_disk_btn.set_accessible_role(AccessibleRole::Radio);
    home_other_disk_btn.set_accessible_role(AccessibleRole::Radio);

    let swap_partition_btn = CheckButton::builder().label("Swap partition").build();
    let swap_file_btn = CheckButton::builder().label("Swap file").build();
    swap_file_btn.set_group(Some(&swap_partition_btn));
    swap_partition_btn.set_accessible_role(AccessibleRole::Radio);
    swap_file_btn.set_accessible_role(AccessibleRole::Radio);

    let swap_file_entry = Entry::builder()
        .placeholder_text("Swap file size in MB")
        .build();
    apply_textbox_role(&swap_file_entry);
    set_accessible_description(
        &swap_file_entry,
        "Only used when swap mode is set to Swap file.",
    );

    let removable_check = CheckButton::builder()
        .label("Install to removable media")
        .build();

    let home_disk_paths: Vec<String> = backend::disk_manager::get_internal_block_devices()
        .unwrap_or_default()
        .into_iter()
        .map(|device| format!("/dev/{}", device.name))
        .collect();
    let home_disk_labels: Vec<String> = home_disk_paths.iter().cloned().collect();
    let home_disk_list = build_list_box("Home disk", "Select the disk used for /home.");
    for label in &home_disk_labels {
        append_list_row(&home_disk_list, label);
    }
    let home_disk_scroller = build_list_scroller(&home_disk_list, 120);
    let home_disk_paths = Rc::new(home_disk_paths);

    let manual_partitions_btn = Button::builder()
        .label("Configure Manual Partitions...")
        .build();
    apply_button_role(&manual_partitions_btn);
    let manual_summary_label = Label::builder()
        .label("")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let format_efi_check = CheckButton::builder().label("Format EFI partition").build();
    let format_root_check = CheckButton::builder().label("Format root partition").build();
    let format_home_check = CheckButton::builder().label("Format /home partition").build();
    let format_swap_check = CheckButton::builder().label("Format swap partition").build();

    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let next_btn = Button::builder().label("Next: Select Desktop Environment").build();
    let back_btn = Button::builder().label("Back").build();
    apply_button_role(&next_btn);
    apply_button_role(&back_btn);

    let setup_field = Box::new(gtk4::Orientation::Vertical, 6);
    setup_field.append(&build_mnemonic_label("_Setup mode", &setup_auto_btn));
    let setup_group = Box::new(gtk4::Orientation::Vertical, 6);
    setup_group.set_accessible_role(AccessibleRole::RadioGroup);
    setup_group.append(&setup_auto_btn);
    setup_group.append(&setup_manual_btn);
    setup_field.append(&setup_group);

    let home_mode_field = Box::new(gtk4::Orientation::Vertical, 6);
    home_mode_field.append(&build_mnemonic_label("_Home mode", &home_on_root_btn));
    let home_mode_group = Box::new(gtk4::Orientation::Vertical, 6);
    home_mode_group.set_accessible_role(AccessibleRole::RadioGroup);
    home_mode_group.append(&home_on_root_btn);
    home_mode_group.append(&home_separate_btn);
    home_mode_field.append(&home_mode_group);

    let home_location_field = Box::new(gtk4::Orientation::Vertical, 6);
    home_location_field.append(&build_mnemonic_label("Home _location", &home_same_disk_btn));
    let home_location_group = Box::new(gtk4::Orientation::Vertical, 6);
    home_location_group.set_accessible_role(AccessibleRole::RadioGroup);
    home_location_group.append(&home_same_disk_btn);
    home_location_group.append(&home_other_disk_btn);
    home_location_field.append(&home_location_group);

    let home_disk_field = Box::new(gtk4::Orientation::Vertical, 6);
    home_disk_field.append(&build_mnemonic_label(
        "Home _disk (if using another disk)",
        &home_disk_list,
    ));
    home_disk_field.append(&home_disk_scroller);

    let swap_mode_field = Box::new(gtk4::Orientation::Vertical, 6);
    swap_mode_field.append(&build_mnemonic_label("S_wap mode", &swap_partition_btn));
    let swap_mode_group = Box::new(gtk4::Orientation::Vertical, 6);
    swap_mode_group.set_accessible_role(AccessibleRole::RadioGroup);
    swap_mode_group.append(&swap_partition_btn);
    swap_mode_group.append(&swap_file_btn);
    swap_mode_field.append(&swap_mode_group);

    let swap_file_field = Box::new(gtk4::Orientation::Vertical, 6);
    swap_file_field.append(&build_mnemonic_label("Swap file size (_MB)", &swap_file_entry));
    swap_file_field.append(&swap_file_entry);
    let manual_partitions_field = Box::new(gtk4::Orientation::Vertical, 6);
    manual_partitions_field.append(&manual_partitions_btn);
    manual_partitions_field.append(&manual_summary_label);

    let refresh_visibility: Rc<dyn Fn()> = {
        let state = state.clone();
        let setup_manual_btn = setup_manual_btn.clone();
        let home_separate_btn = home_separate_btn.clone();
        let home_other_disk_btn = home_other_disk_btn.clone();
        let swap_file_btn = swap_file_btn.clone();
        let swap_partition_btn = swap_partition_btn.clone();
        let home_location_field = home_location_field.clone();
        let home_disk_field = home_disk_field.clone();
        let swap_file_field = swap_file_field.clone();
        let manual_partitions_field = manual_partitions_field.clone();
        let manual_summary_label = manual_summary_label.clone();
        let format_efi_check = format_efi_check.clone();
        let format_root_check = format_root_check.clone();
        let format_home_check = format_home_check.clone();
        let format_swap_check = format_swap_check.clone();
        Rc::new(move || {
            let manual = setup_manual_btn.is_active();
            let separate_home = home_separate_btn.is_active();
            let home_other_disk = home_other_disk_btn.is_active();
            let swap_file = swap_file_btn.is_active();
            let swap_partition = swap_partition_btn.is_active();

            home_location_field.set_visible(!manual && separate_home);
            home_disk_field.set_visible(!manual && separate_home && home_other_disk);
            swap_file_field.set_visible(swap_file);

            manual_partitions_field.set_visible(manual);

            format_efi_check.set_visible(manual);
            format_root_check.set_visible(manual);
            format_home_check.set_visible(manual && separate_home);
            format_swap_check.set_visible(manual && swap_partition);

            if manual {
                let s = state.borrow();
                let efi = if s.manual_efi_partition.is_empty() {
                    "(not set)"
                } else {
                    &s.manual_efi_partition
                };
                let root = if s.manual_root_partition.is_empty() {
                    "(not set)"
                } else {
                    &s.manual_root_partition
                };
                let home = if s.manual_home_partition.is_empty() {
                    "(not set)"
                } else {
                    &s.manual_home_partition
                };
                let swap = if s.manual_swap_partition.is_empty() {
                    "(not set)"
                } else {
                    &s.manual_swap_partition
                };

                let mut lines = vec![format!("EFI: {}", efi), format!("Root: {}", root)];
                if separate_home {
                    lines.push(format!("/home: {}", home));
                }
                if swap_partition {
                    lines.push(format!("Swap: {}", swap));
                }
                manual_summary_label.set_label(&lines.join("\n"));
            } else {
                manual_summary_label.set_label("");
            }
        })
    };

    for btn in [
        setup_auto_btn.clone(),
        setup_manual_btn.clone(),
        home_on_root_btn.clone(),
        home_separate_btn.clone(),
        home_same_disk_btn.clone(),
        home_other_disk_btn.clone(),
        swap_partition_btn.clone(),
        swap_file_btn.clone(),
    ] {
        let refresh_visibility = refresh_visibility.clone();
        btn.connect_toggled(move |_| refresh_visibility());
    }

    {
        let stack = stack.clone();
        let state = state.clone();
        let refresh_visibility = refresh_visibility.clone();
        let home_separate_btn = home_separate_btn.clone();
        let swap_partition_btn = swap_partition_btn.clone();
        manual_partitions_btn.connect_clicked(move |_| {
            open_manual_partitions_dialog(
                &stack,
                state.clone(),
                home_separate_btn.is_active(),
                swap_partition_btn.is_active(),
                refresh_visibility.clone(),
            );
        });
    }

    {
        let app_state = state.borrow();
        // Defaults (ensure each radio group has an active choice).
        setup_auto_btn.set_active(true);
        home_on_root_btn.set_active(true);
        home_same_disk_btn.set_active(true);
        swap_partition_btn.set_active(true);

        match app_state.setup_mode {
            SetupMode::Automatic => setup_auto_btn.set_active(true),
            SetupMode::Manual => setup_manual_btn.set_active(true),
        }
        match app_state.home_mode {
            HomeMode::OnRoot => home_on_root_btn.set_active(true),
            HomeMode::Separate => home_separate_btn.set_active(true),
        }
        match app_state.home_location {
            HomeLocation::SameDisk => home_same_disk_btn.set_active(true),
            HomeLocation::OtherDisk => home_other_disk_btn.set_active(true),
        }
        match app_state.swap_mode {
            SwapMode::Partition => swap_partition_btn.set_active(true),
            SwapMode::File => swap_file_btn.set_active(true),
        }
        swap_file_entry.set_text(&app_state.swap_file_mb.to_string());
        removable_check.set_active(app_state.removable_media);
        format_efi_check.set_active(app_state.format_efi);
        format_root_check.set_active(app_state.format_root);
        format_home_check.set_active(app_state.format_home);
        format_swap_check.set_active(app_state.format_swap);

        if !app_state.home_disk.is_empty() {
            if let Some(index) = home_disk_paths
                .iter()
                .position(|disk| disk == &app_state.home_disk)
            {
                select_list_box_index(&home_disk_list, index);
            }
        }
    }

    refresh_visibility();

    {
        let state = state.clone();
        let stack = stack.clone();
        let status_label = status_label.clone();
        let setup_manual_btn = setup_manual_btn.clone();
        let home_separate_btn = home_separate_btn.clone();
        let home_other_disk_btn = home_other_disk_btn.clone();
        let swap_file_btn = swap_file_btn.clone();
        let swap_file_entry = swap_file_entry.clone();
        let removable_check = removable_check.clone();
        let home_disk_list = home_disk_list.clone();
        let home_disk_paths = home_disk_paths.clone();
        let format_efi_check = format_efi_check.clone();
        let format_root_check = format_root_check.clone();
        let format_home_check = format_home_check.clone();
        let format_swap_check = format_swap_check.clone();
        next_btn.connect_clicked(move |_| {
            let selected_home_disk = selected_list_box_index(&home_disk_list)
                .and_then(|idx| home_disk_paths.get(idx).cloned())
                .unwrap_or_default();

            let parsed_swap_file_mb = swap_file_entry.text().trim().parse::<u64>().unwrap_or(0);

            {
                let mut app_state = state.borrow_mut();
                app_state.setup_mode = if setup_manual_btn.is_active() {
                    SetupMode::Manual
                } else {
                    SetupMode::Automatic
                };
                app_state.home_mode = if home_separate_btn.is_active() {
                    HomeMode::Separate
                } else {
                    HomeMode::OnRoot
                };
                app_state.home_location = if home_other_disk_btn.is_active() {
                    HomeLocation::OtherDisk
                } else {
                    HomeLocation::SameDisk
                };
                app_state.swap_mode = if swap_file_btn.is_active() {
                    SwapMode::File
                } else {
                    SwapMode::Partition
                };
                if parsed_swap_file_mb > 0 {
                    app_state.swap_file_mb = parsed_swap_file_mb;
                }
                app_state.removable_media = removable_check.is_active();
                if app_state.setup_mode == SetupMode::Automatic
                    && app_state.home_mode == HomeMode::Separate
                    && app_state.home_location == HomeLocation::OtherDisk
                {
                    app_state.home_disk = selected_home_disk;
                } else {
                    app_state.home_disk.clear();
                }

                // Manual partition assignments are configured in a separate dialog. Clear stale
                // selections when they no longer apply.
                if app_state.setup_mode == SetupMode::Automatic {
                    app_state.manual_efi_partition.clear();
                    app_state.manual_root_partition.clear();
                    app_state.manual_home_partition.clear();
                    app_state.manual_swap_partition.clear();
                }
                if app_state.home_mode != HomeMode::Separate {
                    app_state.manual_home_partition.clear();
                }
                if app_state.swap_mode != SwapMode::Partition {
                    app_state.manual_swap_partition.clear();
                }
                app_state.format_efi = format_efi_check.is_active();
                app_state.format_root = format_root_check.is_active();
                app_state.format_home = format_home_check.is_active();
                app_state.format_swap = format_swap_check.is_active();
            }

            let selection = {
                let app_state = state.borrow();
                storage_selection_from_state(&app_state)
            };
            match resolve_layout(&selection) {
                Ok(layout) => {
                    state.borrow_mut().resolved_layout = Some(layout);
                    status_label.set_label("");
                    stack.set_visible_child_name("desktop_env");
                }
                Err(e) => {
                    status_label.set_label(&format!("Disk setup invalid: {}", e));
                }
            }
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("disk"));
    }

    vbox.append(&title);
    vbox.append(&subtitle);

    vbox.append(&setup_field);
    vbox.append(&home_mode_field);
    vbox.append(&home_location_field);
    vbox.append(&home_disk_field);
    vbox.append(&swap_mode_field);
    vbox.append(&swap_file_field);
    vbox.append(&manual_partitions_field);
    vbox.append(&removable_check);

    vbox.append(&format_efi_check);
    vbox.append(&format_root_check);
    vbox.append(&format_home_check);
    vbox.append(&format_swap_check);

    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
