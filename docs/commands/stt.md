# `grok-cli stt`

## Purpose

Transcribe a local audio file or remote audio URL into text.

## Common Usage

```bash
grok-cli stt ./sample.wav
```

Specify language:

```bash
grok-cli stt ./sample.mp3 --language zh
```

Transcribe remote audio:

```bash
grok-cli stt --url https://example.com/sample.wav --language auto
```

Advanced transcription parameters:

```bash
grok-cli stt ./meeting.wav --diarize --keyterm Grok --keyterm xAI --filler-words
```

Script or skill usage:

```bash
grok-cli stt --json --file ./sample.wav
```

## Parameters

- `PATH`: positional audio file to transcribe.
- `--file <PATH>`: explicit script-friendly file path.
- `--url <URL>`: transcribe a remote audio URL; cannot be used together with `PATH` or `--file`.
- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--model <MODEL>`: command-level model override, mainly for compatibility and usage tagging.
- `--language <LANG>`: language code, default `en`.
- `--format <true|false>`: whether to request formatted transcription text, default `true`.
- `--audio-format <FORMAT>`: explicitly declare the raw audio format when container metadata is unavailable.
- `--sample-rate <HZ>`: raw audio sample rate.
- `--multichannel`: treat audio as multichannel.
- `--channels <CHANNELS>`: choose channels such as `0,1`.
- `--diarize`: enable speaker diarization.
- `--keyterm <TERM>`: repeatable keyword hint.
- `--filler-words`: keep filler words.
- `--timeout <SECONDS>`: request timeout, default `120`; increase it for large files.

## Behavior

- One of `PATH`, `--file`, or `--url` is required.
- `--url` cannot be combined with local file input.
- Local files must exist, otherwise `invalid_args` is returned.
- Multipart requests send `file` or `url` and include `format`, `language`, `audio_format`, `sample_rate`, `multichannel`, `channels`, `diarize`, `keyterm`, and `filler_words` as needed.
- Access token expiry is checked before the request. If needed, the command refreshes first.
- Responses read `text` or `transcript`.
- If the upstream returns `language`, `duration`, `words`, or `channels`, `--json` preserves those structured fields.
- Successful calls are written to the local usage SQLite database under audio usage.

## JSON Fields

`data` contains:

- `success`
- `provider`
- `credential_source`
- `transcript`
- `language`
- `duration`
- `words`
- `channels`

## Related Docs

- [tts](./tts.md)
- [usage](./usage.md)
