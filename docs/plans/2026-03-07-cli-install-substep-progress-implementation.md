# CLI Install Sub-Step Progress Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add short sub-step progress messages to the CLI installer during long-running install work.

**Architecture:** Introduce an optional backend progress callback and use it to emit sub-step messages from disk execution and system configuration functions. The CLI prints these messages; GTK remains on the no-progress-callback path.

**Tech Stack:** Rust 2021, Cargo workspace, existing CLI installer and shared backend crates.

---

### Task 1: Add Progress Callback Plumbing To Backend

**Files:**
- Modify: `crates/installer-core/src/backend/disk_manager.rs`
- Modify: `crates/installer-core/src/backend/install_worker.rs`

**Step 1: Add a small progress helper**

Use an optional callback signature that can be passed through long-running backend functions.

**Step 2: Instrument disk execution**

Emit progress messages for:

- partition deletion/creation
- disk partitioning
- formatting
- swap activation
- mount target creation and mount operations

**Step 3: Instrument install worker**

Emit progress messages for:

- staging repo clone/update
- overlay copy
- pacstrap start
- fstab generation
- system configuration sub-actions

**Step 4: Preserve compatibility**

If no callback is passed, behavior must remain unchanged.

**Step 5: Verify**

Run:
```bash
cargo test -p installer-core
```

**Step 6: Commit**

```bash
git add crates/installer-core/src/backend/disk_manager.rs crates/installer-core/src/backend/install_worker.rs
git commit -m "feat(core): add install progress callbacks"
```

---

### Task 2: Wire Progress Into CLI Install Flow

**Files:**
- Modify: `cli/src/wizard.rs`

**Step 1: Pass a printing callback from `step_install()`**

Use a closure that prints each backend progress line.

**Step 2: Keep top-level install stages**

Retain the existing `1/7` stage messages and let the callback add sub-step detail underneath them.

**Step 3: Verify**

Run:
```bash
cargo test --workspace
cargo run -p access-os-installer-cli -- --help
```

**Step 4: Commit**

```bash
git add cli/src/wizard.rs
git commit -m "feat(cli): print install sub-step progress"
```

---

### Task 3: Confirm GTK Compatibility

**Files:**
- Modify only if required by compiler errors in GTK call sites

**Step 1: Update any changed function signatures**

Pass `None` where GTK should keep existing behavior.

**Step 2: Final verification**

Run:
```bash
cargo test --workspace
```

**Step 3: Commit**

```bash
git add gtk
git commit -m "refactor(gtk): keep install flow compatible with progress callbacks"
```
