# `grok-cli status`

## Purpose

Check whether the current OAuth state is usable. The command does not make network requests; it only reads the local `auth.json` file and inspects token state, refresh token state, expiry, and error flags.

## Common Usage

```bash
grok-cli status
```

Script or skill usage:

```bash
grok-cli status --json
```

Inspect a custom auth file:

```bash
grok-cli status --auth-file ./auth.json --json
```

## Parameters

- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.

## Behavior

- Reads and validates the OAuth state file.
- Non-JSON output uses a three-column table: field, current value, and English explanation.
- Reports `logged_in`, `access_token_present`, `refresh_token_present`, and `access_token_expiring`.
- Reports `relogin_required` and `entitlement_denied`, which help upstream callers decide whether to re-login or treat the issue as an entitlement problem.
- Does not auto-refresh. Use [`refresh`](./refresh.md) when you need to refresh.

## JSON Fields

`data` contains:

- `logged_in`
- `provider`
- `auth_mode`
- `access_token_present`
- `refresh_token_present`
- `access_token_expiring`
- `relogin_required`
- `entitlement_denied`
- `auth_store_path`
- `base_url`
- `last_refresh`
- `last_auth_error`

## Related Docs

- [login](./login.md)
- [refresh](./refresh.md)
- [state](./state.md)
