# Base Package Main-Repo Audit

Date: 2026-03-03 (America/Chicago)

## Scope

Audit the old installer minimal package list and remove packages that are not in official Arch main repositories (`core`, `extra`, `multilib`) from the new installer base package set.

## Sources

- Old installer index: https://boo15mario.com/scripts/access-os/
- Old minimal package list: https://boo15mario.com/scripts/access-os/pkglist.min.txt
- Arch package API: https://archlinux.org/packages/search/json/

## Packages Not In Main Repositories

These were found in the old minimal list but are not in official main repos:

- `access-grub-boot-menu`
- `access-grub-boot-sound`
- `access-soundcard-picker`
- `downgrade`
- `grub-hook`
- `mkinitcpio-firmware`
- `pacman-systemd-inhibit`
- `update-grub`
- `vi`

## Result

- Old minimal list entries checked: 59
- In main repos: 50
- Not in main repos: 9

The base package list in `src/backend/config_engine.rs` was replaced using the 50 main-repo packages from the old minimal list.

To preserve current installer behavior, these existing main-repo packages were retained as required runtime dependencies:

- `sudo`
- `git`
- `bluez`
- `bluez-utils`
- `cups`
