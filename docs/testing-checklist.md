# Testing Checklist

Use this file to keep installer testing repeatable and honest.

## Fast Dev Checks

Run from the repo root:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
cargo run -p access-os-installer-cli -- --dry-run
```

## CLI Dry-Run Checks

- [ ] installer starts cleanly
- [ ] help text is accurate
- [ ] welcome step is understandable
- [ ] network flow is understandable
- [ ] install option choices are clear
- [ ] disk selection language is clear
- [ ] disk setup summary is understandable
- [ ] review step is trustworthy and readable
- [ ] dry-run plan output is complete enough to debug issues

## VM Install Checks

Environment:
- VM platform: **TBD**
- firmware: UEFI
- test disk layout: single virtual disk

Checklist:
- [ ] live ISO boots
- [ ] speech/accessibility path works in the live environment
- [ ] installer launches
- [ ] automatic partitioning succeeds
- [ ] package selection/profile path works
- [ ] install completes
- [ ] reboot succeeds
- [ ] bootloader is usable
- [ ] installed system reaches expected login/session state
- [ ] installed system remains accessible after reboot

## Manual Partitioning Regression Checks

- [ ] create EFI partition flow is clear
- [ ] create root partition flow is clear
- [ ] create home partition flow is clear
- [ ] create swap partition flow is clear
- [ ] delete confirmation token is unambiguous
- [ ] final plan reflects queued manual actions accurately

## Release Readiness Checks

- [ ] no blocker accessibility regressions known
- [ ] no blocker disk safety regressions known
- [ ] profile/package mapping verified
- [ ] live environment dependencies for real install are present
- [ ] packaging launcher path (`install-access`) works as expected
