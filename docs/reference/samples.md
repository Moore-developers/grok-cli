# Sample State And Outputs

## 1. Sample State File

Sample file:

[`./.sample/auth.json`](../../.sample/auth.json)

Purpose:

- Show the basic shape of the state file schema.
- Show the placement of `provider`, `auth_mode`, `discovery`, `redirect_uri`, and `metadata`.

Note:

- This file uses sample values.
- It cannot be used for real authentication.

## 2. `status --json`

```json
{
  "ok": true,
  "command": "status",
  "data": {
    "logged_in": true,
    "provider": "xai-oauth",
    "auth_mode": "oauth_pkce",
    "access_token_present": true,
    "refresh_token_present": true,
    "access_token_expiring": false,
    "relogin_required": false,
    "entitlement_denied": false,
    "auth_store_path": "~/.grok-cli/auth.json",
    "base_url": "https://api.x.ai/v1"
  }
}
```

## 3. `login --json`

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

## 4. `chat --json`

```json
{
  "ok": true,
  "command": "chat",
  "data": {
    "provider": "xai",
    "model": "grok-4.20-reasoning",
    "protocol": "responses",
    "output_text": "..."
  }
}
```

## 5. `search --json`

```json
{
  "ok": true,
  "command": "search",
  "data": {
    "success": true,
    "provider": "xai",
    "tool": "x_search",
    "model": "grok-4.20-reasoning",
    "query": "..."
  }
}
```

## 6. `usage --json`

```json
{
  "ok": true,
  "command": "usage",
  "data": {
    "provider": "local",
    "session": {},
    "local_usage": {},
    "breakdown": {},
    "recent_rate_limits": []
  }
}
```
