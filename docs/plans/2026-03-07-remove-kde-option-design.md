# Design Doc: Remove KDE Desktop Option

**Date:** 2026-03-07 (America/Chicago)  
**Status:** Approved  
**Scope:** Remove KDE from installer desktop options and remove its backend/profile support.

## Goals

- Remove KDE from the selectable desktop options.
- Remove KDE-specific backend metadata and profile file support.
- Keep the rest of the desktop selection flow unchanged.

## Non-Goals

- Replacing KDE with another desktop in this change.
- Reworking other desktop options.
- Changing unrelated profile loading logic.

## Chosen Approach

Fully remove KDE from the shared desktop enum and its profile file. Both CLI and GTK desktop selectors already build their options from the shared backend, so removing KDE at the source removes it everywhere cleanly.

## Implementation Notes

- Remove `Kde` from `crates/installer-core/src/backend/config_engine.rs`
- Remove KDE label/description/profile/display-manager branches
- Delete `profiles/kde.txt`
- Update any tests or references that expect KDE to exist

## Testing

- Run `cargo test --workspace`
- Run `cargo run -p access-os-installer-cli -- --help`
- Confirm no `Kde` or `kde.txt` references remain
