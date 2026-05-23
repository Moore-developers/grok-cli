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

Read `data` for useful results. Default to lossless human-readable rendering: extract the command-specific field and display its value exactly as returned. The host assistant must not rewrite field values. Return raw JSON only when the user explicitly asks for raw JSON.

## Text

`chat --json`:

- `data.output_text`: return exactly as written
- `data.finish_reason`
- `data.tool_calls`

`search --json`:

- `data.answer`: return exactly as written
- `data.citations`: preserve exactly when exposing citations
- `data.inline_citations`: preserve exactly when exposing citations

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

- `data.transcript`: transcript text; return exactly as written.
- `data.language`, `data.duration`, `data.words`, `data.channels`: only when upstream returns them.

`stt-stream --json`:

- Streaming output is experimental; prefer batch `stt` unless streaming is specifically requested.

## Recovery Results

After install, login, refresh, or another recovery action, return the user's requested Grok result first. Then briefly mention the recovery that was performed.

## Output Mode Rules

- Default mode: render the primary field values exactly as returned, without the JSON envelope.
- Raw mode: if the user asks for raw CLI output or raw JSON, return the complete stdout/stderr payload unchanged except for the minimal fencing needed to display it safely.
- Transformation mode: if the user explicitly asks the host assistant to summarize, translate, rewrite, extract, format, compare, classify, or restructure Grok output, make it clear that the result is transformed output rather than verbatim Grok output.
- Do not summarize, paraphrase, translate, reorder, trim, markdown-polish, or reformat Grok text unless the user explicitly asks for that transformation.
- Preserve whitespace, line breaks, code fences, citations, numbering, and wording as much as the chat surface allows.
- For media commands, return exact paths, URLs, request ids, media tags, or handles from the JSON fields without renaming or editorializing them.
