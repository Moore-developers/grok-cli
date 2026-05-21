# Quickstart

## 1. Install Or Build

If you are using an agent runtime such as Codex, Claude Code, Cursor, or another skill-aware workflow, start with the bundled skill:

```bash
npx --yes skills add https://github.com/Moore-developers/grok-cli --skill grok-cli --global --yes
```

The skill can check whether `grok-cli` is installed, install it when possible, handle OAuth login, and then resume the original Grok task.

For source installs:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

Source installs require Rust 1.88+.

For local development from a checkout:

```bash
cargo test
```

If the tests pass, the local build, auth state handling, task execution, and usage paths are ready for development.

## 2. Check Auth Status

Installed binary:

```bash
grok-cli status --json
```

Local development checkout:

```bash
cargo run -- status --json
```

If no auth state exists yet, a typical response contains:

```json
{
  "ok": false,
  "command": "status",
  "error": {
    "code": "state_file_missing",
    "message": "state file not found: ...",
    "relogin_required": false,
    "entitlement_denied": false
  }
}
```

## 3. Start Browser Login

```bash
grok-cli login
```

For scripts, skills, or validation:

```bash
grok-cli login --json
```

If the environment cannot open a browser or receive a callback, use manual paste mode:

```bash
grok-cli login --json --manual-paste
```

## 4. Run The First Real Tasks

### Optional: choose the shared text model

```bash
grok-cli model --json
grok-cli model --json --model grok-4.3
```

Notes:

- `model` manages the shared default text model for `chat` and `search`.
- In an interactive terminal, `grok-cli model` opens a keyboard selection prompt.
- Image, video, TTS, STT, and streaming STT models are set directly on their commands with `--model`.

### Chat

```bash
grok-cli chat "Introduce Grok in one sentence"
grok-cli chat "Summarize recent AI news" --with-x-search
```

Notes:

- `chat` enables general `web_search` by default.
- `chat` streams readable text by default for humans.
- Use `--no-web-search` for pure chat.
- Use `--no-stream` for a single final text response.
- Use `--with-x-search` to add X search alongside web search.

### X Search

```bash
grok-cli search "What are people saying about xAI on X today?"
```

Use `--no-stream` when you want one final text response instead of readable streaming.

### Image Generation

```bash
grok-cli image "A cinematic skyline at sunrise"
```

### Image Editing

```bash
grok-cli image-edit --image ./source.png --prompt "Make it cinematic"
```

### Video Generation

```bash
grok-cli video "Animate a futuristic skyline" --duration 8
```

### Video Editing

```bash
grok-cli video-edit --video-url https://example.com/source.mp4 --prompt "Make it cinematic"
```

### Video Extension

```bash
grok-cli video-extend --video-url https://example.com/source.mp4 --prompt "Continue the camera move" --duration 6
```

`video-extend` currently requires a remote video URL. Local video paths are intentionally not exposed for this command because real validation reached the upstream service but ended in an xAI internal error.

### Text-To-Speech

```bash
grok-cli tts "Hello from Grok"
```

### Speech-To-Text

```bash
grok-cli stt ./sample.wav
```

### Streaming Speech-To-Text

```bash
grok-cli stt-stream ./sample.wav --interim-results
```

Notes:

- Media and audio commands check whether the access token is close to expiry before sending requests.
- If needed, commands refresh first and then continue the real request.
- `stt-stream` is experimental and exists for WebSocket STT validation.

## 5. Inspect Usage

```bash
grok-cli usage
```

`usage` reports local session usage. It does not query or display account limits.

## 6. Continue Reading

- Command details: [CLI Command Index](../commands/index.md)
- Automation contract: [SKILL integration contract](../reference/skill-integration.md)
- JSON examples: [Sample state and outputs](../reference/samples.md)
- Common errors: [Troubleshooting](./troubleshooting.md)
