# Skill Validation Cases

This document checks whether `skills/grok-cli` can cover every public `grok-cli` capability from both Codex and Claude Code. The focus is not answer quality from the upstream service; the focus is:

- whether the skill chooses the right command
- whether the parameters are shaped correctly
- whether local files are brought in correctly
- whether unsupported capabilities are blocked clearly

## How To Use

In Codex or Claude Code:

1. Install or load `skills/grok-cli`
2. Enter one of the natural-language prompts below
3. Check whether the agent routes to the expected `grok-cli` command
4. For local-file cases, either provide a path directly in the prompt or explicitly say to process the file

## Full Coverage Matrix

| ID | Capability | Prompt a user could actually say | Expected route |
| --- | --- | --- | --- |
| A1 | `login` | Help me log in to Grok | `grok-cli login` |
| A2 | `status` | Check my current Grok login status | `grok-cli status --json` |
| A3 | `refresh` | Refresh my Grok login state | `grok-cli refresh --json` |
| A4 | `logout` | Log out my local Grok session | `grok-cli logout --json` |
| A5 | `state` | Show me a summary of the local auth state | `grok-cli state --json` |
| A6 | `model` | Show me the current default text model | `grok-cli model --json` |
| A7 | `usage` | Show me the local usage stats | `grok-cli usage --json` |
| A8 | `chat` | Use Grok to summarize the recent discussion about Rust CLI design and return structured results | `grok-cli chat --json --prompt "..."` |
| A9 | `search` | Search X and summarize what people are saying today | `grok-cli search --json --query "..."` |
| A10 | `image` | Generate a 1:1 minimal terminal icon and save it locally | `grok-cli image --json --prompt "..." --aspect-ratio 1:1 --output-file ...` |
| A11 | `image` multi | Generate three different terminal mascots and save them to a directory | `grok-cli image --json --prompt "..." --count 3 --output-dir ...` |
| A12 | `image-edit` local | Edit this image to look more like a CLI cover: `./source.png` | `grok-cli image-edit --json --image ./source.png --prompt "..."` |
| A13 | `image-edit` multi | Blend these two images into one validation badge: `./a.png ./b.png` | `grok-cli image-edit --json --image ./a.png --image ./b.png --prompt "..."` |
| A14 | `image-edit` remote | Edit this remote image and save it locally: `https://...` | `grok-cli image-edit --json --image https://... --output-file ... --prompt "..."` |
| A15 | `video` text-to-video | Generate an 8-second terminal-style short video | `grok-cli video --json --prompt "..." --duration 8` |
| A16 | `video` local image to video | Use this local image to make a short video: `./source.png` | `grok-cli video --json --prompt "..." --image ./source.png` |
| A17 | `video` remote image to video | Use this remote image to make a short video: `https://...` | `grok-cli video --json --prompt "..." --image-url https://...` |
| A18 | `video` local reference images | Use these two local reference images to make a product reveal video: `./a.png ./b.png` | `grok-cli video --json --prompt "..." --reference-image ./a.png --reference-image ./b.png` |
| A19 | `video` remote reference images | Use these two remote images to generate a short video: `https://... https://...` | `grok-cli video --json --prompt "..." --reference-image-url https://... --reference-image-url https://...` |
| A20 | `video-edit` local video | Edit this local video to look more cinematic: `./source.mp4` | `grok-cli video-edit --json --video ./source.mp4 --prompt "..."` |
| A21 | `video-edit` remote video | Edit this remote video and make the colors cooler: `https://...` | `grok-cli video-edit --json --video-url https://... --prompt "..."` |
| A22 | `video-extend` remote video | Extend this video by two seconds: `https://example.com/source.mp4` | `grok-cli video-extend --json --video-url https://example.com/source.mp4 --duration 2 --prompt "..."` |
| A23 | `tts` list | List the available TTS voices | `grok-cli tts --json --list-voices` |
| A24 | `tts` synth | Turn this text into an ara-voice MP3 and save it | `grok-cli tts --json --text "..." --voice-id ara --output ... --output-format mp3` |
| A25 | `stt` local audio | Transcribe this audio and keep the keywords Grok and CLI: `./sample.wav` | `grok-cli stt --json --file ./sample.wav --keyterm Grok --keyterm CLI` |
| A26 | `stt` remote audio | Transcribe this remote audio: `https://example.com/sample.wav` | `grok-cli stt --json --url https://example.com/sample.wav` |
| A27 | `stt-stream` | Transcribe this audio in real time: `./sample.wav` | `grok-cli stt-stream --json --file ./sample.wav ...` |

