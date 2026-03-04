# Old Installer Parity Phase 1 Disk Setup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add old-installer-style advanced disk setup to the GTK installer, including automatic/manual setup, optional `/home` on another disk, and layout-driven execution.

**Architecture:** Introduce a backend storage planner that resolves UI choices into a concrete install layout, then drive disk actions/mounting/install from that layout instead of fixed partition assumptions. Keep existing settings/preflight/review/install structure, but insert a new disk-setup step and destructive-action summary/countdown on review.

**Tech Stack:** Rust, GTK4, `sysinfo`, existing backend modules (`disk_manager`, `install_worker`, `preflight`), command-line tools (`lsblk`, `sgdisk`, `mkfs`, `mount`, `swapon`)

**Relevant skills:** @verification-before-completion

---

### Task 1: Add Storage Planner Module

**Files:**
- Create: `src/backend/storage_plan.rs`
- Modify: `src/backend/mod.rs`
- Test: `src/backend/storage_plan.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn auto_home_other_disk_requires_distinct_disk() {
    let cfg = InstallStorageConfig::auto_with_other_disk("/dev/nvme0n1", Some("/dev/nvme0n1"));
    let err = resolve_layout(&cfg).unwrap_err();
    assert!(err.contains("must be different"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test storage_plan::tests::auto_home_other_disk_requires_distinct_disk -- --nocapture`  
Expected: FAIL with unresolved symbols (`InstallStorageConfig`, `resolve_layout`).

**Step 3: Write minimal implementation**

```rust
pub enum SetupMode { Automatic, Manual }
pub enum HomeLocation { Root, SameDisk, OtherDisk }

pub struct InstallStorageConfig { /* selected disks, partitions, toggles */ }
pub struct ResolvedInstallLayout { /* wipe/create/format/mount actions */ }

pub fn resolve_layout(cfg: &InstallStorageConfig) -> Result<ResolvedInstallLayout, String> {
    if matches!(cfg.home_location, HomeLocation::OtherDisk)
        && cfg.install_disk == cfg.home_disk
    {
        return Err("Home disk must be different from install disk.".to_string());
    }
    Ok(ResolvedInstallLayout::default())
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test storage_plan::tests:: -- --nocapture`  
Expected: PASS for new storage planner tests.

**Step 5: Commit**

```bash
git add src/backend/storage_plan.rs src/backend/mod.rs
git commit -m "feat: add storage planner model and layout resolver scaffold"
```

### Task 2: Add Disk and Partition Discovery Helpers

**Files:**
- Modify: `src/backend/disk_manager.rs`
- Test: `src/backend/disk_manager.rs`

**Step 1: Write failing tests**

```rust
#[test]
fn partition_path_supports_selected_partition_refs() {
    assert_eq!(partition_device_path("/dev/nvme0n1", 4), "/dev/nvme0n1p4");
    assert_eq!(partition_device_path("/dev/sda", 2), "/dev/sda2");
}
```

**Step 2: Run test to verify it fails (if helper missing)**

Run: `cargo test disk_manager::tests::partition_path_supports_selected_partition_refs -- --nocapture`  
Expected: FAIL until helper usage is wired for planner structures.

**Step 3: Write minimal implementation**

```rust
pub fn get_partitions_for_disk(disk: &str) -> Result<Vec<PartitionDevice>, String> {
    // parse lsblk -J with children and return partition entries for the chosen disk
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test disk_manager::tests:: -- --nocapture`  
Expected: PASS for existing + new disk manager tests.

**Step 5: Commit**

```bash
git add src/backend/disk_manager.rs
git commit -m "feat: add disk partition discovery helpers for manual setup"
```

### Task 3: Add Disk Setup State and GTK Step

**Files:**
- Modify: `src/main.rs`
- Modify: `src/backend/storage_plan.rs`
- Test: `src/backend/storage_plan.rs`

**Step 1: Write failing planner validation tests for UI choices**

```rust
#[test]
fn manual_requires_efi_and_root() {
    let cfg = InstallStorageConfig::manual("/dev/nvme0n1", None, None, None);
    assert!(resolve_layout(&cfg).is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test storage_plan::tests::manual_requires_efi_and_root -- --nocapture`  
Expected: FAIL until manual validation is implemented.

**Step 3: Write minimal implementation**

