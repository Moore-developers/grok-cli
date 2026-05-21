# Parameter Validation Plan

This document tracks the additional real media parameter checks that still need to be covered. The execution order follows risk and user value: first verify the newer local path support, then fill in high-frequency output, format, and advanced audio parameters.

## Goals

- Real-validate whether uncovered key parameters are accepted upstream.
- Confirm that local path inputs are converted and sent correctly by the CLI.
- Record the input asset, prompt, parameters, generated result, and follow-up action for every real test.
- Keep raw outputs under `.tmp/` and keep reviewable local samples under `docs/project/tests/{timestamp}/`.

## Test Batch

- Timestamp: `2026-05-21T12-26-07+0800`
- Archive directory: `docs/project/tests/2026-05-21T12-26-07+0800/`
- Temp output directory: `.tmp/parameter-validation/2026-05-21T12-26-07+0800/`
- Binary under test: `target/debug/grok-cli`
- OAuth state: reuse the local `~/.grok-cli/auth.json`

## P0 Local Path Video Inputs

Status: partially passed. `video --image`, `video --reference-image`, and `video-edit --video` all passed in real validation; both local MP4 samples for `video-extend --video` reached the upstream service, but generation returned an internal error. Based on that result, local path support for `video-extend` was removed from the CLI surface and only `--video-url` remains.

| ID | Capability | New parameters covered in this round | Input asset | Sample prompt | Acceptance |
| --- | --- | --- | --- | --- | --- |
| P0-1 | `video` image-to-video | `--image`, `--aspect-ratio`, `--resolution`, `--timeout` | `images/image-001.png` | Make the local validation mascot wave gently in a vertical frame | Return a remote video URL, `modality=image` |
| P0-2 | `video` reference-image video | multi-value `--reference-image` | `images/image-001.png`, `images/image-002.png` | Use these two local icons to create a short validation badge reveal animation | Return a remote video URL |
| P0-3 | `video-edit` | `--video`, `--timeout` | `source-video-001.mp4` | Add a soft blue validation glow to the terminal window in this local video | Return a remote video URL, `modality=edit` |
| P0-4 | `video-extend` | originally planned `--video`, `--duration`, `--timeout` | `source-video-001.mp4` | Continue the same motion naturally for two more seconds | Removed after failure; only `--video-url` remains |
