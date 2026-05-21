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
- `auth_missing`: credentials are unavailable; login or refresh before retrying.
- `auth_relogin_required`: refresh cannot recover the session; run `grok-cli login`.
- `xai_oauth_tier_denied`: account/tier permission issue; do not promise reinstall or relogin will fix it.

## Invalid Arguments

If the CLI returns `invalid_args`, fix the command shape:

- Add missing prompts or input files.
- Avoid incompatible flags.
- Use local output flags only with compatible response formats.
- Use the correct media command for the task.

Do not ask the user to debug CLI syntax unless essential information is missing.

## Missing Commands

If the shell says a command such as `image-edit`, `video-edit`, `video-extend`, or `stt-stream` is unrecognized, treat the local installation as incomplete. Reinstall with Cargo and verify `grok-cli --help` before retrying.
