# Changelog

All notable changes to `grok-cli` will be documented in this file.

The project follows semantic versioning once public releases begin.

## Unreleased

### Changed

- Switched the first public release strategy to SKILL-first / source-first distribution.
- Added a bundled `grok-cli` skill that can check installation, install the CLI with Cargo, handle OAuth, and route Grok tasks through JSON-mode commands.

## 0.1.0 - 2026-05-21

### Added

- MIT open source project metadata and release workflow.
- OAuth-first CLI for Grok / xAI chat, X search, media generation, audio, and local usage tracking.
- SuperGrok media command coverage for image generation, image editing, video generation, video editing, video extension, TTS, batch STT, and experimental streaming STT.

### Notes

- `stt-stream` remains experimental. Deep WebSocket mock coverage and chunk-send refinements are intentionally deferred until the protocol path proves useful in real workflows.
