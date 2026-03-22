# Accessibility Checklist

This checklist is focused on the installer experience, especially the CLI path.

## Core Principles

The installer should be:
- usable without sight
- understandable without relying on layout or color
- fully keyboard-driven
- predictable in wording and control flow
- explicit about state, progress, and destructive actions

## CLI Interaction

- [ ] Interface remains line-oriented, not curses/TUI based
- [ ] Global commands are consistent across steps: `next`, `back`, `help`, `quit`
- [ ] Each step clearly names itself
- [ ] Each step clearly states what the user is expected to do
- [ ] Errors are short, specific, and actionable
- [ ] Important information is not buried in large text dumps
- [ ] Prompt wording is stable enough that screen reader users can build muscle memory

## Navigation and Comprehension

- [ ] The current step is obvious
- [ ] The result of each choice is understandable
- [ ] The user can tell when they are blocked and why
- [ ] `back` behavior is predictable
- [ ] `help` gives useful context without overwhelming the screen reader

## Disk Safety and Clarity

- [ ] Selected disk is always clearly identified by device path and size
- [ ] Automatic vs manual setup is clearly distinguished
- [ ] Any deletion is explicitly described as destructive
- [ ] Review step restates destructive changes in plain language
- [ ] Confirmation language cannot be mistaken as informational only

## Review and Progress

- [ ] The review screen summarizes all major choices
- [ ] The install phase reports meaningful progress steps
- [ ] Failures identify the phase that failed
- [ ] Success state is unmistakable
- [ ] Next action after completion is clearly stated

## Accessibility-Specific Validation

- [ ] The live environment can launch the installer without inaccessible workarounds
- [ ] Screen reader output is not flooded by unnecessary text repetition
- [ ] Password and sensitive prompts are handled in a way that remains understandable
- [ ] Audio/speech assumptions are documented, not implicit

## GTK Parity Notes

The GTK path still matters, but it should not become the accessibility reference path until it proves parity on:
- keyboard navigation
- AT-SPI labeling quality
- readable step transitions
- clear announcements for validation and progress
