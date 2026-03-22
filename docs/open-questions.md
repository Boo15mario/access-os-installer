# Open Questions

These are the main unanswered questions blocking a clearly defined first release path for the installer.

## Product Decisions

- The project currently intends to support GNOME, Niri, and Server profiles.
- GNOME uses `gdm`.
- Niri uses `greetd` + `waygreet`.
- The Niri compositor package is `niri-git`.
- Is the CLI installer the official primary path for first release?
- What level of GTK parity is required before release?
- What exact install scenarios are in scope for version 1?

## Storage and Install Decisions

- Is `xfs` intentionally the default filesystem?
- What partitioning scenarios must be supported on day one?
- Is separate `/home` support required for first release, or can it be considered advanced?
- What should happen when networking is unavailable but an offline-capable install might still be possible?

## Accessibility Decisions

- What accessibility behavior is guaranteed in the live session?
- What accessibility behavior is guaranteed after install?
- Which speech/audio packages are considered mandatory?
- What should the installer do when accessibility dependencies are missing or broken?

## Integration Decisions

- How do installer profiles map to package repo groups in `access-os-packages`?
- Which repo is the final source of truth for package selection logic?
- How are `access-grub`, `access-os-plymouth-theme`, wallpaper, and artwork integrated into installed systems?
- Which parts of `access-os-config` must be applied by installer logic versus package/config defaults?

## Testing Decisions

- What VM platform should be the standard test environment?
- What is the minimum regression suite before merging installer changes?
- What should count as a release-blocking accessibility bug?
