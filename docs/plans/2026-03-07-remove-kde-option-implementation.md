# Remove KDE Desktop Option Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove KDE from installer desktop selection and backend profile support.

**Architecture:** Delete KDE from the shared `DesktopEnv` metadata so both frontends stop showing it automatically, then remove the now-unused KDE profile file.

**Tech Stack:** Rust 2021, Cargo workspace, shared desktop metadata in `installer-core`, plain text profile files.

---

### Task 1: Remove KDE From Shared Desktop Metadata

**Files:**
- Modify: `crates/installer-core/src/backend/config_engine.rs`

**Step 1: Delete the `Kde` enum variant**

Remove KDE from:

- `DesktopEnv`
- `DesktopEnv::all()`
- label/description/profile mappings
- display-manager mappings

**Step 2: Update tests if needed**

Adjust any tests or expectations that included KDE.

**Step 3: Commit**

```bash
git add crates/installer-core/src/backend/config_engine.rs
git commit -m "refactor(core): remove KDE desktop profile"
```

---

### Task 2: Remove KDE Profile File

**Files:**
- Delete: `profiles/kde.txt`

**Step 1: Delete the file**

Remove the no-longer-used KDE package profile.

**Step 2: Verify no references remain**

Run:
```bash
rg -n "Kde|kde.txt" .
```

**Step 3: Commit**

```bash
git add profiles/kde.txt
git commit -m "refactor(profiles): remove KDE package list"
```

---

### Task 3: Verify

**Files:**
- Modify only if needed for cleanup

**Step 1: Run verification**

Run:
```bash
cargo test --workspace
cargo run -p access-os-installer-cli -- --help
```

**Step 2: Commit**

```bash
git add .
git commit -m "test: verify KDE removal"
```
