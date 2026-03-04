use crate::app::state::SharedState;
use crate::backend::config_engine::DesktopEnv;
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{AccessibleRole, Align, Box, Button, ComboBoxText, Label, Stack};

pub fn build_de_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 3: Select Desktop Environment")
        .margin_bottom(24)
        .build();

    let de_combo = ComboBoxText::new();
    de_combo.set_accessible_role(AccessibleRole::ComboBox);
    for de in DesktopEnv::all() {
        de_combo.append_text(de.label());
    }
    de_combo.set_active(Some(0));

    let description_label = Label::builder()
        .label("")
        .halign(Align::Start)
        .wrap(true)
        .margin_top(8)
        .build();

    let next_btn = Button::builder().label("Next: Mirror Region").build();
    let back_btn = Button::builder().label("Back").build();

    let update_selection = {
        let description_label = description_label.clone();
        let next_btn = next_btn.clone();
        move |index: usize| {
            if let Some(de) = DesktopEnv::from_index(index) {
                let packages = if de.packages().is_empty() {
                    "(none)".to_string()
                } else {
                    de.packages().join(", ")
                };
                let dm = de.display_manager().unwrap_or("None (headless)");
                description_label.set_label(&format!(
                    "{}\nDisplay manager: {}\nPackages: {}",
                    de.description(),
                    dm,
                    packages
                ));
                next_btn.set_sensitive(de.is_available());
            }
        }
    };

    update_selection(0);

    {
        let update_selection = update_selection.clone();
        de_combo.connect_changed(move |combo| {
            if let Some(selected) = combo.active() {
                update_selection(selected as usize);
            }
        });
    }

    {
        let de_combo = de_combo.clone();
        let state = state.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let selected = de_combo.active().unwrap_or(0) as usize;
            if let Some(de) = DesktopEnv::from_index(selected) {
                if !de.is_available() {
                    return;
                }
                state.borrow_mut().desktop_env = Some(de.clone());
                stack.set_visible_child_name("mirror");
            }
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("disk_setup"));
    }

    vbox.append(&title);
    let de_label = Label::new(Some("_Desktop Environment"));
    de_label.set_use_underline(true);
    de_label.set_mnemonic_widget(Some(&de_combo));
    vbox.append(&de_label);
    vbox.append(&de_combo);
    vbox.append(&description_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
