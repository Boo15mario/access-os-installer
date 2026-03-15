use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use super::config_engine::{self, DesktopEnv, KernelVariant};
use super::emit_progress;

const PIPEWIRE_USER_SERVICES: &[&str] = &["pipewire", "pipewire-pulse", "wireplumber"];
const AUDIO_INIT_SERVICE: &str = "access-os-audio-init.service";
const AUDIO_INIT_SCRIPT_PATH: &str = "/mnt/usr/local/bin/access-os-audio-init";
const AUDIO_INIT_SERVICE_PATH: &str = "/mnt/etc/systemd/system/access-os-audio-init.service";
const AUDIO_INIT_ASOUND_TEMPLATE_PATH: &str = "/mnt/usr/local/share/access-os-audio/asound.conf.in";
const WIREPLUMBER_AUDIO_POLICY_PATH: &str =
    "/mnt/etc/wireplumber/wireplumber.conf.d/50-access-os-audio.conf";
const PIPEWIRE_PULSE_CONFIG_PATH: &str =
    "/mnt/etc/pipewire/pipewire-pulse.conf.d/50-access-os-audio.conf";
const PULSE_CLIENT_CONFIG_PATH: &str = "/mnt/etc/pulse/client.conf";

const AUDIO_INIT_SCRIPT: &str = r#"#!/usr/bin/env bash
set -u

log() {
    systemd-cat -t "access-os-audio-init" printf "%s\n" "$1"
}

card_indices() {
    if [[ -f /proc/asound/cards ]]; then
        sed -n -e 's/^[[:space:]]*\([0-7]\)[[:space:]].*/\1/p' /proc/asound/cards
    fi
}

list_non_pcsp_cards() {
    local card cardfile
    for card in $(card_indices); do
        cardfile="/proc/asound/card${card}/id"
        if [[ -r "$cardfile" && -f "$cardfile" && "$(<"$cardfile")" != "pcsp" ]]; then
            echo "$card"
        fi
    done
}

set_level() {
    local card="$1" control="$2" level="$3"
    amixer -c "$card" set "$control" "$level" unmute >/dev/null 2>&1 || true
}

set_switch() {
    local card="$1" control="$2" state="$3"
    amixer -c "$card" set "$control" "$state" >/dev/null 2>&1 || true
}

mute_control() {
    local card="$1" control="$2"
    amixer -c "$card" set "$control" "0%" mute >/dev/null 2>&1 || true
}

init_card() {
    local card="$1"
    set_level "$card" "Front" "80%"
    set_level "$card" "Master" "80%"
    set_level "$card" "Master Mono" "80%"
    set_level "$card" "Master Digital" "80%"
    set_level "$card" "Playback" "80%"
    set_level "$card" "Headphone" "100%"
    set_level "$card" "PCM" "80%"
    set_level "$card" "PCM,1" "80%"
    set_level "$card" "DAC" "80%"
    set_level "$card" "DAC,0" "80%"
    set_level "$card" "DAC,1" "80%"
    set_level "$card" "Synth" "80%"
    set_level "$card" "CD" "80%"
    set_level "$card" "PC Speaker" "100%"

    mute_control "$card" "Mic"
    mute_control "$card" "IEC958"

    set_switch "$card" "Master Playback Switch" on
    set_switch "$card" "Master Surround" on
    set_level "$card" "Wave" "80%"
    set_level "$card" "Music" "80%"
    set_level "$card" "AC97" "80%"
    set_level "$card" "Dynamic Range Compression" "80%"
    set_level "$card" "Analog Front" "80%"
    set_switch "$card" "IEC958 Capture Monitor" off
    set_level "$card" "IEC958 Playback AC97-SPSA" "0"
    set_level "$card" "VIA DXS,0" "80%"
    set_level "$card" "VIA DXS,1" "80%"
    set_level "$card" "VIA DXS,2" "80%"
    set_level "$card" "VIA DXS,3" "80%"
    set_switch "$card" "Headphone Jack Sense" off
    set_switch "$card" "Line Jack Sense" off
    set_switch "$card" "Audigy Analog/Digital Output Jack" on
    set_switch "$card" "SB Live Analog/Digital Output Jack" on
    set_switch "$card" "Speaker" on
    set_switch "$card" "Headphone" on
    set_level "$card" "Digital" "80%"
}

