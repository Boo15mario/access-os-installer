use crate::backend::network;
use crate::ui::common::a11y::apply_button_role;
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Stack};

pub fn build_welcome_step(stack: &Stack) -> Box {
    let vbox = padded_box(16, 48);
    vbox.set_halign(Align::Center);
    vbox.set_valign(Align::Center);

    let title = Label::builder()
        .label("Welcome to access-OS Installer")
        .margin_bottom(12)
        .build();
    title.set_markup("<span font='28' weight='bold'>Welcome to access-OS Installer</span>");

    let subtitle = Label::builder()
        .label("This installer will guide you through setting up access-OS on your machine.")
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .margin_bottom(24)
        .build();

    let start_btn = Button::builder().label("Get Started").margin_top(16).build();
    apply_button_role(&start_btn);

    {
        let stack = stack.clone();
        start_btn.connect_clicked(move |_| {
            if network::check_connectivity() {
                stack.set_visible_child_name("disk");
            } else {
                stack.set_visible_child_name("wifi");
            }
        });
    }

    vbox.append(&title);
    vbox.append(&subtitle);
    vbox.append(&start_btn);
    vbox
}
