# Advanced Command Rules

Use this reference when combining flags or diagnosing invalid arguments.

## JSON And Streaming

- Prefer `--json` for skills, scripts, and automation.
- Text commands stream formatted text by default for humans.
- Use `--no-stream` for one final human-readable response.
- Use `--raw-stream` only when the caller can parse normalized SSE-style events.

## Image Rules

- `image --count` must be between 1 and 10.
- `image --output-file` only works with a single image.
- Use `image --output-dir` for multiple local files.
- `--output-file` and `--output-dir` imply `response_format=b64_json`.
- Do not combine `--response-format url` with local output flags.

## Image Edit Rules

- `image-edit --image` accepts paths, URLs, or data URIs.
- `--image` is repeatable, up to 3 images.
- Local image paths must exist.
- `--output-file` implies `response_format=b64_json`.

## Video Rules

- `video` supports text-to-video, image-to-video, and reference-image video.
- `--image-url` cannot be combined with `--reference-image-url`.
- `--reference-image-url` is repeatable, up to 7 images.
- `video-edit` requires `--video-url` and a prompt.
- `video-extend` requires `--video-url` and a prompt; duration defaults to 6 seconds.
- Video commands poll until completion or timeout.

## TTS Rules

- `tts` requires text unless `--list-voices` is used.
- If `--output-format` is provided with `--output`, keep it consistent with the file extension.
- `--output-format`, `--sample-rate`, and `--bit-rate` describe the requested audio format.

## STT Rules

- `stt` requires exactly one input source: positional path, `--file`, or `--url`.
- Local files must exist.
- `--keyterm` is repeatable.
- Use `stt-stream` only for streaming experiments; batch `stt` is the default.
