use crate::backend::network;
use crate::ui::common::a11y::{
    apply_button_role, apply_textbox_role, set_accessible_description, set_accessible_label,
};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, ComboBoxText, Label, PasswordEntry, Stack};

pub fn build_wifi_step(stack: &Stack) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder().label("Wi-Fi Setup").margin_bottom(8).build();
    let subtitle = Label::builder()
        .label("No internet connection detected. Connect to Wi-Fi to continue.")
        .wrap(true)
        .margin_bottom(16)
        .build();

    let ssid_combo = ComboBoxText::new();
    set_accessible_label(&ssid_combo, "Available Networks");
    set_accessible_description(&ssid_combo, "Use arrow keys to choose a network.");
    for ssid in network::scan_wifi() {
        ssid_combo.append_text(&ssid);
    }
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

    {
        let ssid_combo = ssid_combo.clone();
        let pass_entry = pass_entry.clone();
        let status_label = status_label.clone();
        let stack = stack.clone();
        connect_btn.connect_clicked(move |_| {
            let ssid = match ssid_combo.active_text() {
                Some(value) => value.to_string(),
                None => {
                    status_label.set_label("Please select a network.");
                    return;
                }
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
        let ssid_combo = ssid_combo.clone();
        let status_label = status_label.clone();
        refresh_btn.connect_clicked(move |_| {
            status_label.set_label("Scanning...");
            ssid_combo.remove_all();
            for ssid in network::scan_wifi() {
                ssid_combo.append_text(&ssid);
            }
            status_label.set_label("Scan complete.");
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
    let networks_label = Label::new(Some("_Available Networks"));
    networks_label.set_use_underline(true);
    networks_label.set_mnemonic_widget(Some(&ssid_combo));
    vbox.append(&networks_label);
    vbox.append(&ssid_combo);
    let password_label = Label::new(Some("_Password"));
    password_label.set_use_underline(true);
    password_label.set_mnemonic_widget(Some(&pass_entry));
    vbox.append(&password_label);
    vbox.append(&pass_entry);
    vbox.append(&status_label);
    vbox.append(&connect_btn);
    vbox.append(&refresh_btn);
    vbox.append(&skip_btn);
    vbox
}
