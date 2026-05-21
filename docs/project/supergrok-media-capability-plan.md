# SuperGrok Media Capability Completion Plan

This is the English summary of the media completion plan for `grok-cli`.

The full Chinese document is preserved at [docs/zh/project/supergrok-media-capability-plan.md](../zh/project/supergrok-media-capability-plan.md).

## Scope

- Compare Hermes Agent behavior with official xAI media docs
- Track remaining gaps for `image`, `tts`, `stt`, and related media parameters
- Keep every new capability or parameter paired with tests

## Current Outcome

- `stt`, `tts`, `image`, `video`, `video-edit`, and `video-extend` have reached the current completion target
- The remaining work is mostly validation, documentation alignment, and regression coverage

## Rule

If a user-visible media parameter changes, update the relevant command docs and add tests in the same change.
