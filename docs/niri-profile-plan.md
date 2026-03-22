# Niri Profile Plan

This document defines the intended first-class Niri desktop profile for Access OS.

## Decisions Already Made

- GNOME uses `gdm`
- Niri uses `greetd` + `waygreet`
- The Niri compositor package is `niri-git`
- `access-launcher` is the default app launcher for the Niri profile
- The server profile remains separate and headless

## Profile Goal

The Access OS Niri profile should be more than "Niri installs." It should be a complete, accessible Wayland desktop option with:

- a working login experience
- accessible screen reader friendly interaction
- keyboard-first navigation
- notifications
- clipboard history
- volume management
- tray access
- default app chooser support
- event and desktop sounds

## Session Model

### GNOME profile
- display manager: `gdm`
- desktop environment: GNOME

### Niri profile
- display manager: `greetd`
- greeter: `waygreet`
- compositor: `niri-git`

### Server profile
- no graphical session manager by default

## Planned Niri Package Groups

These are grouped by role so the installer profile and packaging work can stay organized.

### Core Niri session
Required to boot into a usable Niri session.

- `niri-git`
- `greetd`
- `waygreet`
- `access-launcher`
- `xdg-desktop-portal`
- `xdg-desktop-portal-gtk`
- `wl-clipboard`
- `pipewire`
- `pipewire-pulse`
- `wireplumber`
- `polkit`
- `power-profiles-daemon`
- `brightnessctl`
- `gmrun`

### Accessibility baseline
These should be present so the session is usable by blind users.

- `orca`
- `at-spi2-core`

Additional speech/audio/runtime packages may be required once the exact accessibility stack is finalized.

### Niri desktop utilities
These are the custom utilities planned for the Niri-based Access OS desktop.

- `waynotify`
- `niri-sounds`
- `xdg-chooser`
- `wayclip`
- `wayvol`
- `waytray`
- `soundthemed`

### Basic applications
These should be reviewed and finalized based on what Access OS wants as its default lightweight desktop toolkit.

Current intended baseline includes:
- `lxterminal`
- `firefox`
- `gmrun`
- `libreoffice-fresh`
- `hunspell-en_us`
- `hyphen-en`
- `mythes-en`

### Theme baseline
The current intended visual baseline is:
- `Breeze-Dark` for GTK applications
- `Breeze` with dark colors for Qt applications
- package support via `breeze` and `breeze-gtk`

## Packaging Plan

The following packages should be added to `access-os-packages` under `packages/extra/` first:

- `waynotify`
- `niri-sounds`
- `waygreet`
- `xdg-chooser`
- `wayclip`
- `wayvol`
- `waytray`
- `soundthemed`

## Why `access-os-extra` First

These are upstream third-party projects and are best treated as curated extra packages first. If one later becomes a foundational Access OS component, it can be reconsidered for `access-os-core`.

## Planned Session Integration

The Niri desktop will need Access OS defaults and glue for:

- `greetd` configuration
- `waygreet` configuration
- Niri autostart entries
- default keybindings
- launcher integration via `access-launcher`
- startup of desktop helper daemons
- accessibility defaults

## Planned Niri Startup Components

The intended startup set should include some combination of:

- `waynotify`
- `wayclip-daemon`
- `soundthemed`
- `niri-sounds`
- `waytray-daemon`

These need to be confirmed against real-world session behavior and startup ordering.

## Planned Keybinding Areas

The Niri profile should define default bindings for:

- app launcher (`access-launcher`)
- terminal
- browser
- clipboard history (`wayclip`)
- volume mixer (`wayvol`)
- tray window (`waytray`)
- default app chooser (`xdg-chooser`)
- lock/logout/reboot/shutdown
- focus movement
- workspace movement

## Accessibility Expectations

The Niri profile should guarantee, at minimum:

- keyboard-only usability
- Orca availability
- audible feedback path that works after login
- notifications that are screen reader friendly
- clear launcher access
- clipboard and volume tools that are accessible by keyboard and screen reader

## Tasks to Complete

### Installer profile tasks
- [ ] expand `profiles/niri.txt` beyond `niri` + `gdm`
- [ ] replace `gdm` with `greetd` + `waygreet` in the Niri path
- [ ] add `access-launcher` to the Niri profile
- [ ] add finalized Niri support packages to the profile
- [ ] confirm package names once packaging work is done

### Packaging tasks
- [ ] package `waynotify`
- [ ] package `niri-sounds`
- [ ] package `waygreet`
- [ ] package `xdg-chooser`
- [ ] package `wayclip`
- [ ] package `wayvol`
- [ ] package `waytray`
- [ ] package `soundthemed`
- [ ] add all new package names to `packages/extra.list`

### Integration tasks
- [ ] create `greetd` defaults for the Niri profile
- [ ] create `waygreet` defaults for the Niri profile
- [ ] create default Niri config with startup commands and keybindings
- [ ] decide where that config lives (`access-os-config`, package repo, or both)
- [ ] test Niri login, session start, and helper daemon startup in a VM

## Open Decisions Still Needed

- Which terminal emulator should be the default in the Niri profile?
- Which browser should be the default in the Niri profile?
- Is a file manager included by default?
- Which accessibility/audio helper packages are mandatory beyond `orca` and `at-spi2-core`?
- Which Niri helper daemons are mandatory for first release versus nice-to-have?
