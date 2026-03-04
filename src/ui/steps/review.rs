use crate::app::state::SharedState;
use crate::backend::preflight::CheckStatus;
use crate::backend::storage_plan::{format_destructive_plan, resolve_layout};
use crate::mappers::storage::{format_review_summary, format_warning_lines, storage_selection_from_state};
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, CheckButton, Label, Stack};
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_review_step(stack: &Stack, state: SharedState) -> (Box, Rc<dyn Fn()>) {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 7: Review & Confirm")
        .margin_bottom(12)
        .build();

    let summary_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let warnings_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let destructive_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let countdown_label = Label::builder().label("").halign(Align::Start).wrap(true).build();

    let acknowledge_warning_btn = CheckButton::builder()
        .label("I understand these warnings and want to continue.")
        .build();
    acknowledge_warning_btn.set_visible(false);

    let continue_btn = Button::builder().label("Next: Start Installation").build();
    let back_btn = Button::builder().label("Back").build();

    let layout_valid = Rc::new(RefCell::new(false));
    let countdown_ready = Rc::new(RefCell::new(false));
    let countdown_generation = Rc::new(RefCell::new(0_u64));
    let update_continue: Rc<dyn Fn()> = {
        let state = state.clone();
        let continue_btn = continue_btn.clone();
        let acknowledge_warning_btn = acknowledge_warning_btn.clone();
        let layout_valid = layout_valid.clone();
        let countdown_ready = countdown_ready.clone();
        Rc::new(move || {
            let has_warnings = state
                .borrow()
                .preflight_results
                .iter()
                .any(|result| result.status == CheckStatus::Warn);
            let allow = *layout_valid.borrow()
                && *countdown_ready.borrow()
                && (!has_warnings || acknowledge_warning_btn.is_active());
            continue_btn.set_sensitive(allow);
        })
    };

    {
        let update_continue = update_continue.clone();
        acknowledge_warning_btn.connect_toggled(move |checkbox| {
            let _ = checkbox;
            update_continue();
        });
    }

    let refresh_review: Rc<dyn Fn()> = {
        let state = state.clone();
        let summary_label = summary_label.clone();
        let warnings_label = warnings_label.clone();
        let destructive_label = destructive_label.clone();
        let countdown_label = countdown_label.clone();
        let acknowledge_warning_btn = acknowledge_warning_btn.clone();
        let update_continue = update_continue.clone();
        let layout_valid = layout_valid.clone();
        let countdown_ready = countdown_ready.clone();
        let countdown_generation = countdown_generation.clone();
        Rc::new(move || {
            let app_state = state.borrow();
            summary_label.set_label(&format_review_summary(&app_state));
            warnings_label.set_label(&format_warning_lines(&app_state.preflight_results));

            let has_warnings = app_state
                .preflight_results
                .iter()
                .any(|result| result.status == CheckStatus::Warn);
            drop(app_state);

            if has_warnings {
                acknowledge_warning_btn.set_visible(true);
                acknowledge_warning_btn.set_active(false);
            } else {
                acknowledge_warning_btn.set_visible(false);
            }

            let selection = {
                let app_state = state.borrow();
                storage_selection_from_state(&app_state)
            };
            match resolve_layout(&selection) {
                Ok(layout) => {
                    *layout_valid.borrow_mut() = true;
                    destructive_label.set_label(&format_destructive_plan(&layout));
                    state.borrow_mut().resolved_layout = Some(layout);

                    let token = {
                        let mut generation = countdown_generation.borrow_mut();
                        *generation += 1;
                        *generation
                    };
                    *countdown_ready.borrow_mut() = false;
                    countdown_label.set_label("Confirm enabled in 5 seconds...");

                    let remaining = Rc::new(RefCell::new(5_u32));
                    let countdown_label_tick = countdown_label.clone();
                    let countdown_ready_tick = countdown_ready.clone();
                    let update_continue_tick = update_continue.clone();
                    let countdown_generation_tick = countdown_generation.clone();
                    gtk4::glib::timeout_add_seconds_local(1, move || {
                        if *countdown_generation_tick.borrow() != token {
                            return gtk4::glib::ControlFlow::Break;
                        }

                        let mut seconds_left = remaining.borrow_mut();
                        if *seconds_left <= 1 {
                            countdown_label_tick.set_label(
                                "Destructive actions unlocked. You can start installation.",
                            );
                            *countdown_ready_tick.borrow_mut() = true;
                            update_continue_tick();
                            gtk4::glib::ControlFlow::Break
                        } else {
                            *seconds_left -= 1;
                            countdown_label_tick
                                .set_label(&format!("Confirm enabled in {} seconds...", *seconds_left));
                            gtk4::glib::ControlFlow::Continue
                        }
                    });
                }
                Err(e) => {
                    *layout_valid.borrow_mut() = false;
                    *countdown_ready.borrow_mut() = false;
                    state.borrow_mut().resolved_layout = None;
                    destructive_label.set_label(&format!("Storage plan error: {}", e));
                    countdown_label.set_label("Fix storage plan errors to continue.");
                }
            }

            update_continue();
        })
    };

    {
        let stack = stack.clone();
        continue_btn.connect_clicked(move |_| stack.set_visible_child_name("install"));
    }

    {
        let stack = stack.clone();
        back_btn.connect_clicked(move |_| stack.set_visible_child_name("preflight"));
    }

    vbox.append(&title);
    vbox.append(&Label::new(Some("Selected Configuration")));
    vbox.append(&summary_label);
    vbox.append(&Label::new(Some("Warnings")));
    vbox.append(&warnings_label);
    vbox.append(&Label::new(Some("Destructive Actions")));
    vbox.append(&destructive_label);
    vbox.append(&countdown_label);
    vbox.append(&acknowledge_warning_btn);
    vbox.append(&continue_btn);
    vbox.append(&back_btn);

    (vbox, refresh_review)
}
