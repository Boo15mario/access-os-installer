# Design Doc: Dialog Pickers and Startup Sound

## Goal
Improve the host/settings step UI by showing dropdown pickers when timezone/locale/keymap data are missing, while preserving the host dropdown; play a bundled “Digital Pluck” startup sound the moment the installer window opens.

## Scope
- Keep current host dropdown.
- When a selected host lacks timezone/locale/keymap values:
  - Replace the corresponding text entry with a dropdown of curated options.
  - When values exist, show them as read-only text.
- Bundle a short “Digital Pluck” audio file and play it automatically after the installer window is presented.
- Playback should be best-effort, non-blocking, and fail silently if the system cannot play sound.

## Architecture

### Host Settings Step
- Reuse the existing Step 3 layout.
- On host selection change:
  - Parse `configuration.nix` for `time.timeZone`, `i18n.defaultLocale`, `console.keyMap`.
  - Default to current values if found; otherwise show dropdowns.
- Dropdown catalogs (initial set):
  - Timezones: `America/Chicago`, `America/New_York`, `America/Los_Angeles`, `UTC`.
  - Locales: `en_US.UTF-8`, `en_GB.UTF-8`, `es_US.UTF-8`, `fr_FR.UTF-8`.
  - Keymaps: `us`, `us-intl`, `uk`, `de`, `fr`, `es`.
- Selected/picked values continue to populate `AppState` and the existing local-settings writing logic.

### Startup Sound
- Add `assets/startup-digital-pluck.ogg`.
- After the main window is presented, invoke a try-play procedure (e.g., `paplay` or `aplay`) to play the file once.
- Errors (missing file/player) log for debugging but do not interrupt the UI.

## Data Flow
- When Step 3 is shown, run detection on the selected host’s config directory.
- Keep detection results in memory; once a dropdown choice is made, update the in-memory value so later steps reuse the choice.
- The sound file path resolves relative to the binary (e.g., via `assets/` path).

## Error Handling
- Missing config values simply show dropdowns; no additional alerts required.
- If playback fails, proceed silently (maybe log via `eprintln!`).
- Dropdowns keep consistent options even if host data changes mid-navigation.

## Verification
1. Run `cargo check`.
2. Manual tests:
   - Select host with full time/locale/keymap -> see read-only labels.
   - Select host missing any value -> see dropdown with curated list.
   - Play with each dropdown to ensure state updates.
   - Startup sound plays once on the first window display (verify logs if needed).
