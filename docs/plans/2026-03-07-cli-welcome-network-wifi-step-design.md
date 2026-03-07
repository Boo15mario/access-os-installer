# Design Doc: CLI Welcome Network Status And Wi-Fi Step

**Date:** 2026-03-07 (America/Chicago)  
**Status:** Approved  
**Scope:** Clear the CLI screen on startup, show network status on the welcome screen, and route offline users to a dedicated Wi-Fi step after they type `next`.

## Goals

- Clear the terminal when the CLI first starts.
- Show a simple network status line on the welcome screen.
- Keep `next` as the only way forward from welcome.
- Route offline users to a dedicated Wi-Fi setup step instead of blocking on welcome.
- Send users directly to `InstallOptions` after Wi-Fi is connected.

## Non-Goals

- Auto-connecting to Wi-Fi without user action.
- Adding a curses/TUI interface.
- Changing the rest of the installer step order.

## Chosen Approach

Add a dedicated `WiFiSetup` step between `Welcome` and `InstallOptions`, but only use it when connectivity is unavailable. The welcome screen performs a network check, displays the result, and branches on `next`.

## Behavior

`Welcome` should:

- clear the screen once when the CLI first starts
- show `Network: checking...`, then `connected` or `not connected`
- keep `next`, `help`, `quit`, and `back` behavior simple
- send `next` to `InstallOptions` when online
- send `next` to `WiFiSetup` when offline

`WiFiSetup` should:

- display current connectivity status
- list available Wi-Fi networks
- allow connect/retry/back/help/quit
- send `next` to `InstallOptions` once online

## Implementation Notes

- Add `WiFiSetup` to the CLI step enum in `cli/src/wizard.rs`.
- Add a cached network status field to the wizard so welcome can render status cleanly.
- Emit an ANSI terminal clear once at startup rather than shelling out to `clear`.
- Reuse the existing network scanning/connection backend helpers.

## Testing

- Run `cargo test --workspace`
- Run `cargo run -p access-os-installer-cli -- --help`
- Runtime Wi-Fi connection flow will still need validation in a live/root environment
