use crate::backend::network;
use crate::ui::common::a11y::{
    append_list_row, apply_button_role, apply_textbox_role, build_list_box, build_mnemonic_label,
    clear_list_box, select_list_box_index, selected_list_box_index,
};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, PasswordEntry, ScrolledWindow, Stack};
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_wifi_step(stack: &Stack) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder().label("Wi-Fi Setup").margin_bottom(8).build();
    let subtitle = Label::builder()
        .label("No internet connection detected. Connect to Wi-Fi to continue.")
        .wrap(true)
        .margin_bottom(16)
        .build();

    let ssid_list = build_list_box("Available Networks", "Select a network to connect.");
    let ssid_scroller = ScrolledWindow::builder()
        .child(&ssid_list)
        .hexpand(true)
        .vexpand(true)
        .min_content_height(120)
        .build();
    ssid_scroller.set_focusable(false);

    let ssid_options = Rc::new(RefCell::new(Vec::<String>::new()));

    let pass_entry = PasswordEntry::builder()
        .placeholder_text("Wi-Fi Password")
        .show_peek_icon(true)
        .build();
    apply_textbox_role(&pass_entry);

    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let connect_btn = Button::builder().label("Connect").build();
    let refresh_btn = Button::builder().label("Refresh Networks").build();
    let skip_btn = Button::builder().label("Skip (already connected)").build();
    apply_button_role(&connect_btn);
    apply_button_role(&refresh_btn);
    apply_button_role(&skip_btn);
    connect_btn.set_sensitive(false);

    let rebuild_networks: Rc<dyn Fn()> = {
        let ssid_list = ssid_list.clone();
        let ssid_options = ssid_options.clone();
        let status_label = status_label.clone();
        let connect_btn = connect_btn.clone();
        Rc::new(move || {
            clear_list_box(&ssid_list);
            let scanned = network::scan_wifi();
            *ssid_options.borrow_mut() = scanned.clone();

            if scanned.is_empty() {
                status_label.set_label("No Wi-Fi networks found. Click Refresh Networks.");
                connect_btn.set_sensitive(false);
                let row = append_list_row(&ssid_list, "No networks found");
                row.set_sensitive(false);
                return;
            }

            status_label.set_label("Select a Wi-Fi network.");
            for (idx, ssid) in scanned.iter().enumerate() {
                let row = append_list_row(&ssid_list, ssid);
                if idx == 0 {
                    row.set_widget_name("a11y-default-focus");
                }
            }
            select_list_box_index(&ssid_list, 0);
        })
    };
    rebuild_networks();

    {
        let connect_btn = connect_btn.clone();
        ssid_list.connect_row_selected(move |_, row| {
            connect_btn.set_sensitive(row.is_some());
        });
    }

    {
        let ssid_list = ssid_list.clone();
        let ssid_options = ssid_options.clone();
        let pass_entry = pass_entry.clone();
        let status_label = status_label.clone();
        let stack = stack.clone();
        connect_btn.connect_clicked(move |_| {
            let Some(selected) = selected_list_box_index(&ssid_list) else {
                status_label.set_label("Please select a network.");
                return;
            };
            let Some(ssid) = ssid_options.borrow().get(selected).cloned() else {
                status_label.set_label("Invalid network selection.");
                return;
            };

            let password = pass_entry.text().to_string();
            status_label.set_label("Connecting...");
            match network::connect_wifi(&ssid, &password) {
                Ok(_) => {
                    if network::check_connectivity() {
                        stack.set_visible_child_name("disk");
                    } else {
                        status_label
                            .set_label("Connected to Wi-Fi but no internet. Check password or network.");
                    }
                }
                Err(e) => status_label.set_label(&format!("Failed: {}", e)),
            }
        });
    }

    {
        let rebuild_networks = rebuild_networks.clone();
        let status_label = status_label.clone();
        refresh_btn.connect_clicked(move |_| {
            status_label.set_label("Scanning...");
            rebuild_networks();
        });
    }

    {
        let stack = stack.clone();
        skip_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("disk");
        });
    }

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&build_mnemonic_label("_Available Networks", &ssid_list));
    vbox.append(&ssid_scroller);
    vbox.append(&build_mnemonic_label("_Password", &pass_entry));
    vbox.append(&pass_entry);
    vbox.append(&status_label);
    vbox.append(&connect_btn);
    vbox.append(&refresh_btn);
    vbox.append(&skip_btn);
    vbox
}
