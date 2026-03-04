use crate::app::constants::MIRROR_REGIONS;
use crate::app::state::SharedState;
use crate::ui::common::a11y::apply_button_role;
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, ComboBoxText, Label, Stack};

pub fn build_mirror_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 4: Mirror Region")
        .margin_bottom(24)
        .build();

    let region_combo = ComboBoxText::new();
    region_combo.set_focusable(true);
    for region in MIRROR_REGIONS {
        region_combo.append_text(region);
    }
    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    if let Some(index) = MIRROR_REGIONS
        .iter()
        .position(|region| *region == state.borrow().mirror_region)
    {
        region_combo.set_active(Some(index as u32));
    } else {
        region_combo.set_active(Some(0));
    }

    let next_btn = Button::builder().label("Next: User Settings").build();
    let back_btn = Button::builder().label("Back").build();
    apply_button_role(&next_btn);
    apply_button_role(&back_btn);

    {
        let state = state.clone();
        let stack = stack.clone();
        let status_label = status_label.clone();
        let region_combo = region_combo.clone();
        next_btn.connect_clicked(move |_| {
            let Some(selected) = region_combo.active() else {
                status_label.set_label("Select a mirror region.");
                return;
            };

            let selected_region = MIRROR_REGIONS
                .get(selected as usize)
                .copied()
                .unwrap_or("Worldwide")
                .to_string();
            state.borrow_mut().mirror_region = selected_region;
            status_label.set_label("");
            stack.set_visible_child_name("settings");
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("desktop_env"));
    }

    vbox.append(&title);
    let region_label = Label::new(Some("_Mirror Region"));
    region_label.set_use_underline(true);
    region_label.set_mnemonic_widget(Some(&region_combo));
    vbox.append(&region_label);
    vbox.append(&region_combo);
    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
