use crate::app::constants::MIRROR_REGIONS;
use crate::app::state::SharedState;
use crate::ui::common::a11y::{
    append_list_row, apply_button_role, build_list_box, build_mnemonic_label,
    select_list_box_index, selected_list_box_index,
};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Stack};

pub fn build_mirror_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 4: Mirror Region")
        .margin_bottom(24)
        .build();

    let region_list = build_list_box("Mirror Region", "Use arrow keys to choose a region.");
    for (idx, region) in MIRROR_REGIONS.iter().enumerate() {
        let row = append_list_row(&region_list, region);
        if idx == 0 {
            row.set_widget_name("a11y-default-focus");
        }
    }

    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let initial_index = MIRROR_REGIONS
        .iter()
        .position(|region| *region == state.borrow().mirror_region)
        .unwrap_or(0);
    select_list_box_index(&region_list, initial_index);

    let next_btn = Button::builder().label("Next: User Settings").build();
    let back_btn = Button::builder().label("Back").build();
    apply_button_role(&next_btn);
    apply_button_role(&back_btn);

    {
        let state = state.clone();
        let stack = stack.clone();
        let status_label = status_label.clone();
        let region_list = region_list.clone();
        next_btn.connect_clicked(move |_| {
            let Some(selected) = selected_list_box_index(&region_list) else {
                status_label.set_label("Select a mirror region.");
                return;
            };

            let selected_region = MIRROR_REGIONS
                .get(selected)
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
    vbox.append(&build_mnemonic_label("_Mirror Region", &region_list));
    vbox.append(&region_list);
    vbox.append(&status_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}

