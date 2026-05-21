# grok-cli Documentation

Default documentation is in English. The preserved Chinese version is available at [docs/zh](./zh/index.md).

`grok-cli` is an OAuth-first Grok / xAI CLI for browser login, local state management, Grok capability calls, and local usage tracking.

## Recommended Reading Path

1. New users should start with the [Guides Index](./guides/index.md) or go directly to the [Quickstart](./guides/quickstart.md).
2. Daily command lookup belongs in the [CLI Command Index](./commands/index.md).
3. Automation and SKILL integration details live in the [Reference Index](./reference/index.md), [SKILL integration contract](./reference/skill-integration.md), bundled [`grok-cli` skill](../skills/grok-cli/SKILL.md), and [skills README](../skills/README.md). Install the skill with `npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes`.
4. Release and installation ownership is documented in the [Release and Installation Guide](./guides/release.md). The current strategy is SKILL-first; macOS Apple Silicon can use a maintainer-uploaded tarball, macOS Intel and Linux are source-first, and Windows uses a GitHub Release binary.
5. Development planning and validation live in the [Project Index](./project/index.md). Media capability work is tracked in the [SuperGrok media capability plan](./project/supergrok-media-capability-plan.md).
6. Historical design material lives in the [Archive](./archive/index.md).

## CLI Command Specs

### Authentication

- [`login`](./commands/login.md): open a real browser and complete xAI OAuth login.
- [`status`](./commands/status.md): read local OAuth state and report whether it is usable.
- [`refresh`](./commands/refresh.md): refresh the saved access token with the refresh token.
- [`logout`](./commands/logout.md): remove local OAuth state.

### Local State And Models

- [`state`](./commands/state.md): inspect a redacted local OAuth state summary.
- [`model`](./commands/model.md): manage the shared default text model used by `chat` and `search`.

### Text

- [`chat`](./commands/chat.md): run Grok chat through the Responses API with web search enabled by default.
- [`search`](./commands/search.md): search X through Grok `x_search`.

### Media

- [`image`](./commands/image.md): generate images.
- [`image-edit`](./commands/image-edit.md): edit one or more reference images.
- [`video`](./commands/video.md): generate text-to-video, image-to-video, or reference-image video.
- [`video-edit`](./commands/video-edit.md): edit an existing video.
- [`video-extend`](./commands/video-extend.md): extend an existing remote video URL.

### Audio

- [`tts`](./commands/tts.md): convert text to speech.
- [`stt`](./commands/stt.md): transcribe local or remote audio.
- [`stt-stream`](./commands/stt-stream.md): experimental streaming speech-to-text over WebSocket.

### Usage

- [`usage`](./commands/usage.md): inspect local session usage and recent rate-limit snapshots.

## Reference Docs

- [Sample state and outputs](./reference/samples.md)
- [Internal auth recovery entrypoints](./reference/internal-auth.md)
- [`usage` command spec](./reference/usage-command-spec.md)
- [SKILL integration contract](./reference/skill-integration.md)
- [Extension points](./reference/extension-points.md)

## Project Docs

- [Acceptance examples](./project/acceptance.md)
- [Plan task](./plan-task.md)
- [SuperGrok media capability plan](./project/supergrok-media-capability-plan.md)
- [Pre-release validation plan](./project/pre-release-validation-plan.md)
- [Pre-release validation results](./project/pre-release-validation-results.md)
- [Parameter validation plan](./project/parameter-validation-plan.md)
- [Skill validation cases](./project/skill-validation-cases.md)

## Historical Archive

Archived design documents are kept for traceability. They may mention early commands such as `grok-cli auth ...`, `grok-cli task ...`, `proxy`, or `debug`; when archive docs conflict with the README or command specs, use the current README and command docs as authoritative.