set_default_card() {
    local card="$1"
    local template="/usr/local/share/access-os-audio/asound.conf.in"

    if [[ ! -f "$template" ]]; then
        log "ASound template missing; skipping default card selection"
        return 0
    fi

    sed -e "s/%card%/$card/g" "$template" >/etc/asound.conf
    log "Configured ALSA default card ${card} in /etc/asound.conf"
}

main() {
    local card found=0 selected_card=""
    for card in $(card_indices); do
        found=1
        log "Initializing ALSA controls on card ${card}"
        init_card "$card"
    done

    if [[ "$found" -eq 0 ]]; then
        log "No ALSA cards detected during audio initialization"
        return 0
    fi

    for card in $(list_non_pcsp_cards); do
        selected_card="$card"
        break
    done

    if [[ -n "$selected_card" ]]; then
        set_default_card "$selected_card"
    else
        log "No non-pcsp ALSA cards detected for default device selection"
    fi

    if alsactl store >/dev/null 2>&1; then
        log "Stored initialized ALSA state"
    else
        log "Failed to store ALSA state"
    fi
}

main "$@"
"#;

const AUDIO_INIT_UNIT: &str = r#"[Unit]
Description=Initialize ALSA mixer controls for Access OS
Wants=systemd-udev-settle.service
After=systemd-udev-settle.service sound.target
Before=espeakup.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/access-os-audio-init

[Install]
WantedBy=sound.target
"#;

const AUDIO_INIT_ASOUND_TEMPLATE: &str = r#"defaults.ctl.card %card%;
defaults.pcm.card %card%;
"#;

const WIREPLUMBER_AUDIO_POLICY: &str = r#"monitor.alsa.rules = [
  {
    matches = [
      { device.name = "~alsa_card.*" }
    ]
    actions = {
      update-props = {
        session.suspend-timeout-seconds = 0
        api.alsa.disable-power-save = true
      }
    }
  }
  {
    matches = [
      { node.name = "~alsa_input.*" }
      { node.name = "~alsa_output.*" }
    ]
    actions = {
      update-props = {
        session.suspend-timeout-seconds = 0
      }
    }
  }
]
"#;

const PIPEWIRE_PULSE_CONFIG: &str = r#"pulse.properties = {
    server.address = [
        "unix:native"
        "unix:/tmp/pulse.sock"
    ]
}

pulse.rules = [
    {
        matches = [ { application.name = "~speech-dispatcher*" } ]
        actions = {
            update-props = {
                pulse.min.req = 1024/48000
                pulse.min.quantum = 1024/48000
            }
        }
    }
]
"#;

const PULSE_CLIENT_CONFIG: &str = r#"# Configured for PipeWire socket access.
default-server = unix:/tmp/pulse.sock
autospawn = no
"#;

const GNOME_EXTENSIONS: &[(&str, &str)] = &[
    ("no-overview@fthx", "https://github.com/fthx/no-overview"),
    (
        "notification-timeout@chlumskyvaclav.gmail.com",
        "https://github.com/vchlum/notification-timeout",
    ),
    (
        "overviewbackground@github.com.orbitcorrection",
        "https://github.com/howbea/overview-background",
    ),
];
const STAGED_CONFIG_REPO_PATH: &str = "/access-os-config";

pub struct InstallConfig {
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub timezone: String,
    pub locale: String,
    pub keymap: String,
    pub desktop_env: DesktopEnv,
    pub kernel: KernelVariant,
    pub nvidia: bool,
    pub removable_media: bool,
}

fn run_command(program: &str, args: &[&str], context: &str) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{}: {}", context, e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!(
                "{}: command exited with {}",
                context, output.status
            ))
        } else {
            Err(format!("{}: {}", context, stderr))
        }
    }
}

fn run_chroot(args: &[&str], context: &str) -> Result<(), String> {
    let mut full_args = vec!["/mnt"];
    full_args.extend_from_slice(args);
    run_command("arch-chroot", &full_args, context)
}

