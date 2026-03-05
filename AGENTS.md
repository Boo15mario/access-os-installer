# Repository Guidelines

## Project Structure & Module Organization

- `Cargo.toml`: workspace root (Rust 2021).
- `crates/installer-core/`: shared backend library (disk/network/preflight/storage planning + services).
- `cli/`: line-based CLI wizard intended to work well with screen readers (no curses/TUI).
- `gtk/`: GTK4 installer app (wizard UI + accessibility helpers). `access-os-installer.desktop` targets this binary.
- `assets/`: runtime assets (e.g. `assets/login.wav`, `assets/access-os-installer.svg`).
- `docs/plans/`: design docs and implementation plans for major changes.

## Build, Test, and Development Commands

Run these from the repo root:

```bash
cargo build --workspace                     # compile all workspace crates
cargo test --workspace                      # run unit tests
cargo run -p access-os-installer-cli -- --dry-run
cargo run -p access-os-installer            # GTK; requires a display session
cargo fmt --all                             # format (rustfmt)
cargo clippy --workspace --all-targets      # lint (Clippy)
cargo build -p access-os-installer --release
```

Notes:
- This is a GUI app; headless shells will fail with "Failed to open display".
- Prefer `--dry-run` while iterating; it prints the computed plan without touching disks or installing packages.
- For screen reader debugging, ensure AT-SPI is active (the app sets `GTK_A11Y=atspi` when unset).

## Coding Style & Naming

- `cargo fmt` is the baseline; keep diffs clean and mechanical formatting out of review.
- Indentation: 4 spaces; keep functions small and step builders readable/testable.
- Naming: `snake_case` (fns/modules), `CamelCase` (types), `SCREAMING_SNAKE_CASE` (constants).
- Put install logic in `crates/installer-core/` and call it from `cli/` and `gtk/` (avoid duplicating behavior).
- GTK: prefer `gtk/src/ui/common/a11y.rs` helpers over ad-hoc accessible labels/roles.

## Testing Guidelines

- Unit tests live alongside code under `#[cfg(test)]` (mostly in `crates/installer-core/src/...`).
- Run `cargo test` before pushing.
- UI changes require a quick manual pass: keyboard-only navigation and Orca announcements across step transitions.

## Commit & Pull Request Guidelines

- Use Conventional Commit style seen in history. Examples: `fix(a11y): ...`, `feat(storage): ...`, `docs: ...`.
- PRs should explain what changed and why.
- PRs should state how it was tested (`cargo test`, plus manual UI/Orca notes).
- PRs should include screenshots/screencast for UI behavior changes.

## Safety (Installer)

Parts of `crates/installer-core/` perform destructive disk actions. Do not test the install path on your primary machine; use a VM or a spare disk and double-check the selected `/dev/...` target.
