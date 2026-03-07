# access-os-installer

Installers for access-OS, with a strong focus on accessibility.

## What’s Here

This repository is a Cargo workspace with multiple installers sharing one backend:

- `gtk/` (`access-os-installer`): GTK4 wizard UI. This is what the desktop entry launches.
- `cli/` (`access-os-installer-cli`): line-oriented wizard designed to work well with screen readers (no curses/TUI). Supports typed navigation commands like `next` / `back`.
- `crates/installer-core/` (`installer-core`): shared install backend (disk/network/preflight/storage planning + install pipeline).

Contributor guidelines live in [AGENTS.md](/home/alek/git/access-os-installer/AGENTS.md).

## Build and Run (From Repo Root)

```bash
cargo build --workspace
cargo test --workspace

# CLI (recommended for iteration): prints plan only, performs no disk/install actions
cargo run -p access-os-installer-cli -- --dry-run

# GTK installer (requires a graphical session)
cargo run -p access-os-installer
```

Useful dev commands:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
```

## Requirements

The real install path expects an Arch-based live environment with tools such as `pacstrap`, `arch-chroot`, `sgdisk`, filesystem format tools (e.g. `mkfs.fat`/`mkfs.ext4`), and networking via NetworkManager (`nmcli`).

## Safety

The backend can perform destructive disk operations (e.g. wiping a target drive and repartitioning). Prefer `--dry-run` while developing, and only test real installs in a VM or on a spare disk.

CLI manual partitioning can queue role-based `EFI`, `root`, `home`, and `swap` partition creates/deletes on the selected install disk and optional separate `/home` disk. Deletes require an explicit confirmation token and are applied during the final install phase.

## Accessibility Notes

- GTK: the app enables AT-SPI by setting `GTK_A11Y=atspi` when it’s not already set.
- CLI: keeps interaction line-based and avoids dense “key help” text; use `help` when needed.
