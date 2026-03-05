use crate::app::state::SharedState;
use crate::backend::preflight;
use crate::mappers::storage::{format_check_group, preflight_context_from_state};
use crate::ui::common::a11y::apply_button_role;
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Stack};
use std::rc::Rc;

pub fn build_preflight_step(stack: &Stack, state: SharedState) -> (Box, Rc<dyn Fn()>) {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 6: Preflight Checks")
        .margin_bottom(12)
        .build();
    let subtitle = Label::builder()
        .label("Hard blockers must pass before you can continue.")
        .halign(Align::Start)
        .wrap(true)
        .build();

    let hard_checks_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let warning_checks_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let status_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let rerun_btn = Button::builder().label("Re-run Checks").build();
    let continue_btn = Button::builder().label("Next: Review & Confirm").build();
    continue_btn.set_sensitive(false);
    let back_btn = Button::builder().label("Back").build();
    apply_button_role(&rerun_btn);
    apply_button_role(&continue_btn);
    apply_button_role(&back_btn);

    let refresh_preflight: Rc<dyn Fn()> = {
        let state = state.clone();
        let hard_checks_label = hard_checks_label.clone();
        let warning_checks_label = warning_checks_label.clone();
        let status_label = status_label.clone();
        let continue_btn = continue_btn.clone();
        Rc::new(move || {
            let context = {
                let app_state = state.borrow();
                preflight_context_from_state(&app_state)
            };

            let results = preflight::evaluate_checks(&context);
            let hard_fail = preflight::has_hard_fail(&results);

            {
                let mut app_state = state.borrow_mut();
                app_state.preflight_results = results.clone();
            }

            hard_checks_label.set_label(&format_check_group(&results, true));
            warning_checks_label.set_label(&format_check_group(&results, false));

            if hard_fail {
                status_label.set_label("Preflight failed. Resolve hard blockers before continuing.");
                continue_btn.set_sensitive(false);
            } else {
                status_label.set_label("Preflight passed. Continue to review and confirm.");
                continue_btn.set_sensitive(true);
            }
        })
    };

    {
        let refresh_preflight = refresh_preflight.clone();
        rerun_btn.connect_clicked(move |_| refresh_preflight());
    }

    {
        let stack = stack.clone();
        continue_btn.connect_clicked(move |_| stack.set_visible_child_name("review"));
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("settings"));
    }

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&Label::new(Some("Hard blockers")));
    vbox.append(&hard_checks_label);
    vbox.append(&Label::new(Some("Soft warnings")));
    vbox.append(&warning_checks_label);
    vbox.append(&status_label);
    vbox.append(&rerun_btn);
    vbox.append(&continue_btn);
    vbox.append(&back_btn);

    (vbox, refresh_preflight)
}
