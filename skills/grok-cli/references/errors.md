# Errors

`grok-cli` JSON failures use a stable envelope:

```json
{
  "ok": false,
  "command": "chat",
  "error": {
    "code": "auth_missing",
    "message": "...",
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

## Structured Recovery

Prefer `error.recovery_action` when it is present:

- `refresh_then_retry`: run `grok-cli refresh --json`, then retry the original command once.
- `login_then_retry`: run `grok-cli login`, then retry the original command once.
- `wait_then_retry`: wait for `error.retry_after_seconds`, then retry once.
- `fix_args_then_retry`: fix clear command-shape mistakes, then retry once.
- `stop_billing`: official account billing, balance, credits, or spend cap must be fixed outside the CLI.
- `stop_quota`: quota or usage limit is exhausted.
- `stop_rate_limit`: upstream rate limit did not provide a retry window.
- `stop_entitlement`: account or subscription lacks the capability.
- `stop_unknown` / `none`: return the original error with the minimum useful note.

## Auth Recovery

Use these rules only when an old `grok-cli` binary does not include `recovery_action`.

- `state_file_missing`: no saved auth state; run `grok-cli login`, then resume the original task.
- `auth_missing`: credentials are unavailable; run `grok-cli refresh --json` first, then retry. If refresh fails because local auth state is missing or relogin is required, run `grok-cli login`.
- `auth_expired`: the upstream request rejected the access token; run `grok-cli refresh --json`, then retry the original command once.
- `auth_relogin_required`: refresh cannot recover the session; run `grok-cli login`.
- `access_token_expiring` from an explicit `status --json` diagnostic: refresh with `grok-cli refresh --json` when the user asked for status or diagnostics.
- Credential validation messages such as `bad-credentials` or `The OAuth2 access token could not be validated`: run `grok-cli refresh --json`, then retry the original command once before asking the user to log in again. This rule takes priority even when the same envelope also contains `xai_oauth_tier_denied` or `entitlement_denied: true`.
- `billing_required`, `payment_required`, insufficient balance, insufficient funds, credits, or spend cap wording: stop and explain the official account billing blocker.
- `quota_exhausted`, `insufficient_quota`, quota exceeded, or usage limit wording: stop and explain the official quota blocker.
- `rate_limited` or rate-limit wording: wait and retry only when a retry-after window is available.
- Pure `xai_oauth_tier_denied` with no credential-validation wording: account/tier permission issue; do not promise reinstall, refresh, or relogin will fix it.

The recovery order is failure-driven: run the user's real command first, recover from its actual error, then retry the original command once. Do not run status checks or permission probes before routine user tasks.

## Invalid Arguments

If the CLI returns `invalid_args`, fix the command shape:

- Add missing prompts or input files.
- Avoid incompatible flags.
- Use local output flags only with compatible response formats.
- Use the correct media command for the task.

Do not ask the user to debug CLI syntax unless essential information is missing.

## Sparse Search Results

When `search --json` succeeds, return `data.answer` under the output mode contract. Do not add host-assistant sufficiency judgments unless the user explicitly asks for analysis.

If the user asks you to analyze a sparse result, then say so directly. Include the query and date range used when they are available, then name the likely cause:

- no visible public discussion in that window
- query mismatch or overly broad wording
- auth or entitlement failure
- upstream search did not return enough grounded evidence

Do not turn a generic model explanation into a claim about X sentiment.

## Missing Commands

If the shell says a command such as `image-edit`, `video-edit`, `video-extend`, or `stt-stream` is unrecognized, treat the local installation as incomplete. Repair through the platform-specific install path in `references/install-and-auth.md`: use release binaries on macOS Apple Silicon and Windows x64, and use Cargo only on source-first platforms or when the user explicitly requests a source build. Verify `grok-cli --help` before retrying.

If `~/.local/bin/grok-cli` exists but `command -v grok-cli` fails, treat it as a PATH configuration issue. Temporarily export `PATH="$HOME/.local/bin:$PATH"` or call the binary by absolute path, then explain the permanent PATH fix briefly.
