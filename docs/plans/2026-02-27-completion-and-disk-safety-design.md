# Design Doc: Completion Actions and Disk Safety

## Goal
Add a dedicated post-install completion screen with `Reboot` and `Exit Installer` actions, and make disk selection safer by replacing manual drive entry with internal-drive selection plus destructive confirmation.

## Scope
- Add Step 5 (`Installation Complete`) to the GTK stack.
- On successful install, navigate from Step 4 to Step 5.
- Add shared unmount cleanup for both reboot and exit actions.
- If cleanup fails, show force fallback actions (`Force Reboot`, `Force Exit`).
- Replace Step 1 disk text entry with an internal-drive dropdown.
- Require explicit erase confirmation text before allowing progression.

## Architecture

### UI Flow
1. `Welcome` -> `Wi-Fi` (if needed) -> `Disk` -> `Repo` -> `Host` -> `Installing` -> `Complete`.
2. Disk step now shows detected internal drives only.
3. Disk step blocks progression until:
- a drive is selected
- confirmation text matches required token (`ERASE`)
4. Installing step continues to show progress and errors.
5. Complete step handles reboot/exit action selection and force fallback.

### Backend and System Action Behavior
1. Disk discovery uses `lsblk -J` and filters to internal disks.
2. Cleanup routine attempts unmount order:
- `/mnt/boot`
- `/mnt`
3. Action handling:
- `Reboot`: cleanup success -> reboot command
- `Exit Installer`: cleanup success -> close window
4. Cleanup failure handling:
- Show failure details in status label
- Reveal force actions that bypass cleanup and proceed

## Error Handling
- No internal drives found: show error label and keep Next disabled.
- Invalid erase confirmation: keep Next disabled.
- Cleanup failure: no hard block; force actions become available.
- Reboot command failure: show command error in status label.
- Install failure: remain on Step 4 and show error; do not navigate to Step 5.

## Accessibility
- Keep all status and errors in persistent labels (no transient-only dialogs).
- Ensure button labels are explicit (`Exit Installer`, `Force Reboot`, `Force Exit`).

## Verification
1. Build checks: `cargo check`.
2. Manual behavior checks:
- Internal-only drives appear in Step 1.
- Next is disabled until selection + `ERASE`.
- Successful install moves to Step 5.
- Reboot/Exit attempts unmount first.
- Unmount failure exposes force fallback actions.
