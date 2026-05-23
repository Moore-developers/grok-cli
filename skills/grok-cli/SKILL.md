---
name: grok-cli
description: Use this skill whenever the user wants to use Grok or xAI through the grok-cli command line, including chat, X search, image generation/editing, video generation/editing/extension, text-to-speech, speech-to-text, OAuth login/status, or local usage stats. This skill should also be used when grok-cli may need to be installed first; it runs the user's requested grok-cli command directly when possible, repairs install/auth/permission problems only after an actual failure, retries the original task after recovery, and returns Grok/grok-cli results without assistant-side rewriting unless the user explicitly asks for transformation.
---

# Grok CLI Skill

This skill turns a user's Grok / xAI request into a deterministic `grok-cli` workflow. It is SKILL-first. On covered release platforms, prefer the prebuilt GitHub Release binary instead of compiling from source. Today that means macOS Apple Silicon uses the maintainer-uploaded release tarball, Windows x64 uses the GitHub Actions-built release zip, and macOS Intel / Linux stay source-first through Cargo.

Repository:

```text
https://github.com/Moore-developers/grok-cli
```

## Core Workflow

1. Identify the user's intended Grok capability.
2. Preserve the user's original prompt, query, text, file paths, URLs, and output instructions when constructing the command.
3. Prefer explicit flags plus `--json` for automation, parsing, and reliable error handling.
4. Fast path: run the user's requested `grok-cli` command directly when `grok-cli` is available. Do not run `status`, `state`, login checks, refresh checks, entitlement checks, or capability probes before routine `chat`, `search`, `usage`, media, or audio commands.
5. If the command succeeds, render the result according to the output mode contract below.
6. If the command fails, inspect the actual shell error or JSON error envelope and run only the matching recovery action.
7. After a successful recovery action, retry the user's original command once, unchanged except for mechanical fixes required by the recovered environment.
8. If the retry fails, return the original error code and message with the minimum actionable note needed.

Keep the original task in mind while handling installation, login, refresh, or permission issues. These are recovery steps after a real failure, not preflight work.

## Output Mode Contract

Default output mode is lossless human-readable rendering: parse the JSON envelope, extract command-specific primary fields, and display their values exactly as returned. Return full raw JSON only when the user explicitly asks for raw JSON.

The host assistant is a renderer for Grok results, not a second editor by default. Preserve both the user's request and Grok's response unless the user explicitly asks for translation, summarization, restructuring, rewriting, formatting, or analysis by the host assistant.

Input handling:

- Pass the user's substantive prompt, query, text, URLs, file paths, and requested output format to `grok-cli` exactly as written.
- Do not translate, summarize, expand, shorten, clean up, normalize, improve, or reinterpret the user's wording before sending it to Grok.
- Do not add hidden instructions, extra context, preferred structure, safety language, style rules, or examples to the Grok prompt unless the user explicitly provided them.
- Only make mechanical changes required by the shell or CLI, such as quoting, escaping, choosing the correct flag, adding explicit date flags requested by the user, or resolving a local path.
- Setup probes, auth checks, and permission checks are separate from the user's original task. Never mix their prompts or outputs into the user's Grok request.

Output handling:

- For text commands, render the exact text value produced by Grok: `data.output_text` for `chat --json`, `data.answer` for `search --json`, and `data.transcript` for `stt --json`.
- Do not summarize, paraphrase, translate, reorder, trim, markdown-polish, add headings, add bullets, correct grammar, merge citations into prose, or otherwise rewrite Grok's text.
- Preserve Grok's whitespace, line breaks, code fences, citations, numbering, and wording as much as the chat surface allows.
- For media commands, return the exact generated file path, URL, data URL, request id, media tag, or handle from the JSON fields. Do not rename or editorialize them.
- If the user asks for raw CLI output or raw JSON, return the complete stdout/stderr payload unchanged except for the minimal fencing needed to display it safely.
- If the user explicitly asks the host assistant to summarize, translate, rewrite, extract, format, compare, classify, or restructure Grok output, enter transformation mode and make it clear that the result is transformed output rather than verbatim Grok output.
- If recovery happened, mention it only after the Grok result, in a short separate note. Do not insert recovery commentary into the Grok result itself.
- If a command fails, return the original error code and message. Add only the minimum actionable recovery note needed for auth, entitlement, or command-shape blockers.

