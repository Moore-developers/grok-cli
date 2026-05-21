# Acceptance Examples

This document gives a small set of reproducible acceptance paths so we can quickly confirm whether the feature set is still intact during integration, delivery, or regression checks.

## 1. State And Authentication

### Acceptance 1: State Summary

```bash
cargo run -- state --json
```

Expected:

- `ok: true`
- `command: "state"`
- a redacted state summary or `exists: false`

### Acceptance 2: Browser Login

```bash
cargo run -- login --json
```

Expected:

- a real browser opens
- the login result is written to disk on success

### Acceptance 3: Refresh

```bash
cargo run -- refresh --json
```

Expected:

- the access token is refreshed
- `last_refresh` updates

## 2. Text Commands

### Acceptance 4: Chat

```bash
cargo run -- chat "Explain Grok CLI in one sentence" --json
```

Expected:

- `ok: true`
- `command: "chat"`
- text output is returned

### Acceptance 5: X Search

```bash
cargo run -- search "What are people saying about xAI on X today?" --json
```

Expected:

- `ok: true`
- `command: "search"`
- X search output is returned

## 3. Media And Audio

### Acceptance 6: Image Generation

```bash
cargo run -- image "A cinematic skyline at sunrise" --json
```

Expected:

- a returned image URL or local file path

### Acceptance 7: Image Editing

```bash
cargo run -- image-edit --image ./source.png --prompt "Make it cinematic" --json
```

Expected:

- edited image output is returned

### Acceptance 8: Video Generation

```bash
cargo run -- video "Animate a futuristic skyline" --duration 8 --json
```

Expected:

- a generated video URL is returned

### Acceptance 9: Video Editing

```bash
cargo run -- video-edit --video-url https://example.com/source.mp4 --prompt "Make it cinematic" --json
```

Expected:

- an edited video URL is returned

### Acceptance 10: Video Extension

```bash
cargo run -- video-extend --video-url https://example.com/source.mp4 --prompt "Continue the camera move" --duration 6 --json
```

Expected:

- an extended video URL is returned

### Acceptance 11: TTS

```bash
cargo run -- tts "Hello from Grok" --json
```

Expected:

- a local audio file path is returned

### Acceptance 12: STT

```bash
cargo run -- stt ./sample.wav --json
```

Expected:

- a transcript is returned

### Acceptance 13: Streaming STT

```bash
cargo run -- stt-stream ./sample.wav --json
```

Expected:

- a stream event list is returned

## 4. Usage

### Acceptance 14: Usage

```bash
cargo run -- usage --json
```

Expected:

- local usage data is returned
- account limits are not queried

## 5. Regression

### Acceptance 15: Re-run the main flow after login

Expected:

- after login, the original task can continue without asking the user to repeat it
