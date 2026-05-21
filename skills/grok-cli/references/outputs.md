# Outputs

Use these stable fields when reading JSON output.

## Success Envelope

```json
{
  "ok": true,
  "command": "image",
  "data": {}
}
```

Read `data` for useful results. Avoid returning raw JSON to the user unless asked.

## Text

`chat --json`:

- `data.output_text`
- `data.finish_reason`
- `data.tool_calls`

`search --json`:

- `data.answer`
- `data.citations`
- `data.inline_citations`

## Media

`image --json`:

- `data.image`: first URL/path/data URL.
- `data.images`: full list.

`image-edit --json`:

- `data.image`
- `data.images`

`video --json`, `video-edit --json`, `video-extend --json`:

- `data.video`: final video URL.
- `data.request_id`: upstream async request id.
- `data.modality`: generation/edit/extension where available.

`tts --json`:

- `data.file_path`: local audio path.
- `data.media_tag`: media marker.
- `data.output_format`: when available.

`stt --json`:

- `data.transcript`: transcript text.
- `data.language`, `data.duration`, `data.words`, `data.channels`: only when upstream returns them.

`stt-stream --json`:

- Streaming output is experimental; prefer batch `stt` unless streaming is specifically requested.

## Setup Results

After install or login, return the user's requested Grok result first. Then briefly mention setup that was performed.
