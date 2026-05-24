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

- Prefer `error.recovery_action` when the JSON error includes it. This field is the CLI's single recovery decision and should override ad hoc interpretation of `code`, `message`, `relogin_required`, or `entitlement_denied`.
- `refresh_then_retry`: run `refresh --json`, then retry the original command once.
- `login_then_retry`: run `login`, then retry the original command once.
- `wait_then_retry`: wait for `error.retry_after_seconds`, then retry the original command once.
- `fix_args_then_retry`: correct a clear command-shape issue, then retry once.
- `stop_billing`, `stop_quota`, `stop_rate_limit`, and `stop_entitlement` mean the caller should stop and surface the blocker. Do not reinstall, refresh, or relogin for these blockers.
- For old binaries without `recovery_action`, use fallback matching: credential-validation failures refresh, relogin-required failures login, pure billing/quota/rate-limit/entitlement blockers stop.
- Invalid arguments should be corrected in the command shape rather than surfaced to the user as CLI confusion.

## 4. Pass-Through Output Contract

The skill should default to lossless human-readable rendering: parse the JSON envelope, extract command-specific primary fields, and display their values exactly as returned. Mention any recovery work such as installation, login, or refresh in a separate short note after the result.

- For `chat --json`, return `data.output_text` exactly.
- For `search --json`, return `data.answer` exactly and preserve `data.citations` / `data.inline_citations` when exposing citations.
- For `stt --json`, return `data.transcript` exactly.
- For media commands, return exact paths, URLs, request ids, media tags, or handles from the JSON fields.
- Do not summarize, paraphrase, translate, reorder, trim, markdown-polish, or reformat Grok text unless the user explicitly asks for that transformation.
- If the user asks for raw CLI output or raw JSON, return the complete stdout/stderr payload unchanged except for the minimal fencing needed to display it safely.
