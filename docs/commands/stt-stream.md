# `grok-cli stt-stream`

## Purpose

Run real-time transcription through the xAI WebSocket STT interface.

This is an experimental entrypoint and does not replace the batch transcription command [`stt`](./stt.md). The first version sends a local audio file as binary frames over WebSocket, then sends `{"type":"audio.done"}`, and keeps reading transcription events until the stream ends.

## Common Usage

```bash
grok-cli stt-stream --file ./sample.wav --language en --interim-results
```

Print JSON event summaries:

```bash
grok-cli stt-stream --file ./sample.wav --json
```

Use raw PCM parameters:

```bash
grok-cli stt-stream --file ./sample.raw --encoding pcm_s16le --sample-rate 16000 --language en
```

## Parameters

- `PATH`: positional audio file to transcribe.
- `--file <PATH>`: explicit script-friendly audio file input.
- `--json`: use the standard JSON envelope. `data.events` contains the received event list.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--model <MODEL>`: override the streaming STT model, default `grok-transcribe`.
- `--language <LANG>`: language code, default `en`.
- `--interim-results`: request interim transcription results.
- `--endpointing <VALUE>`: pass through the official endpointing parameter.
- `--encoding <ENCODING>`: raw audio encoding, such as `pcm_s16le`.
- `--sample-rate <HZ>`: raw audio sample rate.
- `--diarize`: enable speaker diarization.
- `--filler-words`: keep filler words.
- `--multichannel`: treat audio as multichannel.
- `--channels <LIST>`: choose channels such as `0,1`.
- `--keyterm <TERM>`: keyword boosting. Repeatable.
- `--timeout <SECONDS>`: reserved WebSocket session timeout parameter. It is mainly retained for CLI compatibility.

## Behavior

- The real connection URL is derived from the OAuth state `base_url`, for example `https://api.x.ai/v1` becomes `wss://api.x.ai/v1/stt`.
- Streaming STT configuration parameters go into the URL query string.
- Access token expiry is checked before the request. If needed, the command refreshes first.
- Non-JSON output prints `interim: ...` or `final: ...` events.
- JSON output returns the event array after the connection ends. Each event preserves normalized fields and the raw event.
- This is streaming STT; it does not replace the batch [`stt`](./stt.md) multipart file / URL transcription path.

## JSON Fields

`data` contains:

- `success`
- `provider`
- `credential_source`
- `events`

Each `events[]` item contains:

- `event_type`
- `transcript`
- `is_final`
- `raw`

## Related Docs

- [stt](./stt.md)
- [tts](./tts.md)
