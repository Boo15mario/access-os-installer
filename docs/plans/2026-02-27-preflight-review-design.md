# Design Doc: Preflight and Review Gates

## Goal
Add a dedicated `Preflight` step and `Review & Confirm` step that prevent avoidable installation failures while preserving a fast installer flow.

## Scope
- Add `Preflight` screen between host/user setup and install.
- Add `Review & Confirm` screen before install execution.
- Enforce hard blockers in preflight:
  - UEFI mode detected
  - Target disk selected
  - Internet reachable
- Show soft warnings:
  - RAM below `8 GiB`
  - Target disk capacity below `128 GiB`
- Require explicit acknowledgement checkbox on review screen before install when warnings exist.

## User Flow
`Welcome` -> `Wi-Fi (if needed)` -> `Disk` -> `Repo` -> `Host/User` -> `Preflight` -> `Review & Confirm` -> `Installing` -> `Complete`

## Architecture

### Preflight Step
- Runs checks when entering the screen.
- Displays grouped results with status:
  - `Pass`
  - `Warn`
  - `Fail`
- Continue button remains disabled while any hard blocker is `Fail`.
- Provides back navigation to edit previous selections.

### Review & Confirm Step
- Displays final install summary from `AppState`:
  - selected disk
  - repository URL
  - selected host
  - username/timezone/locale
- Displays any preflight soft warnings.
- Install button is disabled until acknowledgement checkbox is enabled when warnings are present.

## Data and State Model
- Keep preflight results in memory for the current run.
- Recompute preflight checks every time the user returns to `Preflight`.
- If a user changes prior inputs and returns, results are refreshed and review state updates accordingly.

## Check Definitions

### Hard Blockers
1. UEFI mode: pass if `/sys/firmware/efi` exists.
2. Disk selected: pass if `state.drive` is non-empty and valid.
3. Internet reachable: pass when connectivity check succeeds.

### Soft Warnings
1. Low RAM: warn if RAM `< 8 GiB`.
2. Low disk capacity: warn if selected disk size `< 128 GiB`.

## Error Handling
- Hard-fail checks keep user on `Preflight` with clear failure text.
- Warning-only results allow progression.
- `Review & Confirm` cannot proceed without warning acknowledgement when warnings exist.
- If check execution fails unexpectedly (command/system error), render a fail with actionable text.

## Accessibility
- Persistent labels for all result and error text.
- Status labels use explicit words (`Pass`, `Warn`, `Fail`) instead of color-only signaling.
- Warning acknowledgement control has explicit label text.

## Verification
1. Build passes (`cargo check`) in supported shell.
2. Manual checks:
  - Simulate no UEFI -> preflight blocks.
  - Clear selected disk -> preflight blocks.
  - Simulate no network -> preflight blocks.
  - Low RAM/disk thresholds show warnings but allow continue.
  - Review screen requires acknowledgement before install when warnings exist.