## Fast Path And Recovery

Use optimistic execution for normal user tasks:

1. Build the user's original command, for example `grok-cli search --json --query "..."`.
2. Run that command directly.
3. If it succeeds, render the result.
4. If it fails, recover based on the actual error, then retry the original command once.

Do not run readiness probes such as `grok-cli search --json --query "Grok"` or `grok-cli chat --json --prompt "Reply with exactly: ok"` before a real user request. A user asking to search should get the real search first.

Recovery map:

- Shell cannot find `grok-cli`: install it, verify `grok-cli --version` and `grok-cli --help`, then retry the original command.
- Shell says a subcommand is missing: repair or upgrade the install, verify `grok-cli --help`, then retry the original command.
- JSON error `auth_missing`, invalid auth, credential validation failure such as `bad-credentials`, expired token, or stale token: run `grok-cli refresh --json` first, then retry the original command.
- If refresh fails because local auth state is missing, refresh cannot recover the session, or `relogin_required` is true, run `grok-cli login`, then retry the original command.
- JSON error `state_file_missing`, `auth_relogin_required`, or `relogin_required: true` from the original command means refresh is unlikely to help; run `grok-cli login`, then retry the original command.
- `access_token_expiring` from `status --json`: refresh only when the user specifically asked for `status` or diagnostics. Do not run status just to discover this before routine tasks.
- `entitlement_denied` or `xai_oauth_tier_denied`: explain that the account or subscription cannot access that capability. Do not retry, reinstall, or relogin unless the error also explicitly requires relogin.
- `invalid_args`: fix the command shape if the correct shape is clear, then retry once. If essential information is missing, ask the user for that missing input.

After recovery, retry the original user command, not a probe command. Preserve the original prompt or query exactly.

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

Version alone is not enough. Older pre-release installs can report a valid version while missing newer commands. Verify that top-level help includes these required commands:

```text
image-edit
video-edit
video-extend
stt-stream
```

If `grok-cli` is missing or any required command is absent, first identify the platform and choose the install path in this order:

- macOS Apple Silicon: prefer the GitHub Release asset `grok-cli-macos-aarch64-apple-darwin.tar.gz`.
- Windows x64: prefer the GitHub Release asset `grok-cli-windows-x86_64-pc-windows-msvc.zip`.
- macOS Intel and Linux: use Cargo source install.
- If a covered release asset is missing unexpectedly, surface that clearly and do not silently fall back to Cargo on a product platform.
- Only use Cargo when the user explicitly asks for a source build, or when the platform is source-first.

Only check whether Cargo is available when the platform is source-first or when the user explicitly wants a source build:

```bash
command -v cargo
```

For source installs, also check the active compiler version:

```bash
rustc --version
```

`grok-cli` source installs require Rust 1.88 or newer because the crate uses edition 2024 and declares `rust-version = "1.88"`. CI and local development are pinned to Rust 1.92.0 through `rust-toolchain.toml`.

Use GitHub Release binaries by default when the user's platform is covered:

- macOS Apple Silicon: `grok-cli-macos-aarch64-apple-darwin.tar.gz`
- Windows x64: `grok-cli-windows-x86_64-pc-windows-msvc.zip`

If Cargo is missing on macOS Intel or Linux, do not invent a binary install path; tell the user they need Rust/Cargo first and point them to install Rust with `rustup`.

If `rustc --version` is older than 1.88, stop and explain the exact requirement instead of telling the user to "upgrade Rust" generically. Say that `grok-cli` source install requires Rust 1.88+ and that the repository toolchain is currently 1.92.0. Then suggest either:

