# `grok-cli refresh`

## Purpose

Force a refresh of the saved access token by using the refresh token.

## Common Usage

```bash
grok-cli refresh
```

Script or skill usage:

```bash
grok-cli refresh --json
```

## Parameters

- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.

## Behavior

- Reads the local OAuth state file.
- Uses the refresh token to call the xAI OAuth token endpoint.
- On success, atomically writes the new access token, any updated refresh token, and `last_refresh`.
- On failure, writes `last_auth_error` while preserving endpoint, phase, grant type, redirect URI, and other troubleshooting context.
- Performs lightweight retry for network connect / timeout / request errors.

## Related Docs

- [login](./login.md)
- [status](./status.md)