fn get_root_uuid(root_partition: &str) -> Result<String, String> {
    let output = Command::new("blkid")
        .args(&["-s", "UUID", "-o", "value", root_partition])
        .output()
        .map_err(|e| format!("Failed to get root UUID: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "blkid failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let uuid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uuid.is_empty() {
        return Err("blkid returned empty UUID for root partition".to_string());
    }
    Ok(uuid)
}

fn install_audio_init_assets() -> Result<(), String> {
    let script_path = Path::new(AUDIO_INIT_SCRIPT_PATH);
    if let Some(parent) = script_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create audio init script directory: {}", e))?;
    }
    fs::write(script_path, AUDIO_INIT_SCRIPT)
        .map_err(|e| format!("Failed to write audio init script: {}", e))?;
    fs::set_permissions(script_path, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to mark audio init script executable: {}", e))?;

    let service_path = Path::new(AUDIO_INIT_SERVICE_PATH);
    if let Some(parent) = service_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create audio init service directory: {}", e))?;
    }
    fs::write(service_path, AUDIO_INIT_UNIT)
        .map_err(|e| format!("Failed to write audio init service: {}", e))?;

    let asound_template_path = Path::new(AUDIO_INIT_ASOUND_TEMPLATE_PATH);
    if let Some(parent) = asound_template_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create ASound template directory: {}", e))?;
    }
    fs::write(asound_template_path, AUDIO_INIT_ASOUND_TEMPLATE)
        .map_err(|e| format!("Failed to write ASound template: {}", e))?;

    let wireplumber_policy_path = Path::new(WIREPLUMBER_AUDIO_POLICY_PATH);
    if let Some(parent) = wireplumber_policy_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create WirePlumber config directory: {}", e))?;
    }
    fs::write(wireplumber_policy_path, WIREPLUMBER_AUDIO_POLICY)
        .map_err(|e| format!("Failed to write WirePlumber audio policy: {}", e))?;

    let pipewire_pulse_config_path = Path::new(PIPEWIRE_PULSE_CONFIG_PATH);
    if let Some(parent) = pipewire_pulse_config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create PipeWire Pulse config directory: {}", e))?;
    }
    fs::write(pipewire_pulse_config_path, PIPEWIRE_PULSE_CONFIG)
        .map_err(|e| format!("Failed to write PipeWire Pulse config: {}", e))?;

    let pulse_client_config_path = Path::new(PULSE_CLIENT_CONFIG_PATH);
    if let Some(parent) = pulse_client_config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create Pulse client config directory: {}", e))?;
    }
    fs::write(pulse_client_config_path, PULSE_CLIENT_CONFIG)
        .map_err(|e| format!("Failed to write Pulse client config: {}", e))?;

    Ok(())
}

pub fn run_pacstrap(
    config: &InstallConfig,
    progress: Option<&super::ProgressCallback>,
) -> Result<(), String> {
    emit_progress(progress, "Installing packages with pacstrap");
    let packages =
        config_engine::full_package_list(&config.desktop_env, &config.kernel, config.nvidia)?;
    let mut args: Vec<&str> = vec!["-K", "/mnt"];
    args.extend(packages.iter().map(String::as_str));
    run_command("pacstrap", &args, "pacstrap failed")
}

