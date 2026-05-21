# `grok-cli video-extend`

## Purpose

Extend an existing video with Grok Imagine.

This command is separate from [`video`](./video.md) and [`video-edit`](./video-edit.md). `video` generates new video, `video-edit` edits an existing video, and `video-extend` appends new footage to the end of an existing MP4 video.

Note: `video-extend` only supports a remote video URL. It does not expose local video paths because real validation showed that local MP4 files were encoded and sent to the upstream service, but the final xAI extension job returned an internal error. To extend a local video, upload it first and then pass the remote URL with `--video-url`.

## Common Usage

```bash
grok-cli video-extend --video-url https://example.com/source.mp4 --prompt "The camera pans left" --duration 6
```

Script or skill usage:

```bash
grok-cli video-extend --json --video-url https://example.com/source.mp4 --prompt "Continue the camera move"
```

## Parameters

- `PROMPT`: positional extension prompt.
- `--prompt <PROMPT>`: explicit script-friendly prompt.
- `--video-url <URL>`: source video URL to extend.
- `--duration <SECONDS>`: extension length, default `6`, clamped to `2..=10`.
- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--model <MODEL>`: override the model for this video extension request only.
- `--timeout <SECONDS>`: total video polling timeout, default `600`; individual HTTP requests still stay within the media request ceiling.

## Behavior

- Default model is `grok-imagine-video`.
- The command calls `POST /videos/extensions` and sends `video: {"url": ...}` plus the normalized `duration`.
- `--video-url` is required; local video paths are not part of the public capability surface.
- `aspect_ratio` and `resolution` are not sent; the extension inherits the input video properties.
- The command reads `request_id` from the create response and polls `GET /videos/{request_id}` until completion.
- Successful calls are written to the local usage SQLite database under video usage.

## JSON Fields

`data` contains:

- `provider`
- `credential_source`
- `model`
- `video`
- `modality`
- `duration`
- `extra.request_id`

`modality` is fixed to `extension`.

## Related Docs

- [video](./video.md)
- [video-edit](./video-edit.md)
