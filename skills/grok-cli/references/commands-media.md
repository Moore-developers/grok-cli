# Media Commands

Use this reference for common image, video, TTS, and STT tasks. Prefer `--json` and explicit input flags for automation.

## Image Generation

```bash
grok-cli image --json --prompt "A cinematic skyline at sunrise"
```

Common flags:

- `--count <N>`: generate 1 to 10 images.
- `--response-format url|b64_json`: choose URL or base64 output.
- `--output-file <PATH>`: save one base64 image locally.
- `--output-dir <PATH>`: save multiple base64 images locally.
- `--aspect-ratio <RATIO>`: for example `16:9` or `1:1`.
- `--resolution <VALUE>`: for example `1k`.

## Image Editing

```bash
grok-cli image-edit --json --image ./source.png --prompt "Make it cinematic"
```

Notes:

- `--image <PATH_OR_URL>` is repeatable, up to 3 images.
- `--output-file <PATH>` saves base64 output locally.
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

Notes:

- `--reference-image-url` is repeatable, up to 7 images.
- `--image-url` cannot be combined with `--reference-image-url`.
- `--duration`, `--aspect-ratio`, `--resolution`, and `--model` are optional.

## Video Editing

```bash
grok-cli video-edit --json --video-url https://example.com/source.mp4 --prompt "Make it cinematic"
```

`video-edit` edits an existing MP4 URL and does not send duration, aspect ratio, or resolution.

## Video Extension

```bash
grok-cli video-extend --json --video-url https://example.com/source.mp4 --prompt "Continue the camera move" --duration 6
```

`--duration` defaults to 6 seconds and is clamped to the supported range.

## Text To Speech

```bash
grok-cli tts --json --text "Hello from Grok"
```

Common flags:

- `--voice-id <VOICE>`.
- `--language <LANG>`.
- `--output <PATH>`.
- `--output-format mp3|wav`.
- `--sample-rate <HZ>`.
- `--bit-rate <BPS>`.
- `--list-voices`: list available voices without synthesizing audio.

## Speech To Text

Local file:

```bash
grok-cli stt --json --file ./sample.wav
```

Remote URL:

```bash
grok-cli stt --json --url https://example.com/sample.wav --language auto
```

Common flags:

- `--language <LANG>`.
- `--format true|false`.
- `--audio-format <FORMAT>`.
- `--sample-rate <HZ>`.
- `--multichannel`.
- `--channels <LIST>`.
- `--diarize`.
- `--keyterm <TERM>` repeatable.
- `--filler-words`.

## Streaming STT

```bash
grok-cli stt-stream --json --file ./sample.wav --interim-results
```

This is experimental. Use `stt` for normal batch transcription unless the user specifically needs streaming behavior.
