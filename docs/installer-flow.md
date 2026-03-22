# Installer Flow

This document describes the current intended installer flow, with emphasis on the CLI path because it is the strongest accessibility-first interface in the project.

## Current CLI Step Order

Based on `cli/src/wizard.rs`, the wizard currently moves through these major steps:

1. Welcome
2. Wi-Fi Setup (only when internet is not already available)
3. Install Options
4. Disk Selection
5. Disk Setup
6. Regional
7. User Settings
8. Review
9. Install
10. Complete

## Step Summary

### 1. Welcome
Purpose:
- introduce the installer
- confirm root access
- show current network status
- establish the global commands: `next`, `back`, `help`, `quit`

Expected behavior:
- clear version string
- clear statement that the interface is line-based
- no surprises before the user chooses to continue

### 2. Wi-Fi Setup
Purpose:
- connect to Wi-Fi if internet is not already available
- allow retry/check before continuing

Expected behavior:
- user can scan and connect
- user can re-check network status
- `next` only continues once internet is available

### 3. Install Options
Purpose:
- choose desktop environment
- choose kernel variant
- choose whether Nvidia support is enabled
- choose whether this is a removable media install

Expected behavior:
- choices are easy to review in text form
- changing an option does not hide other state
- desktop selection is required before continuing

### 4. Disk Selection
Purpose:
- choose the target install disk
- establish the main disk context before any partitioning choices

Expected behavior:
- all listed disks are clearly identified
- size and device path should be obvious
- destructive risk should begin being communicated here

### 5. Disk Setup
Purpose:
- choose automatic or manual storage setup
- choose filesystem and swap/home behavior
- define partitioning actions

Expected behavior:
- destructive actions are explicit
- planned layout can be reviewed before execution
- manual mode makes creates/deletes very clear
- role-based partition language should stay consistent: EFI, root, home, swap

### 6. Regional
Purpose:
- choose mirror region
- choose timezone
- choose locale
- choose keymap

Expected behavior:
- defaults should be sensible
- choices should be readable linearly in a screen reader
- terminology should match installed system behavior

### 7. User Settings
Purpose:
- collect hostname
- collect username
- collect password

Expected behavior:
- validation should be clear and immediate
- password prompts should be explicit about confirmation requirements
- wording should make it obvious what becomes the installed account

### 8. Review
Purpose:
- show the computed install plan before execution
- confirm all important choices in one place

Expected behavior:
- this should be the trust-building step
- disk actions, package/profile choices, and user settings should be summarized plainly
- destructive consequences should be restated here

### 9. Install
Purpose:
- execute the install plan
- show progress in understandable text

Expected behavior:
- progress should be meaningful, not noisy
- failures should identify the failing phase clearly
- user should not be left guessing whether the system is still working

### 10. Complete
Purpose:
- clearly state whether install succeeded
- provide next action guidance

Expected behavior:
- explain whether a reboot is expected
- explain any post-install note the user must know before rebooting

## Global CLI Principles

The CLI path should remain:
- line-oriented
- fully keyboard-driven
- predictable in command vocabulary
- readable in sequence by a screen reader
- explicit about destructive actions

## Immediate Documentation Gap

This doc tracks the intended high-level flow, but it still needs a deeper pass on:
- exact subflows inside disk setup
- exact validation rules for user/account input
- exact review output structure
- exact install progress phases
