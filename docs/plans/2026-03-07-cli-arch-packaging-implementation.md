# CLI Arch Packaging Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Arch packaging for the CLI installer as `access-os-installer-cli` with the user-facing command `install-access`.

**Architecture:** Package the compiled CLI binary under `/usr/share/access-os-installer/`, ship profile files under `/usr/share/access-os-installer/profiles`, and provide a root-elevating launcher script in `/usr/bin/install-access`.

**Tech Stack:** Rust 2021, Cargo workspace, Arch `PKGBUILD`, shell launcher script.

---

### Task 1: Add Packaged Runtime Path Support

**Files:**
- Modify: `crates/installer-core/src/backend/config_engine.rs`

**Step 1: Add packaged profile search path**

Teach the loader to check:

- `/usr/share/access-os-installer/profiles`

in addition to existing development paths.

**Step 2: Verify**

Run:
```bash
cargo test -p installer-core
```

**Step 3: Commit**

```bash
git add crates/installer-core/src/backend/config_engine.rs
git commit -m "feat(core): support packaged profile path"
```

---

### Task 2: Add Launcher Script

**Files:**
- Create: `packaging/install-access`

**Step 1: Write launcher**

Behavior:

- if root: exec packaged binary
- else: try `sudo`
- else: try `pkexec`
- else: print a clear error

**Step 2: Keep paths explicit**

Use the packaged binary path:

- `/usr/share/access-os-installer/install-access-real`

**Step 3: Commit**

```bash
git add packaging/install-access
git commit -m "feat(packaging): add install-access launcher"
```

---

### Task 3: Add PKGBUILD

**Files:**
- Create: `PKGBUILD`

**Step 1: Package the CLI only**

Install:

- launcher to `/usr/bin/install-access`
- built binary to `/usr/share/access-os-installer/install-access-real`
- profile files to `/usr/share/access-os-installer/profiles/`

**Step 2: Use workspace-aware cargo build**

Build only `access-os-installer-cli`.

**Step 3: Commit**

```bash
git add PKGBUILD
git commit -m "feat(packaging): add Arch PKGBUILD for CLI installer"
```

---

### Task 4: Update Docs And Verify

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`

**Step 1: Add packaging notes**

Document:

- package name
- launcher command
- installed profile path

**Step 2: Verify**

Run:
```bash
cargo test --workspace
cargo run -p access-os-installer-cli -- --help
```

If available:

```bash
makepkg -f
```

**Step 3: Commit**

```bash
git add README.md AGENTS.md
git commit -m "docs: describe CLI Arch packaging"
```
