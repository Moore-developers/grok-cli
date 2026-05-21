# `grok-cli login`

## Purpose

Start xAI OAuth PKCE login. By default, the command opens the system browser and exchanges the authorization code for tokens when the local loopback callback arrives.

Default auth state path:

```text
~/.grok-cli/auth.json
```

## Common Usage

```bash
grok-cli login
```

Script or skill usage:

```bash
grok-cli login --json
```

If the browser cannot complete the local callback, use manual paste mode:

```bash
grok-cli login --manual-paste
```

Print login flow status and use a custom port:

```bash
grok-cli login --port 56121 --timeout 180
```

## Parameters

- `--json`: use the standard JSON envelope.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--no-browser`: do not automatically open a browser; only prepare the local login state.
- `--manual-paste`: use manual callback / authorization code paste mode.
- `--timeout <SECONDS>`: loopback callback wait timeout.
- `--port <PORT>`: local callback port.

## Behavior

- Generates the PKCE verifier / challenge, OAuth `state`, and `nonce`.
- Builds the xAI authorization URL with `referrer=hermes-agent` and `plan=generic`.
- Writes the pending OAuth session to `auth.json`.
- Opens the real system browser by default.
- On a successful loopback callback, automatically exchanges the authorization code.
- If the callback times out in an interactive terminal, the command falls back to manual paste.
- On success, writes access token, refresh token, redirect URI, and last refresh metadata.

## JSON Output

Success:

```json
{
  "ok": true,
  "command": "login",
  "data": {
    "provider": "xai-oauth",
    "auth_mode": "oauth_pkce",
    "saved": true,
    "auth_store_path": "~/.grok-cli/auth.json",
    "redirect_uri": "http://127.0.0.1:56121/callback",
    "base_url": "https://api.x.ai/v1"
  }
}
```

Failures follow the standard error envelope. Common errors include `auth_callback_timeout`, `auth_state_mismatch`, and `auth_token_exchange_failed`.

## Related Docs

- [status](./status.md)
- [Internal auth recovery entrypoints](../reference/internal-auth.md)
- [Troubleshooting](../guides/troubleshooting.md)
