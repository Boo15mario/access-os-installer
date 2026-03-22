# Installer Work Log

Use this file for short dated notes as installer work happens.

---

## 2026-03-19

- Added a focused docs hub under `docs/` for current installer work.
- Documented the current CLI step sequence from `cli/src/wizard.rs`.
- Wrote a proposed golden path centered on a VM-based CLI install.
- Added accessibility and testing checklists to guide iteration.
- Captured open product, accessibility, storage, integration, and testing questions.
- Added a dedicated Niri profile plan reflecting current decisions: `gdm` for GNOME, `greetd` + `waygreet` for Niri, `niri-git` as the compositor package, and `access-launcher` as the Niri launcher.
- Updated the Niri profile/config direction to remove `waybar`, keep `lxterminal`, add `gmrun` back as the app runner, and leave power profile handling to `waytray` while keeping brightness controls.
- Added `libreoffice-fresh` plus English spell-checking packages to the Niri profile and extended theme defaults toward a system-wide Breeze Dark setup for GTK and Qt apps.
