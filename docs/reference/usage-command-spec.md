# `usage` Command Spec

## 1. Goal

`grok-cli usage` is a stable top-level command for answering how much the current Grok session has consumed locally.

It is intentionally separate from auth commands, `state`, and general capability commands because it crosses local session accounting, recent rate-limit snapshots, and usage breakdowns.

Recommended command shape:

```text
grok-cli usage [options]
```

## 2. Command Semantics

### 2.1 First version

```bash
cargo run -- usage --json
```

Recommended parameters:

- `--json`
- `--auth-file`
- `--session-db`
- `--session-id`
- `--timeout`
- `--local-only`

### 2.2 Output goals

- Surface text, image, video, and audio usage separately.
- Keep token and cost estimates readable.
- Keep the output suitable for both humans and automation.

## 3. JSON Expectations

`data` should include:

- `provider`
- `session`
- `local_usage`
- `breakdown`
- `recent_rate_limits`

## 4. Why It Is Top-Level

`usage` is not an auth action.
It is not a one-shot task execution command.
It is not merely a state file display.
It is the local accounting view for the current session.
