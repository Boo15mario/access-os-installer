use gtk4::Label;

pub fn append_log_line(log_label: &Label, line: &str) {
    let current = log_label.label().to_string();
    if current.is_empty() {
        log_label.set_label(line);
    } else {
        log_label.set_label(&format!("{}\n{}", current, line));
    }
}
