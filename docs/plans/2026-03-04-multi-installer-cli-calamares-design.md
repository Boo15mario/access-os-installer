# Design Doc: Multi-Installer Layout (CLI + Calamares + GTK)

**Date:** 2026-03-04 (America/Chicago)  
**Status:** Approved  
**Scope:** Add a screen-reader friendly CLI installer and a Calamares GUI installer, while moving the current GTK installer into `gtk/` for later work.

## Goals

- Provide two primary install paths:
  - `access-os-installer-cli`: line-based wizard (no curses), optimized for console screen readers.
  - Calamares: graphical installer, configured for **online** package-based installs.
- Keep the current GTK installer available, but not on the critical path for now.
- Share one install engine across frontends to avoid duplicated logic and drift.

## Non-Goals (This Phase)

- Finish GTK accessibility parity work (paused once moved under `gtk/`).
- Build or publish the ISO in this repository (ISO integration happens in the `access-os` build repo).

## Repository Structure (Target)

- `crates/installer-core/`: shared backend logic and data model (disk, storage plan, preflight, config, install worker).
- `cli/`: new CLI binary crate (depends on `installer-core`).
- `gtk/`: existing GTK app moved from repo root (depends on `installer-core`).
- `calamares/`: Calamares configuration + branding for Access OS (dropped into `/etc/calamares/` in the live ISO).

## CLI UX Requirements

- Every prompt is one line of input.
- Global commands always available: `next`, `back`, `help`, `quit`.
- Steps print:
  - a short title
  - current selections (compact)
  - numbered options (or explicit free-form fields)
  - a single `>` prompt
- Validation happens on `next` with actionable, single-line errors.
- Destructive actions require explicit confirmation (e.g. type `install` or `WIPE /dev/nvme0n1`).

## CLI Flow (Initial)

1. Welcome + prerequisites (root required; network check).
2. Install options: profile (minimal/desktop), kernel, Nvidia toggle, removable-media toggle.
3. Storage:
   - select install disk
   - automatic vs manual partitioning
   - optional separate `/home` (same disk or other disk)
   - swap: partition vs file (size prompt if file)
   - show a computed destructive plan before proceeding
4. Regional: mirror region, timezone, locale, keymap.
5. System: hostname, username, password.
6. Review: show final plan; require typed confirmation.
7. Install: execute partition/format/mount, pacstrap, fstab, system configuration, bootloader, users, post-config.

## Calamares (Online Installer)

- Provide `calamares/` configs and branding for an **online** install:
  - partitioning handled by Calamares
  - packages installed from repos (pacman / pacstrap-style flow)
  - post-install hooks apply Access OS-specific configuration
- Keep logs verbose and easy to retrieve when installs fail (mirror/keyring issues are common with online installs).

## Risks / Mitigations

- Online install reliability (mirrors/keyring/network):
  - CLI: explicit keyring init step and retry around package install.
  - Calamares: use scripted pre-steps for keyring + mirror sanity; enable verbose logging.
- Disk safety:
  - default to internal disks only (reuse existing internal-disk filter).
  - require explicit destructive confirmation strings.

## Success Criteria

- CLI can complete an end-to-end install in a VM using only typed commands (`next`/`back`), with clear review output.
- Calamares config boots in the live ISO and can complete an online install using Access OS branding and package selection.
- GTK app still builds from `gtk/` (even if not feature-complete).