- using the release binary on macOS Apple Silicon or Windows x64, or
- running `rustup update` before retrying the source install on source-first platforms.

For Release binary installs:

1. Confirm the platform is exactly covered by the asset name.
2. Download the asset and matching `.sha256` from the latest GitHub Release.
3. Verify the checksum when possible.
4. Extract the binary.
5. Put `grok-cli` or `grok-cli.exe` in a directory already on `PATH`; if none is suitable, use `~/.local/bin`, temporarily add it to the current shell `PATH`, and tell the user how to make that permanent.
6. If `command -v grok-cli` still fails but `~/.local/bin/grok-cli` exists and is executable, treat this as a PATH configuration issue, not a failed install. Continue by temporarily exporting `PATH="$HOME/.local/bin:$PATH"` or using `~/.local/bin/grok-cli` directly for verification.
7. Run `grok-cli --version` and `grok-cli --help`.
8. Retry the original Grok task. If that task reports auth trouble, follow OAuth handling below.

If Cargo is the chosen path, install from the latest repository state when the user asked for latest:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked --force
```

For a pinned public version:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.1 --locked --force
```

After installation, verify:

```bash
grok-cli --version
grok-cli --help
```

If reinstalling because a command was missing, rerun the original user task after verification. Do not stop at installation.

## OAuth Handling

Do not check OAuth status before routine Grok calls. Let the user's real command run first, then recover only if it reports an auth problem.

If the real command says auth is missing, invalid, expired, stale, or `bad-credentials`, try refresh first:

```bash
grok-cli refresh --json
```

Then retry the original command once.

If refresh fails because local auth state is missing, refresh cannot recover the session, or relogin is required, run:

```bash
grok-cli login
```

Then retry the original command once. If the retry still fails, explain the auth or entitlement error clearly and ask the user to run login only when relogin is actually required.

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

For X search tasks:

- Include relevant date flags when the user asks for today, recent discussion, or a bounded time window.
- Pass the search query text through exactly as the user wrote it.
- After running the command, return `data.answer` exactly as Grok returned it.
- Preserve `data.citations` and `data.inline_citations` exactly when exposing citations.
- Do not add host-assistant conclusions about sufficiency, sentiment, or meaning unless the user explicitly asks for your analysis.
- If the command itself reports sparse or missing results, pass that message through rather than turning it into a separate summary.

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

Minimum routing and pass-through checks:

- `用 Grok 总结一下最近关于 Rust CLI 设计的讨论，返回结构化结果`
  Expected route: `grok-cli chat --json --prompt ...`; the prompt text must remain unchanged.
- `搜索一下 X 上大家今天怎么评价 Grok CLI`
  Expected route: `grok-cli search --json --query ...`; the query text must remain unchanged.
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

- If auth is missing, invalid, expired, stale, or `bad-credentials`, run `grok-cli refresh --json` first, then retry the original task.
- If refresh fails because local auth state is missing, refresh cannot recover the session, or `relogin_required` is true, run `grok-cli login`, then retry the original task.
- If `entitlement_denied` is true, explain that this is an account or tier permission issue, not something a reinstall or relogin necessarily fixes.
- If `state_file_missing` appears, run login before retrying the original task.
- If a command returns invalid arguments, fix the command shape instead of asking the user to debug CLI syntax.

## Output To User

Default to lossless human-readable rendering: parse the JSON envelope, extract the command-specific primary fields, and display their values exactly as returned. Then briefly mention any recovery performed, such as installing `grok-cli`, refreshing credentials, or completing OAuth.

For text commands, copy `data.output_text`, `data.answer`, or `data.transcript` exactly. For media commands, report the exact returned file path, URL, request id, media tag, or generated media handle. Return raw JSON only when the user asks for raw JSON; when they do ask, return the raw JSON unchanged.
