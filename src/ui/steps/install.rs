use crate::app::constants::DOTFILES_REPO_URL;
use crate::app::state::SharedState;
use crate::backend;
use crate::backend::config_engine::DesktopEnv;
use crate::mappers::storage::storage_selection_from_state;
use crate::services::log::append_log_line;
use crate::services::mirror::apply_mirror_region;
use crate::services::mount::prepare_install_targets;
use crate::ui::common::a11y::apply_button_role;
use crate::ui::common::layout::padded_box;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, ComboBoxText, Label, Stack};
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_install_step(stack: &Stack, state: SharedState) -> Box {
    let vbox = padded_box(12, 24);
    let title = Label::builder()
        .label("Step 8: Installation Progress")
        .margin_bottom(24)
        .build();
    let fs_combo = ComboBoxText::new();
    fs_combo.append_text("xfs");
    fs_combo.append_text("ext4");
    if state.borrow().fs_type == "ext4" {
        fs_combo.set_active(Some(1));
    } else {
        fs_combo.set_active(Some(0));
    }
    let swap_hint = Label::builder()
        .label(&format!("Suggested swap size: {} GiB", state.borrow().swap_gb))
        .halign(Align::Start)
        .wrap(true)
        .build();
    let progress_label = Label::builder()
        .label("Ready to install...")
        .halign(Align::Start)
        .wrap(true)
        .build();
    let log_label = Label::builder().label("").halign(Align::Start).wrap(true).build();
    let start_btn = Button::builder().label("Start Installation").build();
    let retry_pacstrap_btn = Button::builder().label("Retry pacstrap").build();
    let retry_config_btn = Button::builder().label("Retry configuration").build();
    apply_button_role(&start_btn);
    apply_button_role(&retry_pacstrap_btn);
    apply_button_role(&retry_config_btn);
    retry_pacstrap_btn.set_visible(false);
    retry_config_btn.set_visible(false);
    let install_password = Rc::new(RefCell::new(None::<String>));

    {
        let state = state.clone();
        fs_combo.connect_changed(move |combo| {
            let fs_type = combo
                .active_text()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "xfs".to_string());
            state.borrow_mut().fs_type = fs_type;
        });
    }

    {
        let stack = stack.clone();
        let progress_label = progress_label.clone();
        let log_label = log_label.clone();
        let retry_pacstrap_btn = retry_pacstrap_btn.clone();
        let retry_config_btn = retry_config_btn.clone();
        let install_password = install_password.clone();
        let state = state.clone();
        start_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            retry_pacstrap_btn.set_visible(false);
            retry_config_btn.set_visible(false);
            log_label.set_label("");

            let s = state.borrow();
            let selection = storage_selection_from_state(&s);
            let fs_type = s.fs_type.clone();
            let username = s.username.clone();
            let password = s.password.clone();
            let hostname = s.hostname.clone();
            let timezone = s.timezone.clone();
            let locale = s.locale.clone();
            let keymap = s.keymap.clone();
            let mirror_region = s.mirror_region.clone();
            let desktop_env = s.desktop_env.clone();
            let kernel = s.kernel.clone();
            let nvidia = s.nvidia;
            let removable_media = s.removable_media;

            drop(s);

            let layout = match crate::backend::storage_plan::resolve_layout(&selection) {
                Ok(layout) => layout,
                Err(e) => {
                    progress_label.set_label(&format!("Installation failed: invalid disk setup: {}", e));
                    append_log_line(&log_label, &format!("FAIL: invalid disk setup: {}", e));
                    btn.set_sensitive(true);
                    return;
                }
            };
            let root_partition = layout.root_partition.clone();

            if fs_type != "xfs" && fs_type != "ext4" {
                progress_label.set_label("Installation failed: unsupported filesystem selection.");
                append_log_line(&log_label, &format!("FAIL: unsupported filesystem '{}'.", fs_type));
                btn.set_sensitive(true);
                return;
            }

            if password.is_empty() {
                progress_label.set_label(
                    "Installation failed: password is empty. Go back and re-enter it.",
                );
                append_log_line(
                    &log_label,
                    "FAIL: missing password in install state; return to user settings.",
                );
                btn.set_sensitive(true);
                return;
            }

            let Some(de) = desktop_env else {
                progress_label.set_label("Installation failed: no desktop environment selected.");
                append_log_line(&log_label, "FAIL: no desktop environment selected.");
                btn.set_sensitive(true);
                return;
            };

            *install_password.borrow_mut() = Some(password.clone());
            state.borrow_mut().resolved_layout = Some(layout.clone());

            progress_label.set_label("Partitioning and formatting...");
            append_log_line(
                &log_label,
                &format!(
                    "INFO: applying storage layout (mode={:?}, fs={}).",
                    layout.setup_mode, fs_type
                ),
            );
            if let Err(e) = prepare_install_targets(&layout) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }

            progress_label.set_label("Staging system config...");
            append_log_line(
                &log_label,
                &format!(
                    "INFO: cloning system config repo to /access-os-config from '{}'.",
                    DOTFILES_REPO_URL
                ),
            );
            if let Err(e) = backend::install_worker::stage_system_config_repo(DOTFILES_REPO_URL) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }
            append_log_line(
                &log_label,
                "INFO: applying staged system config into /mnt before pacstrap.",
            );
            if let Err(e) = backend::install_worker::overlay_staged_config_to_target() {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }
            append_log_line(
                &log_label,
                "SUCCESS: staged system config copied into /mnt.",
            );

            append_log_line(
                &log_label,
                &format!("INFO: applying mirror region '{}'.", mirror_region),
            );
            if let Err(e) = apply_mirror_region(&mirror_region) {
                append_log_line(
                    &log_label,
                    &format!("WARN: mirror region apply failed (non-fatal): {}. Continuing.", e),
                );
            } else {
                append_log_line(&log_label, "SUCCESS: mirror region applied.");
            }

            progress_label.set_label("Installing base system (pacstrap)...");
            append_log_line(
                &log_label,
                &format!("INFO: running pacstrap with {} packages.", de.label()),
            );

            let install_config = backend::install_worker::InstallConfig {
                username: username.clone(),
                password: password.clone(),
                hostname: hostname.clone(),
                timezone: timezone.clone(),
                locale: locale.clone(),
                keymap: keymap.clone(),
                desktop_env: de.clone(),
                kernel: kernel.clone(),
                nvidia,
                removable_media,
            };

            if let Err(e) = backend::install_worker::run_pacstrap(&install_config) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                retry_pacstrap_btn.set_visible(true);
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: pacstrap completed.");

            if let Err(e) = backend::disk_manager::setup_swap_file(&layout) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                retry_config_btn.set_visible(true);
                btn.set_sensitive(true);
                return;
            }
            progress_label.set_label("Generating fstab...");
            append_log_line(&log_label, "INFO: generating fstab.");
            if let Err(e) = backend::install_worker::generate_fstab() {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                retry_config_btn.set_visible(true);
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: fstab generated.");

            progress_label.set_label("Configuring system...");
            append_log_line(&log_label, "INFO: configuring timezone, locale, bootloader, user.");
            if let Err(e) = backend::install_worker::configure_system(&install_config, &root_partition) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                retry_config_btn.set_visible(true);
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: system configured.");

            if de == DesktopEnv::Gnome {
                progress_label.set_label("Configuring GNOME...");
                append_log_line(&log_label, "INFO: applying GNOME theme and extensions.");
                if let Err(e) = backend::install_worker::configure_gnome(&username) {
                    append_log_line(&log_label, &format!("WARN: GNOME config failed (non-fatal): {}", e));
                } else {
                    append_log_line(&log_label, "SUCCESS: GNOME configured.");
                }
            }

            state.borrow_mut().password.clear();

            append_log_line(&log_label, "SUCCESS: installation finished.");
            *install_password.borrow_mut() = None;
            progress_label.set_label("Installation complete. Opening completion options...");
            stack.set_visible_child_name("complete");
        });
    }

    {
        let stack = stack.clone();
        let progress_label = progress_label.clone();
        let log_label = log_label.clone();
        let retry_config_btn = retry_config_btn.clone();
        let install_password = install_password.clone();
        let state = state.clone();
        retry_pacstrap_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            retry_config_btn.set_visible(false);

            let s = state.borrow();
            let username = s.username.clone();
            let hostname = s.hostname.clone();
            let timezone = s.timezone.clone();
            let locale = s.locale.clone();
            let keymap = s.keymap.clone();
            let mirror_region = s.mirror_region.clone();
            let desktop_env = s.desktop_env.clone();
            let kernel = s.kernel.clone();
            let nvidia = s.nvidia;
            let removable_media = s.removable_media;
            let layout = s.resolved_layout.clone();

            drop(s);

            let Some(layout) = layout else {
                progress_label.set_label(
                    "Installation failed: storage layout is unavailable. Return to review.",
                );
                append_log_line(&log_label, "FAIL: retry requested without resolved storage layout.");
                btn.set_sensitive(true);
                return;
            };
            let root_partition = layout.root_partition.clone();

            let Some(password) = install_password.borrow().clone() else {
                progress_label.set_label(
                    "Installation failed: password is unavailable. Return to user settings.",
                );
                append_log_line(&log_label, "FAIL: retry requested but no password is cached.");
                btn.set_sensitive(true);
                return;
            };

            let Some(de) = desktop_env else {
                progress_label.set_label("Installation failed: no desktop environment selected.");
                btn.set_sensitive(true);
                return;
            };

            let install_config = backend::install_worker::InstallConfig {
                username: username.clone(),
                password: password.clone(),
                hostname,
                timezone,
                locale,
                keymap,
                desktop_env: de,
                kernel,
                nvidia,
                removable_media,
            };

            progress_label.set_label("Refreshing staged system config...");
            append_log_line(
                &log_label,
                &format!(
                    "INFO: refreshing staged config repo at /access-os-config from '{}'.",
                    DOTFILES_REPO_URL
                ),
            );
            if let Err(e) = backend::install_worker::stage_system_config_repo(DOTFILES_REPO_URL) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }
            if let Err(e) = backend::install_worker::overlay_staged_config_to_target() {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }
            append_log_line(
                &log_label,
                "SUCCESS: staged system config refreshed for pacstrap retry.",
            );

            append_log_line(
                &log_label,
                &format!("INFO: applying mirror region '{}'.", mirror_region),
            );
            if let Err(e) = apply_mirror_region(&mirror_region) {
                append_log_line(
                    &log_label,
                    &format!("WARN: mirror region apply failed (non-fatal): {}. Continuing.", e),
                );
            } else {
                append_log_line(&log_label, "SUCCESS: mirror region applied.");
            }

            progress_label.set_label("Retrying pacstrap...");
            append_log_line(&log_label, "INFO: retrying pacstrap.");
            if let Err(e) = backend::install_worker::run_pacstrap(&install_config) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: pacstrap completed.");

            if let Err(e) = backend::disk_manager::setup_swap_file(&layout) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                retry_config_btn.set_visible(true);
                btn.set_sensitive(true);
                return;
            }

            progress_label.set_label("Generating fstab...");
            if let Err(e) = backend::install_worker::generate_fstab() {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                retry_config_btn.set_visible(true);
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: fstab generated.");

            progress_label.set_label("Configuring system...");
            if let Err(e) = backend::install_worker::configure_system(&install_config, &root_partition) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                retry_config_btn.set_visible(true);
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: system configured.");

            if install_config.desktop_env == DesktopEnv::Gnome {
                progress_label.set_label("Configuring GNOME...");
                append_log_line(&log_label, "INFO: applying GNOME theme and extensions.");
                if let Err(e) = backend::install_worker::configure_gnome(&username) {
                    append_log_line(&log_label, &format!("WARN: GNOME config failed (non-fatal): {}", e));
                } else {
                    append_log_line(&log_label, "SUCCESS: GNOME configured.");
                }
            }

            state.borrow_mut().password.clear();

            append_log_line(&log_label, "SUCCESS: installation finished after retry.");
            *install_password.borrow_mut() = None;
            progress_label.set_label("Installation complete. Opening completion options...");
            stack.set_visible_child_name("complete");
        });
    }

    {
        let stack = stack.clone();
        let progress_label = progress_label.clone();
        let log_label = log_label.clone();
        let install_password = install_password.clone();
        let state = state.clone();
        retry_config_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);

            let s = state.borrow();
            let username = s.username.clone();
            let hostname = s.hostname.clone();
            let timezone = s.timezone.clone();
            let locale = s.locale.clone();
            let keymap = s.keymap.clone();
            let desktop_env = s.desktop_env.clone();
            let kernel = s.kernel.clone();
            let nvidia = s.nvidia;
            let removable_media = s.removable_media;
            let layout = s.resolved_layout.clone();

            drop(s);

            let Some(layout) = layout else {
                progress_label.set_label(
                    "Installation failed: storage layout is unavailable. Return to review.",
                );
                append_log_line(
                    &log_label,
                    "FAIL: config retry requested without resolved storage layout.",
                );
                btn.set_sensitive(true);
                return;
            };
            let root_partition = layout.root_partition.clone();

            let Some(password) = install_password.borrow().clone() else {
                progress_label.set_label(
                    "Installation failed: password is unavailable. Return to user settings.",
                );
                append_log_line(&log_label, "FAIL: retry requested but no password is cached.");
                btn.set_sensitive(true);
                return;
            };

            let Some(de) = desktop_env else {
                progress_label.set_label("Installation failed: no desktop environment selected.");
                btn.set_sensitive(true);
                return;
            };

            let install_config = backend::install_worker::InstallConfig {
                username,
                password,
                hostname,
                timezone,
                locale,
                keymap,
                desktop_env: de,
                kernel,
                nvidia,
                removable_media,
            };

            progress_label.set_label("Retrying fstab generation...");
            append_log_line(&log_label, "INFO: retrying fstab generation.");
            if let Err(e) = backend::install_worker::generate_fstab() {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: fstab generated.");

            progress_label.set_label("Retrying system configuration...");
            append_log_line(&log_label, "INFO: retrying system configuration.");
            if let Err(e) = backend::install_worker::configure_system(&install_config, &root_partition) {
                progress_label.set_label(&format!("Installation failed: {}", e));
                append_log_line(&log_label, &format!("FAIL: {}", e));
                btn.set_sensitive(true);
                return;
            }
            append_log_line(&log_label, "SUCCESS: system configured.");

            let username = install_config.username.clone();

            if install_config.desktop_env == DesktopEnv::Gnome {
                progress_label.set_label("Configuring GNOME...");
                append_log_line(&log_label, "INFO: applying GNOME theme and extensions.");
                if let Err(e) = backend::install_worker::configure_gnome(&username) {
                    append_log_line(&log_label, &format!("WARN: GNOME config failed (non-fatal): {}", e));
                } else {
                    append_log_line(&log_label, "SUCCESS: GNOME configured.");
                }
            }

            state.borrow_mut().password.clear();

            append_log_line(&log_label, "SUCCESS: installation finished after config retry.");
            *install_password.borrow_mut() = None;
            progress_label.set_label("Installation complete. Opening completion options...");
            stack.set_visible_child_name("complete");
        });
    }

    vbox.append(&title);
    let fs_label = Label::new(Some("_Root Filesystem"));
    fs_label.set_use_underline(true);
    fs_label.set_mnemonic_widget(Some(&fs_combo));
    vbox.append(&fs_label);
    vbox.append(&fs_combo);
    vbox.append(&swap_hint);
    vbox.append(&progress_label);
    vbox.append(&log_label);
    vbox.append(&start_btn);
    vbox.append(&retry_pacstrap_btn);
    vbox.append(&retry_config_btn);
    vbox
}
