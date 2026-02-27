# Preflight and Review Gates Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a dedicated preflight gate and final review confirmation gate so unsafe or unsupported install runs are blocked early, while warning-only conditions require explicit user acknowledgement before install.

**Architecture:** Introduce a preflight evaluation module with testable check classification (`Pass/Warn/Fail`) and wire two new GTK stack steps: `Preflight` and `Review & Confirm`. Persist preflight results in app state, gate navigation based on hard failures, and require a warning acknowledgement checkbox on the review screen before install.

**Tech Stack:** Rust, GTK4, existing backend modules, `std::process::Command`, `sysinfo`

---

### Task 1: Add Preflight Evaluation Module

**Files:**
- Create: `src/backend/preflight.rs`
- Modify: `src/backend/mod.rs`
- Test: `src/backend/preflight.rs` (`#[cfg(test)]` unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn hard_fail_blocks_progress_when_uefi_missing() {
    let ctx = PreflightContext { is_uefi: false, has_disk: true, online: true, ram_gib: 16, disk_gib: Some(256) };
    let results = evaluate_checks(&ctx);
    assert!(results.iter().any(|r| r.is_hard && matches!(r.status, CheckStatus::Fail)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test preflight::tests::hard_fail_blocks_progress_when_uefi_missing -- --nocapture`
Expected: FAIL because preflight module and types do not exist yet.

**Step 3: Write minimal implementation**

```rust
pub enum CheckStatus { Pass, Warn, Fail }
pub struct CheckResult { pub id: &'static str, pub label: &'static str, pub status: CheckStatus, pub message: String, pub is_hard: bool }
pub struct PreflightContext { pub is_uefi: bool, pub has_disk: bool, pub online: bool, pub ram_gib: u64, pub disk_gib: Option<u64> }
pub fn evaluate_checks(ctx: &PreflightContext) -> Vec<CheckResult> { /* hard + soft checks */ }
```

Include thresholds:
- RAM warning: `< 8 GiB`
- Disk warning: `< 128 GiB`

**Step 4: Run test to verify it passes**

Run: `cargo test preflight::tests:: -- --nocapture`
Expected: PASS with coverage for hard fail, warning classification, and all-pass case.

**Step 5: Commit**

```bash
git add src/backend/preflight.rs src/backend/mod.rs
git commit -m "feat: add preflight check evaluation module"
```

### Task 2: Persist Disk Size and Preflight Data in App State

**Files:**
- Modify: `src/main.rs`
- Modify: `src/backend/disk_manager.rs`
- Test: `src/backend/disk_manager.rs` (extend tests for size handling helpers if added)

**Step 1: Write the failing test**

```rust
#[test]
fn selected_disk_capacity_is_available_for_preflight() {
    // verify helper preserves numeric capacity for selected drive
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test disk_manager::tests::selected_disk_capacity_is_available_for_preflight -- --nocapture`
Expected: FAIL because selection capacity helper/state does not exist yet.

**Step 3: Write minimal implementation**

```rust
// disk_manager: ensure block device size is represented in bytes and format separately for UI labels
// main.rs AppState: add selected_disk_gib: Option<u64> and preflight_results: Vec<CheckResult>
// Step 1 selection: store both drive path and capacity metadata in AppState
```

**Step 4: Run tests and build**

Run: `cargo test disk_manager::tests:: -- --nocapture`
Expected: PASS.

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs src/backend/disk_manager.rs
git commit -m "feat: persist selected disk capacity for preflight"
```

### Task 3: Add Dedicated Preflight Step and Navigation Gate

**Files:**
- Modify: `src/main.rs`
- Test: `src/backend/preflight.rs` (if helper APIs evolve)

**Step 1: Write the failing test**

```rust
// N/A for GTK step wiring in current codebase; use deterministic manual verification for flow gating.
```

**Step 2: Run baseline build**

Run: `cargo check`
Expected: PASS before UI step insertion.

**Step 3: Write minimal implementation**

```rust
// add Step "preflight" to Stack between host and install
// evaluate checks on entry to preflight
// render grouped hard/soft results in labels
// disable continue button if any hard check fails
// add Back button to return to host step
```

**Step 4: Run build**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add preflight screen with hard blocker gating"
```

### Task 4: Add Review & Confirm Step with Warning Acknowledgement

**Files:**
- Modify: `src/main.rs`

**Step 1: Write the failing test**

```rust
// N/A for GTK-specific widget interaction in current codebase; use manual verification checklist.
```

**Step 2: Run baseline build**

Run: `cargo check`
Expected: PASS before review-step insertion.

**Step 3: Write minimal implementation**

```rust
// add Step "review" between preflight and install
// show selected disk/repo/host/user/timezone/locale summary
// show warning list from preflight results
// add acknowledgement checkbox
// install/continue button enabled only when:
//   - no warnings OR checkbox checked
```

**Step 4: Run build**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add review step with warning acknowledgement gate"
```

### Task 5: End-to-End Verification

**Files:**
- Modify: none

**Step 1: Compile verification**

Run: `cargo check`
Expected: PASS.

**Step 2: Unit verification**

Run: `cargo test preflight::tests:: -- --nocapture`
Expected: PASS.

Run: `cargo test disk_manager::tests:: -- --nocapture`
Expected: PASS.

**Step 3: Manual flow verification**

Run installer and verify:
- Missing UEFI blocks progression on preflight.
- No selected disk blocks progression on preflight.
- No internet blocks progression on preflight.
- RAM below `8 GiB` shows warning only.
- Selected disk below `128 GiB` shows warning only.
- Review screen shows summary and warnings.
- Install remains disabled until acknowledgement checkbox is checked when warnings exist.

**Step 4: Final commit**

```bash
git add src/main.rs src/backend/preflight.rs src/backend/mod.rs src/backend/disk_manager.rs
git commit -m "feat: add preflight and review confirmation gates"
```
