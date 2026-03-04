# Design Doc: GTK A11y Parity Sweep

Date: 2026-03-04 (America/Chicago)

## Goal
Bring `access-os-installer` GTK4 accessibility behavior in line with the patterns used in:
- `access-launcher` (explicit accessible label/description on widgets that lack good names)
- `universal-startup-manager` (explicit `AccessibleRole` on primary interactive widgets + mnemonic labels for form fields)

Success looks like:
- Every text input and selection control has a visible label and keyboard mnemonic where applicable.
- Buttons/text inputs have explicit accessible roles set (matching `universal-startup-manager` style).
- Widgets that otherwise have poor accessible names get explicit accessible labels/descriptions (matching `access-launcher` style).
- No new dependencies; only GTK4 APIs already in use.

## Non-Goals
- No custom screen-reader announcement system (no Orca-specific IPC, no “live region” emulation).
- No UI redesign or step flow changes.
- No changes to backend install behavior.

## Current State (Installer)
The installer already:
- Enables GTK accessibility plumbing early (`GTK_A11Y=1`, clears `NO_AT_BRIDGE`).
- Uses labels + mnemonics for some combo boxes (e.g. Wi-Fi network, disk, DE, mirror, filesystem).

Gaps relative to the reference apps:
- Many `Entry`/`PasswordEntry` fields are preceded by plain labels without mnemonics (or no label at all).
- No explicit `AccessibleRole` assignments for buttons/textboxes.
- No shared helper for `Accessible` properties (label/description).

## Approach
Implement a “full parity sweep”:
1. Add a small `ui/common/a11y` helper module for:
   - setting accessible label/description (`update_property`)
   - applying standard roles (button/textbox) in one-liners
2. Update every step UI to:
   - add mnemonic labels for all text inputs and relevant selection controls
   - set `AccessibleRole::Button` for buttons
   - set `AccessibleRole::TextBox` for text inputs (`Entry`/`PasswordEntry`)
   - add explicit accessible label/description only where GTK would otherwise have a weak accessible name

## Implementation Notes
- Prefer mnemonics + `set_mnemonic_widget(...)` for input labeling (matches `universal-startup-manager`).
- Use explicit accessible label/description sparingly for widgets that don’t inherit names well (matches `access-launcher`).
- Keep widget construction mostly unchanged; only add the minimal a11y calls and missing labels.

## Files Expected To Change
- Create: `src/ui/common/a11y.rs`
- Modify: `src/ui/common/mod.rs`
- Modify: `src/ui/steps/*.rs` (all steps) to apply roles and improve field labeling

## Verification
- Build validation: `cargo check` (and `cargo test` if the crate already has tests enabled for this environment).
- Manual spot-check (optional): run the installer in a GTK session with a screen reader and verify:
  - tab order lands on the first interactive control per step
  - form fields read with correct names
  - “Back”/“Next” buttons read as buttons and are reachable by keyboard

