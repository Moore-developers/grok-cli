# Troubleshooting

## 1. `state_file_missing`

Meaning:

- No usable local auth state file exists.

Fix:

- Run `grok-cli login`.
- For scripts or skills, run `grok-cli login --json`.

## 2. `auth_relogin_required`

Meaning:

- Refresh can no longer recover the current session.
- A fresh browser login is required.

Fix:

- Run `grok-cli login`.

## 3. `xai_oauth_tier_denied`

Meaning:

- The OAuth account does not have permission for the requested API or capability.

Fix:

- Do not keep refreshing in a loop.
- Do not assume reinstalling will help.
- Check the account subscription, entitlement, and capability availability.

Additional note:

- If the upstream body says `The OAuth2 access token could not be validated`, it is not always a true subscription-tier failure.
- During real media validation on 2026-05-20, this text was also seen when the access token was close to expiry; refresh recovered the request.
- Current media commands refresh before requests when the token is close to expiry.
- If you see this on an old binary, upgrade first and retry.

## 4. Browser Login Succeeds But The CLI Fails Later

Common symptoms:

- The browser reports that authorization succeeded.
- The CLI then fails during token exchange or refresh.

Known root cause observed in this project:

- Browser and `curl` access to `auth.x.ai` worked.
- Rust `reqwest` could still pick a problematic IPv6 route in the local environment.

Current handling:

- The shared HTTP client prefers IPv4 outbound binding.

## 5. `stt` Reports A Missing File

Meaning:

- `--file` points to a local audio path that does not exist.

Fix:

- Use an absolute path or a path that exists relative to the current working directory.

## 6. `chat` Or `search` Streaming Fails

Suggested isolation order:

1. Run non-streaming JSON first, such as `grok-cli chat --json --prompt "..."`.
2. If `chat` fails, compare `chat --stream`, `chat --no-stream`, and `chat --raw-stream`.
3. If `search` fails, compare `search --json`, `search --no-stream`, and `search --raw-stream`.

This helps separate auth issues, Responses API issues, and SSE event compatibility issues.

## 7. `stt` Returns `Field 'language' is required when 'format' is true`

Meaning:

- The STT request asked for formatted transcription, and xAI requires `language`.

Current handling:

- `stt` sends `language=en` by default in multipart requests.

If you want another language:

```bash
grok-cli stt ./sample.wav --language zh
```
