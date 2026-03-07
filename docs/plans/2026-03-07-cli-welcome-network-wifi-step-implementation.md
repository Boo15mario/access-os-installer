# CLI Welcome Network Status And Wi-Fi Step Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Clear the CLI screen on startup, show welcome-screen network status, and move offline users to a dedicated Wi-Fi step after `next`.

**Architecture:** Extend the CLI wizard with a `WiFiSetup` step and a cached connectivity state, keeping the rest of the installer flow unchanged.

**Tech Stack:** Rust 2021, Cargo workspace, existing CLI wizard and `installer_core::backend::network`.

---

### Task 1: Add Welcome Network Status State

**Files:**
- Modify: `cli/src/wizard.rs`

**Step 1: Extend wizard state**

Add:

- `WiFiSetup` step enum variant
- cached network status field
- startup clear-screen flag if needed

**Step 2: Clear screen once at startup**

Use ANSI clear output before the first welcome render.

**Step 3: Render network status on welcome**

Show:

- checking
- connected
- not connected

**Step 4: Commit**

```bash
git add cli/src/wizard.rs
git commit -m "feat(cli): show welcome network status"
```

---

### Task 2: Add Dedicated Wi-Fi Step

**Files:**
- Modify: `cli/src/wizard.rs`

**Step 1: Add `step_wifi_setup()`**

Provide:

- current status
- scanned SSID list
- connect/retry/back/help/quit
- `next` to `InstallOptions` when online

**Step 2: Route `next` from welcome**

If online:

- `next` -> `InstallOptions`

If offline:

- `next` -> `WiFiSetup`

**Step 3: Keep navigation predictable**

- `back` from Wi-Fi returns to welcome
- successful connect allows `next` to continue to install options

**Step 4: Commit**

```bash
git add cli/src/wizard.rs
git commit -m "feat(cli): add dedicated Wi-Fi setup step"
```

---

### Task 3: Verify

**Files:**
- Modify only if needed for compile/test cleanup

**Step 1: Run verification**

Run:
```bash
cargo test --workspace
cargo run -p access-os-installer-cli -- --help
```

**Step 2: Commit**

```bash
git add .
git commit -m "test(cli): verify welcome network and Wi-Fi step flow"
```
