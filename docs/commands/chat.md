# `grok-cli chat`

## Purpose

Run text chat, summarization, reasoning, and Q&A through the Grok Responses API. General `web_search` is enabled by default, which makes the command useful for recent facts.

For human use, `chat` streams readable text by default without exposing raw SSE events or status messages such as `Thinking...`. For scripts, skills, and automation, prefer `--json` for a stable single response.

## Common Usage

```bash
grok-cli chat "Summarize recent AI news"
```

Pure chat without search:

```bash
grok-cli chat "Explain OAuth PKCE" --no-web-search
```

Use both web search and X search:

```bash
grok-cli chat "What are the hottest AI discussions from the last 48 hours?" --with-x-search
```

Script or skill usage:

```bash
grok-cli chat --json --prompt "Summarize today's AI news"
```

Disable default streaming:

```bash
grok-cli chat "Tell a short story" --no-stream
```

Explicitly request formatted streaming:

```bash
grok-cli chat "Tell a short story" --stream
```

Output raw normalized stream events:

```bash
grok-cli chat "Tell a short story" --raw-stream
```

## Parameters

- `PROMPT`: positional user prompt.
- `--prompt <PROMPT>`: explicit script-friendly prompt.
- `--json`: use the standard JSON envelope. This disables streaming by default and returns one stable result.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--system <TEXT>`: system instruction, mapped to Responses API `instructions`.
- `--model <MODEL>`: override the model for this request only.
- `--no-web-search`: disable default `web_search`.
- `--with-x-search`: also attach `x_search`.
- `--allowed-domain <DOMAIN>`: restrict web search domains. Repeatable, up to 10.
- `--excluded-domain <DOMAIN>`: exclude web search domains. Repeatable, up to 10.
- `--enable-image-understanding`: enable image understanding for search tools.
- `--allowed-x-handle <HANDLE>`: restrict X search handles. Repeatable, up to 10.
- `--excluded-x-handle <HANDLE>`: exclude X search handles. Repeatable, up to 10.
- `--from-date <YYYY-MM-DD>`: X search start date.
- `--to-date <YYYY-MM-DD>`: X search end date.
- `--enable-video-understanding`: enable video understanding for X search.
- `--stream`: explicitly use formatted streaming output.
- `--no-stream`: disable default streaming and print one final result.
- `--raw-stream`: output raw normalized stream events for debugging or programmatic consumption.
- `--timeout <SECONDS>`: request timeout, default `3600`.

## Behavior

- Default model is `grok-4.20-reasoning`; it can be changed with [`model --model ...`](./model.md).
- Requests use `store: false`.
- `web_search` is attached by default.
- Non-JSON output defaults to readable text streaming and does not print raw event wrappers.
- `response.created`, reasoning, and tool events are not printed as human-visible status messages. Text comes from `response.output_text.delta`.
- `--stream` uses the same readable streaming mode.
- `--raw-stream` switches to raw normalized event output.
- `--json` uses the non-streaming stable result path.
- `--with-x-search` adds `x_search` alongside default `web_search`.
- `--no-web-search --with-x-search` attaches only `x_search`.
- When tools are attached, requests set `tool_choice: "auto"` and `parallel_tool_calls: true`.
- Non-streaming responses are written to the local usage SQLite database.
- Raw event streams may include `response.output_text.delta`, `response.output_text.done`, `response.output_item.done`, `response.completed`, and `response.failed`.

## JSON Fields

Non-streaming `data` contains:

- `provider`
- `model`
- `protocol`
- `output_text`
- `finish_reason`
- `tool_calls`

## Related Docs

- [search](./search.md)
- [model](./model.md)
- [Sample outputs](../reference/samples.md)
