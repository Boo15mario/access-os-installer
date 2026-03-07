use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum DesktopEnv {
    Gnome,
    Kde,
    Server,
    Niri,
}

impl DesktopEnv {
    pub fn all() -> &'static [DesktopEnv] {
        &[
            DesktopEnv::Gnome,
            DesktopEnv::Kde,
            DesktopEnv::Server,
            DesktopEnv::Niri,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            DesktopEnv::Gnome => "GNOME (Custom)",
            DesktopEnv::Kde => "KDE Plasma",
            DesktopEnv::Server => "Server (Headless)",
            DesktopEnv::Niri => "Niri (Coming Soon)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DesktopEnv::Gnome => "GNOME with Access OS custom configuration",
            DesktopEnv::Kde => "KDE Plasma with default settings",
            DesktopEnv::Server => {
                "Headless server profile with Docker, Docker Compose, Tailscale, and SSH"
            }
            DesktopEnv::Niri => "Niri scrollable tiling Wayland compositor — not yet available",
        }
    }

    pub fn profile_filename(&self) -> &'static str {
        match self {
            DesktopEnv::Gnome => "gnome.txt",
            DesktopEnv::Kde => "kde.txt",
            DesktopEnv::Server => "server.txt",
            DesktopEnv::Niri => "niri.txt",
        }
    }

    pub fn extra_services(&self) -> &'static [&'static str] {
        match self {
            DesktopEnv::Server => &["docker", "tailscaled", "sshd"],
            _ => &[],
        }
    }

    pub fn display_manager(&self) -> Option<&'static str> {
        match self {
            DesktopEnv::Gnome | DesktopEnv::Niri => Some("gdm"),
            DesktopEnv::Kde => Some("sddm"),
            DesktopEnv::Server => None,
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            DesktopEnv::Gnome | DesktopEnv::Kde | DesktopEnv::Server => true,
            DesktopEnv::Niri => false,
        }
    }

    pub fn from_index(index: usize) -> Option<&'static DesktopEnv> {
        DesktopEnv::all().get(index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KernelVariant {
    Standard,
    Lts,
    Zen,
    Hardened,
}

impl KernelVariant {
    pub fn all() -> &'static [KernelVariant] {
        &[
            KernelVariant::Standard,
            KernelVariant::Lts,
            KernelVariant::Zen,
            KernelVariant::Hardened,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            KernelVariant::Standard => "Linux (Standard)",
            KernelVariant::Lts => "Linux LTS",
            KernelVariant::Zen => "Linux Zen",
            KernelVariant::Hardened => "Linux Hardened",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            KernelVariant::Standard => "Latest stable kernel — recommended for most hardware",
            KernelVariant::Lts => "Long-term support kernel — maximum stability",
            KernelVariant::Zen => "Performance-tuned kernel — optimized for desktop use",
            KernelVariant::Hardened => "Security-focused kernel — extra hardening patches",
        }
    }

    pub fn profile_filename(&self) -> &'static str {
        match self {
            KernelVariant::Standard => "kernel-standard.txt",
            KernelVariant::Lts => "kernel-lts.txt",
            KernelVariant::Zen => "kernel-zen.txt",
            KernelVariant::Hardened => "kernel-hardened.txt",
        }
    }

    pub fn vmlinuz(&self) -> &'static str {
        match self {
            KernelVariant::Standard => "/vmlinuz-linux",
            KernelVariant::Lts => "/vmlinuz-linux-lts",
            KernelVariant::Zen => "/vmlinuz-linux-zen",
            KernelVariant::Hardened => "/vmlinuz-linux-hardened",
        }
    }

    pub fn initramfs(&self) -> &'static str {
        match self {
            KernelVariant::Standard => "/initramfs-linux.img",
            KernelVariant::Lts => "/initramfs-linux-lts.img",
            KernelVariant::Zen => "/initramfs-linux-zen.img",
            KernelVariant::Hardened => "/initramfs-linux-hardened.img",
        }
    }

    pub fn from_index(index: usize) -> Option<&'static KernelVariant> {
        KernelVariant::all().get(index)
    }
}