## Local File Scenarios

These are the local-file forms the skill must handle correctly. Users do not need to know the CLI flag names; they only need to tell the skill what file to process.

| Input type | Example user phrasing | Expected command |
| --- | --- | --- |
| Local single image edit | Process this image: `./source.png` | `image-edit --image ./source.png` |
| Local multi-image edit | Process these two images: `./a.png ./b.png` | `image-edit --image ./a.png --image ./b.png` |
| Local image-to-video | Turn this image into a video: `./source.png` | `video --image ./source.png` |
| Local reference-image video | Use these two images as references: `./a.png ./b.png` | `video --reference-image ./a.png --reference-image ./b.png` |
| Local video edit | Process this video: `./source.mp4` | `video-edit --video ./source.mp4` |
| Local audio transcription | Transcribe this audio: `./sample.wav` | `stt --file ./sample.wav` |
| Local streaming transcription | Transcribe this audio in real time: `./sample.wav` | `stt-stream --file ./sample.wav` |

## Negative Cases

### N1. Do not invent a local video extension command

User prompt:

```text
Extend this local video by two seconds: ./source.mp4
```

Expected behavior:

- Do not generate `grok-cli video-extend --video ./source.mp4`
- Explain clearly that `video-extend` currently only supports `--video-url`
- Ask the user to upload the local video to a public URL first, then continue with extension

### N2. Do not route image editing to image generation

User prompt:

```text
Make a small change to this image: ./source.png
```

Expected behavior:

- Choose `image-edit`
- Do not choose `image`

### N3. Do not route video editing to video generation

User prompt:

```text
Adjust the color of this existing video: ./source.mp4
```

Expected behavior:

- Choose `video-edit`
- Do not choose `video`

### N4. Do not route normal transcription to streaming transcription

User prompt:

```text
Transcribe this audio file: ./sample.wav
```

Expected behavior:

- Default to `stt`
- Only choose `stt-stream` when the user explicitly asks for real time, streaming, or interim results

## Parameter Cross-Check Matrix

This matrix verifies that the main `SKILL.md` keeps only common parameters and the `references/` files retain the full parameter detail. Unless noted otherwise, the prompts below are intended as `--json` calls. If the user explicitly gives a local auth file, the skill should add `--auth-file`.

