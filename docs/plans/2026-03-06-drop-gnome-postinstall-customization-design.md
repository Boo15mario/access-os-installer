# Design Doc: Drop GNOME Post-Install Customization

**Date:** 2026-03-06 (America/Chicago)  
**Status:** Approved  
**Scope:** Keep GNOME as an install option, but stop applying Access OS-specific GNOME post-install customization.

## Goals

- Preserve GNOME as a supported desktop environment choice.
- Keep GNOME package selection and display-manager setup unchanged.
- Stop applying post-install GNOME customization such as dconf/theme/extension tweaks.

## Non-Goals

- Renaming the GNOME profile in this change.
- Changing GNOME package selection.
- Adding a new installer toggle or profile split for custom vs standard GNOME.

## Chosen Approach

Keep the existing GNOME profile and package list, but remove the `configure_gnome()` call from the install flows. This is the smallest safe change because it affects only post-install behavior and does not alter package selection, boot setup, or general system configuration.

This is preferred over adding a new option or feature flag because the user explicitly wants the non-custom path for now, and a configuration toggle would add state and UI complexity without immediate value.

## User-Visible Behavior

- GNOME still appears as a desktop environment choice.
- Selecting GNOME still installs the GNOME session and enables `gdm`.
- The installed system uses upstream/default GNOME settings instead of the Access OS GNOME tweaks.

## Implementation Notes

- Remove GNOME-specific post-install hook calls from:
  - `cli/src/wizard.rs`
  - `gtk/src/ui/steps/install.rs`
- Leave `crates/installer-core/src/backend/config_engine.rs` unchanged so GNOME package selection remains stable.
- Leave `crates/installer-core/src/backend/install_worker.rs::configure_gnome()` in place for now to minimize churn; it simply becomes unused.

## Testing

- Run `cargo test --workspace`.
- Confirm the GNOME post-install hook is no longer called from the CLI and GTK install flows.
