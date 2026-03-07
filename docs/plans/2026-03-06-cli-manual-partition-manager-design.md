# Design Doc: CLI Manual Partition Manager

**Date:** 2026-03-06 (America/Chicago)  
**Status:** Approved  
**Scope:** Extend the CLI installer's manual disk setup flow so users can create, delete, assign, and format partitions for installer roles.

## Goals

- Allow manual disk setup in the CLI to create new partitions, not just select existing ones.
- Allow deleting partitions on the selected install disk and optional separate `/home` disk.
- Keep the UX line-oriented and predictable for screen reader users.
- Preserve strict role validation for `EFI`, `root`, `home`, and `swap`.

## Non-Goals

- Generic partition editing outside installer roles.
- Managing disks outside the selected install disk and optional `/home` disk.
- Reworking the overall CLI step order.

## Chosen Approach

Extend the existing `Disk Setup -> Manual partitions` flow into a role-driven partition manager. The CLI remains a simple numbered menu with short prompts and explicit confirmations. The backend gains explicit manual partition operations instead of inferring everything from final partition selections alone.

This is preferred over a command-shell style interface because it keeps prompts stable, shorter, and easier to use with a screen reader. It also avoids splitting partition edits across multiple screens or deferring too much state until install time.

## UX Flow

In `Manual partitions`, the CLI should first speak a compact summary:

- current assigned roles (`EFI`, `root`, optional `home`, optional `swap`)
- current format flags
- current partitions on the managed disks

The menu then offers these actions:

- `create partition`
- `delete partition`
- `assign role`
- `toggle format flags`
- `back`

`create partition` is role-based only. The user chooses the target disk, role, and size where applicable. `EFI`, `home`, and `swap` always require a size. `root` may use the remaining free space or a user-provided size.

`delete partition` may target any partition on the managed disks, including one currently assigned to an installer role. If the partition is assigned, the CLI warns that the assignment will be cleared and requires an explicit typed confirmation before deleting it.

After every create or delete action, the partition list refreshes and stale role assignments are cleared automatically.

## Backend Design

Add explicit manual partition intent to the shared storage model in `crates/installer-core`:

- pending create operations with target disk, role, size, and filesystem/type metadata
- pending delete operations by partition path

The disk manager should expose focused helpers to:

- list partitions on allowed disks
- create a single GPT partition for a requested installer role
- delete a selected partition

Role mapping stays fixed:

- `EFI` -> GPT type `ef00`, formatted `vfat`
- `root` -> GPT type `8300`, formatted as the selected root filesystem
- `home` -> GPT type `8300`, formatted `ext4`
- `swap` -> GPT type `8200`, formatted `swap`

## Safety Rules

- Operations are limited to the selected install disk and, when configured, the separate `/home` disk.
- Only one partition may be assigned to each installer role.
- `home` is only available when `/home` is separate.
- `swap` is only available when swap mode is `Partition`.
- Delete requires an explicit confirmation token.
- Review must show pending partition edits and the final resolved install layout before `install`.

## Error Handling

- Reject any create/delete request outside the allowed disks.
- Reject impossible role combinations early with a direct message.
- Surface partitioning command errors directly from the backend.
- If an operation fails, return to the manual partition manager with refreshed state.

## Testing

- Add `installer-core` unit tests for manual operation validation and stale-assignment clearing.
- Add CLI-focused tests where practical for confirmation parsing and role availability.
- Verify with `cargo test --workspace`.
- Verify behavior manually with `cargo run -p access-os-installer-cli -- --dry-run`.
