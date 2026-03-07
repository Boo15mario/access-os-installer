# CLI Manual Partition Manager Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend the CLI installer's manual storage flow so users can create, delete, assign, and format installer-role partitions on the selected install and `/home` disks.

**Architecture:** Keep the CLI line-based and role-driven. Add explicit manual partition operation state in `installer-core`, expose narrow disk-manager helpers for single-partition create/delete, and have the CLI manual partition screen drive those operations and refresh assignments after each change.

**Tech Stack:** Rust 2021, Cargo workspace, `std::io` CLI prompts, `lsblk`, `sgdisk`, existing storage-plan and disk-manager modules.

---

### Task 1: Extend Shared Storage State For Manual Partition Operations

**Files:**
- Modify: `crates/installer-core/src/backend/storage_plan.rs`
- Test: `crates/installer-core/src/backend/storage_plan.rs`

**Step 1: Add manual operation data types**

Define structs/enums for:

- `ManualPartitionRole` (`Efi`, `Root`, `Home`, `Swap`)
- `ManualCreatePartition` (disk, role, size, use_remaining flag)
- storage state fields for pending create/delete actions if needed

**Step 2: Add helper methods**

Implement helpers to:

- list valid roles for the current `StorageSelection`
- clear stale role assignments after delete
- validate that manual operations only target allowed disks

**Step 3: Update layout resolution**

Ensure `resolve_layout()`:

- still validates final role assignments
- reports pending create/delete actions in review-friendly output
- preserves existing manual install behavior after partition edits are applied

**Step 4: Add unit tests**

Run:
```bash
cargo test -p installer-core
```

Expected coverage:

- `home` role hidden unless separate `/home`
- `swap` role hidden unless swap partition mode
- deleting an assigned partition clears the assignment
- operations outside allowed disks are rejected

**Step 5: Commit**

```bash
git add crates/installer-core/src/backend/storage_plan.rs
git commit -m "feat(core): model manual partition manager state"
```

---

### Task 2: Add Disk Manager Helpers For Manual Create/Delete

**Files:**
- Modify: `crates/installer-core/src/backend/disk_manager.rs`
- Test: `crates/installer-core/src/backend/disk_manager.rs`

**Step 1: Add partition listing helper for managed disks**

Create a helper that filters partitions to:

- selected install disk
- optional separate `/home` disk

**Step 2: Add create helper**

Implement a helper that creates one partition for a requested role using `sgdisk`, with:

- `EFI` -> `ef00`
- `root`/`home` -> `8300`
- `swap` -> `8200`

Support either a fixed size or remaining free space for `root`.

**Step 3: Add delete helper**

Implement a helper that deletes a partition by number/path only if it belongs to an allowed disk.

**Step 4: Add tests for pure helpers**

Run:
```bash
cargo test -p installer-core
```

Expected coverage:

- path-to-disk scoping works for NVMe and SATA names
- role-to-GPT-type mapping is correct
- delete rejects out-of-scope partitions

**Step 5: Commit**

```bash
git add crates/installer-core/src/backend/disk_manager.rs
git commit -m "feat(core): add manual partition create and delete helpers"
```

---

### Task 3: Rework CLI Manual Partition Screen

**Files:**
- Modify: `cli/src/wizard.rs`
- Test: `cli/src/wizard.rs` or new focused CLI test module if extracted

**Step 1: Replace the current selector-only manual menu**

Change `edit_manual_partitions()` so it presents:

- role assignments
- format flags
- current partitions on managed disks
- numbered actions for create, delete, assign, toggle, back

**Step 2: Add create flow**

Implement prompts for:

- target disk
- role
- size or use remaining space for `root`

After creation:

- refresh the partition list
- allow immediate assignment of the new partition

**Step 3: Add delete flow**

Implement prompts for:

- partition selection
- explicit confirmation token

After delete:

- clear stale assignments
- refresh the partition list

**Step 4: Keep assign/toggle flows concise**

Preserve the existing assignment and format-flag behavior where still valid, but drive it from the refreshed managed-disk partition list.

**Step 5: Smoke test**

Run:
```bash
cargo run -p access-os-installer-cli -- --dry-run
```

Validate manually:

- create EFI/root/home/swap partitions
- delete an assigned partition and confirm the assignment clears
- move through `Review` and confirm the destructive summary is understandable

**Step 6: Commit**

```bash
git add cli/src/wizard.rs
git commit -m "feat(cli): add manual partition create and delete flow"
```

---

### Task 4: Expand Review Output For Manual Partition Edits

**Files:**
- Modify: `cli/src/wizard.rs`
- Modify: `crates/installer-core/src/backend/storage_plan.rs`

**Step 1: Show pending manual partition edits**

Update review output so it clearly states:

- partitions to create
- partitions to delete
- final role assignments
- format actions

**Step 2: Keep dry-run useful**

Ensure `--dry-run` prints the new manual partition intent clearly even though no changes are executed.

**Step 3: Verify**

Run:
```bash
cargo test --workspace
cargo run -p access-os-installer-cli -- --dry-run
```

**Step 4: Commit**

```bash
git add cli/src/wizard.rs crates/installer-core/src/backend/storage_plan.rs
git commit -m "feat(cli): expand manual partition review output"
```

---

### Task 5: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`

**Step 1: Update README**

Document that CLI manual mode can:

- create role-based partitions
- delete partitions on the selected managed disks
- require explicit confirmation for deletes

**Step 2: Update contributor guide if needed**

Add a short note about where the manual partition manager logic lives.

**Step 3: Verify**

Run:
```bash
cargo test --workspace
```

**Step 4: Commit**

```bash
git add README.md AGENTS.md
git commit -m "docs: describe CLI manual partition manager"
```
