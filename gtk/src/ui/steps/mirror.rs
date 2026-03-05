use crate::app::constants::{KEYMAPS, LOCALES, MIRROR_REGIONS, TIMEZONES};
use crate::app::state::SharedState;
use crate::ui::common::a11y::{apply_button_role, build_mnemonic_label};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{AccessibleRole, Align, Box, Button, CheckButton, Label, ScrolledWindow, Stack};

pub fn build_mirror_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 4: Regional Settings")
        .margin_bottom(16)
        .build();
    let subtitle = Label::builder()
        .label("Choose a mirror region and system language/keyboard defaults.")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let content = Box::new(gtk4::Orientation::Vertical, 12);

    let mirror_field = Box::new(gtk4::Orientation::Vertical, 6);
    let mirror_first_btn = CheckButton::builder().label(MIRROR_REGIONS[0]).build();
    mirror_first_btn.set_accessible_role(AccessibleRole::Radio);
    mirror_first_btn.set_widget_name("a11y-default-focus");
    let mut mirror_btns = vec![mirror_first_btn.clone()];
    for region in &MIRROR_REGIONS[1..] {
        let btn = CheckButton::builder().label(*region).build();
        btn.set_group(Some(&mirror_first_btn));
        btn.set_accessible_role(AccessibleRole::Radio);
        mirror_btns.push(btn);
    }
    // Ensure the group always has a selection, then restore saved state if possible.
    mirror_first_btn.set_active(true);
    if let Some(idx) = MIRROR_REGIONS
        .iter()
        .position(|region| *region == state.borrow().mirror_region)
    {
        mirror_btns[idx].set_active(true);
    }
    let mirror_group = Box::new(gtk4::Orientation::Vertical, 6);
    mirror_group.set_accessible_role(AccessibleRole::RadioGroup);
    for btn in &mirror_btns {
        mirror_group.append(btn);
    }
    mirror_field.append(&build_mnemonic_label("_Mirror Region", &mirror_first_btn));
    mirror_field.append(&mirror_group);

    let tz_field = Box::new(gtk4::Orientation::Vertical, 6);
    let tz_first_btn = CheckButton::builder().label(TIMEZONES[0]).build();
    tz_first_btn.set_accessible_role(AccessibleRole::Radio);
    let mut tz_btns = vec![tz_first_btn.clone()];
    for tz in &TIMEZONES[1..] {
        let btn = CheckButton::builder().label(*tz).build();
        btn.set_group(Some(&tz_first_btn));
        btn.set_accessible_role(AccessibleRole::Radio);
        tz_btns.push(btn);
    }
    tz_first_btn.set_active(true);
    if let Some(idx) = TIMEZONES
        .iter()
        .position(|tz| *tz == state.borrow().timezone)
    {
        tz_btns[idx].set_active(true);
    }
    let tz_group = Box::new(gtk4::Orientation::Vertical, 6);
    tz_group.set_accessible_role(AccessibleRole::RadioGroup);
    for btn in &tz_btns {
        tz_group.append(btn);
    }
    tz_field.append(&build_mnemonic_label("_Timezone", &tz_first_btn));
    tz_field.append(&tz_group);

    let locale_field = Box::new(gtk4::Orientation::Vertical, 6);
    let locale_first_btn = CheckButton::builder().label(LOCALES[0]).build();
    locale_first_btn.set_accessible_role(AccessibleRole::Radio);
    let mut locale_btns = vec![locale_first_btn.clone()];
    for locale in &LOCALES[1..] {
        let btn = CheckButton::builder().label(*locale).build();
        btn.set_group(Some(&locale_first_btn));
        btn.set_accessible_role(AccessibleRole::Radio);
        locale_btns.push(btn);
    }
    locale_first_btn.set_active(true);
    if let Some(idx) = LOCALES
        .iter()
        .position(|locale| *locale == state.borrow().locale)
    {
        locale_btns[idx].set_active(true);
    }
    let locale_group = Box::new(gtk4::Orientation::Vertical, 6);
    locale_group.set_accessible_role(AccessibleRole::RadioGroup);
    for btn in &locale_btns {
        locale_group.append(btn);
    }
    locale_field.append(&build_mnemonic_label("_Locale", &locale_first_btn));
    locale_field.append(&locale_group);

    let keymap_field = Box::new(gtk4::Orientation::Vertical, 6);
    let keymap_first_btn = CheckButton::builder().label(KEYMAPS[0]).build();
    keymap_first_btn.set_accessible_role(AccessibleRole::Radio);
    let mut keymap_btns = vec![keymap_first_btn.clone()];
    for keymap in &KEYMAPS[1..] {
        let btn = CheckButton::builder().label(*keymap).build();
        btn.set_group(Some(&keymap_first_btn));
        btn.set_accessible_role(AccessibleRole::Radio);
        keymap_btns.push(btn);
    }
    keymap_first_btn.set_active(true);
    if let Some(idx) = KEYMAPS
        .iter()
        .position(|keymap| *keymap == state.borrow().keymap)
    {
        keymap_btns[idx].set_active(true);
    }
    let keymap_group = Box::new(gtk4::Orientation::Vertical, 6);
    keymap_group.set_accessible_role(AccessibleRole::RadioGroup);
    for btn in &keymap_btns {
        keymap_group.append(btn);
    }
    keymap_field.append(&build_mnemonic_label("Key_map", &keymap_first_btn));
    keymap_field.append(&keymap_group);

    content.append(&mirror_field);
    content.append(&tz_field);
    content.append(&locale_field);
    content.append(&keymap_field);

    let scroller = ScrolledWindow::builder()
        .child(&content)
        .hexpand(true)
        .vexpand(true)
        .min_content_height(260)
        .build();
    scroller.set_focusable(false);

    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let next_btn = Button::builder().label("Next: User Settings").build();
    let back_btn = Button::builder().label("Back").build();
    apply_button_role(&next_btn);
    apply_button_role(&back_btn);

    {
        let state = state.clone();
        let stack = stack.clone();
        let status_label = status_label.clone();
        let mirror_btns = mirror_btns.clone();
        let tz_btns = tz_btns.clone();
        let locale_btns = locale_btns.clone();
        let keymap_btns = keymap_btns.clone();
        next_btn.connect_clicked(move |_| {
            let selected_region = mirror_btns
                .iter()
                .find(|btn| btn.is_active())
                .and_then(|btn| btn.label())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "Worldwide".to_string());
            let selected_tz = tz_btns
                .iter()
                .find(|btn| btn.is_active())
                .and_then(|btn| btn.label())
                .map(|v| v.to_string())
                .unwrap_or_else(|| TIMEZONES[0].to_string());
            let selected_locale = locale_btns
                .iter()
                .find(|btn| btn.is_active())
                .and_then(|btn| btn.label())
                .map(|v| v.to_string())
                .unwrap_or_else(|| LOCALES[0].to_string());
            let selected_keymap = keymap_btns
                .iter()
                .find(|btn| btn.is_active())
                .and_then(|btn| btn.label())
                .map(|v| v.to_string())
                .unwrap_or_else(|| KEYMAPS[0].to_string());

            {
                let mut s = state.borrow_mut();
                s.mirror_region = selected_region;
                s.timezone = selected_tz;
                s.locale = selected_locale;
                s.keymap = selected_keymap;
            }
            status_label.set_label("");
            stack.set_visible_child_name("settings");
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("desktop_env"));
    }

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&scroller);
    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
