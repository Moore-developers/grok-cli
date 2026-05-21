# `grok-cli tts`

## Purpose

Convert text to speech and save the audio locally.

## Common Usage

```bash
grok-cli tts "Hello from Grok"
```

Choose voice, language, and output file:

```bash
grok-cli tts "Hello, I am Grok" --voice-id eve --language zh --output ./out/grok.mp3
```

Script or skill usage:

```bash
grok-cli tts --json --text "Hello from Grok"
```

List available voices:

```bash
grok-cli tts --list-voices --json
```

Control output format explicitly:

```bash
grok-cli tts "Hello" --output-format mp3 --sample-rate 24000 --bit-rate 128000
```

## Parameters

- `TEXT`: positional input text.
- `--text <TEXT>`: explicit script-friendly text.
- `--list-voices`: list available TTS voices without synthesizing audio.
- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--voice-id <VOICE>`: voice id, default `eve`.
- `--language <LANG>`: language code, default `en`; `auto` is allowed.
- `--output <PATH>`: output audio path.
- `--output-format <FORMAT>`: output format, such as `mp3` or `wav`.
- `--sample-rate <HZ>`: output sample rate.
- `--bit-rate <BPS>`: output bit rate.
- `--optimize-streaming-latency <MODE>`: pass through xAI TTS streaming latency optimization mode.
- `--text-normalization <MODE>`: pass through xAI TTS text normalization mode.
- `--model <MODEL>`: command-level model override, mainly for compatibility and usage tagging.
- `--timeout <SECONDS>`: request timeout, default `120`.

## Behavior

- Default output path lives under `~/.hermes/cache/audio/audio_cache/`.
- If the output file extension is `.wav`, the request sends `output_format=wav`.
- If `--output-format` is passed explicitly, it must match the output file extension.
- The xAI TTS request body sends `text`, `voice_id`, `language`, and optionally `output_format`, `optimize_streaming_latency`, and `text_normalization`.
- `--list-voices` calls `GET /v1/tts/voices` and returns a voice list without requiring text.
- Access token expiry is checked before the request. If needed, the command refreshes first.
- Successful calls are written to the local usage SQLite database under audio usage.

## JSON Fields

`data` contains:

- `success`
- `provider`
- `credential_source`
- `file_path`
- `media_tag`
- `voice_compatible`
- `output_format`

When `--list-voices --json` is used, `data` contains:

- `success`
- `provider`
- `credential_source`
- `voices`

## Related Docs

- [stt](./stt.md)
- [usage](./usage.md)
