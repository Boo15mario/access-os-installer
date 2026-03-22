# Golden Path

The goal is to define the first install path that should become boring, reliable, and easy to test.

## Proposed First Golden Path

Target:
- VM install first
- UEFI boot
- single disk
- automatic partitioning
- CLI installer path
- one desktop environment
- one kernel default
- no optional complexity unless needed for baseline success

## Why This Path First

This path gives the project the fastest route to a complete and repeatable success case:
- easier to test safely
- fewer variables when debugging
- strongest accessibility story because the CLI path is line-based
- creates a stable baseline before manual partitioning and wider hardware support are expanded

## Proposed Baseline Choices

These should be confirmed and edited as the project decides them.

- Installer interface: CLI
- Storage mode: automatic
- Boot mode: UEFI
- Disk layout: single disk
- Home layout: on root
- Swap: default recommended setting
- Filesystem: **TBD** (`xfs` appears to be the current default in CLI state)
- Desktop environment: **TBD**
- Kernel: standard
- Nvidia support: off by default for the golden path unless the test target requires it
- Removable media install: off

## Golden Path Success Criteria

A test of the golden path should count as successful only if all of the following are true:

1. the live environment boots
2. the CLI installer launches cleanly
3. prompts are understandable with a screen reader
4. automatic partitioning computes a valid plan
5. install completes without manual repair
6. the installed system reboots successfully
7. the installed system is usable with keyboard-first interaction
8. the core accessibility path works after reboot

## Scope Excluded From First Golden Path

These can be added after the baseline is stable:
- manual partitioning edge cases
- separate `/home` disk installs
- removable media installs
- Nvidia-specific path as a primary target
- multiple desktop targets being equally polished
- GTK installer parity for every flow

## Immediate Decisions Needed

- Which desktop environment is the first-class target?
- Is `xfs` the intended default filesystem, or just a current implementation choice?
- What exact accessibility stack is guaranteed in the live and installed environments?
- What package profile combination defines the first successful supported install?
