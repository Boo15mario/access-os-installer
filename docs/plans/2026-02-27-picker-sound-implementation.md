# Dialog Pickers and Startup Sound Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show dropdowns for missing timezone/locale/keymap values after host selection, keep host dropdown as-is, and play a bundled “Digital Pluck” effect when the installer window opens.

**Architecture:** Extend Step 3 to detect host-provided settings, swapping entries for dropdowns when values are absent, and keep a new playback helper in `build_ui` to fire once after the window is presented. Include the new `assets/startup-digital-pluck.ogg` file bundled with the binary.

**Tech Stack:** Rust, GTK4, `std::fs`, `std::process::Command`, `rodio`/system audio tool.

---

### Task 1: Detect host-provided settings and track pickable state

**Files:**
- Modify: `src/main.rs`
- Modify: `src/backend/config_engine.rs` (if needed for detection helper)

**Step 1: Write the failing test**

```rust
// No test for UI logic; rely on manual verification once wired.
```

**Step 2: Run baseline**

Run: `cargo check`
Expected: PASS.

**Step 3: Implement detection logic**

```rust
// After host selection: inspect config_engine::check_settings()
// Update AppState with Option<String> for timezone/locale/keymap detection mode.
// Provide helper to know if dropdown is needed.
```

**Step 4: Run build**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs src/backend/config_engine.rs
git commit -m "feat: detect host settings for conditional pickers"
```

### Task 2: Render conditional dropdowns in Step 3

**Files:**
- Modify: `src/main.rs`

**Step 1: Write the failing test**

```rust
// UI change; no automated test beyond cargo check.
```

**Step 2: Run baseline**

Run: `cargo check`
Expected: PASS.

**Step 3: Implement UI updates**

```rust
// Replace timezone/locale/keymap Entry builders with logic:
// if detection flag present -> show label
// else -> show DropDown with curated options
// store selection back into AppState.
```

**Step 4: Run build**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: render dialog-style pickers for missing settings"
```

### Task 3: Add bundled startup sound and playback helper

**Files:**
- Add: `assets/startup-digital-pluck.ogg`
- Modify: `src/main.rs`
- Update: `Cargo.toml` if new dependency (e.g., `rodio`)

**Step 1: Write the failing test**

```rust
// Not applicable; rely on manual verification.
```

**Step 2: Run baseline**

Run: `cargo check`
Expected: PASS.

**Step 3: Implement playback**

```rust
// Add helper to invoke `Command::new("paplay").arg(asset_path)` after window.present.
// Fallback: if playback fails, log and continue.
```

**Step 4: Run build**

Run: `cargo check`
Expected: PASS.

**Step 5: Commit**

```bash
git add assets/startup-digital-pluck.ogg src/main.rs
git commit -m "feat: add bundled startup sound"
```

### Task 4: Verification

**Files:**
- Modify: none

**Step 1: Compile**

Run: `cargo check`
Expected: PASS.

**Step 2: Manual verification**

Run the installer and confirm:
- Host with full settings shows labels.
- Host missing any field shows dropdown with defined catalogs.
- Making selections updates state.
- Startup sound plays once on initial window show.

**Step 3: Final commit (if needed)**

```bash
git add src/main.rs assets/startup-digital-pluck.ogg
git commit -m "feat: finalize dialog pickers and startup sound"
```
