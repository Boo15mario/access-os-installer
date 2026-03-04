use crate::app::state::SharedState;
use crate::backend::config_engine::{DesktopEnv, KernelVariant};
use crate::ui::common::a11y::{apply_button_role, apply_textbox_role, build_mnemonic_label};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, CheckButton, ComboBoxText, Entry, Label, PasswordEntry, Stack,
};
use std::rc::Rc;

pub fn build_settings_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 5: User Settings")
        .margin_bottom(24)
        .build();

    let hostname_entry = Entry::builder()
        .placeholder_text("Hostname (e.g. my-pc)")
        .build();
    let user_entry = Entry::builder().placeholder_text("Username").build();
    let pass_entry = PasswordEntry::builder()
        .placeholder_text("Password")
        .show_peek_icon(true)
        .build();
    let tz_entry = Entry::builder().text("America/Chicago").build();
    let locale_entry = Entry::builder().text("en_US.UTF-8").build();
    let keymap_entry = Entry::builder().text("us").build();
    apply_textbox_role(&hostname_entry);
    apply_textbox_role(&user_entry);
    apply_textbox_role(&pass_entry);
    apply_textbox_role(&tz_entry);
    apply_textbox_role(&locale_entry);
    apply_textbox_role(&keymap_entry);

    let kernel_combo = ComboBoxText::new();
    for kernel in KernelVariant::all() {
        kernel_combo.append_text(kernel.label());
    }
    kernel_combo.set_active(Some(0));

    let kernel_desc = Label::builder()
        .label(KernelVariant::Standard.description())
        .halign(Align::Start)
        .wrap(true)
        .build();
    {
        let kernel_desc = kernel_desc.clone();
        kernel_combo.connect_changed(move |combo| {
            if let Some(selected) = combo.active() {
                if let Some(k) = KernelVariant::from_index(selected as usize) {
                    kernel_desc.set_label(k.description());
                }
            }
        });
    }

    let nvidia_check = CheckButton::builder()
        .label("Install Nvidia drivers (nvidia-dkms)")
        .build();
    let nvidia_note = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let next_btn = Button::builder().label("Next: Run Preflight Checks").build();
    let back_btn = Button::builder().label("Back").build();
    apply_button_role(&next_btn);
    apply_button_role(&back_btn);

    let refresh_nvidia_toggle: Rc<dyn Fn()> = {
        let state = state.clone();
        let nvidia_check = nvidia_check.clone();
        let nvidia_note = nvidia_note.clone();
        Rc::new(move || {
            let is_server = matches!(state.borrow().desktop_env, Some(DesktopEnv::Server));
            if is_server {
                nvidia_check.set_active(false);
                nvidia_check.set_sensitive(false);
                nvidia_note
                    .set_label("Nvidia drivers are disabled for the Server (Headless) profile.");
            } else {
                nvidia_check.set_sensitive(true);
                nvidia_note.set_label("");
            }
        })
    };
    refresh_nvidia_toggle();
    {
        let stack = stack.clone();
        let refresh_nvidia_toggle = refresh_nvidia_toggle.clone();
        stack.connect_visible_child_name_notify(move |stack| {
            if stack.visible_child_name().as_deref() == Some("settings") {
                refresh_nvidia_toggle();
            }
        });
    }

    {
        let state = state.clone();
        let status_label = status_label.clone();
        let hostname_entry = hostname_entry.clone();
        let user_entry = user_entry.clone();
        let pass_entry = pass_entry.clone();
        let tz_entry = tz_entry.clone();
        let locale_entry = locale_entry.clone();
        let keymap_entry = keymap_entry.clone();
        let kernel_combo = kernel_combo.clone();
        let nvidia_check = nvidia_check.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let hostname = hostname_entry.text().trim().to_string();
            if hostname.is_empty() {
                status_label.set_label("Enter a hostname.");
                return;
            }

            let username = user_entry.text().trim().to_string();
            if username.is_empty() {
                status_label.set_label("Enter a username.");
                return;
            }

            let password = pass_entry.text().to_string();
            if password.is_empty() {
                status_label.set_label("Enter a password.");
                return;
            }
            pass_entry.set_text("");

            let timezone = tz_entry.text().trim().to_string();
            if timezone.is_empty() {
                status_label.set_label("Enter a timezone.");
                return;
            }

            let locale = locale_entry.text().trim().to_string();
            if locale.is_empty() {
                status_label.set_label("Enter a locale.");
                return;
            }

            let keymap = keymap_entry.text().trim().to_string();
            if keymap.is_empty() {
                status_label.set_label("Enter a keymap.");
                return;
            }

            let kernel = KernelVariant::from_index(kernel_combo.active().unwrap_or(0) as usize)
                .cloned()
                .unwrap_or(KernelVariant::Standard);
            let server_profile = matches!(state.borrow().desktop_env, Some(DesktopEnv::Server));

            let mut s = state.borrow_mut();
            s.hostname = hostname;
            s.username = username;
            s.password = password;
            s.timezone = timezone;
            s.locale = locale;
            s.keymap = keymap;
            s.kernel = kernel;
            s.nvidia = !server_profile && nvidia_check.is_active();
            status_label.set_label("");
            stack.set_visible_child_name("preflight");
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("mirror"));
    }

    vbox.append(&title);
    let hostname_label = build_mnemonic_label("_Hostname", &hostname_entry);
    vbox.append(&hostname_label);
    vbox.append(&hostname_entry);
    let username_label = build_mnemonic_label("_Username", &user_entry);
    vbox.append(&username_label);
    vbox.append(&user_entry);
    let password_label = build_mnemonic_label("_Password", &pass_entry);
    vbox.append(&password_label);
    vbox.append(&pass_entry);
    let timezone_label = build_mnemonic_label("_Timezone", &tz_entry);
    vbox.append(&timezone_label);
    vbox.append(&tz_entry);
    let locale_label = build_mnemonic_label("_Locale", &locale_entry);
    vbox.append(&locale_label);
    vbox.append(&locale_entry);
    let keymap_label = build_mnemonic_label("Key_map", &keymap_entry);
    vbox.append(&keymap_label);
    vbox.append(&keymap_entry);
    let kernel_label = Label::new(Some("_Kernel"));
    kernel_label.set_use_underline(true);
    kernel_label.set_mnemonic_widget(Some(&kernel_combo));
    vbox.append(&kernel_label);
    vbox.append(&kernel_combo);
    vbox.append(&kernel_desc);
    vbox.append(&nvidia_check);
    vbox.append(&nvidia_note);
    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
