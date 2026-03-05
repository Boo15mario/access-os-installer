# Repository Guidelines

## Project Structure

- `src/`: Rust sources (GTK4 app).
- `src/app/`: application bootstrap + shared state.
- `src/ui/steps/`: wizard-style screens (Welcome, Wi-Fi, Disk, Install, etc.).
- `src/ui/common/`: shared UI helpers (layout + accessibility helpers in `a11y.rs`).
- `src/backend/`: system probing + install logic (disk/network/preflight/storage planning).
- `src/services/` and `src/mappers/`: supporting utilities and data mapping.
- `assets/`: runtime assets (e.g. `assets/login.wav`, `assets/access-os-installer.svg`).
- `docs/plans/`: design docs and implementation plans for major changes.

## Build, Test, and Development Commands

Run these from the repo root:

```bash
cargo build          # compile debug binary
cargo test           # run unit tests
cargo run            # run the GTK installer locally (requires a display session)
cargo fmt            # format (rustfmt)
cargo clippy         # lint (Clippy)
cargo build --release
```

Notes:
- This is a GUI app; headless shells will fail with "Failed to open display".
- For screen reader debugging, ensure AT-SPI is active (the app sets `GTK_A11Y=atspi` when unset).

## Coding Style & Naming

- Rust 2021 edition (`Cargo.toml`).
- Indentation: 4 spaces; keep functions small and step builders readable.
- Naming: `snake_case` (fns/modules), `CamelCase` (types), `SCREAMING_SNAKE_CASE` (constants).
- Prefer using `src/ui/common/a11y.rs` helpers instead of ad-hoc accessible labels/roles.

## Testing Guidelines

- Unit tests live alongside code under `#[cfg(test)]` (e.g. in `src/backend/*`).
- Run `cargo test` before pushing.
- UI changes require a quick manual pass: keyboard-only navigation and Orca announcements across step transitions.

## Commit & Pull Request Guidelines

- Use Conventional Commit style seen in history. Examples: `fix(a11y): ...`, `feat(storage): ...`, `docs: ...`.
- PRs should explain what changed and why.
- PRs should state how it was tested (`cargo test`, plus manual UI/Orca notes).
- PRs should include screenshots/screencast for UI behavior changes.

## Safety (Installer)

Parts of `src/backend/` perform destructive disk actions. Do not test the install path on your primary machine; use a VM or a spare disk and double-check the selected `/dev/...` target.
