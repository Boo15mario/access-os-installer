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
use gtk4::{Align, Box, Button, CheckButton, Entry, Label, ListBox, ScrolledWindow, Stack};
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

    let setup_mode_list = build_list_box("Setup mode", "Use arrow keys to choose an option.");
    let setup_auto_row = append_list_row(&setup_mode_list, "Automatic");
    setup_auto_row.set_widget_name("a11y-default-focus");
    append_list_row(&setup_mode_list, "Manual");

    let home_mode_list = build_list_box("Home mode", "Use arrow keys to choose an option.");
    append_list_row(&home_mode_list, "Home on root filesystem");
    append_list_row(&home_mode_list, "Separate /home");

    let home_location_list = build_list_box("Home location", "Use arrow keys to choose an option.");
    append_list_row(&home_location_list, "Same disk");
    append_list_row(&home_location_list, "Another disk");

    let swap_mode_list = build_list_box("Swap mode", "Use arrow keys to choose an option.");
    append_list_row(&swap_mode_list, "Swap partition");
    append_list_row(&swap_mode_list, "Swap file");

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
    let home_disk_list = build_list_box("Home disk", "Use arrow keys to choose a disk.");
    for label in &home_disk_labels {
        append_list_row(&home_disk_list, label);
    }
    let home_disk_scroller = build_list_scroller(&home_disk_list, 120);
    let home_disk_paths = Rc::new(home_disk_paths);

    let partition_info = backend::disk_manager::get_partition_devices().unwrap_or_default();
    let partition_paths: Vec<String> = partition_info.iter().map(|part| part.path.clone()).collect();
    let partition_labels: Vec<String> = partition_info
        .iter()
        .map(|part| {
            let size = backend::disk_manager::human_gib_label(part.size_bytes);
            let fstype = part.fstype.clone().unwrap_or_else(|| "unknown".to_string());
            format!("{} | {} | {} | {}", part.path, size, part.parent_disk, fstype)
        })
        .collect();

    let efi_list = build_list_box(
        "Manual EFI partition",
        "Use arrow keys to choose an EFI partition.",
    );
    let root_list = build_list_box(
        "Manual root partition",
        "Use arrow keys to choose a root partition.",
    );
    let home_list = build_list_box(
        "Manual /home partition",
        "Use arrow keys to choose a /home partition.",
    );
    let swap_list = build_list_box(
        "Manual swap partition",
        "Use arrow keys to choose a swap partition.",
    );
    for label in &partition_labels {
        append_list_row(&efi_list, label);
        append_list_row(&root_list, label);
        append_list_row(&home_list, label);
        append_list_row(&swap_list, label);
    }
    let efi_scroller = build_list_scroller(&efi_list, 140);
    let root_scroller = build_list_scroller(&root_list, 140);
    let home_scroller = build_list_scroller(&home_list, 140);
    let swap_scroller = build_list_scroller(&swap_list, 140);
    let partition_paths = Rc::new(partition_paths);

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
    setup_field.append(&build_mnemonic_label("_Setup mode", &setup_mode_list));
    setup_field.append(&setup_mode_list);

    let home_mode_field = Box::new(gtk4::Orientation::Vertical, 6);
    home_mode_field.append(&build_mnemonic_label("_Home mode", &home_mode_list));
    home_mode_field.append(&home_mode_list);

    let home_location_field = Box::new(gtk4::Orientation::Vertical, 6);
    home_location_field.append(&build_mnemonic_label("Home _location", &home_location_list));
    home_location_field.append(&home_location_list);

    let home_disk_field = Box::new(gtk4::Orientation::Vertical, 6);
    home_disk_field.append(&build_mnemonic_label(
        "Home _disk (if using another disk)",
        &home_disk_list,
    ));
    home_disk_field.append(&home_disk_scroller);

    let swap_mode_field = Box::new(gtk4::Orientation::Vertical, 6);
    swap_mode_field.append(&build_mnemonic_label("S_wap mode", &swap_mode_list));
    swap_mode_field.append(&swap_mode_list);

    let swap_file_field = Box::new(gtk4::Orientation::Vertical, 6);
    swap_file_field.append(&build_mnemonic_label("Swap file size (_MB)", &swap_file_entry));
    swap_file_field.append(&swap_file_entry);

    let efi_field = Box::new(gtk4::Orientation::Vertical, 6);
    efi_field.append(&build_mnemonic_label("Manual _EFI partition", &efi_list));
    efi_field.append(&efi_scroller);

    let root_field = Box::new(gtk4::Orientation::Vertical, 6);
    root_field.append(&build_mnemonic_label("Manual _root partition", &root_list));
    root_field.append(&root_scroller);

    let manual_home_field = Box::new(gtk4::Orientation::Vertical, 6);
    manual_home_field.append(&build_mnemonic_label("Manual /_home partition", &home_list));
    manual_home_field.append(&home_scroller);

    let manual_swap_field = Box::new(gtk4::Orientation::Vertical, 6);
    manual_swap_field.append(&build_mnemonic_label("Manual s_wap partition", &swap_list));
    manual_swap_field.append(&swap_scroller);

    let refresh_visibility: Rc<dyn Fn()> = {
        let setup_mode_list = setup_mode_list.clone();
        let home_mode_list = home_mode_list.clone();
        let home_location_list = home_location_list.clone();
        let swap_mode_list = swap_mode_list.clone();
        let home_location_field = home_location_field.clone();
        let home_disk_field = home_disk_field.clone();
        let swap_file_field = swap_file_field.clone();
        let efi_field = efi_field.clone();
        let root_field = root_field.clone();
        let manual_home_field = manual_home_field.clone();
        let manual_swap_field = manual_swap_field.clone();
        let format_efi_check = format_efi_check.clone();
        let format_root_check = format_root_check.clone();
        let format_home_check = format_home_check.clone();
        let format_swap_check = format_swap_check.clone();
        Rc::new(move || {
            let manual = selected_list_box_index(&setup_mode_list) == Some(1);
            let separate_home = selected_list_box_index(&home_mode_list) == Some(1);
            let home_other_disk = selected_list_box_index(&home_location_list) == Some(1);
            let swap_file = selected_list_box_index(&swap_mode_list) == Some(1);
            let swap_partition = selected_list_box_index(&swap_mode_list) == Some(0);

            home_location_field.set_visible(separate_home);
            home_disk_field.set_visible(separate_home && home_other_disk);
            swap_file_field.set_visible(swap_file);

            efi_field.set_visible(manual);
            root_field.set_visible(manual);
            manual_home_field.set_visible(manual && separate_home);
            manual_swap_field.set_visible(manual && swap_partition);

            format_efi_check.set_visible(manual);
            format_root_check.set_visible(manual);
            format_home_check.set_visible(manual && separate_home);
            format_swap_check.set_visible(manual && swap_partition);
        })
    };

    for list in [
        setup_mode_list.clone(),
        home_mode_list.clone(),
        home_location_list.clone(),
        swap_mode_list.clone(),
    ] {
        let refresh_visibility = refresh_visibility.clone();
        list.connect_row_selected(move |_, _| refresh_visibility());
    }

    {
        let app_state = state.borrow();
        select_list_box_index(
            &setup_mode_list,
            match app_state.setup_mode {
                SetupMode::Automatic => 0,
                SetupMode::Manual => 1,
            },
        );
        select_list_box_index(
            &home_mode_list,
            match app_state.home_mode {
                HomeMode::OnRoot => 0,
                HomeMode::Separate => 1,
            },
        );
        select_list_box_index(
            &home_location_list,
            match app_state.home_location {
                HomeLocation::SameDisk => 0,
                HomeLocation::OtherDisk => 1,
            },
        );
        select_list_box_index(
            &swap_mode_list,
            match app_state.swap_mode {
                SwapMode::Partition => 0,
                SwapMode::File => 1,
            },
        );
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
        if !app_state.manual_efi_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_efi_partition)
            {
                select_list_box_index(&efi_list, index);
            }
        }
        if !app_state.manual_root_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_root_partition)
            {
                select_list_box_index(&root_list, index);
            }
        }
        if !app_state.manual_home_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_home_partition)
            {
                select_list_box_index(&home_list, index);
            }
        }
        if !app_state.manual_swap_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_swap_partition)
            {
                select_list_box_index(&swap_list, index);
            }
        }
    }

    refresh_visibility();

    {
        let state = state.clone();
        let stack = stack.clone();
        let status_label = status_label.clone();
        let setup_mode_list = setup_mode_list.clone();
        let home_mode_list = home_mode_list.clone();
        let home_location_list = home_location_list.clone();
        let swap_mode_list = swap_mode_list.clone();
        let swap_file_entry = swap_file_entry.clone();
        let removable_check = removable_check.clone();
        let home_disk_list = home_disk_list.clone();
        let home_disk_paths = home_disk_paths.clone();
        let efi_list = efi_list.clone();
        let root_list = root_list.clone();
        let home_list = home_list.clone();
        let swap_list = swap_list.clone();
        let partition_paths = partition_paths.clone();
        let format_efi_check = format_efi_check.clone();
        let format_root_check = format_root_check.clone();
        let format_home_check = format_home_check.clone();
        let format_swap_check = format_swap_check.clone();
        next_btn.connect_clicked(move |_| {
            let selected_home_disk = selected_list_box_index(&home_disk_list)
                .and_then(|idx| home_disk_paths.get(idx).cloned())
                .unwrap_or_default();
            let selected_efi = selected_list_box_index(&efi_list)
                .and_then(|idx| partition_paths.get(idx).cloned())
                .unwrap_or_default();
            let selected_root = selected_list_box_index(&root_list)
                .and_then(|idx| partition_paths.get(idx).cloned())
                .unwrap_or_default();
            let selected_home = selected_list_box_index(&home_list)
                .and_then(|idx| partition_paths.get(idx).cloned())
                .unwrap_or_default();
            let selected_swap = selected_list_box_index(&swap_list)
                .and_then(|idx| partition_paths.get(idx).cloned())
                .unwrap_or_default();

            let parsed_swap_file_mb = swap_file_entry.text().trim().parse::<u64>().unwrap_or(0);

            {
                let mut app_state = state.borrow_mut();
                app_state.setup_mode = if selected_list_box_index(&setup_mode_list) == Some(1) {
                    SetupMode::Manual
                } else {
                    SetupMode::Automatic
                };
                app_state.home_mode = if selected_list_box_index(&home_mode_list) == Some(1) {
                    HomeMode::Separate
                } else {
                    HomeMode::OnRoot
                };
                app_state.home_location =
                    if selected_list_box_index(&home_location_list) == Some(1) {
                    HomeLocation::OtherDisk
                } else {
                    HomeLocation::SameDisk
                };
                app_state.swap_mode = if selected_list_box_index(&swap_mode_list) == Some(1) {
                    SwapMode::File
                } else {
                    SwapMode::Partition
                };
                if parsed_swap_file_mb > 0 {
                    app_state.swap_file_mb = parsed_swap_file_mb;
                }
                app_state.removable_media = removable_check.is_active();
                app_state.home_disk = selected_home_disk;
                app_state.manual_efi_partition = selected_efi;
                app_state.manual_root_partition = selected_root;
                app_state.manual_home_partition = selected_home;
                app_state.manual_swap_partition = selected_swap;
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
    vbox.append(&removable_check);

    vbox.append(&efi_field);
    vbox.append(&root_field);
    vbox.append(&manual_home_field);
    vbox.append(&manual_swap_field);
    vbox.append(&format_efi_check);
    vbox.append(&format_root_check);
    vbox.append(&format_home_check);
    vbox.append(&format_swap_check);

    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
