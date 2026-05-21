# CLI Command Index

This directory contains specs and usage notes for every public `grok-cli` command. Use this index for day-to-day command lookup. Deeper output examples and internal design notes live in [`../reference/`](../reference/).

## Top-Level Commands

```text
grok-cli <login|status|refresh|logout|state|model|usage|chat|search|image|image-edit|video|video-edit|video-extend|tts|stt|stt-stream> ...
```

## Authentication

| Command | Doc | Purpose |
| --- | --- | --- |
| `login` | [`login.md`](./login.md) | Open a real browser, complete xAI OAuth login, and save tokens. |
| `status` | [`status.md`](./status.md) | Read local OAuth state and report whether login is usable. |
| `refresh` | [`refresh.md`](./refresh.md) | Refresh the saved access token with the refresh token. |
| `logout` | [`logout.md`](./logout.md) | Delete local OAuth state. |

## State And Models

| Command | Doc | Purpose |
| --- | --- | --- |
| `state` | [`state.md`](./state.md) | Inspect a redacted local OAuth state summary. |
| `model` | [`model.md`](./model.md) | Manage the shared default text model for `chat` and `search`. |

## Text

| Command | Doc | Purpose |
| --- | --- | --- |
| `chat` | [`chat.md`](./chat.md) | Run Grok text chat with web search enabled by default. |
| `search` | [`search.md`](./search.md) | Search X through Grok `x_search`. |

## Media

| Command | Doc | Purpose |
| --- | --- | --- |
| `image` | [`image.md`](./image.md) | Generate images with Grok Imagine. |
| `image-edit` | [`image-edit.md`](./image-edit.md) | Edit one or more reference images. |
| `video` | [`video.md`](./video.md) | Generate text-to-video, image-to-video, or reference-image video. |
| `video-edit` | [`video-edit.md`](./video-edit.md) | Edit an existing video from a URL or local file. |
| `video-extend` | [`video-extend.md`](./video-extend.md) | Extend an existing remote video URL. |

## Audio

| Command | Doc | Purpose |
| --- | --- | --- |
| `tts` | [`tts.md`](./tts.md) | Convert text to speech and save audio locally. |
| `stt` | [`stt.md`](./stt.md) | Transcribe local audio files or remote audio URLs. |
| `stt-stream` | [`stt-stream.md`](./stt-stream.md) | Experimental WebSocket speech-to-text. |

## Usage

| Command | Doc | Purpose |
| --- | --- | --- |
| `usage` | [`usage.md`](./usage.md) | Inspect local session usage and recent rate-limit snapshots. |

## Notes

- Public commands are intentionally flat. There is no public `auth`, `task`, `proxy`, or `debug` command group.
- `chat` and `search` stream readable text by default for humans.
- Use `--json` for skills, scripts, and automation.
- Use `--raw-stream` only when the caller can consume normalized stream events.
- Internal auth recovery entrypoints are not listed here; see [`../reference/internal-auth.md`](../reference/internal-auth.md).
