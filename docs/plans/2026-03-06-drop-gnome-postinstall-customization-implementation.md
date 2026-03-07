# Drop GNOME Post-Install Customization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Stop applying Access OS-specific GNOME post-install customization while keeping GNOME package installation and display-manager setup unchanged.

**Architecture:** Remove the `configure_gnome()` call sites from the CLI and GTK installers, leaving the shared backend package and system configuration flow intact.

**Tech Stack:** Rust 2021, Cargo workspace, existing installer frontend flows in `cli/` and `gtk/`.

---

### Task 1: Remove GNOME Post-Install Hook From CLI

**Files:**
- Modify: `cli/src/wizard.rs`

**Step 1: Delete the conditional GNOME post-install block**

Remove the best-effort `configure_gnome()` call that runs after `configure_system()`.

**Step 2: Verify**

Run:
```bash
cargo test --workspace
```

**Step 3: Commit**

```bash
git add cli/src/wizard.rs
git commit -m "refactor(cli): drop GNOME post-install customization"
```

---

### Task 2: Remove GNOME Post-Install Hook From GTK

**Files:**
- Modify: `gtk/src/ui/steps/install.rs`

**Step 1: Delete the GNOME-specific post-install blocks**

Remove the `configure_gnome()` call sites from the GTK install step paths.

**Step 2: Verify**

Run:
```bash
cargo test --workspace
```

**Step 3: Commit**

```bash
git add gtk/src/ui/steps/install.rs
git commit -m "refactor(gtk): drop GNOME post-install customization"
```

---

### Task 3: Verify And Document

**Files:**
- Optionally modify: `README.md` if GNOME customization is mentioned

**Step 1: Confirm docs do not promise custom GNOME behavior**

Update only if needed.

**Step 2: Final verification**

Run:
```bash
cargo test --workspace
```

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: remove GNOME customization references"
```
