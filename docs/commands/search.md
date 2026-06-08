# `grok-cli search`

## Purpose

Search X through the Grok Responses API `x_search` tool. This command is the dedicated X search entrypoint and is separate from the general `web_search` attached by default to [`chat`](./chat.md).

For human use, `search` streams readable text by default without exposing raw SSE events or status messages such as `Searching X...`. For scripts, skills, and automation, prefer `--json` for a stable single response.

## Common Usage

```bash
grok-cli search "What are builders saying about Grok today?"
```

Limit date range:

```bash
grok-cli search "AI news" --from-date 2026-05-18 --to-date 2026-05-20
```

Restrict or exclude handles:

```bash
grok-cli search "Grok" --allowed-x-handle xai --excluded-x-handle example
```

Script or skill usage:

```bash
grok-cli search --json --query "Grok Hermes latest updates"
```

Disable default streaming:

```bash
grok-cli search "AI news" --no-stream
```

Output raw normalized stream events:

```bash
grok-cli search "AI news" --raw-stream
```

## Parameters

- `QUERY`: positional search query.
- `--query <QUERY>`: explicit script-friendly query.
- `--json`: use the standard JSON envelope. This disables streaming by default and returns one stable result.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--allowed-x-handle <HANDLE>`: restrict X handles. Repeatable, up to 10.
- `--excluded-x-handle <HANDLE>`: exclude X handles. Repeatable, up to 10.
- `--from-date <YYYY-MM-DD>`: start date.
- `--to-date <YYYY-MM-DD>`: end date.
- `--enable-image-understanding`: enable image understanding.
- `--enable-video-understanding`: enable video understanding.
- `--model <MODEL>`: override the model for this request only.
- `--stream`: explicitly use formatted streaming output.
- `--no-stream`: disable default streaming and print one final result.
- `--raw-stream`: output raw normalized stream events for debugging or programmatic consumption.
- `--timeout <SECONDS>`: request timeout, default `3600`.

## Behavior

- Default model is `grok-4.3`; it can be changed with [`model --model ...`](./model.md).
- Request tool is fixed to `x_search`.
- Non-JSON output defaults to readable text streaming and does not print raw event wrappers.
- `response.created` and `x_search` tool events are not printed as human-visible status messages. Text comes from `response.output_text.delta`.
- `--stream` uses the same readable streaming mode.
- `--raw-stream` switches to raw normalized event output.
- `--json` uses the non-streaming stable result path.
- Requests set `tool_choice: "auto"`, `parallel_tool_calls: true`, and `store: false`.
- Responses extract the message answer, citations, and inline citations.
- Successful calls are written to the local usage SQLite database.

## JSON Fields

`data` contains:

- `success`
- `provider`
- `credential_source`
- `tool`
- `model`
- `query`
- `answer`
- `citations`
- `inline_citations`

## Related Docs

- [chat](./chat.md)
- [model](./model.md)
- [Sample outputs](../reference/samples.md)
