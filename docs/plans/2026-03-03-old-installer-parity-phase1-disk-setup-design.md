# Old Installer Parity Phase 1 Disk Setup Design

**Date:** 2026-03-03  
**Status:** Approved  
**Scope:** Make the new GTK installer storage flow behave like the old installer's advanced disk setup path, while keeping the new preflight/review structure.

## Goal

Add a storage setup flow that supports:
- Automatic or manual partitioning
- Optional separate `/home`
- Optional `/home` on another disk
- Automatic partitioning of the second disk when `/home` is separate and automatic mode is selected
- Explicit destructive-action summary before install

## In Scope

- New `Disk Setup` step after disk selection
- New app state for storage setup choices
- Storage planner/resolver backend model
- Layout-driven install execution (not hardcoded `p1/p2/p3`)
- Review screen destructive plan summary and countdown confirmation

## Out of Scope (Phase 1)

- Mirror-region selection
- Install profile parity (`minimal`, `hyprland`, `i3`)
- Post-install advanced menu (chroot/extra packages)

## User Flow

1. `Disk Selection` (existing):
- User selects install target disk from internal disks.

2. `Disk Setup` (new):
- Setup mode: `Automatic` or `Manual`
- Swap mode: `Swap partition` or `Swap file`
- If swap file: enter size in MB
- Home mode: `On root` or `Separate /home`
- If separate `/home`: location `Same disk` or `Another disk`
- If `Another disk`: select home disk
- If `Manual`: select EFI and root partitions, plus optional home partition
- Format toggles in manual mode: `Format EFI`, `Format root`, `Format /home`
- `Removable media` toggle

3. `Settings`, `Preflight` (existing):
- Continue through existing host/user/kernel settings and checks.

4. `Review` (enhanced):
- Existing summary plus explicit destructive plan:
  - Disks to wipe
  - Partitions to create
  - Partitions to format
  - Mount map (`/`, `/boot`, optional `/home`)
- Countdown gate before install starts.

5. `Install`:
- Uses resolved storage layout actions.

## Architecture

### Frontend

- Add `DiskSetupState` to `AppState` in `src/main.rs`.
- Add `build_disk_setup_step` between `build_step1` and `build_de_step`.
- Update stack transitions:
  - `disk -> disk_setup -> desktop_env`
- Add validation on step transition and block Next with clear inline errors.

### Backend

- Create `src/backend/storage_plan.rs` with:
  - setup mode enums
  - swap/home policy enums
  - selected disk/partition references
  - `ResolvedInstallLayout` with concrete actions
- Expose module from `src/backend/mod.rs`.
- Add resolver validation functions that produce actionable errors.

### Execution

- Update partitioning/mount logic in `src/backend/disk_manager.rs` and installer orchestration in `src/main.rs`:
  - automatic mode creates required partitions on selected disks
  - manual mode uses selected partitions and format flags
  - optional `/home` mount supported
  - swap partition or swap file creation supported
- Keep GNOME customization and dotfiles non-fatal.

## Validation Rules

- Automatic mode:
  - install disk required
  - if separate `/home` on another disk: second disk required and must differ from install disk
  - swap file size must be numeric and above minimum threshold

- Manual mode:
  - EFI and root required
  - if separate home enabled: home partition required
  - home cannot equal EFI/root

- General:
  - selected partitions/disks must still exist at execution time
  - if unavailable at install start, abort before destructive actions

## Error Handling

- Fatal:
  - invalid or stale storage selections
  - partitioning/formatting/mount failures
- Non-fatal warnings:
  - GNOME post-config failures
  - dotfiles application failures
- Existing retry flow remains for pacstrap and configuration stages.

## Testing Strategy

- Unit tests in new `storage_plan` module:
  - auto same-disk home
  - auto other-disk home
  - manual mapping validation failures
  - swap partition vs swap file
- Extend `disk_manager` tests for:
  - partition path and mount-plan helpers
  - layout-driven operations where testable
- Manual verification:
  - auto install with home on root
  - auto install with home on second disk
  - manual install with existing partitions
  - review countdown blocks premature install

## Migration Notes

- Existing default flow should continue to work with automatic mode preselected.
- Review summary remains the final human confirmation checkpoint.
