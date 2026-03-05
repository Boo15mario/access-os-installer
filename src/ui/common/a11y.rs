use gtk4::prelude::*;
use gtk4::{self, AccessibleRole, Label, ListBox, ListBoxRow, SelectionMode};

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

pub fn build_list_box(accessible_label: &str, description: &str) -> ListBox {
    let list_box = ListBox::new();
    list_box.set_accessible_role(AccessibleRole::List);
    list_box.set_selection_mode(SelectionMode::Single);
    list_box.set_focusable(true);
    set_accessible_label(&list_box, accessible_label);
    if !description.is_empty() {
        set_accessible_description(&list_box, description);
    }
    list_box
}

pub fn append_list_row(list_box: &ListBox, label_text: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_focusable(true);
    row.set_accessible_role(AccessibleRole::ListItem);
    set_accessible_label(&row, label_text);

    let label = Label::new(Some(label_text));
    label.set_xalign(0.0);
    label.set_wrap(true);
    row.set_child(Some(&label));

    list_box.append(&row);
    row
}

pub fn clear_list_box(list_box: &ListBox) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
}

pub fn select_list_box_index(list_box: &ListBox, index: usize) {
    let Some(row) = list_box.row_at_index(index as i32) else {
        return;
    };
    list_box.select_row(Some(&row));
}

pub fn selected_list_box_index(list_box: &ListBox) -> Option<usize> {
    list_box
        .selected_row()
        .and_then(|row| usize::try_from(row.index()).ok())
}
