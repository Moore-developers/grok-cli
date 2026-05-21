---
name: grok-cli
description: Use this skill whenever the user wants to use Grok or xAI through the grok-cli command line, including chat, X search, image generation/editing, video generation/editing/extension, text-to-speech, speech-to-text, OAuth login/status, or local usage stats. This skill should also be used when grok-cli may need to be installed first; it handles checking for the CLI, installing it from GitHub with Cargo, running JSON-mode commands, and resuming the user's original Grok task after login.
---

# Grok CLI Skill

This skill turns a user's Grok / xAI request into a deterministic `grok-cli` workflow. It is SKILL-first and source-first: users do not need prebuilt release binaries. If `grok-cli` is missing or missing required command surfaces, install it from GitHub with Cargo, then run the requested command.

Repository:

```text
https://github.com/Moore-developers/grok-cli
```

## Core Workflow

1. Identify the user's intended Grok capability.
2. Ensure `grok-cli` is installed, runnable, and exposes the required commands.
3. Check OAuth status with `grok-cli status --json`.
4. If login is missing or expired, run `grok-cli login`.
5. Resume the original user task with the correct `grok-cli` command.
6. Prefer `--json` for automation, parsing, and reliable error handling.

Keep the original task in mind while handling installation or login. Authentication and installation are setup steps, not the final answer.

## Installation Check

First check whether the command exists:

```bash
command -v grok-cli
grok-cli --version
grok-cli --help
```

Version alone is not enough. `v0.1.0` was retagged during pre-release validation, so an older local binary can still report `grok-cli 0.1.0` while missing newer commands. Verify that top-level help includes these required commands:

```text
image-edit
video-edit
video-extend
stt-stream
```

If `grok-cli` is missing or any required command is absent, check whether Cargo is available:

```bash
command -v cargo
```

If Cargo is missing, tell the user they need Rust/Cargo first and point them to install Rust with `rustup`. Do not attempt a prebuilt binary install path; this project intentionally uses source-first distribution.

If Cargo exists, install from the latest repository state when the user asked for latest:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked --force
```

For a pinned public version:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked --force
```

After installation, verify:

```bash
grok-cli --version
grok-cli --help
```

If reinstalling because a command was missing, rerun the original user task after verification. Do not stop at installation.

## OAuth Handling

Check status before real capability calls:

```bash
grok-cli status --json
```

If the status says auth is missing, invalid, or relogin is required, run:

```bash
grok-cli login
```

After login completes, rerun:

```bash
grok-cli status --json
```

Then resume the original task. Do not ask the user to repeat it unless essential information is missing.

## Command Routing

Default to explicit flags plus `--json` for reliable automation.

Use `chat` for general Grok reasoning or answering:

```bash
grok-cli chat --json --prompt "Summarize this topic"
```

Use `search` for X / Twitter / social discussion search:

```bash
grok-cli search --json --query "What are builders saying about Grok today?"
```

Use `image` for image generation:

```bash
grok-cli image --json --prompt "A cinematic skyline at sunrise"
```

Use `image-edit` for image editing:

```bash
grok-cli image-edit --json --image ./source.png --prompt "Make it cinematic"
```

Use `video` for video generation:

```bash
grok-cli video --json --prompt "Animate a futuristic skyline" --duration 8
```

Use `video-edit` for editing an existing video:

```bash
grok-cli video-edit --json --video-url https://example.com/source.mp4 --prompt "Make it cinematic"
```

Use `video-extend` for extending an existing video:

```bash
grok-cli video-extend --json --video-url https://example.com/source.mp4 --prompt "Continue the camera move" --duration 6
```

Use `tts` for text-to-speech:

```bash
grok-cli tts --json --text "Hello from Grok"
```

Use `stt` for batch speech-to-text:

```bash
grok-cli stt --json --file ./sample.wav
```

Use `usage` for local usage stats:

```bash
grok-cli usage --json
```

For complete command options, read only the relevant reference file:

- `references/install-and-auth.md`: install, upgrade, status, login, refresh, logout, state.
- `references/commands-basic.md`: chat, search, model, usage.
- `references/commands-media.md`: image, image-edit, video, video-edit, video-extend, tts, stt, stt-stream common use.
- `references/commands-advanced.md`: advanced flags and combination rules.
- `references/errors.md`: JSON errors, auth recovery, entitlement handling.
- `references/outputs.md`: stable JSON fields to read.

## Error Handling

Read JSON errors from the standard envelope:

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

Important handling rules:

- If `relogin_required` is true, run `grok-cli login`, then resume the task.
- If `entitlement_denied` is true, explain that this is an account or tier permission issue, not something a reinstall or relogin necessarily fixes.
- If `state_file_missing` or `auth_missing` appears, run login before retrying the original task.
- If a command returns invalid arguments, fix the command shape instead of asking the user to debug CLI syntax.

## Output To User

Return the useful Grok result first. Then briefly mention any setup performed, such as installing `grok-cli` or completing OAuth.

Avoid dumping raw JSON unless the user asked for it. For media commands, report the returned file path, URL, or generated media handle clearly.
