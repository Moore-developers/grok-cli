# `grok-cli state`

## Purpose

Inspect a redacted summary of the local OAuth state file. `state` does not call the xAI network; it only reads local `auth.json`.

If you only want to know whether the session is currently usable, prefer [`status`](./status.md). `state` is mainly for troubleshooting what was saved locally.

## Common Usage

```bash
grok-cli state
grok-cli state --json
```

Inspect a custom state file:

```bash
grok-cli state --auth-file ./auth.json --json
```

## Parameters

- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.

## Behavior

- `state` is a redacted show command. It no longer exposes `path`, `show`, or `validate` subcommands.
- The command never prints full token values; it only shows whether tokens exist, whether they are expired, and safe summary fields such as provider, auth mode, base URL, last refresh, and last auth error.

## JSON Fields

`data` contains:

- `provider`
- `auth_mode`
- `auth_store_path`
- `base_url`
- `access_token_present`
- `refresh_token_present`
- `access_token_expiring`
- `last_refresh`
- `last_auth_error`

## Related Docs

- [status](./status.md)
