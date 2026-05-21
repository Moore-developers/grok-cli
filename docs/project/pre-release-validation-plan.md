# Pre-release Validation Plan

This document covers the final checks still needed before `grok-cli` is ready for public promotion. It does not run real tests yet; the goal is to confirm the plan first, then execute each item in order.

## Goals

- Confirm that the new media capabilities work in real xAI / SuperGrok OAuth conditions.
- Make the bundled `grok-cli` skill the user-facing entrypoint: core capabilities should be available directly in the main `SKILL.md`, while advanced parameters and full command coverage live in linked reference files.
- Keep the performance analysis minimal: only record rough post-install CPU, memory, and binary size.
- Keep the first public release strategy SKILL-first; macOS Apple Silicon can use a maintainer-built local tarball upload, macOS Intel / Linux remain source-first, and Windows uses a GitHub Release binary.

## Non-goals

- Do not restore the macOS / Linux GitHub Actions release-binary workflow yet.
- Do not add deep WebSocket mock / chunked-send tests for `stt-stream`.
- Do not publish to crates.io, Homebrew, winget, or Scoop yet.
- Do not commit real OAuth tokens, session DB files, media files, or transcription content.

## Phase 0: Release And Installation Loop

Goal: confirm that users can obtain the CLI and the skill through the public paths that exist today.

Tasks:

- [ ] Verify `cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked` from a clean environment on macOS / Linux / Windows.
- [ ] Verify macOS Apple Silicon local tarball packaging, naming, checksum, and upload instructions.
- [ ] Verify Windows Release binary packaging, naming, checksum, and download instructions.
- [ ] Verify `grok-cli --version`, `grok-cli --help`, and `grok-cli status --json` after installation.
- [ ] Verify that `skills/grok-cli` can be installed with `npx --yes skills add https://github.com/Moore-developers/grok-cli --skill grok-cli --global --yes`.
- [ ] Verify that the skill can follow the install-check path when the CLI is missing.


## Phase 1: Skill Capability Coverage

Goal: make sure the skill exposes the useful command surface without overloading the main body.

Tasks:

- [ ] Keep the basic capability list in the top-level skill body.
- [ ] Move advanced parameters into linked reference files.
- [ ] Keep `references/` discoverable from the skill.
- [ ] Provide test cases for Codex and Claude Code routing.

## Phase 2: Performance Snapshot

Goal: keep only coarse installation metrics.

Tasks:

- [ ] Record approximate install-time CPU, memory, and binary size.
- [ ] Keep the recording simple and repeatable.
