# `grok-cli video-edit`

## Purpose

Edit an existing video with Grok Imagine.

This command is separate from [`video`](./video.md). `video` handles text-to-video, image-to-video, and reference-image video; `video-edit` handles editing an existing MP4 video.

## Common Usage

```bash
grok-cli video-edit --video-url https://example.com/source.mp4 --prompt "Give the woman a silver necklace"
```

Use a local video file:

```bash
grok-cli video-edit --video ./source.mp4 --prompt "Give the woman a silver necklace"
```

Script or skill usage:

```bash
grok-cli video-edit --json --video-url https://example.com/source.mp4 --prompt "Make the scene more cinematic"
```

## Parameters

- `PROMPT`: positional edit prompt.
- `--prompt <PROMPT>`: explicit script-friendly prompt.
- `--video-url <URL>`: source video URL to edit.
- `--video <PATH>`: local source video path to edit.
- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--model <MODEL>`: override the model for this video edit request only.
- `--timeout <SECONDS>`: total video polling timeout, default `600`; individual HTTP requests still stay within the media request ceiling.

## Behavior

- Default model is `grok-imagine-video`.
- The command calls `POST /videos/edits` and sends `video: {"url": ...}`.
- Local videos are encoded as data URIs before being sent as `video.url`.
- The command does not send `duration`, `aspect_ratio`, or `resolution`; the edited output inherits the input video properties.
- It reads `request_id` from the create response and then polls `GET /videos/{request_id}` until completion.
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

`modality` is fixed to `edit`.

## Related Docs

- [video](./video.md)
- [image-edit](./image-edit.md)
