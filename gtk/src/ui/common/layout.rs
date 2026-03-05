use gtk4::{Box, Orientation};

pub fn padded_box(spacing: i32, margin: i32) -> Box {
    Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(spacing)
        .margin_top(margin)
        .margin_bottom(margin)
        .margin_start(margin)
        .margin_end(margin)
        .build()
}
