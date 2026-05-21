# Extension Points

This file captures the stable boundaries future `grok-cli` features should reuse instead of reinventing.

The full Chinese reference is preserved at [docs/zh/reference/extension-points.md](../zh/reference/extension-points.md).

## Stable Boundaries

- Flat top-level commands
- Unified JSON success and error envelopes
- Shared runtime credential resolution
- Shared upstream execution layers

## Recommended Rule For New Features

1. Decide the command shape first
2. Reuse the shared auth and request layers where possible
3. Keep the JSON output contract stable
4. Add module tests, command tests, and docs together

## Do Not Change Lightly

- Existing JSON envelope shape
- Existing public error code strings
- The main `chat`, `search`, `image`, `video`, `tts`, and `stt` output fields
