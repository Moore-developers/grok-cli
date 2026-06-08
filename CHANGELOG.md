# Changelog

All notable changes to `grok-cli` will be documented in this file.

The project follows semantic versioning once public releases begin.

## Unreleased

No changes yet.

## 0.1.4 - 2026-06-08

### Changed

- Updated the shared Grok text model catalog to `grok-4.3`, `grok-4.20-0309-reasoning`, `grok-4.20-0309-non-reasoning`, and `grok-4.20-multi-agent-0309`.
- Changed the default `chat` and `search` text model to `grok-4.3`.

## 0.1.1 - 2026-05-21

### Changed

- Switched the first public release strategy to SKILL-first / source-first distribution.
- Added a bundled `grok-cli` skill that can check installation, install the CLI with Cargo, handle OAuth, and route Grok tasks through JSON-mode commands.
- Added GitHub Release packaging guidance for maintainer-built macOS Apple Silicon assets and GitHub Actions-built Windows x64 assets.
- Added English default documentation with a Chinese mirror under `docs/zh/`.

## 0.1.0 - 2026-05-21

### Added

- MIT open source project metadata and release workflow.
- OAuth-first CLI for Grok / xAI chat, X search, media generation, audio, and local usage tracking.
- SuperGrok media command coverage for image generation, image editing, video generation, video editing, video extension, TTS, batch STT, and experimental streaming STT.

### Notes

- `stt-stream` remains experimental. Deep WebSocket mock coverage and chunk-send refinements are intentionally deferred until the protocol path proves useful in real workflows.
