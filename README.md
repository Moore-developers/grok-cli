# grok-cli

[中文说明](README.zh-CN.md)

> Grok / xAI in a terminal-first, scriptable, and agent-ready CLI.

## Features

- **OAuth-only auth** — sign in with SuperGrok or X Premium+, no API key needed.
- **Flat command surface** — one CLI for chat, search, image, video, audio, and usage.
- **Streaming by default** — readable text for humans, `--json` for automation.
- **Media inputs** — local files and remote URLs for image, video, and audio.
- **Cross-platform** — pre-built for macOS Apple Silicon and Windows x64.

## Installation

```bash
# Agent runtime (recommended)
npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes

# From source (Rust 1.88+)
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked

# Release binary
# Download from GitHub Releases
```

Source installs require Rust 1.88+ (toolchain pinned to 1.92.0). On macOS Apple Silicon and Windows x64, the bundled skill prefers the release binary.

## Quick Start

```bash
# 1. Log in with your browser
grok-cli login

# 2. Verify the session
grok-cli status

# 3. Run your first task
grok-cli chat "Summarize the latest AI news"

# 4. Check usage
grok-cli usage
```

Headless environment? Use `grok-cli login --manual-paste` for a code-based login flow.

## Usage

### Chat & Search

```bash
# Stream a response (default)
grok-cli chat "What's new in AI?"
grok-cli search "Thoughts on Grok 3?"

# Non-streaming
grok-cli chat "Summarize AI news" --no-stream

# With explicit X search results
grok-cli chat "Latest xAI updates" --with-x-search

# Pure chat without web search
grok-cli chat "Hello" --no-web-search

# JSON for scripts
grok-cli search --json --query "Grok updates"
```

### Image & Video

```bash
grok-cli image "A cinematic skyline at sunrise"
grok-cli image-edit --image ./source.png --prompt "Make it cinematic"
grok-cli video "Animate a futuristic skyline" --duration 8
grok-cli video-edit --video-url https://example.com/source.mp4 --prompt "Make it cinematic"
grok-cli video-extend --video-url https://example.com/source.mp4 --prompt "Continue the camera move" --duration 6
```

### Audio

```bash
grok-cli tts "Hello from Grok"
grok-cli stt ./sample.wav
grok-cli stt-stream ./sample.wav --interim-results
```

### Model

```bash
# View current model
grok-cli model

# Set a specific model for chat and search
grok-cli model --model grok-4.3
```

## JSON Output

All commands accept `--json` for stable structured output. The envelope is consistent:

```json
{
  "ok": true,
  "command": "chat",
  "data": {}
}
```

On failure:

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

Designed for Codex, Claude Code, Cursor, and other agent runtimes. Install the bundled skill — it handles auth, command routing, and install checks automatically:

```bash
npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes
```

## Commands

| Command | Description |
| --- | --- |
| `login` | Start xAI OAuth login in the system browser |
| `status` | Check OAuth session status |
| `refresh` | Refresh the saved access token |
| `logout` | Delete local auth state |
| `chat` | Text chat with Grok (includes web search by default) |
| `search` | Search X via Grok `x_search` |
| `image` | Generate an image |
| `image-edit` | Edit reference images |
| `video` | Generate a video |
| `video-edit` | Edit a video |
| `video-extend` | Extend a video |
| `tts` | Text-to-speech |
| `stt` | Speech-to-text |
| `stt-stream` | Streaming STT over WebSocket (experimental) |
| `usage` | Show local session usage and rate-limit snapshots |
| `model` | Set the default text model for `chat` and `search` |
| `state` | Inspect redacted local auth state |

Use `--help` on any command for details.

## State

- **Auth tokens**: `~/.grok-cli/auth.json`
- **Usage history**: `~/.grok-cli/session.db` (SQLite)

Usage history tracks session totals, per-command events, media-type breakdowns, and rate-limit snapshots. Media files are not stored.

## Development

```bash
cargo test
cargo build --release
cargo install --path . --force
```

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## Documentation

- [Quickstart](docs/guides/quickstart.md)
- [Command reference](docs/commands/index.md)
- [Troubleshooting](docs/guides/troubleshooting.md)
- [Changelog](CHANGELOG.md)
