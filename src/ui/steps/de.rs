use crate::app::state::SharedState;
use crate::backend::config_engine::DesktopEnv;
use crate::ui::common::a11y::{
    append_list_row, apply_button_role, build_list_box, build_mnemonic_label,
    select_list_box_index, selected_list_box_index, set_accessible_description,
};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Stack};

pub fn build_de_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 3: Select Desktop Environment")
        .margin_bottom(24)
        .build();

    let de_list = build_list_box(
        "Desktop Environment",
        "Select a desktop environment.",
    );
    for (idx, de) in DesktopEnv::all().iter().enumerate() {
        let row = append_list_row(&de_list, de.label());
        set_accessible_description(&row, de.description());
        if idx == 0 {
            row.set_widget_name("a11y-default-focus");
        }
    }

    let description_label = Label::builder()
        .label("")
        .halign(Align::Start)
        .wrap(true)
        .margin_top(8)
        .build();

    let next_btn = Button::builder().label("Next: Regional Settings").build();
    let back_btn = Button::builder().label("Back").build();
    apply_button_role(&next_btn);
    apply_button_role(&back_btn);

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

    let initial_index = state
        .borrow()
        .desktop_env
        .as_ref()
        .and_then(|selected| DesktopEnv::all().iter().position(|de| de == selected))
        .unwrap_or(0);
    select_list_box_index(&de_list, initial_index);
    update_selection(initial_index);

    {
        let update_selection = update_selection.clone();
        de_list.connect_row_selected(move |_, row| {
            let Some(row) = row else {
                return;
            };
            let Some(index) = usize::try_from(row.index()).ok() else {
                return;
            };
            update_selection(index);
        });
    }

    {
        let de_list = de_list.clone();
        let state = state.clone();
        let stack = stack.clone();
        next_btn.connect_clicked(move |_| {
            let selected = selected_list_box_index(&de_list).unwrap_or(0);
            let Some(de) = DesktopEnv::from_index(selected) else {
                return;
            };
            if !de.is_available() {
                return;
            }
            state.borrow_mut().desktop_env = Some(de.clone());
            stack.set_visible_child_name("mirror");
        });
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("disk_setup"));
    }

    vbox.append(&title);
    vbox.append(&build_mnemonic_label("_Desktop Environment", &de_list));
    vbox.append(&de_list);
    vbox.append(&description_label);
    vbox.append(&next_btn);
    vbox.append(&back_btn);
    vbox
}
