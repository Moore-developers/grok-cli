# `grok-cli logout`

## Purpose

Delete the local OAuth state file so the CLI returns to an unauthenticated state.

## Common Usage

```bash
grok-cli logout
```

Script or skill usage:

```bash
grok-cli logout --json
```

## Parameters

- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.

## Behavior

- Deletes the specified or default `auth.json`.
- Succeeds even if the file does not exist and returns `removed: false`.
- Does not delete the session usage SQLite database.

## JSON Fields

`data` contains:

- `removed`
- `auth_store_path`

## Related Docs

- [login](./login.md)
- [status](./status.md)
