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
    "model": "grok-4.3",
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
    "model": "grok-4.3",
    "query": "..."
  }
}
```

## Error Envelope

```json
{
  "ok": false,
  "command": "chat",
  "error": {
    "code": "auth_expired",
    "message": "responses request auth failed: The OAuth2 access token could not be validated. [WKE=unauthenticated:bad-credentials]",
    "relogin_required": false,
    "entitlement_denied": false,
    "category": "auth_refreshable",
    "recovery_action": "refresh_then_retry",
    "retryable": true,
    "retry_after_seconds": null,
    "billing_required": false,
    "quota_exhausted": false,
    "rate_limited": false
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
