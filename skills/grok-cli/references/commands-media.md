# Media Commands

Use this reference for common image, video, TTS, and STT tasks. Prefer `--json` and explicit input flags for automation.

## Image Generation

```bash
grok-cli image --json --prompt "A cinematic skyline at sunrise"
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional prompt or `--prompt <TEXT>`: image prompt.
- `--model <MODEL>`: override the image model for this request.
- `--count <N>`: generate 1 to 10 images.
- `--response-format url|b64_json`: choose URL or base64 output.
- `--output-file <PATH>`: save one base64 image locally.
- `--output-dir <PATH>`: save multiple base64 images locally.
- `--aspect-ratio <RATIO>`: for example `16:9` or `1:1`.
- `--resolution <VALUE>`: for example `1k`.
- `--timeout <SECONDS>`: override the image request timeout.

## Image Editing

```bash
grok-cli image-edit --json --image ./source.png --prompt "Make it cinematic"
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional prompt or `--prompt <TEXT>`: edit prompt.
- `--image <PATH_OR_URL>` is repeatable, up to 3 images.
- `--model <MODEL>`: override the image edit model.
- `--aspect-ratio <RATIO>`: for example `16:9` or `1:1`.
- `--resolution <VALUE>`: for example `1k`.
- `--response-format url|b64_json`: choose URL or base64 output.
- `--output-file <PATH>` saves base64 output locally.
- `--timeout <SECONDS>`: override the image edit request timeout.
- Use `--response-format url` only when not writing a local file.

## Video Generation

Text-to-video:

```bash
grok-cli video --json --prompt "Animate a futuristic skyline" --duration 8
```

Image-to-video:

```bash
grok-cli video --json --prompt "Make this scene move" --image-url https://example.com/source.png
```

Reference image video:

```bash
grok-cli video --json --prompt "Create a product reveal" --reference-image-url https://example.com/ref-1.png
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional prompt or `--prompt <TEXT>`: video prompt.
- `--image-url <URL>`: remote source image for image-to-video.
- `--image <PATH>`: local source image for image-to-video.
- `--reference-image-url` is repeatable, up to 7 images.
- `--reference-image <PATH>` is repeatable for local reference images.
- `--duration <SECONDS>`: requested video duration.
- `--aspect-ratio <RATIO>`: for example `16:9` or `1:1`.
- `--resolution <VALUE>`: for example `720p`.
- `--model <MODEL>`: override the video model.
- `--timeout <SECONDS>`: total polling timeout.
- `--image-url` cannot be combined with `--reference-image-url`.

## Video Editing

```bash
grok-cli video-edit --json --video-url https://example.com/source.mp4 --prompt "Make it cinematic"
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional prompt or `--prompt <TEXT>`: edit prompt.
- `--video-url <URL>`: remote source MP4.
- `--video <PATH>`: local source MP4.
- `--model <MODEL>`: override the video edit model.
- `--timeout <SECONDS>`: total polling timeout.

`video-edit` edits an existing MP4. It does not send duration, aspect ratio, or resolution.

## Video Extension

```bash
grok-cli video-extend --json --video-url https://example.com/source.mp4 --prompt "Continue the camera move" --duration 6
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional prompt or `--prompt <TEXT>`: extension prompt.
- `--video-url <URL>`: remote source MP4.
- `--duration <SECONDS>`: requested extension duration.
- `--model <MODEL>`: override the video extension model.
- `--timeout <SECONDS>`: total polling timeout.

`video-extend` only supports `--video-url`. Do not route local file paths to `video-extend --video`; real validation showed that local MP4 data URI input can reach xAI, but the extension task ends in upstream internal error. `--duration` defaults to 6 seconds and is clamped to the supported range.

## Text To Speech

```bash
grok-cli tts --json --text "Hello from Grok"
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional text or `--text <TEXT>`: text to synthesize.
- `--list-voices`: list available voices without synthesizing audio.
- `--voice-id <VOICE>`.
- `--language <LANG>`.
- `--output <PATH>`.
- `--output-format mp3|wav`.
- `--sample-rate <HZ>`.
- `--bit-rate <BPS>`.
- `--optimize-streaming-latency <MODE>`.
- `--text-normalization <MODE>`.
- `--model <MODEL>`.
- `--timeout <SECONDS>`.

## Speech To Text

Local file:

```bash
grok-cli stt --json --file ./sample.wav
```

Remote URL:

```bash
grok-cli stt --json --url https://example.com/sample.wav --language auto
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional path or `--file <PATH>`: local audio file.
- `--url <URL>`: remote audio file.
- `--model <MODEL>`.
- `--language <LANG>`.
- `--format true|false`.
- `--audio-format <FORMAT>`.
- `--sample-rate <HZ>`.
- `--multichannel`.
- `--channels <LIST>`.
- `--diarize`.
- `--keyterm <TERM>` repeatable.
- `--filler-words`.
- `--timeout <SECONDS>`.

## Streaming STT

```bash
grok-cli stt-stream --json --file ./sample.wav --interim-results
```

Public flags:

- `--json`: machine-readable response for agents and scripts.
- `--auth-file <PATH>`: use an alternate local OAuth state file.
- positional path or `--file <PATH>`: local audio file.
- `--model <MODEL>`.
- `--language <LANG>`.
- `--interim-results`: request interim transcript events.
- `--endpointing <MODE_OR_DURATION>`: endpointing value accepted by xAI.
- `--encoding <ENCODING>`: raw audio encoding, for example `pcm_s16le`.
- `--sample-rate <HZ>`.
- `--diarize`.
- `--filler-words`.
- `--multichannel`.
- `--channels <LIST>`.
- `--keyterm <TERM>` repeatable.
- `--timeout <SECONDS>`.

This is experimental. Use `stt` for normal batch transcription unless the user specifically needs streaming behavior.
