# Design Doc: Text Profile Package Lists

**Date:** 2026-03-06 (America/Chicago)  
**Status:** Approved  
**Scope:** Move installer package lists out of Rust code and into editable text files under `profiles/`.

## Goals

- Make package lists easier to edit without touching Rust code.
- Keep one plain text file per package group/profile.
- Preserve current install behavior on the first pass.
- Reuse the same package source for install logic and UI package previews.

## Non-Goals

- Adding profile metadata beyond package names.
- Moving service/display-manager logic into text files.
- Introducing TOML, JSON, or another structured config format.

## Chosen Approach

Add a top-level `profiles/` directory with one text file per package group:

- `profiles/base.txt`
- `profiles/gnome.txt`
- `profiles/kde.txt`
- `profiles/server.txt`
- `profiles/nvidia.txt`
- `profiles/kernel-standard.txt`
- `profiles/kernel-lts.txt`
- `profiles/kernel-zen.txt`
- `profiles/kernel-hardened.txt`

Each file contains one package per line. Blank lines and `#` comments are ignored.

## Backend Design

Keep desktop environments and kernel variants in `config_engine.rs` for labels, descriptions, availability, services, and display-manager behavior. Replace hardcoded package arrays with profile filename mappings.

`full_package_list()` should:

1. load `base.txt`
2. load the selected desktop file
3. load the selected kernel file
4. optionally load `nvidia.txt`
5. deduplicate while preserving order

Package file lookup should:

1. check `profiles/` in the current working directory
2. check `profiles/` relative to the executable location
3. return a clear error if the files are missing

## Compatibility

Seed the profile files with the exact package lists currently embedded in Rust so the first pass does not change installed packages.

Update UI package previews to read from the same package loader instead of stale hardcoded arrays, so contributors and users see the same source of truth.

## Testing

- Add unit tests for:
  - comment/blank-line parsing
  - deduplication order
  - profile composition for at least one desktop and one kernel
- Run `cargo test --workspace`
- Run `cargo run -p access-os-installer-cli -- --help` to confirm startup still works