pub fn generate_fstab(progress: Option<&super::ProgressCallback>) -> Result<(), String> {
    emit_progress(progress, "Generating fstab");
    let output = Command::new("genfstab")
        .args(&["-U", "/mnt"])
        .output()
        .map_err(|e| format!("genfstab failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "genfstab failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    std::fs::write("/mnt/etc/fstab", &output.stdout)
        .map_err(|e| format!("Failed to write /mnt/etc/fstab: {}", e))
}

pub fn configure_system(
    config: &InstallConfig,
    root_partition: &str,
    progress: Option<&super::ProgressCallback>,
) -> Result<(), String> {
    // 1. Timezone
    emit_progress(progress, "Setting timezone");
    let tz_path = format!("/usr/share/zoneinfo/{}", config.timezone);
    run_chroot(
        &["ln", "-sf", &tz_path, "/etc/localtime"],
        "Failed to set timezone symlink",
    )?;
    run_chroot(&["hwclock", "--systohc"], "Failed to sync hardware clock")?;

    // 2. Locale
    emit_progress(progress, "Generating locales");
    let sed_pattern = format!("s/^#\\({}\\)/\\1/", config.locale);
    run_chroot(
        &["sed", "-i", &sed_pattern, "/etc/locale.gen"],
        "Failed to uncomment locale in locale.gen",
    )?;
    run_chroot(&["locale-gen"], "Failed to generate locales")?;

    let locale_conf = format!("LANG={}", config.locale);
    std::fs::write("/mnt/etc/locale.conf", format!("{}\n", locale_conf))
        .map_err(|e| format!("Failed to write locale.conf: {}", e))?;

    // 3. Keymap
    emit_progress(progress, "Writing console keymap");
    let vconsole = format!("KEYMAP={}", config.keymap);
    std::fs::write("/mnt/etc/vconsole.conf", format!("{}\n", vconsole))
        .map_err(|e| format!("Failed to write vconsole.conf: {}", e))?;

    // 4. Hostname
    emit_progress(progress, "Writing hostname and hosts file");
    std::fs::write("/mnt/etc/hostname", format!("{}\n", config.hostname))
        .map_err(|e| format!("Failed to write hostname: {}", e))?;

    // 5. /etc/hosts
    let hosts = format!(
        "127.0.0.1\tlocalhost\n::1\t\tlocalhost\n127.0.1.1\t{}\n",
        config.hostname
    );
    std::fs::write("/mnt/etc/hosts", hosts)
        .map_err(|e| format!("Failed to write /etc/hosts: {}", e))?;

    // 6. Bootloader (systemd-boot)
    emit_progress(progress, "Installing bootloader");
    if config.removable_media {
        run_chroot(
            &["bootctl", "install", "--no-variables"],
            "Failed to install systemd-boot for removable media",
        )?;
    } else {
        run_chroot(&["bootctl", "install"], "Failed to install systemd-boot")?;
    }

    let loader_conf = "default access-os.conf\ntimeout 3\neditor no\n";
    std::fs::write("/mnt/boot/loader/loader.conf", loader_conf)
        .map_err(|e| format!("Failed to write loader.conf: {}", e))?;

    let root_uuid = get_root_uuid(root_partition)?;
    let entry = format!(
        "title   Access OS\nlinux   {}\ninitrd  /amd-ucode.img\ninitrd  /intel-ucode.img\ninitrd  {}\noptions root=UUID={} rw\n",
        config.kernel.vmlinuz(),
        config.kernel.initramfs(),
        root_uuid
    );

    std::fs::create_dir_all("/mnt/boot/loader/entries")
        .map_err(|e| format!("Failed to create loader entries dir: {}", e))?;
    std::fs::write("/mnt/boot/loader/entries/access-os.conf", entry)
        .map_err(|e| format!("Failed to write boot entry: {}", e))?;

    // 7. Create user
    emit_progress(progress, "Creating user account");
    run_chroot(
        &[
            "useradd",
            "-m",
            "-G",
            "audio,video,storage,power,wheel",
            "-s",
            "/bin/bash",
            &config.username,
        ],
        "Failed to create user",
    )?;

    // Set password via chpasswd
    let mut child = Command::new("arch-chroot")
        .args(&["/mnt", "chpasswd"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn chpasswd: {}", e))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or("Failed to open stdin for chpasswd")?;
    let input = format!("{}:{}\n", config.username, config.password);
    stdin
        .write_all(input.as_bytes())
        .map_err(|e| format!("Failed to write to chpasswd stdin: {}", e))?;
    drop(stdin);

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for chpasswd: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            return Err("chpasswd failed to set user password".to_string());
        }
        return Err(format!("chpasswd failed: {}", stderr));
    }

    // 7. Sudoers — uncomment %wheel ALL=(ALL:ALL) ALL
    emit_progress(progress, "Configuring sudo access");
    run_chroot(
        &[
            "sed",
            "-i",
            "s/^# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/",
            "/etc/sudoers",
        ],
        "Failed to configure sudoers",
    )?;

    // 9. Enable services
    emit_progress(progress, "Installing audio initialization service");
    install_audio_init_assets()?;

    emit_progress(progress, "Enabling system services");
    let mut services = vec![
        AUDIO_INIT_SERVICE,
        "NetworkManager",
        "bluetooth",
        "cups",
        "cronie",
        "ntpd",
        "espeakup",
    ];
    if let Some(dm) = config.desktop_env.display_manager() {
        services.push(dm);
    }
    services.extend_from_slice(config.desktop_env.extra_services());
    for service in services {
        run_chroot(
            &["systemctl", "enable", service],
            &format!("Failed to enable {}", service),
        )?;
    }

    // Enable PipeWire audio user services for all users
    emit_progress(progress, "Enabling PipeWire audio services");
    for service in PIPEWIRE_USER_SERVICES {
        run_chroot(
            &["systemctl", "--global", "enable", service],
            &format!("Failed to enable {} user service", service),
        )?;
    }

    Ok(())
}

pub fn configure_gnome(username: &str) -> Result<(), String> {
    let home = format!("/home/{}", username);

    // Set dark theme, Breeze-Dark GTK theme, and orange accent via dconf
    // Using dconf directly since gsettings needs a running session
    let dconf_settings = "\
[org/gnome/desktop/interface]\n\
color-scheme='prefer-dark'\n\
gtk-theme='Breeze-Dark'\n\
accent-color='orange'\n\
";

    let dconf_dir = format!("/mnt{}/.config/dconf", home);
    std::fs::create_dir_all(&dconf_dir)
        .map_err(|e| format!("Failed to create dconf dir: {}", e))?;

    // Write dconf database using dconf load inside chroot as the user
    let mut child = Command::new("arch-chroot")
        .args(&["/mnt", "su", "-", username, "-c", "dconf load /"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn dconf load: {}", e))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or("Failed to open stdin for dconf load")?;
    stdin
        .write_all(dconf_settings.as_bytes())
        .map_err(|e| format!("Failed to write dconf settings: {}", e))?;
    drop(stdin);

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for dconf load: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("dconf load failed: {}", stderr));
    }

    // Install GNOME extensions to userspace (~/.local/share/gnome-shell/extensions/)
    let ext_base = format!("/mnt{}/.local/share/gnome-shell/extensions", home);
    std::fs::create_dir_all(&ext_base)
        .map_err(|e| format!("Failed to create extensions dir: {}", e))?;

    for (uuid, repo_url) in GNOME_EXTENSIONS {
        let ext_dir = format!("{}/{}", ext_base, uuid);
        let clone_output = Command::new("git")
            .args(&["clone", "--depth", "1", repo_url, &ext_dir])
            .output()
            .map_err(|e| format!("Failed to clone extension {}: {}", uuid, e))?;

        if !clone_output.status.success() {
            return Err(format!(
                "Failed to clone extension {}: {}",
                uuid,
                String::from_utf8_lossy(&clone_output.stderr)
            ));
        }

        // Remove .git dir from the cloned extension
        let git_dir = format!("{}/.git", ext_dir);
        let _ = std::fs::remove_dir_all(&git_dir);
    }

    // Enable extensions via dconf
    let ext_uuids: Vec<String> = GNOME_EXTENSIONS
        .iter()
        .map(|(uuid, _)| format!("'{}'", uuid))
        .collect();
    let enable_setting = format!(
        "[org/gnome/shell]\nenabled-extensions=[{}]\n",
        ext_uuids.join(", ")
    );

    let mut child = Command::new("arch-chroot")
        .args(&["/mnt", "su", "-", username, "-c", "dconf load /"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn dconf load for extensions: {}", e))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or("Failed to open stdin for dconf load")?;
    stdin
        .write_all(enable_setting.as_bytes())
        .map_err(|e| format!("Failed to write extension enable settings: {}", e))?;
    drop(stdin);

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for dconf load: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("dconf load for extensions failed: {}", stderr));
    }

    // Fix ownership of all user config
    run_chroot(
        &["chown", "-R", &format!("{}:{}", username, username), &home],
        "Failed to fix ownership of GNOME config",
    )?;

    Ok(())
}

pub fn stage_system_config_repo(
    url: &str,
    progress: Option<&super::ProgressCallback>,
) -> Result<(), String> {
    if url.is_empty() {
        return Ok(());
    }

    emit_progress(progress, "Staging system configuration repository");
    if Path::new(STAGED_CONFIG_REPO_PATH).exists() {
        fs::remove_dir_all(STAGED_CONFIG_REPO_PATH).map_err(|e| {
            format!(
                "Failed to remove existing staged repo {}: {}",
                STAGED_CONFIG_REPO_PATH, e
            )
        })?;
    }

    run_command(
        "git",
        &["clone", "--depth", "1", url, STAGED_CONFIG_REPO_PATH],
        "Failed to clone system config repo",
    )
}

pub fn overlay_staged_config_to_target(
    progress: Option<&super::ProgressCallback>,
) -> Result<(), String> {
    let repo = Path::new(STAGED_CONFIG_REPO_PATH);
    if !repo.exists() {
        return Err(format!(
            "Staged config repo not found at {}",
            STAGED_CONFIG_REPO_PATH
        ));
    }

    let entries =
        fs::read_dir(repo).map_err(|e| format!("Failed to read staged repo directory: {}", e))?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".git" {
            continue;
        }

        emit_progress(progress, &format!("Copying staged config {}", name_str));
        let src = entry.path();
        let dest = Path::new("/mnt").join(&name);
        let output = Command::new("cp")
            .args(["-a", &src.to_string_lossy(), &dest.to_string_lossy()])
            .output()
            .map_err(|e| format!("Failed to copy staged item {}: {}", name_str, e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to copy {}: {}",
                name_str,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }

    Ok(())
}
