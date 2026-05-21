# Basic Commands

Use this reference for text, search, model, and local usage tasks. Prefer `--json` for automation.

## Chat

General Grok reasoning:

```bash
grok-cli chat --json --prompt "Summarize this topic"
```

Useful flags:

- `--system <TEXT>`: instruction/system text.
- `--model <MODEL>`: per-request model override.
- `--no-web-search`: disable default web search.
- `--with-x-search`: add X search.
- `--allowed-domain <DOMAIN>` / `--excluded-domain <DOMAIN>`: repeatable web filters.
- `--allowed-x-handle <HANDLE>` / `--excluded-x-handle <HANDLE>`: repeatable X filters.
- `--from-date YYYY-MM-DD` / `--to-date YYYY-MM-DD`: date filters.
- `--enable-image-understanding` / `--enable-video-understanding`: search tool understanding flags.
- `--no-stream`: final human-readable response.
- `--raw-stream`: normalized SSE-style events for callers that can parse streams.

## Search

Search X / social discussion:

```bash
grok-cli search --json --query "What are builders saying about Grok today?"
```

Useful flags:

- `--allowed-x-handle <HANDLE>` / `--excluded-x-handle <HANDLE>`.
- `--from-date YYYY-MM-DD` / `--to-date YYYY-MM-DD`.
- `--enable-image-understanding`.
- `--enable-video-understanding`.
- `--model <MODEL>`.
- `--no-stream` or `--raw-stream` when not using JSON.

## Model

Read selected text model:

```bash
grok-cli model --json
```

Set shared default text model for chat and search:

```bash
grok-cli model --json --model grok-4.3
```

Media model overrides are passed directly to media commands with `--model`.

## Usage

Read local usage accounting:

```bash
grok-cli usage --json
```

Useful flags:

- `--session-db <PATH>`: override local SQLite usage database.
- `--session-id <ID>`: read a specific session.

Usage is local accounting, not a live provider quota lookup.
