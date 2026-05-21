# `grok-cli image-edit`

## Purpose

Edit one or more reference images with Grok Imagine.

This command is separate from [`image`](./image.md) so that pure generation and reference-image editing do not share one entrypoint.

## Common Usage

Edit one image:

```bash
grok-cli image-edit --image ./source.png --prompt "Make it cinematic"
```

Use a remote image URL:

```bash
grok-cli image-edit --image https://example.com/source.png --prompt "Change the background to sunset"
```

Edit multiple images, up to 3:

```bash
grok-cli image-edit \
  --image ./a.png \
  --image ./b.png \
  --image ./c.png \
  --prompt "Blend these references into one editorial image"
```

Save a base64 edit result:

```bash
grok-cli image-edit --image ./source.png --prompt "Make it cinematic" --output-file ./out/edited.png
```

## Parameters

- `PROMPT`: positional edit prompt.
- `--prompt <PROMPT>`: explicit script-friendly prompt.
- `--image <PATH_OR_URL>`: reference image input, repeatable up to 3 times; supports local paths, `http(s)` URLs, or data URIs.
- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--model <MODEL>`: override the model for this image edit request only.
- `--aspect-ratio <RATIO>`: output ratio, such as `16:9` or `1:1`.
- `--resolution <VALUE>`: output resolution, such as `1k`.
- `--response-format <url|b64_json>`: explicitly choose URL or base64 output.
- `--output-file <PATH>`: require base64 output and save one local file.
- `--timeout <SECONDS>`: request timeout, default `120`.

## Behavior

- Default model is `grok-imagine-image`.
- One image sends official field `image`; multiple images send official field `images`.
- Local images are encoded as `data:image/<ext>;base64,...` before being sent as image URLs.
- More than 3 images returns `invalid_args`.
- `--output-file` implicitly uses `response_format=b64_json`.
- If `--response-format url` is passed explicitly, it cannot be combined with `--output-file`.
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

- `image` is the first edit result or the local file path.
- `images` returns the full edit result list; single-image calls still return a one-item array.

## Related Docs

- [image](./image.md)
- [video](./video.md)
