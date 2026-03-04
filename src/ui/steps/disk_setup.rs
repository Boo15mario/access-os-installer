use crate::app::state::SharedState;
use crate::backend;
use crate::backend::storage_plan::{resolve_layout, HomeLocation, HomeMode, SetupMode, SwapMode};
use crate::mappers::storage::storage_selection_from_state;
use crate::ui::common::a11y::{
    apply_button_role, apply_textbox_role, build_mnemonic_label, set_accessible_description,
};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, CheckButton, ComboBoxText, Entry, Label, Stack};
use std::rc::Rc;

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

    let setup_mode_combo = ComboBoxText::new();
    setup_mode_combo.append_text("Automatic");
    setup_mode_combo.append_text("Manual");

    let home_mode_combo = ComboBoxText::new();
    home_mode_combo.append_text("Home on root filesystem");
    home_mode_combo.append_text("Separate /home");

    let home_location_combo = ComboBoxText::new();
    home_location_combo.append_text("Same disk");
    home_location_combo.append_text("Another disk");

    let swap_mode_combo = ComboBoxText::new();
    swap_mode_combo.append_text("Swap partition");
    swap_mode_combo.append_text("Swap file");

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
    let home_disk_labels: Vec<String> = home_disk_paths.iter().map(|path| path.to_string()).collect();
    let home_disk_combo = ComboBoxText::new();
    for label in &home_disk_labels {
        home_disk_combo.append_text(label);
    }
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

    let efi_combo = ComboBoxText::new();
    let root_combo = ComboBoxText::new();
    let home_combo = ComboBoxText::new();
    let swap_combo = ComboBoxText::new();
    for combo in [&efi_combo, &root_combo, &home_combo, &swap_combo] {
        for label in &partition_labels {
            combo.append_text(label);
        }
    }
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

    {
        let app_state = state.borrow();
        setup_mode_combo.set_active(Some(match app_state.setup_mode {
            SetupMode::Automatic => 0,
            SetupMode::Manual => 1,
        }));
        home_mode_combo.set_active(Some(match app_state.home_mode {
            HomeMode::OnRoot => 0,
            HomeMode::Separate => 1,
        }));
        home_location_combo.set_active(Some(match app_state.home_location {
            HomeLocation::SameDisk => 0,
            HomeLocation::OtherDisk => 1,
        }));
        swap_mode_combo.set_active(Some(match app_state.swap_mode {
            SwapMode::Partition => 0,
            SwapMode::File => 1,
        }));
        swap_file_entry.set_text(&app_state.swap_file_mb.to_string());
        removable_check.set_active(app_state.removable_media);
        format_efi_check.set_active(app_state.format_efi);
        format_root_check.set_active(app_state.format_root);
        format_home_check.set_active(app_state.format_home);
        format_swap_check.set_active(app_state.format_swap);

        if !app_state.home_disk.is_empty() {
            if let Some(index) = home_disk_paths.iter().position(|disk| disk == &app_state.home_disk) {
                home_disk_combo.set_active(Some(index as u32));
            }
        }
        if !app_state.manual_efi_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_efi_partition)
            {
                efi_combo.set_active(Some(index as u32));
            }
        }
        if !app_state.manual_root_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_root_partition)
            {
                root_combo.set_active(Some(index as u32));
            }
        }
        if !app_state.manual_home_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_home_partition)
            {
                home_combo.set_active(Some(index as u32));
            }
        }
        if !app_state.manual_swap_partition.is_empty() {
            if let Some(index) = partition_paths
                .iter()
                .position(|part| part == &app_state.manual_swap_partition)
            {
                swap_combo.set_active(Some(index as u32));
            }
        }
    }

    let refresh_visibility: Rc<dyn Fn()> = {
        let setup_mode_combo = setup_mode_combo.clone();
        let home_mode_combo = home_mode_combo.clone();
        let home_location_combo = home_location_combo.clone();
        let swap_mode_combo = swap_mode_combo.clone();
        let swap_file_entry = swap_file_entry.clone();
        let home_disk_combo = home_disk_combo.clone();
        let efi_combo = efi_combo.clone();
        let root_combo = root_combo.clone();
        let home_combo = home_combo.clone();
        let swap_combo = swap_combo.clone();
        let format_efi_check = format_efi_check.clone();
        let format_root_check = format_root_check.clone();
        let format_home_check = format_home_check.clone();
        let format_swap_check = format_swap_check.clone();
        Rc::new(move || {
            let manual = setup_mode_combo.active() == Some(1);
            let separate_home = home_mode_combo.active() == Some(1);
            let home_other_disk = home_location_combo.active() == Some(1);
            let swap_file = swap_mode_combo.active() == Some(1);
            let swap_partition = swap_mode_combo.active() == Some(0);

            home_location_combo.set_visible(separate_home);
            home_disk_combo.set_visible(separate_home && home_other_disk);
            swap_file_entry.set_visible(swap_file);

            efi_combo.set_visible(manual);
            root_combo.set_visible(manual);
            home_combo.set_visible(manual && separate_home);
            swap_combo.set_visible(manual && swap_partition);
            format_efi_check.set_visible(manual);
            format_root_check.set_visible(manual);
            format_home_check.set_visible(manual && separate_home);
            format_swap_check.set_visible(manual && swap_partition);
        })
    };

    for combo in [
        setup_mode_combo.clone(),
        home_mode_combo.clone(),
        home_location_combo.clone(),
        swap_mode_combo.clone(),
    ] {
        let refresh_visibility = refresh_visibility.clone();
        combo.connect_changed(move |_| refresh_visibility());
    }
    refresh_visibility();

    {
        let state = state.clone();
        let stack = stack.clone();
        let status_label = status_label.clone();
        let setup_mode_combo = setup_mode_combo.clone();
        let home_mode_combo = home_mode_combo.clone();
        let home_location_combo = home_location_combo.clone();
        let swap_mode_combo = swap_mode_combo.clone();
        let swap_file_entry = swap_file_entry.clone();
        let removable_check = removable_check.clone();
        let home_disk_combo = home_disk_combo.clone();
        let home_disk_paths = home_disk_paths.clone();
        let efi_combo = efi_combo.clone();
        let root_combo = root_combo.clone();
        let home_combo = home_combo.clone();
        let swap_combo = swap_combo.clone();
        let partition_paths = partition_paths.clone();
        let format_efi_check = format_efi_check.clone();
        let format_root_check = format_root_check.clone();
        let format_home_check = format_home_check.clone();
        let format_swap_check = format_swap_check.clone();
        next_btn.connect_clicked(move |_| {
            let selected_home_disk = home_disk_combo
                .active()
                .and_then(|idx| home_disk_paths.get(idx as usize).cloned())
                .unwrap_or_default();
            let selected_efi = efi_combo
                .active()
                .and_then(|idx| partition_paths.get(idx as usize).cloned())
                .unwrap_or_default();
            let selected_root = root_combo
                .active()
                .and_then(|idx| partition_paths.get(idx as usize).cloned())
                .unwrap_or_default();
            let selected_home = home_combo
                .active()
                .and_then(|idx| partition_paths.get(idx as usize).cloned())
                .unwrap_or_default();
            let selected_swap = swap_combo
                .active()
                .and_then(|idx| partition_paths.get(idx as usize).cloned())
                .unwrap_or_default();

            let parsed_swap_file_mb = swap_file_entry.text().trim().parse::<u64>().unwrap_or(0);

            {
                let mut app_state = state.borrow_mut();
                app_state.setup_mode = if setup_mode_combo.active() == Some(1) {
                    SetupMode::Manual
                } else {
                    SetupMode::Automatic
                };
                app_state.home_mode = if home_mode_combo.active() == Some(1) {
                    HomeMode::Separate
                } else {
                    HomeMode::OnRoot
                };
                app_state.home_location = if home_location_combo.active() == Some(1) {
                    HomeLocation::OtherDisk
                } else {
                    HomeLocation::SameDisk
                };
                app_state.swap_mode = if swap_mode_combo.active() == Some(1) {
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

    let setup_label = Label::new(Some("_Setup mode"));
    setup_label.set_use_underline(true);
    setup_label.set_mnemonic_widget(Some(&setup_mode_combo));
    vbox.append(&setup_label);
    vbox.append(&setup_mode_combo);

    let home_mode_label = Label::new(Some("_Home mode"));
    home_mode_label.set_use_underline(true);
    home_mode_label.set_mnemonic_widget(Some(&home_mode_combo));
    vbox.append(&home_mode_label);
    vbox.append(&home_mode_combo);

    let home_location_label = Label::new(Some("Home _location"));
    home_location_label.set_use_underline(true);
    home_location_label.set_mnemonic_widget(Some(&home_location_combo));
    vbox.append(&home_location_label);
    vbox.append(&home_location_combo);

    let home_disk_label = Label::new(Some("Home _disk (if using another disk)"));
    home_disk_label.set_use_underline(true);
    home_disk_label.set_mnemonic_widget(Some(&home_disk_combo));
    vbox.append(&home_disk_label);
    vbox.append(&home_disk_combo);

    let swap_mode_label = Label::new(Some("S_wap mode"));
    swap_mode_label.set_use_underline(true);
    swap_mode_label.set_mnemonic_widget(Some(&swap_mode_combo));
    vbox.append(&swap_mode_label);
    vbox.append(&swap_mode_combo);

    let swap_file_label = build_mnemonic_label("Swap file size (_MB)", &swap_file_entry);
    vbox.append(&swap_file_label);
    vbox.append(&swap_file_entry);
    vbox.append(&removable_check);

    let efi_label = Label::new(Some("Manual _EFI partition"));
    efi_label.set_use_underline(true);
    efi_label.set_mnemonic_widget(Some(&efi_combo));
    vbox.append(&efi_label);
    vbox.append(&efi_combo);

    let root_label = Label::new(Some("Manual _root partition"));
    root_label.set_use_underline(true);
    root_label.set_mnemonic_widget(Some(&root_combo));
    vbox.append(&root_label);
    vbox.append(&root_combo);

    let home_label = Label::new(Some("Manual /_home partition"));
    home_label.set_use_underline(true);
    home_label.set_mnemonic_widget(Some(&home_combo));
    vbox.append(&home_label);
    vbox.append(&home_combo);

    let swap_label = Label::new(Some("Manual s_wap partition"));
    swap_label.set_use_underline(true);
    swap_label.set_mnemonic_widget(Some(&swap_combo));
    vbox.append(&swap_label);
    vbox.append(&swap_combo);

    vbox.append(&format_efi_check);
    vbox.append(&format_root_check);
    vbox.append(&format_home_check);
    vbox.append(&format_swap_check);
    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