fn profile_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join("profiles"));
    }
    let manifest_profiles = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("profiles");
    paths.push(manifest_profiles);
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("profiles"));
            if let Some(parent) = dir.parent() {
                paths.push(parent.join("profiles"));
            }
        }
    }
    paths
}

fn profiles_dir() -> Result<PathBuf, String> {
    for candidate in profile_search_paths() {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "Could not find profiles directory. Checked: {}",
        profile_search_paths()
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn parse_package_list(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

fn load_profile_file_from_dir(dir: &Path, filename: &str) -> Result<Vec<String>, String> {
    let path = dir.join(filename);
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profile {}: {}", path.display(), e))?;
    Ok(parse_package_list(&contents))
}

pub fn load_profile_packages(filename: &str) -> Result<Vec<String>, String> {
    let dir = profiles_dir()?;
    load_profile_file_from_dir(&dir, filename)
}

fn merge_packages(groups: &[Vec<String>]) -> Vec<String> {
    let mut packages = Vec::new();
    for group in groups {
        for pkg in group {
            if !packages.contains(pkg) {
                packages.push(pkg.clone());
            }
        }
    }
    packages
}

pub fn desktop_profile_packages(de: &DesktopEnv) -> Result<Vec<String>, String> {
    load_profile_packages(de.profile_filename())
}

pub fn full_package_list(
    de: &DesktopEnv,
    kernel: &KernelVariant,
    nvidia: bool,
) -> Result<Vec<String>, String> {
    let mut groups = vec![
        load_profile_packages("base.txt")?,
        desktop_profile_packages(de)?,
        load_profile_packages(kernel.profile_filename())?,
    ];
    if nvidia {
        groups.push(load_profile_packages("nvidia.txt")?);
    }
    Ok(merge_packages(&groups))
}

#[cfg(test)]
mod tests {
    use super::{
        DesktopEnv, KernelVariant, desktop_profile_packages, full_package_list, merge_packages,
        parse_package_list,
    };

    #[test]
    fn parser_ignores_comments_and_blank_lines() {
        let packages = parse_package_list(
            "\n# comment\nbase\n\nnetworkmanager\n  # indented comment\nvim\n",
        );
        assert_eq!(packages, vec!["base", "networkmanager", "vim"]);
    }

    #[test]
    fn merge_keeps_first_seen_order() {
        let merged = merge_packages(&[
            vec!["base".to_string(), "vim".to_string()],
            vec!["vim".to_string(), "gdm".to_string()],
            vec!["gdm".to_string(), "linux".to_string()],
        ]);
        assert_eq!(merged, vec!["base", "vim", "gdm", "linux"]);
    }

    #[test]
    fn desktop_and_kernel_map_to_expected_files() {
        assert_eq!(DesktopEnv::Gnome.profile_filename(), "gnome.txt");
        assert_eq!(DesktopEnv::Server.profile_filename(), "server.txt");
        assert_eq!(KernelVariant::Standard.profile_filename(), "kernel-standard.txt");
        assert_eq!(KernelVariant::Hardened.profile_filename(), "kernel-hardened.txt");
    }

    #[test]
    fn desktop_profile_packages_load_from_text_file() {
        let packages = desktop_profile_packages(&DesktopEnv::Kde).unwrap();
        assert_eq!(packages, vec!["plasma-meta", "kde-applications-meta", "sddm"]);
    }

    #[test]
    fn full_package_list_merges_base_desktop_kernel_and_nvidia() {
        let packages = full_package_list(&DesktopEnv::Gnome, &KernelVariant::Standard, true).unwrap();
        assert!(packages.contains(&"base".to_string()));
        assert!(packages.contains(&"gnome".to_string()));
        assert!(packages.contains(&"linux".to_string()));
        assert!(packages.contains(&"nvidia-dkms".to_string()));
        assert_eq!(
            packages.iter().filter(|pkg| pkg.as_str() == "gdm").count(),
            1
        );
    }
}
