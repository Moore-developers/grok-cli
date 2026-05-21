# SKILL Integration Contract

## Goal

Let higher-level skills rely on `grok-cli`'s JSON output, error codes, and recovery flow instead of scattering auth, state, and capability logic across multiple scripts.

The repository includes [`skills/grok-cli/SKILL.md`](../../skills/grok-cli/SKILL.md) as the recommended user entrypoint. That skill checks whether `grok-cli` already exists, installs it from GitHub with Cargo when needed, handles OAuth login, and resumes the user's original Grok task after setup.

## 1. Command Calling Principles

- Prefer `--json`.
- Do not rely on the default streaming behavior for automation.
- Treat standard output as machine-consumable by default.
- Use standard error for logs and debug details only.
- Do not assume the state file exists before calling a command.
- Do not assume the CLI is already installed when running through a skill.

Notes:

- `chat` and `search` stream readable text by default for humans.
- Skills, scripts, and automation should use `--json` consistently.
- Use `--raw-stream` only when the caller can handle normalized stream events.

## 2. Recommended Call Order

### Normal capability calls

1. `status --json`
2. If status is missing or unusable, enter `login`
3. After login completes, resume the original top-level capability command

### Install-first flow

1. Check for `grok-cli`
2. Install it if needed
3. Verify `--version` and `--help`
4. Check auth status
5. Log in if needed
6. Resume the original Grok task

## 3. Error Handling

- `auth_missing` or `state_file_missing` means the caller should run login before retrying.
- `relogin_required` means a fresh login is required.
- `entitlement_denied` means the account or tier does not have the capability, so reinstalling will not fix it.
- Invalid arguments should be corrected in the command shape rather than surfaced to the user as CLI confusion.

## 4. Output Expectations

The skill should return the useful Grok result first, then mention any setup work such as installation or OAuth.
