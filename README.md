# grok-cli

OAuth-first command-line access to Grok / xAI capabilities.

[中文说明](README.zh-CN.md)

`grok-cli` gives local workflows, scripts, and agent skills a single CLI for:

- Browser OAuth login with token refresh
- Chat through Grok Responses
- X search through Grok `x_search`
- Image generation and image editing
- Video generation, editing, and extension
- Text-to-speech, batch speech-to-text, and experimental streaming speech-to-text
- Local session usage accounting in SQLite

The recommended public distribution path is SKILL-first. Cargo install works across platforms; macOS Apple Silicon users may also use a maintainer-uploaded GitHub Release tarball when available, while Windows users can download a GitHub Actions-built release binary. macOS Intel and Linux remain source-first through Cargo. A bundled skill at [`skills/grok-cli/SKILL.md`](skills/grok-cli/SKILL.md) can check whether `grok-cli` is installed, install it from GitHub with Cargo when needed, run OAuth login, and then execute the user's Grok task. See [`skills/README.md`](skills/README.md) for skill installation notes.

Text commands are optimized for both humans and automation:

- `chat` and `search` stream readable text by default for human use
- `--json` keeps stable non-stream output for scripts, skills, and automation
- `--stream` explicitly keeps formatted text streaming on
- `--raw-stream` exposes the raw normalized event stream when you need it

The public command surface is intentionally flat:

```text
grok-cli <login|status|refresh|logout|state|model|usage|chat|search|image|image-edit|video|video-edit|video-extend|tts|stt|stt-stream> ...
```

## Quick Start

Install directly from GitHub:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

macOS Apple Silicon and Windows users can also download covered release assets from [GitHub Releases](https://github.com/Moore-developers/grok-cli/releases/latest).

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

## Script Mode

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
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked
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
scripts/package-local-macos-release.sh v0.1.0 --upload
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
