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

    pub fn packages(&self) -> &'static [&'static str] {
        match self {
            DesktopEnv::Gnome => &["gnome", "gnome-tweaks", "gdm", "breeze-gtk"],
            DesktopEnv::Kde => &["plasma-meta", "kde-applications-meta", "sddm"],
            DesktopEnv::Server => &["docker", "docker-compose", "tailscale", "openssh"],
            DesktopEnv::Niri => &["niri", "gdm"],
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

    pub fn packages(&self) -> &'static [&'static str] {
        match self {
            KernelVariant::Standard => &["linux", "linux-headers"],
            KernelVariant::Lts => &["linux-lts", "linux-lts-headers"],
            KernelVariant::Zen => &["linux-zen", "linux-zen-headers"],
            KernelVariant::Hardened => &["linux-hardened", "linux-hardened-headers"],
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

pub fn base_packages(kernel: &KernelVariant) -> Vec<&'static str> {
    let mut packages = vec![
        // Old installer minimal package list, filtered to official Arch repos.
        "base",
        "dialog",
        "alsa-card-profiles",
        "alsa-firmware",
        "alsa-utils",
        "amd-ucode",
        "archlinux-keyring",
        "aspell",
        "aspell-en",
        "base-devel",
        "broadcom-wl-dkms",
        "cifs-utils",
        "cronie",
        "dnsmasq",
        "dosfstools",
        "edk2-ovmf",
        "efibootmgr",
        "espeak-ng",
        "espeakup",
        "grub",
        "icu",
        "intel-ucode",
        "iptables-nft",
        "linux-firmware",
        "linux-firmware-marvell",
        "lvm2",
        "man-db",
        "man-pages",
        "mkinitcpio",
        "mtools",
        "net-tools",
        "nano",
        "networkmanager",
        "ntfs-3g",
        "ntp",
        "openssh",
        "os-prober",
        "pacman-contrib",
        "python",
        "python-pip",
        "reflector",
        "ruby",
        "rust",
        "sof-firmware",
        "traceroute",
        "ufw",
        "usbutils",
        "util-linux",
        "vim",
        "xfsprogs",
        // Required by current installer behavior (dotfiles + enabled services).
        "sudo",
        "git",
        "bluez",
        "bluez-utils",
        "cups",
    ];
    packages.extend_from_slice(kernel.packages());
    packages
}

pub fn nvidia_packages() -> &'static [&'static str] {
    &["nvidia-dkms", "nvidia-utils", "lib32-nvidia-utils"]
}

pub fn full_package_list(de: &DesktopEnv, kernel: &KernelVariant, nvidia: bool) -> Vec<&'static str> {
    let mut combined = base_packages(kernel);
    combined.extend_from_slice(de.packages());
    if nvidia {
        combined.extend_from_slice(nvidia_packages());
    }

    let mut packages = Vec::new();
    for pkg in combined {
        if !packages.contains(&pkg) {
            packages.push(pkg);
        }
    }
    packages
}
