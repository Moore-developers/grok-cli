# `grok-cli video`

## Purpose

Generate video with Grok Imagine, including text-to-video, image-to-video, and reference-image video.

## Common Usage

Text-to-video:

```bash
grok-cli video "Animate a futuristic skyline" --duration 8
```

Image-to-video from a URL:

```bash
grok-cli video "Make the scene slowly move" --image-url "https://example.com/source.png"
```

Image-to-video from a local file:

```bash
grok-cli video "Make the scene slowly move" --image ./source.png
```

Reference-image video from a URL:

```bash
grok-cli video "Create a product reveal" --reference-image-url "https://example.com/ref-1.png"
```

Reference-image video from local files:

```bash
grok-cli video "Create a product reveal" --reference-image ./ref-1.png --reference-image ./ref-2.png
```

Script or skill usage:

```bash
grok-cli video --json --prompt "Animate a futuristic skyline" --duration 8
```

## Parameters

- `PROMPT`: positional video prompt.
- `--prompt <PROMPT>`: explicit script-friendly prompt.
- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--image-url <URL>`: source image URL for image-to-video.
- `--image <PATH>`: local source image path for image-to-video.
- `--reference-image-url <URL>`: repeatable reference image URL, up to 7.
- `--reference-image <PATH>`: repeatable local reference image path, up to 7.
- `--duration <SECONDS>`: video duration, normalized by the command.
- `--aspect-ratio <RATIO>`: aspect ratio, supports `1:1`, `16:9`, `9:16`, `4:3`, `3:4`, `3:2`, and `2:3`.
- `--resolution <VALUE>`: resolution, supports `480p` and `720p`.
- `--model <MODEL>`: override the model for this video request only.
- `--timeout <SECONDS>`: total polling timeout, default `600`; individual create / poll requests still use a 120 second ceiling.

## Behavior

- Default model is `grok-imagine-video`.
- `grok-cli model` does not manage the video default model; pass `--model` directly when needed.
- Only one input mode is allowed: `--image-url`, `--image`, `--reference-image-url`, or `--reference-image`.
- `--reference-image-url` and `--reference-image` together accept up to 7 items.
- Local images are encoded as data URIs before being sent as `image.url` or `reference_images[].url`.
- Default duration is 8 seconds. Normal videos max out at 15 seconds. Reference-image videos max out at 10 seconds.
- Default aspect ratio is `16:9` and default resolution is `720p`.
- The command first calls `POST /videos/generations`, then polls `GET /videos/{request_id}`.
- Access token expiry is checked before requests and polling. If needed, the command refreshes first.
- Successful calls are written to the local usage SQLite database under video usage.

## JSON Fields

`data` contains:

- `provider`
- `credential_source`
- `model`
- `video`
- `modality`
- `aspect_ratio`
- `duration`
- `extra.request_id`

## Related Docs

- [image](./image.md)
- [usage](./usage.md)
