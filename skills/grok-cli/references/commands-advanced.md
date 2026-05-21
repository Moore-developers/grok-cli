# Advanced Command Rules

Use this reference when combining flags or diagnosing invalid arguments.

## JSON And Streaming

- Prefer `--json` for skills, scripts, and automation.
- Text commands stream formatted text by default for humans.
- Use `--no-stream` for one final human-readable response.
- Use `--raw-stream` only when the caller can parse normalized SSE-style events.
- `--auth-file <PATH>` is available on auth-backed and task commands when an alternate local OAuth state file is needed.

## Image Rules

- `image --count` must be between 1 and 10.
- `image --output-file` only works with a single image.
- Use `image --output-dir` for multiple local files.
- `--output-file` and `--output-dir` imply `response_format=b64_json`.
- Do not combine `--response-format url` with local output flags.
- `image` supports `--model`, `--aspect-ratio`, `--resolution`, and `--timeout`.

## Image Edit Rules

- `image-edit --image` accepts paths, URLs, or data URIs.
- `--image` is repeatable, up to 3 images.
- Local image paths must exist.
- `--output-file` implies `response_format=b64_json`.
- `image-edit` supports `--model`, `--aspect-ratio`, `--resolution`, `--response-format`, and `--timeout`.

## Video Rules

- `video` supports text-to-video, image-to-video, and reference-image video.
- `--image-url` cannot be combined with `--reference-image-url`.
- `--reference-image-url` is repeatable, up to 7 images.
- `video` supports local `--image` and local `--reference-image` inputs too.
- `video-edit` requires a prompt and accepts either `--video-url` or local `--video`.
- `video-edit` supports `--model` and `--timeout` only; it does not use duration, aspect ratio, or resolution.
- `video-extend` requires `--video-url` and a prompt; duration defaults to 6 seconds.
- `video-extend` supports `--model`, `--duration`, and `--timeout`.
- Do not route a local file directly to `video-extend`; ask for or help produce a remote URL first.
- Video commands poll until completion or timeout.

## TTS Rules

- `tts` requires text unless `--list-voices` is used.
- If `--output-format` is provided with `--output`, keep it consistent with the file extension.
- `--output-format`, `--sample-rate`, and `--bit-rate` describe the requested audio format.
- `tts` also supports `--voice-id`, `--language`, `--optimize-streaming-latency`, `--text-normalization`, `--model`, and `--timeout`.

## STT Rules

- `stt` requires exactly one input source: positional path, `--file`, or `--url`.
- Local files must exist.
- `--keyterm` is repeatable.
- `stt` supports `--model`, `--language`, `--format`, `--audio-format`, `--sample-rate`, `--multichannel`, `--channels`, `--diarize`, `--filler-words`, and `--timeout`.
- Use `stt-stream` only for streaming experiments; batch `stt` is the default.
- `stt-stream` supports `--model`, `--language`, `--interim-results`, `--endpointing`, `--encoding`, `--sample-rate`, `--diarize`, `--filler-words`, `--multichannel`, `--channels`, `--keyterm`, and `--timeout`.
