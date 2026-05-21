# `grok-cli image`

## Purpose

Generate images with Grok Imagine.

## Common Usage

```bash
grok-cli image "A cinematic skyline at sunrise"
```

Set aspect ratio and resolution:

```bash
grok-cli image "A cinematic skyline" --aspect-ratio 16:9 --resolution 1k
```

Save a base64 image to a local file:

```bash
grok-cli image "A logo mark" --output-file ./out/logo.png
```

Generate multiple images:

```bash
grok-cli image "A cinematic skyline" --count 4 --response-format url --json
```

Save multiple base64 images to a directory:

```bash
grok-cli image "A logo mark" --count 4 --output-dir ./out/logos
```

Script or skill usage:

```bash
grok-cli image --json --prompt "A cinematic skyline"
```

## Parameters

- `PROMPT`: positional image prompt.
- `--prompt <PROMPT>`: explicit script-friendly prompt.
- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--model <MODEL>`: override the model for this image request only.
- `--aspect-ratio <RATIO>`: output ratio, such as `16:9` or `1:1`.
- `--resolution <VALUE>`: output resolution, such as `1k`.
- `--count <N>`: number of images to generate, range `1..=10`, default `1`; maps to xAI field `n`.
- `--response-format <url|b64_json>`: explicitly choose URL or base64 output.
- `--output-file <PATH>`: require base64 output and save one local file.
- `--output-dir <PATH>`: require base64 output and save multiple files as `image-001.png`, `image-002.png`, and so on.
- `--timeout <SECONDS>`: request timeout, default `120`.

## Behavior

- Default model is `grok-imagine-image`.
- `grok-cli model` does not manage the image default model; pass `--model` directly when needed.
- Access token expiry is checked before the request. If needed, the command refreshes first.
- Default request uses `n=1` and does not explicitly send `response_format`.
- `--count` outside `1..=10` returns `invalid_args`.
- If `--output-file` is omitted, the command returns an image URL or data URL.
- With `--output-file`, the decoded file is written locally and the output reports the local path.
- `--output-file` only supports a single image. Use `--output-dir` for multiple images.
- `--output-file` and `--output-dir` implicitly use `response_format=b64_json`.
- If `--response-format url` is passed explicitly, it cannot be combined with `--output-file` or `--output-dir`.
- Successful calls are written to the local usage SQLite database under image usage.

## JSON Fields

`data` contains:

- `provider`
- `credential_source`
- `model`
- `image`
- `images`
- `aspect_ratio`
- `extra`

Compatibility note:

- `image` always remains present and points to the first image or the first local file path.
- `images` returns the full list; single-image calls still return a one-item array.

## Related Docs

- [video](./video.md)
- [usage](./usage.md)
