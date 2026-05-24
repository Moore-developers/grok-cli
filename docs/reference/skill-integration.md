# SKILL Integration Contract

## Goal

Let higher-level skills rely on `grok-cli`'s JSON output, error codes, and recovery flow instead of scattering auth, state, and capability logic across multiple scripts.

The repository includes [`skills/grok-cli/SKILL.md`](../../skills/grok-cli/SKILL.md) as the recommended user entrypoint. That skill runs the user's requested command directly when possible, installs or repairs `grok-cli` only when needed, handles OAuth after real auth failures, and retries the user's original Grok task after recovery.

## 1. Command Calling Principles

- Prefer `--json`.
- Do not rely on the default streaming behavior for automation.
- Treat standard output as machine-consumable by default.
- Use standard error for logs and debug details only.
- Do not assume the CLI is already installed when running through a skill.
- Do not run `status`, login checks, refresh checks, entitlement checks, or capability probes before routine user tasks.
- Run the user's real command first; recover only from actual shell or JSON errors.
- Preserve the user's prompt, query, text, file paths, URLs, and requested output format exactly when building the command. Only quote, escape, choose flags, or resolve paths as mechanically required.

Notes:

- `chat` and `search` stream readable text by default for humans.
- Skills, scripts, and automation should use `--json` consistently.
- Use `--raw-stream` only when the caller can handle normalized stream events.

## 2. Recommended Call Order

### Normal capability calls

1. Build the user's original command, for example `search --json --query "..."`.
2. Run it directly.
3. If it succeeds, render the result.
4. If it fails, recover from the actual error and retry the original command once.

### Install-first flow

1. Enter this flow only if the shell cannot find `grok-cli` or the required subcommand is missing.
2. Install or repair it.
3. Verify `--version` and `--help`.
4. Retry the original Grok task.
5. If that task reports auth trouble, enter auth recovery.

## 3. Error Handling

- `auth_missing`, `auth_expired`, invalid auth, or stale credential messages such as `bad-credentials` or `The OAuth2 access token could not be validated` mean the caller should try `refresh --json` before retrying.
- Credential-validation wording takes priority over `entitlement_denied` flags. If both appear together, refresh first, then retry the original command once.
- `state_file_missing`, `auth_relogin_required`, or `relogin_required` means a fresh login is required.
- If refresh fails because local auth state is missing or relogin is required, run login before retrying.
- Pure `entitlement_denied` without credential-validation wording means the account or tier does not have the capability, so reinstalling, refreshing, or relogin will not fix it.
- Invalid arguments should be corrected in the command shape rather than surfaced to the user as CLI confusion.

## 4. Pass-Through Output Contract

The skill should default to lossless human-readable rendering: parse the JSON envelope, extract command-specific primary fields, and display their values exactly as returned. Mention any recovery work such as installation, login, or refresh in a separate short note after the result.

- For `chat --json`, return `data.output_text` exactly.
- For `search --json`, return `data.answer` exactly and preserve `data.citations` / `data.inline_citations` when exposing citations.
- For `stt --json`, return `data.transcript` exactly.
- For media commands, return exact paths, URLs, request ids, media tags, or handles from the JSON fields.
- Do not summarize, paraphrase, translate, reorder, trim, markdown-polish, or reformat Grok text unless the user explicitly asks for that transformation.
- If the user asks for raw CLI output or raw JSON, return the complete stdout/stderr payload unchanged except for the minimal fencing needed to display it safely.