| ID | Command | Focus parameters | Example user phrasing | Expected route |
| --- | --- | --- | --- | --- |
| P1 | `login` | `--no-browser` `--manual-paste` `--timeout` `--port` | This machine cannot open a browser automatically, help me log in to Grok with manual paste, 300 second timeout, port 8787 | `grok-cli login --no-browser --manual-paste --timeout 300 --port 8787` |
| P2 | `status` / `state` | `--auth-file` | Check the login state using this auth file: `./tmp/auth.json` | `grok-cli status --json --auth-file ./tmp/auth.json` |
| P3 | `model` | `--model` | Switch the default text model to `grok-4.3` | `grok-cli model --json --model grok-4.3` |
| P4 | `usage` | `--session-db` `--session-id` | Look up the usage for session `abc123` from this session database: `./session.db` | `grok-cli usage --json --session-db ./session.db --session-id abc123` |
| P5 | `chat` | `--system` `--model` `--no-web-search` `--with-x-search` `--allowed-domain` `--excluded-domain` `--allowed-x-handle` `--excluded-x-handle` `--from-date` `--to-date` `--enable-image-understanding` `--enable-video-understanding` `--timeout` | Use Grok to look only at X and the allowed domains, restrict to `example.com`, exclude `blocked.example.com`, include only `@xAI` and `@grok`, use the date range 2026-05-01 to 2026-05-21, enable image and video understanding, and return structured results | `grok-cli chat --json --system "..." --model ... --no-web-search --with-x-search --allowed-domain example.com --excluded-domain blocked.example.com --allowed-x-handle xAI --allowed-x-handle grok --from-date 2026-05-01 --to-date 2026-05-21 --enable-image-understanding --enable-video-understanding --timeout ... --prompt "..."` |
| P6 | `chat` | `--stream` `--no-stream` `--raw-stream` | I want to watch the generation live, but I do not want a second final summary | `grok-cli chat --stream ...` or `grok-cli chat --no-stream ...` or `grok-cli chat --raw-stream ...` |
| P7 | `search` | `--query` `--model` `--allowed-x-handle` `--excluded-x-handle` `--from-date` `--to-date` `--enable-image-understanding` `--enable-video-understanding` `--timeout` | Search X for discussion about `@xAI` and `@grok`, look only at the last week, and enable video understanding | `grok-cli search --json --query "..." --allowed-x-handle xAI --allowed-x-handle grok --from-date ... --to-date ... --enable-video-understanding --timeout ...` |
| P8 | `image` | `--count` `--response-format` `--output-file` `--output-dir` `--aspect-ratio` `--resolution` `--model` `--timeout` | Generate three 1:1 terminal-style images, save them to a directory, and also keep the base64 output locally | `grok-cli image --json --prompt "..." --count 3 --output-dir ./out --aspect-ratio 1:1 --resolution 1k --response-format b64_json --model ... --timeout ...` |
| P9 | `image-edit` | repeat `--image` `--response-format` `--output-file` `--aspect-ratio` `--resolution` `--model` `--timeout` | Blend these two images into a more CLI-like cover: `./a.png ./b.png` | `grok-cli image-edit --json --image ./a.png --image ./b.png --prompt "..." --response-format b64_json --output-file ./out.png --aspect-ratio 16:9 --resolution 1k --model ... --timeout ...` |
| P10 | `video` | `--image-url` `--image` `--reference-image-url` `--reference-image` `--duration` `--aspect-ratio` `--resolution` `--model` `--timeout` | Turn this local image into an 8-second video, then try a second version using two reference images | `grok-cli video --json --prompt "..." --image ./source.png --duration 8 --aspect-ratio 16:9 --resolution 720p --model ... --timeout ...` |
| P11 | `video-edit` | `--video-url` `--video` `--model` `--timeout` | Edit this local video and make it more cinematic: `./source.mp4` | `grok-cli video-edit --json --video ./source.mp4 --prompt "..." --model ... --timeout ...` |
| P12 | `video-extend` | `--video-url` `--duration` `--model` `--timeout` | Extend this remote video by two seconds: `https://example.com/source.mp4` | `grok-cli video-extend --json --video-url https://example.com/source.mp4 --duration 2 --prompt "..." --model ... --timeout ...` |
| P13 | `tts` | `--list-voices` `--text` `--voice-id` `--language` `--output` `--output-format` `--sample-rate` `--bit-rate` `--optimize-streaming-latency` `--text-normalization` `--model` `--timeout` | First list the available voices, then turn this text into an ara-voice MP3 and save it | `grok-cli tts --json --list-voices` and `grok-cli tts --json --text "..." --voice-id ara --language en --output ./out.mp3 --output-format mp3 --sample-rate ... --bit-rate ... --optimize-streaming-latency ... --text-normalization ... --model ... --timeout ...` |
| P14 | `stt` | `--file` `--url` `--model` `--language` `--format` `--audio-format` `--sample-rate` `--multichannel` `--channels` `--diarize` `--keyterm` `--filler-words` `--timeout` | Transcribe this local audio and keep the keywords Grok and CLI: `./sample.wav` | `grok-cli stt --json --file ./sample.wav --keyterm Grok --keyterm CLI --language auto --format true --audio-format wav --sample-rate 16000 --multichannel --channels 0,1 --diarize --filler-words --model ... --timeout ...` |
| P15 | `stt-stream` | `--file` `--model` `--language` `--interim-results` `--endpointing` `--encoding` `--sample-rate` `--diarize` `--filler-words` `--multichannel` `--channels` `--keyterm` `--timeout` | Transcribe this audio in real time and turn on interim results: `./sample.wav` | `grok-cli stt-stream --json --file ./sample.wav --interim-results --endpointing ... --encoding pcm_s16le --sample-rate 16000 --diarize --filler-words --multichannel --channels 0,1 --keyterm Grok --model ... --language auto --timeout ...` |

## Acceptance Criteria

- Every public capability has at least one executable skill test case.
- Every capability that accepts local files can be triggered correctly just by giving the path.
- `video-extend`'s local path restriction is clearly explained by the skill.
- The parameter cross-check matrix covers the public parameters for `login`, `chat`, `search`, `image`, `image-edit`, `video`, `video-edit`, `video-extend`, `tts`, `stt`, and `stt-stream`.
- Codex and Claude Code can complete validation using only `SKILL.md` and this test case document.
