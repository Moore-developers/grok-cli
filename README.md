# grok-cli

> Grok / xAI in a terminal-first, scriptable, and agent-ready CLI.

## Overview

`grok-cli` brings Grok / xAI into terminal-first, scriptable, and agent-driven workflows. It supports SuperGrok or X Premium+ through direct OAuth login, so you do not need a separate API key or standalone billing setup.

It gives you one CLI for login, chat, search, media, audio, and usage tracking. It also keeps authentication, automation output, local files, remote URLs, and platform-specific installs in one place.

OpenClaw and Hermes Agent cover the officially supported integration paths; `grok-cli` is for Codex, Claude Code, Cursor, custom automation, agent runtimes, skills, scripts, CI jobs, and validation flows.

## Highlights

- Direct OAuth login with SuperGrok or X Premium+.
- Flat command surface for login, chat, search, media, audio, state, model, and usage.
- Human-readable streaming by default, with `--json` and `--raw-stream` for automation.
- Local file inputs and remote URLs for image, video, and audio workflows.
- Skill-ready for Codex, Claude Code, Cursor, and other agent runtimes.
- Release builds for macOS Apple Silicon and Windows x64.

## Quick Install

Pick the path that matches how you want to use `grok-cli`:

| Need | Best path | Example |
| --- | --- | --- |
| Use it in Codex, Claude Code, Cursor, or another agent runtime | Skill | `npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes` |
| Build from source | Cargo | `cargo install --git https://github.com/Moore-developers/grok-cli.git --locked` |
| Skip Rust and use a prebuilt binary | Release binary | Download from [GitHub Releases](https://github.com/Moore-developers/grok-cli/releases/latest) |

If you are not sure, start with Skill for agent workflows or Cargo for source builds.

Text commands are optimized for both humans and automation:

- `chat` and `search` stream readable text by default for human use
- `--json` keeps stable non-stream output for scripts, skills, and automation
- `--stream` explicitly keeps formatted text streaming on
- `--raw-stream` exposes the raw normalized event stream when you need it

The public command surface is intentionally flat:

```text
grok-cli <login|status|refresh|logout|state|model|usage|chat|search|image|image-edit|video|video-edit|video-extend|tts|stt|stt-stream> ...
```

## For Humans

Use `grok-cli` directly when you want a reliable command instead of a live browser session.

Log in with the browser:

```bash
grok-cli login
```

Check the saved session:

```bash
grok-cli status
```

Ask Grok:

```bash
grok-cli chat "Summarize the latest AI news"
```

Search X:

```bash
grok-cli search "What are builders saying about Grok today?"
```

Generate media:

```bash
grok-cli image "A cinematic skyline at sunrise"
grok-cli image-edit --image ./source.png --prompt "Make it cinematic"
grok-cli video "Animate a futuristic skyline" --duration 8
grok-cli video-edit --video-url https://example.com/source.mp4 --prompt "Make it cinematic"
grok-cli video-extend --video-url https://example.com/source.mp4 --prompt "Continue the camera move" --duration 6
grok-cli tts "Hello from Grok"
grok-cli stt ./sample.wav
grok-cli stt-stream ./sample.wav --interim-results
```

Show local usage:

```bash
grok-cli usage
```

## For Scripts

Human-friendly commands use positional arguments by default. Scripts can keep using explicit flags and JSON output:

```bash
grok-cli chat --json --prompt "Summarize today's AI news"
grok-cli search --json --query "Grok Hermes latest updates"
grok-cli image --json --prompt "A cinematic skyline"
grok-cli image-edit --json --image ./source.png --prompt "Make it cinematic"
grok-cli tts --json --text "Hello from Grok"
grok-cli stt --json --file ./sample.wav
grok-cli stt-stream --json --file ./sample.wav
grok-cli usage --json
```

If you want a single final human-readable response instead of streaming, add `--no-stream`:

```bash
grok-cli chat "Summarize today's AI news" --no-stream
grok-cli search "What are builders saying about Grok today?" --no-stream
```

Successful JSON output uses a stable envelope:

```json
{
  "ok": true,
  "command": "chat",
  "data": {}
}
```

Failed JSON output uses the same shape:

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

## For AI Agents

`grok-cli` is designed for Codex, Claude Code, Cursor, custom automation, agent runtimes, skills, scripts, CI jobs, and validation flows. OpenClaw and Hermes Agent cover the officially supported integration paths.

Install the bundled skill:

```bash
npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes
```

Use the skill when you want the assistant to handle install checks, OAuth login, and command routing for you.

## Core Concepts

| Concept | What it means |
| --- | --- |
| Flat command surface | One CLI entrypoint covers login, chat, search, media, audio, model, state, and usage. |
| Streaming defaults | `chat` and `search` stream readable text by default for humans. |
| Script mode | `--json` keeps output stable for automation; `--no-stream` and `--raw-stream` refine the output mode. |
| Local files | Image, video, and audio commands accept local paths where the upstream flow supports them. |
| Local state | OAuth tokens live in `auth.json`; usage history lives in SQLite. |

## Commands

- `login`: start xAI OAuth login in the system browser.
- `status`: show whether a usable OAuth session exists.
- `refresh`: refresh the saved access token.
- `logout`: delete local auth state.
- `chat`: run text chat through Grok Responses. By default this includes web search.
- `search`: run X search through Grok `x_search`.
- `image`: generate an image with Grok Imagine.
- `image-edit`: edit one or more reference images with Grok Imagine.
- `video`: generate a video with Grok Imagine.
- `video-edit`: edit an existing video with Grok Imagine.
- `video-extend`: extend an existing video with Grok Imagine.
- `tts`: convert text to speech.
- `stt`: transcribe speech to text.
- `stt-stream`: stream speech to text over WebSocket. This is an experimental entry point.
- `usage`: show local session usage and rate-limit snapshots.
- `model`: configure the shared default text model for `chat` and `search`.
- `state`: inspect the redacted local auth state.

Use `--help` on any command:

```bash
grok-cli chat --help
grok-cli usage --help
```

## State Files

Default paths:

- OAuth state: `~/.grok-cli/auth.json`
- Session usage database: `~/.grok-cli/session.db`

OAuth tokens are stored in `auth.json`. Usage history is stored in SQLite and includes session totals, per-command events, text/image/video/audio breakdowns, and recent rate-limit snapshots.

Media file bodies are not stored in SQLite.

## Installation

From source:

```bash
git clone https://github.com/Moore-developers/grok-cli.git
cd grok-cli
cargo install --path .
```

From GitHub after the repository is public:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

From a tag:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.1 --locked
```

Covered release assets:

- macOS Apple Silicon: `grok-cli-macos-aarch64-apple-darwin.tar.gz`
- Windows x64: `grok-cli-windows-x86_64-pc-windows-msvc.zip`

Each release asset should have a matching `.sha256` checksum file. Prebuilt binaries are intentionally targeted rather than a full platform matrix. The recommended path is `cargo install --git`, or using the bundled [`grok-cli` skill](skills/grok-cli/SKILL.md) to perform installation and command execution automatically.

## Development

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request, and use [SECURITY.md](SECURITY.md) for private vulnerability reports.

Run tests:

```bash
cargo test
```

Build a release binary:

```bash
cargo build --release
```

Package and upload a local macOS Apple Silicon release asset:

```bash
scripts/package-local-macos-release.sh v0.1.1 --upload
```

Install the local release binary:

```bash
cargo install --path . --force
```

## Documentation

- [中文说明](README.zh-CN.md)
- [Documentation index](docs/index.md)
- [Quickstart](docs/guides/quickstart.md)
- [Command reference](docs/commands/index.md)
- [Usage command spec](docs/reference/usage-command-spec.md)
- [Release and installation guide](docs/guides/release.md)
- [Troubleshooting](docs/guides/troubleshooting.md)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
