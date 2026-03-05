# Multi-Installer (CLI + Calamares + GTK) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restructure this repo into a Cargo workspace that ships (1) a screen-reader friendly CLI installer, (2) Calamares config for an online GUI installer, and (3) the existing GTK installer moved under `gtk/` (paused for later).

**Architecture:** Extract shared install logic into `crates/installer-core` and keep both frontends thin. The CLI is a line-based wizard with `next`/`back` typed commands and an explicit destructive-review gate.

**Tech Stack:** Rust 2021, Cargo workspace, std::io CLI interaction, Calamares config + branding files.

---

### Task 1: Convert Repo To Cargo Workspace

**Files:**
- Modify: `Cargo.toml`
- Create: `gtk/Cargo.toml`
- Move: `src/` -> `gtk/src/`

**Step 1: Replace root `Cargo.toml` with a virtual workspace**

```toml
[workspace]
members = [
  "gtk",
  "cli",
  "crates/installer-core",
]
resolver = "2"
```

**Step 2: Create `gtk/Cargo.toml` based on the current package**
- Keep package/binary name `access-os-installer` to preserve `access-os-installer.desktop`.
- Add a path dependency on `installer-core` (added in Task 2).

**Step 3: Move sources**

Run:
```bash
git mv src gtk/src
```

**Step 4: Verify GTK still builds**

Run:
```bash
cargo build -p access-os-installer
```

Commit:
```bash
git commit -m "refactor: convert to workspace and move GTK app under gtk/"
```

---

### Task 2: Extract Shared Install Engine (`installer-core`)

**Files:**
- Create: `crates/installer-core/Cargo.toml`
- Create: `crates/installer-core/src/lib.rs`
- Create: `crates/installer-core/src/backend/mod.rs`
- Move: `gtk/src/backend/*` -> `crates/installer-core/src/backend/*`
- Create/Modify: `gtk/src/backend/mod.rs` (shim re-export)
- Move: `gtk/src/services/{mirror,mount,power}.rs` -> `crates/installer-core/src/services/*`
- Create/Modify: `gtk/src/services/mod.rs` (keep `log.rs` local, re-export the rest)

**Step 1: Create the library crate**
- `installer-core` dependencies: `serde`, `serde_json`, `sysinfo`.

**Step 2: Move backend modules into core**
- Preserve module names to keep the public API stable (`backend::disk_manager`, `backend::storage_plan`, etc.).

**Step 3: Add GTK shims to avoid touching the paused GTK UI**

Example `gtk/src/backend/mod.rs`:
```rust
pub use installer_core::backend::*;
```

Example `gtk/src/services/mod.rs`:
```rust
pub mod log; // GTK-only
pub use installer_core::services::{mirror, mount, power};
```

**Step 4: Verify unit tests still pass**

Run:
```bash
cargo test
```

Commit:
```bash
git commit -m "refactor: extract installer-core library"
```

---

### Task 3: Add Line-Based CLI Installer

**Files:**
- Create: `cli/Cargo.toml`
- Create: `cli/src/main.rs`
- Create: `cli/src/wizard.rs`
- Create: `cli/src/steps/*` (one module per step)

**Step 1: CLI skeleton**
- A `Wizard` struct holds selections and the current step index.
- A single input loop reads one line, trims it, then:
  - handles `next`/`back`/`help`/`quit`
  - handles numeric selection (e.g. `1`, `2`, …)
  - handles free-form fields (hostname, username, etc.)

**Step 2: Minimum end-to-end flow (no Wi-Fi management initially)**
- Welcome + root/network checks (use `installer_core::backend::network::check_connectivity()`).
- Disk list (use `installer_core::backend::disk_manager::get_internal_block_devices()`).
- Storage options (automatic/manual, swap mode/size, optional `/home`).
- Regional options (mirror region, timezone, locale, keymap) using the same value lists as GTK.
- System options (hostname, username, password).
- Review prints:
  - selected disks/partitions
  - destructive summary via `installer_core::backend::storage_plan::format_destructive_plan(...)`
- Require typed confirmation: `install`.

**Step 3: Install execution**
- Use the same backend pipeline as the GTK step:
  - `storage_plan::resolve_layout`
  - `disk_manager::execute_layout`
  - `install_worker::{stage_system_config_repo, overlay_staged_config_to_target, run_pacstrap, generate_fstab, configure_system, configure_gnome}`
- Add a `--dry-run` flag that stops after Review (prints plan only).

**Step 4: Smoke test**

Run:
```bash
cargo run -p access-os-installer-cli -- --dry-run
```

Commit:
```bash
git commit -m "feat(cli): add line-based installer wizard"
```

---

### Task 4: Add Calamares Config Tree (Online Installer)

**Files:**
- Create: `calamares/README.md`
- Create: `calamares/settings.conf`
- Create: `calamares/modules.conf`
- Create: `calamares/branding/access-os/branding.desc`
- Create: `calamares/scripts/*` (pre/post install hooks)

**Step 1: Skeleton that boots Calamares with Access OS branding**
- Keep configs self-contained so the ISO build can copy `calamares/` into `/etc/calamares/`.

**Step 2: Online package install**
- Add a module path that installs packages from repos (pacman/pacstrap-based).
- Include a pre-step that ensures:
  - keyring is initialized/updated
  - mirrorlist exists (or is generated)
- Include a post-step that applies Access OS customization (services, dotfiles overlay, etc.).

Commit:
```bash
git commit -m "feat(calamares): add initial Access OS config and branding"
```

---

### Task 5: Update Contributor Docs

**Files:**
- Modify: `AGENTS.md`

Update paths and commands to reflect:
- `cargo build -p access-os-installer` (GTK)
- `cargo run -p access-os-installer-cli` (CLI)
- new workspace directories (`gtk/`, `cli/`, `crates/installer-core/`, `calamares/`).

Commit:
```bash
git commit -m "docs: update contributor guide for workspace layout"
```

