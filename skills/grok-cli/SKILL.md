---
name: grok-cli
description: Use this skill whenever the user wants to use Grok or xAI through the grok-cli command line, including chat, X search, image generation/editing, video generation/editing/extension, text-to-speech, speech-to-text, OAuth login/status, or local usage stats. This skill should also be used when grok-cli may need to be installed first; it handles checking for the CLI, installing it from GitHub with Cargo, running JSON-mode commands, and resuming the user's original Grok task after login.
---

# Grok CLI Skill

This skill turns a user's Grok / xAI request into a deterministic `grok-cli` workflow. It is SKILL-first. macOS (Intel and Apple Silicon) and Linux users stay source-first through Cargo, while Windows users can use the GitHub Release binary. If `grok-cli` is missing or missing required command surfaces, install it from GitHub with Cargo, then run the requested command.

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

## What Users Can Do Through This Skill

Users should be able to rely on this skill as the primary interface for `grok-cli`. Route requests to these supported capabilities:

- General Grok reasoning and writing with `chat`
- X / Twitter search with `search`
- Image generation with `image`
- Image editing with `image-edit`
- Video generation with `video`
- Video editing with `video-edit`
- Video extension with `video-extend`
- Text-to-speech with `tts`
- Batch speech-to-text with `stt`
- Experimental streaming speech-to-text with `stt-stream`
- OAuth install / login / refresh / status / logout / state
- Local usage inspection with `usage`

Important support rules:

- `video` supports text-to-video, `--image-url`, local `--image`, `--reference-image-url`, and local `--reference-image`.
- `video-edit` supports both `--video-url` and local `--video`.
- `video-extend` supports `--video-url` only.
- Do not offer `video-extend --video <PATH>`. Real validation showed that local MP4 input can reach xAI, but extension generation ends in upstream internal error. Users should upload the video first and then use `--video-url`.
- `image-edit --image` accepts local paths and remote URLs.
- `stt` supports local files and remote `--url`.
- `stt-stream` is experimental; prefer `stt` unless the user explicitly needs streaming behavior.

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

If Cargo is missing, tell the user they need Rust/Cargo first and point them to install Rust with `rustup`. For Windows users who do not want Rust/Cargo, point them to the GitHub Release binary instead of trying to invent a source install workaround.

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

## Common Parameter Cheat Sheet

Use this page for the usual knobs and a few representative values. If the user asks for a rare flag, an exact value range, or a long combination rule, stop here and open the matching reference file below instead of trying to squeeze everything into the main SKILL.

- `login`: use `--no-browser`, `--manual-paste`, `--timeout 300`, and `--port 8787` when the machine cannot complete a normal browser callback.
- `status`, `refresh`, `logout`, `state`: use `--auth-file <PATH>` when the user wants an alternate local OAuth state file.
- `model`: use `--model grok-4.3` to set the shared default text model for `chat` and `search`.
- `usage`: use `--session-db <PATH>` and `--session-id <ID>` for a specific local usage record.
- `chat`: use `--prompt` or a positional prompt, `--system`, `--model`, `--no-web-search`, `--with-x-search`, `--allowed-domain example.com`, `--stream`, `--no-stream`, `--raw-stream`, and `--timeout 3600`.
- `search`: use `--query` or a positional query, `--model`, `--allowed-x-handle xAI`, `--from-date 2026-05-01`, `--to-date 2026-05-21`, `--stream`, `--no-stream`, `--raw-stream`, and `--timeout 3600`.
- `image`: use `--count 1-10`, `--response-format url|b64_json`, `--output-file <PATH>`, `--output-dir <PATH>`, `--aspect-ratio 1:1`, and `--resolution 1k`.
- `image-edit`: use repeat `--image` up to 3 times, `--output-file <PATH>`, and `--response-format b64_json` when writing locally.
- `video`: use `--image ./source.png`, `--reference-image ./a.png`, `--duration 8`, `--aspect-ratio 16:9`, and `--resolution 720p`.
- `video-edit`: use `--video ./source.mp4` or `--video-url https://...`.
- `video-extend`: use `--video-url https://...` and `--duration 6`.
- `tts`: use `--list-voices`, `--text`, `--voice-id ara`, `--language en`, `--output ./out.mp3`, and `--output-format mp3`.
- `stt`: use `--file ./sample.wav`, `--url`, `--language auto`, `--format true`, `--diarize`, `--keyterm Grok`, and `--filler-words`.
- `stt-stream`: use `--file ./sample.wav`, `--interim-results`, `--language auto`, and `--sample-rate 16000`.

## Reference Map

Use the matching reference file when the user wants exact flag behavior, uncommon combinations, or the full parameter surface:

- `references/install-and-auth.md`: install, status, login, refresh, logout, state, and auth-file handling.
- `references/commands-basic.md`: chat, search, model, usage, and text-command filters.
- `references/commands-media.md`: image, image-edit, video, video-edit, video-extend, TTS, and STT common use.
- `references/commands-advanced.md`: exact values, rare flags, combination rules, and edge cases.
- `references/errors.md`: JSON errors, auth recovery, and entitlement handling.
- `references/outputs.md`: stable JSON fields to read.

When the user gives a local media path:

- `image-edit --image ./file.png`: supported
- `video --image ./file.png`: supported
- `video --reference-image ./file.png`: supported
- `video-edit --video ./file.mp4`: supported
- `video-extend --video ./file.mp4`: not supported in the current public skill surface; convert to a remote URL workflow instead

## Skill Test Prompts

Use these prompts to verify that Codex or Claude Code routes through this skill correctly and picks the expected `grok-cli` command shape. For the full matrix, including every public capability and local-file coverage, use:

- `docs/project/skill-validation-cases.md`

Minimum routing checks:

- `用 Grok 总结一下最近关于 Rust CLI 设计的讨论，返回结构化结果`
  Expected route: `grok-cli chat --json --prompt ...`
- `搜索一下 X 上大家今天怎么评价 Grok CLI`
  Expected route: `grok-cli search --json --query ...`
- `生成一张 1:1 的极简终端图标，保存到本地`
  Expected route: `grok-cli image --json --prompt ... --aspect-ratio 1:1 --output-file ...`
- `把这张图片改得更像命令行工具封面：./source.png`
  Expected route: `grok-cli image-edit --json --image ./source.png --prompt ...`
- `用这张本地图片做一个短视频：./source.png`
  Expected route: `grok-cli video --json --prompt ... --image ./source.png`
- `编辑这个本地视频，让画面更有电影感：./source.mp4`
  Expected route: `grok-cli video-edit --json --video ./source.mp4 --prompt ...`
- `把这个视频再延长两秒：https://example.com/source.mp4`
  Expected route: `grok-cli video-extend --json --video-url https://example.com/source.mp4 --duration 2 --prompt ...`
- `把这段文字转成 ara 声音的 mp3 并保存`
  Expected route: `grok-cli tts --json --text ... --voice-id ara --output ... --output-format mp3`
- `转写这个音频，并保留关键词 Grok 和 CLI：./sample.wav`
  Expected route: `grok-cli stt --json --file ./sample.wav --keyterm Grok --keyterm CLI ...`

Negative routing checks:

- If the user asks to extend a local video file directly, the skill must not invent `video-extend --video ./source.mp4`.
- Instead, the skill should explain that `video-extend` currently requires a remote URL and ask for or help produce an upload URL workflow.

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
