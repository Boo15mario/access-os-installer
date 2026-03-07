# Text Profile Package Lists Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace hardcoded installer package lists with editable text files under `profiles/` while preserving current profile behavior.

**Architecture:** Keep profile metadata in Rust, move package membership into text files, and load/merge those files in `installer-core` for both install execution and UI previews.

**Tech Stack:** Rust 2021, Cargo workspace, plain text package files, std filesystem APIs.

---

### Task 1: Add Profile Text Files

**Files:**
- Create: `profiles/base.txt`
- Create: `profiles/gnome.txt`
- Create: `profiles/kde.txt`
- Create: `profiles/server.txt`
- Create: `profiles/nvidia.txt`
- Create: `profiles/kernel-standard.txt`
- Create: `profiles/kernel-lts.txt`
- Create: `profiles/kernel-zen.txt`
- Create: `profiles/kernel-hardened.txt`

**Step 1: Copy the current package lists into text files**

Use one package per line. Preserve current package membership exactly.

**Step 2: Add comments only where helpful**

Keep the files simple and easy to edit.

**Step 3: Commit**

```bash
git add profiles
git commit -m "feat(profiles): add text package lists"
```

---

### Task 2: Refactor Package Loading In `installer-core`

**Files:**
- Modify: `crates/installer-core/src/backend/config_engine.rs`
- Test: `crates/installer-core/src/backend/config_engine.rs`

**Step 1: Replace hardcoded package arrays with filename mappings**

Keep labels/descriptions/services in Rust, but add helpers that map desktop and kernel choices to profile filenames.

**Step 2: Add profile loading helpers**

Implement helpers to:

- resolve the `profiles/` directory
- read and parse package files
- ignore comments and blank lines
- merge/deduplicate package lists in order

**Step 3: Change package assembly to return owned strings**

Update `full_package_list()` and any dependent APIs accordingly.

**Step 4: Add tests**

Run:
```bash
cargo test -p installer-core
```

Cover:

- parser behavior
- deduplication order
- selected profile composition

**Step 5: Commit**

```bash
git add crates/installer-core/src/backend/config_engine.rs
git commit -m "refactor(core): load package profiles from text files"
```

---

### Task 3: Update Call Sites And UI Preview

**Files:**
- Modify: `crates/installer-core/src/backend/install_worker.rs`
- Modify: `gtk/src/ui/steps/de.rs`

**Step 1: Update install worker**

Handle the new owned package list type when building the `pacstrap` command.

**Step 2: Update desktop preview UI**

Show package previews from the same profile loader used by the installer. If loading fails, show a short error instead of crashing.

**Step 3: Verify**

Run:
```bash
cargo test --workspace
```

**Step 4: Commit**

```bash
git add crates/installer-core/src/backend/install_worker.rs gtk/src/ui/steps/de.rs
git commit -m "refactor(ui): use text package profiles for previews"
```

---

### Task 4: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`

**Step 1: Document `profiles/`**

Point contributors to `profiles/*.txt` for package changes.

**Step 2: Final verification**

Run:
```bash
cargo test --workspace
cargo run -p access-os-installer-cli -- --help
```

**Step 3: Commit**

```bash
git add README.md AGENTS.md
git commit -m "docs: describe text package profiles"
```
