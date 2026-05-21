# `grok-cli usage`

## Purpose

Inspect local session usage. The command reads the SQLite session store and summarizes text, image, video, and audio usage, along with token, cost estimate, and context-length information.

Default database:

```text
~/.grok-cli/session.db
```

## Common Usage

```bash
grok-cli usage
```

Only read local stats and skip any provider account lookup:

```bash
grok-cli usage --local-only
```

Note: `usage` now defaults to local-only behavior. `--local-only` exists for compatibility and is not needed day to day.

Script or skill usage:

```bash
grok-cli usage --json
```

Inspect a specific session:

```bash
grok-cli usage --session-id sess_01_example
```

## Parameters

- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--session-db <PATH>`: override the SQLite session database path.
- `--session-id <ID>`: inspect a specific session instead of the active session.
- `--timeout <SECONDS>`: hidden compatibility parameter; the command no longer queries account limits.
- `--local-only`: compatibility parameter; the command already shows local stats only.

## Behavior

- Successful local statistics still count as a successful command.
- Text commands report token and cost estimates.
- Image, video, and audio commands report request count, model, and rate-limit snapshots; when no token data exists, values show as `0` or `n/a`.
- Token values are abbreviated with `K`, `M`, or `B`, such as `124.8K` or `2.8M`.
- The command does not query, display, or return account limits.

## Human-Readable Output

Default output includes:

- `Session Usage`
- `Usage Breakdown`
- `Session metadata`

## JSON Fields

`data` contains:

- `provider`
- `session`
- `local_usage`
- `breakdown`
- `recent_rate_limits`

## Related Docs

- [`usage` deep spec](../reference/usage-command-spec.md)
- [Sample outputs](../reference/samples.md)
