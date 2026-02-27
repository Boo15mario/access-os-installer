# Design Doc: access-OS Installer

## Goal
To create a modern, accessible, and transparent GTK4-based installer for NixOS configurations (both Flake and non-Flake) that follows the "Transparent Workspace" philosophy.

## Architecture

### 1. User Interface (GTK4 / Rust)
- **Welcome & Disk**: Hardware discovery (disks, RAM) and partitioning scheme selection.
- **Source Selection**: A blank Git URL input field. The installer clones the repo to a temporary path (`/tmp/installer-source`) to scan for metadata.
- **Host Discovery**: List detected NixOS host configurations from the cloned repo.
- **Settings Interrogation**: Scans the selected host for `time.timeZone`, `i18n.defaultLocale`, and `console.keyMap`. Prompts user if any are undefined.
- **Credentials**: Collects username, hostname, and password (held in memory, never written to disk).

### 2. Backend Components
- **`disk_manager.rs`**:
    - Uses `sfdisk` for GPT partitioning.
    - 1GB EFI partition (FAT32) is mandatory.
    - Swap size calculated as `RAM * 2` (if RAM <= 16GB) or `RAM * 1` (if RAM > 16GB), with user override.
    - Root partition (XFS or EXT4).
    - Optional separate partition/drive for `/home`.
- **`config_engine.rs`**:
    - Clones the Git repo directly to `/mnt/etc/nixos` during the install phase.
    - Runs `nixos-generate-config --root /mnt` to place `hardware-configuration.nix` in the correct host folder.
    - **Local Overrides**: If settings (timezone, etc.) were missing, it creates `<host>/local-settings.nix` and adds `imports = [ ./local-settings.nix ];` to the host's `configuration.nix`.
- **`install_worker.rs`**:
    - Executes `nixos-install --flake .#<host>`.
    - Securely injects the password via `chroot /mnt chpasswd`.

## Data Flow
1. User enters Git URL -> Clone to `/tmp` -> Scan for hosts.
2. User selects host -> Scan for missing settings -> UI prompts user.
3. User triggers Install -> Partition/Format/Mount -> Clone to `/mnt` -> Generate hardware config -> Apply `local-settings.nix` -> `nixos-install` -> Set password via `chpasswd`.

## Success Criteria
- The resulting system has the user's Git repo already set up in `/etc/nixos`.
- The system boots into the requested configuration with the correct local settings and user password.
- The installer is fully navigable via screen reader (GTK A11y).
