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
    "entitlement_denied": false
  }
}
```

## Auth Recovery

- `state_file_missing`: no saved auth state; run `grok-cli login`, then resume the original task.
- `auth_missing`: credentials are unavailable; run `grok-cli refresh --json` first, then retry. If refresh fails because local auth state is missing or relogin is required, run `grok-cli login`.
- `auth_relogin_required`: refresh cannot recover the session; run `grok-cli login`.
- `access_token_expiring` from an explicit `status --json` diagnostic: refresh with `grok-cli refresh --json` when the user asked for status or diagnostics.
- `xai_oauth_tier_denied`: account/tier permission issue; do not promise reinstall or relogin will fix it.
- Credential validation messages such as `bad-credentials`: run `grok-cli refresh --json`, then retry the original command once before asking the user to log in again.

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
