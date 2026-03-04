# GTK A11y Parity Sweep Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Apply `access-launcher` and `universal-startup-manager` GTK4 accessibility patterns across `access-os-installer` (explicit roles, properly labeled form fields, and explicit accessible label/description where GTK defaults are weak).

**Architecture:** Add a small shared a11y helper module in `src/ui/common/a11y.rs`, then sweep every step builder in `src/ui/steps/*.rs` to (1) ensure inputs have labels + mnemonics and (2) set `AccessibleRole` on primary interactive widgets (buttons and textboxes). No backend changes.

**Tech Stack:** Rust 2021, `gtk4` crate (0.7.x), GTK AT-SPI accessibility (already enabled via env setup in `src/app/bootstrap.rs`).

---

### Task 1: Add Shared A11y Helpers

**Files:**
- Create: `src/ui/common/a11y.rs`
- Modify: `src/ui/common/mod.rs`

**Step 1: Add module export (intentional compile-fail)**

Modify `src/ui/common/mod.rs` to `pub mod a11y;` (without creating the file yet).

**Step 2: Run build to verify it fails**

Run: `cargo check`
Expected: FAIL with a missing module/file error for `ui::common::a11y`.

**Step 3: Create minimal helper module**

Create `src/ui/common/a11y.rs` with:
- `set_accessible_label(widget, label)`
- `set_accessible_description(widget, description)`
- `apply_button_role(button)`
- `apply_textbox_role(entry_like)`

Use the same GTK APIs as the reference apps:
- `update_property(&[gtk4::accessible::Property::Label/Description(...)])`
- `set_accessible_role(AccessibleRole::Button/TextBox)`

**Step 4: Run build to verify it passes**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit (optional, ask first)**

```bash
git add src/ui/common/mod.rs src/ui/common/a11y.rs
git commit -m "feat(a11y): add shared GTK accessibility helpers"
```

---

### Task 2: Sweep Step 0 (Welcome)

**Files:**
- Modify: `src/ui/steps/welcome.rs`

**Step 1: Apply roles/descriptions**
- Set `AccessibleRole::Button` on the “Get Started” button.
- Add an accessible description if needed (keep strings short).

**Step 2: Verify build**

Run: `cargo check`
Expected: PASS.

**Step 3: Commit (optional, ask first)**

```bash
git add src/ui/steps/welcome.rs
git commit -m "feat(a11y): label and role tweaks for welcome step"
```

---

### Task 3: Sweep Wi-Fi Step

**Files:**
- Modify: `src/ui/steps/wifi.rs`

**Step 1: Apply roles**
- Set `AccessibleRole::TextBox` on the `PasswordEntry`.
- Set `AccessibleRole::Button` on connect/refresh/skip buttons.

**Step 2: Verify build**

Run: `cargo check`
Expected: PASS.

---

### Task 4: Sweep Disk Selection + Disk Setup Steps

**Files:**
- Modify: `src/ui/steps/disk.rs`
- Modify: `src/ui/steps/disk_setup.rs`

**Step 1: Apply roles**
- Buttons: `AccessibleRole::Button`
- Swap file entry: `AccessibleRole::TextBox`

**Step 2: Fix missing mnemonics**
- Convert “Swap file size (MB)” label to a mnemonic label and bind it to the swap size entry.

**Step 3: Verify build**

Run: `cargo check`
Expected: PASS.

---

### Task 5: Sweep DE + Mirror Steps

**Files:**
- Modify: `src/ui/steps/de.rs`
- Modify: `src/ui/steps/mirror.rs`

**Step 1: Apply roles**
- Buttons: `AccessibleRole::Button`

**Step 2: Verify build**

Run: `cargo check`
Expected: PASS.

---

### Task 6: Sweep Settings Step (Biggest Form)

**Files:**
- Modify: `src/ui/steps/settings.rs`

**Step 1: Add mnemonic labels for every text field**
- Hostname, Username, Password, Timezone, Locale, Keymap

**Step 2: Apply roles**
- `Entry`/`PasswordEntry`: `AccessibleRole::TextBox`
- Buttons: `AccessibleRole::Button`

**Step 3: Verify build**

Run: `cargo check`
Expected: PASS.

---

### Task 7: Sweep Preflight + Review Steps

**Files:**
- Modify: `src/ui/steps/preflight.rs`
- Modify: `src/ui/steps/review.rs`

**Step 1: Apply roles**
- Buttons: `AccessibleRole::Button`

**Step 2: Verify build**

Run: `cargo check`
Expected: PASS.

---

### Task 8: Sweep Install Step

**Files:**
- Modify: `src/ui/steps/install.rs`

**Step 1: Apply roles**
- Buttons (start/retry): `AccessibleRole::Button`

**Step 2: Verify build**

Run: `cargo check`
Expected: PASS.

---

### Task 9: Sweep Completion Step

**Files:**
- Modify: `src/ui/steps/complete.rs`

**Step 1: Add a real label for the packages entry**
- Add a mnemonic label (don’t rely on placeholder text alone).

**Step 2: Apply roles**
- Packages `Entry`: `AccessibleRole::TextBox`
- Buttons: `AccessibleRole::Button`

**Step 3: Verify build**

Run: `cargo check`
Expected: PASS.

---

### Task 10: Final Verification

**Step 1: Run full build checks**

Run: `cargo check`
Expected: PASS.

**Step 2: (Optional) Run tests if present**

Run: `cargo test`
Expected: PASS.

**Step 3: Manual a11y spot-check**
- Launch with Orca enabled and tab through each step to ensure fields/buttons have sensible names.

---

Plan complete and saved to `docs/plans/2026-03-04-gtk-a11y-parity-implementation.md`.

Two execution options:
1. Subagent-Driven (this session): implement task-by-task with checkpoints.
2. Parallel Session (separate): run the plan in a new worktree/session.

