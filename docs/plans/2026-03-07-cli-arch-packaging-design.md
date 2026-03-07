# Design Doc: Arch Packaging For CLI Installer

**Date:** 2026-03-07 (America/Chicago)  
**Status:** Approved  
**Scope:** Package the CLI installer for an Arch-based custom repo as `access-os-installer-cli` with the public launcher command `install-access`.

## Goals

- Add a standard Arch `PKGBUILD` to this repository.
- Package only the CLI installer for now.
- Install profile files under `/usr/share/access-os-installer/profiles`.
- Expose the public command as `install-access`.
- Ensure packaged startup attempts elevation when not already running as root.

## Non-Goals

- Packaging the GTK installer in this change.
- Renaming the Cargo crate/package.
- Adding repo metadata generation in this repository.

## Chosen Approach

Keep the Rust binary build as `access-os-installer-cli`, install it under a private package path, and provide a launcher script `install-access` in `/usr/bin`. The launcher handles elevation and then execs the real binary.

## Install Layout

- `/usr/bin/install-access` — launcher script
- `/usr/share/access-os-installer/install-access-real` — actual CLI binary
- `/usr/share/access-os-installer/profiles/*` — package profile files

## Elevation Behavior

When `install-access` is run as a non-root user:

1. try `sudo`
2. if unavailable, try `pkexec`
3. if neither works, print a clear error and exit

If already root, exec the real binary directly.

## Backend Support

Update the package-profile loader so it explicitly checks `/usr/share/access-os-installer/profiles` as a packaged runtime location in addition to the current development paths.

## Packaging Files

- `PKGBUILD`
- `packaging/install-access` launcher script

An `.install` file is not required unless later install-time hooks become necessary.

## Testing

- `cargo test --workspace`
- `cargo run -p access-os-installer-cli -- --help`
- if available, `makepkg -f`
- verify the package installs the launcher, real binary, and profile files at the expected paths