```rust
struct DiskSetupState {
    setup_mode: SetupMode,
    swap_mode: SwapMode,
    swap_file_mb: Option<u64>,
    home_mode: HomeMode,
    home_location: HomeLocation,
    home_disk: Option<String>,
    removable_media: bool,
    // manual selections
    efi_partition: Option<String>,
    root_partition: Option<String>,
    home_partition: Option<String>,
    format_efi: bool,
    format_root: bool,
    format_home: bool,
}
```

Add `build_disk_setup_step(...)` and transition: `disk -> disk_setup -> desktop_env`.

**Step 4: Run compile check**

Run: `cargo check`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs src/backend/storage_plan.rs
git commit -m "feat: add disk setup step and state for auto/manual storage choices"
```

### Task 4: Resolve Layout Before Install and Execute From Layout

**Files:**
- Modify: `src/main.rs`
- Modify: `src/backend/disk_manager.rs`
- Modify: `src/backend/install_worker.rs`
- Modify: `src/backend/storage_plan.rs`
- Test: `src/backend/storage_plan.rs`

**Step 1: Write failing tests for auto-home-other-disk layout actions**

```rust
#[test]
fn auto_with_other_disk_home_emits_two_disk_actions() {
    let cfg = /* auto with install disk + home disk */;
    let layout = resolve_layout(&cfg).unwrap();
    assert!(layout.disks_to_partition.len() == 2);
    assert!(layout.mounts.iter().any(|m| m.target == "/mnt/home"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test storage_plan::tests::auto_with_other_disk_home_emits_two_disk_actions -- --nocapture`  
Expected: FAIL until layout actions are implemented.

**Step 3: Write minimal implementation**

```rust
let storage_cfg = state.borrow().to_storage_config()?;
let layout = storage_plan::resolve_layout(&storage_cfg)?;
disk_manager::execute_layout(&layout)?;
mount_install_targets_from_layout(&layout)?;
```

Implement layout-driven formatting/mounts/swap behavior and keep retry behavior unchanged.

**Step 4: Run checks**

Run: `cargo test storage_plan::tests:: -- --nocapture && cargo check`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs src/backend/disk_manager.rs src/backend/install_worker.rs src/backend/storage_plan.rs
git commit -m "feat: execute installation using resolved storage layout"
```

### Task 5: Review Destructive Summary and Countdown Gate

**Files:**
- Modify: `src/main.rs`
- Modify: `src/backend/storage_plan.rs`
- Test: `src/backend/storage_plan.rs`

**Step 1: Write failing tests for destructive summary rendering**

```rust
#[test]
fn destructive_summary_lists_wipes_and_formats() {
    let summary = format_destructive_plan(&layout_fixture());
    assert!(summary.contains("Disks to wipe"));
    assert!(summary.contains("Partitions to format"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test storage_plan::tests::destructive_summary_lists_wipes_and_formats -- --nocapture`  
Expected: FAIL until formatter exists.

**Step 3: Write minimal implementation**

```rust
fn format_destructive_plan(layout: &ResolvedInstallLayout) -> String { /* render lists */ }
```

Add review countdown gate (e.g., 5-second disable before `Next: Start Installation` activates).

**Step 4: Run checks**

Run: `cargo test storage_plan::tests:: -- --nocapture && cargo check`  
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs src/backend/storage_plan.rs
git commit -m "feat: add destructive plan summary and countdown confirmation"
```

### Task 6: Final Verification and Docs

**Files:**
- Modify: `docs/plans/2026-03-03-old-installer-parity-phase1-disk-setup-design.md` (if behavior changed)
- Modify: `docs/plans/2026-03-03-old-installer-parity-phase1-disk-setup-implementation.md` (mark completed notes if desired)

**Step 1: Run full verification**

Run: `cargo test -- --nocapture && cargo check`  
Expected: PASS.

**Step 2: Run focused manual checks**

Run installer and validate:
- Auto setup with `/home` on root.
- Auto setup with `/home` on another disk.
- Manual setup with existing partitions.
- Review shows wipe/format/mount lists and countdown.

Expected: All scenarios complete without regression in preflight/review/install transitions.

**Step 3: Commit**

```bash
git add src/main.rs src/backend/*.rs docs/plans/2026-03-03-old-installer-parity-phase1-disk-setup-design.md docs/plans/2026-03-03-old-installer-parity-phase1-disk-setup-implementation.md
git commit -m "feat: implement phase 1 old installer disk setup parity"
```
