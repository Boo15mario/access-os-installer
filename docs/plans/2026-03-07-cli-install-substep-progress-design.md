# Design Doc: CLI Install Sub-Step Progress

**Date:** 2026-03-07 (America/Chicago)  
**Status:** Approved  
**Scope:** Add short sub-step progress messages to the CLI install stage without dumping raw command output.

## Goals

- Keep the current coarse `1/7` install stages.
- Add short, readable sub-status lines during long-running install work.
- Reuse one backend progress mechanism instead of duplicating status logic in the CLI.
- Preserve current behavior for callers that do not need sub-step progress.

## Non-Goals

- Streaming raw stdout/stderr from installer commands.
- Building a curses/TUI progress UI.
- Reworking the GTK progress model in this change.

## Chosen Approach

Add optional progress callbacks to the shared backend and emit one-line status messages before major sub-actions. The CLI will pass a printing callback so screen readers get continuous progress updates. GTK will pass no callback and keep its current higher-level progress handling.

## Behavior

Examples of new CLI sub-status lines:

- `Partitioning disk /dev/nvme0n1`
- `Formatting /dev/nvme0n1p1 as vfat`
- `Mounting /dev/nvme0n1p3 at /mnt`
- `Installing packages with pacstrap`
- `Generating fstab`
- `Setting timezone`
- `Generating locales`
- `Creating user account`
- `Enabling NetworkManager`

Messages should be emitted before the corresponding action starts, so the user hears forward movement even when a step takes time.

## Implementation Notes

- Add a lightweight progress callback type in `installer-core`.
- Thread `Option<&dyn Fn(&str)>` through the long-running backend functions.
- Instrument:
  - `disk_manager::execute_layout()`
  - `disk_manager::setup_swap_file()`
  - `install_worker::stage_system_config_repo()`
  - `install_worker::overlay_staged_config_to_target()`
  - `install_worker::run_pacstrap()`
  - `install_worker::generate_fstab()`
  - `install_worker::configure_system()`
- Update the CLI install flow to pass a closure that prints the sub-status lines.
- Keep GTK on the existing no-callback path for now.

## Testing

- Run `cargo test --workspace`
- Run `cargo run -p access-os-installer-cli -- --help`
- Verify behavior remains unchanged when the callback is omitted
