# Completion and Disk Safety Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a dedicated completion screen with reboot/exit actions (including unmount cleanup + force fallback) and implement safer internal-disk selection with destructive confirmation.

**Architecture:** Extend the GTK stack with a new completion step, keep install logic in Step 4, and add a shared cleanup/action path for reboot and exit. Replace free-form disk input with lsblk-backed internal-disk discovery and explicit confirmation gating before install progression.

**Tech Stack:** Rust, GTK4, `std::process::Command`, existing backend modules (`disk_manager`, `install_worker`)

---

### Task 1: Add Internal-Disk Filtering in Backend

**Files:**
- Modify: `src/backend/disk_manager.rs`
- Test: `src/backend/disk_manager.rs` (`#[cfg(test)]` unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn internal_filter_excludes_usb_or_removable_disks() {
    // Build BlockDevice fixtures and assert only internal drives pass.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test disk_manager::tests::internal_filter_excludes_usb_or_removable_disks -- --nocapture`
Expected: FAIL because filter helper/function does not exist yet.

**Step 3: Write minimal implementation**

```rust
fn is_internal_device(device: &BlockDevice) -> bool {
    // disk type + not removable + not usb transport
}

pub fn get_internal_block_devices() -> Result<Vec<BlockDevice>, String> {
    // call lsblk and filter with is_internal_device
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test disk_manager::tests::internal_filter_excludes_usb_or_removable_disks -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/backend/disk_manager.rs
git commit -m "feat: filter installer disk list to internal devices"
```

### Task 2: Replace Disk Entry with Internal Drive Picker + Confirmation Gate

**Files:**
- Modify: `src/main.rs` (`build_step1`, related state handling)

**Step 1: Write the failing test**

```rust
// N/A for GTK widget wiring in current codebase; use manual verification checklist.
```

**Step 2: Run check to capture baseline**

Run: `cargo check`
Expected: PASS before UI edits.

**Step 3: Write minimal implementation**

```rust
// In Step 1:
// - populate dropdown from get_internal_block_devices()
// - require drive selection + ERASE text
// - enable Next only when both conditions are true
```

**Step 4: Run check to verify it builds**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add internal drive picker with erase confirmation gate"
```

### Task 3: Add Step 5 Completion Screen with Cleanup + Force Fallback

**Files:**
- Modify: `src/main.rs` (new completion step, cleanup helper, action handlers)

**Step 1: Write the failing test**

```rust
// N/A for process + GTK integration in current codebase; verify through manual flow.
```

**Step 2: Run check to capture baseline**

Run: `cargo check`
Expected: PASS before Step 5 changes.

**Step 3: Write minimal implementation**

```rust
// - Add build_step5()
// - On install success: stack.set_visible_child_name("complete")
// - Add shared cleanup unmount sequence (/mnt/boot then /mnt)
// - Reboot/Exit use cleanup first
// - On cleanup failure, reveal Force Reboot / Force Exit
```

**Step 4: Run check to verify it builds**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add completion step with reboot/exit cleanup and force fallback"
```

### Task 4: End-to-End Verification

**Files:**
- Modify: none

**Step 1: Run full compile checks**

Run: `cargo check`
Expected: PASS.

**Step 2: Manual verification**

Run installer and confirm:
- Disk step lists only internal drives.
- `Next` is disabled until a drive is selected and `ERASE` is entered.
- Successful install path transitions to Step 5.
- Reboot/Exit attempts unmount first.
- Cleanup failure reveals force fallback buttons.

**Step 3: Final commit**

```bash
git add src/main.rs src/backend/disk_manager.rs docs/plans/2026-02-27-completion-and-disk-safety-design.md docs/plans/2026-02-27-completion-and-disk-safety-implementation.md
git commit -m "feat: add post-install actions and safer internal disk selection"
```
