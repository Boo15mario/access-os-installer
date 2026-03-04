use gtk4::prelude::*;
use gtk4::{self, AccessibleRole, Label};

pub fn set_accessible_label<A: IsA<gtk4::Accessible>>(widget: &A, label: &str) {
    widget.update_property(&[gtk4::accessible::Property::Label(label)]);
}

pub fn set_accessible_description<A: IsA<gtk4::Accessible>>(widget: &A, description: &str) {
    widget.update_property(&[gtk4::accessible::Property::Description(description)]);
}

pub fn apply_button_role<W: IsA<gtk4::Accessible>>(widget: &W) {
    widget.set_accessible_role(AccessibleRole::Button);
}

pub fn apply_textbox_role<W: IsA<gtk4::Accessible>>(widget: &W) {
    widget.set_accessible_role(AccessibleRole::TextBox);
}

pub fn build_mnemonic_label<W: IsA<gtk4::Widget>>(label: &str, widget: &W) -> Label {
    let label = Label::new(Some(label));
    label.set_use_underline(true);
    label.set_mnemonic_widget(Some(widget));
    label
}
