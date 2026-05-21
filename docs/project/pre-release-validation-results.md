# Pre-release Validation Results

This document records redacted pre-release validation results. Sensitive tokens, account identifiers, real media contents, and private URLs are not written to the repository. Temporary outputs stay under `.tmp/`.

## 2026-05-21 Phase 0: Release And Installation Loop

Status: passed, with 1 skill-install-check issue discovered and fixed.

### Validation Environment

- Platform: macOS, local validation.
- Rust / Cargo: Cargo available, version `1.92.0`.
- Install method: isolated `cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked`.
- Install source: GitHub tag `v0.1.0`, resolved to commit `d95b84a`.
- Install result: success.
- Build time: about 5 minutes 55 seconds, recorded only as an installation observation.
- Installed binary size: about `7.9M`.

### Verified Items

- `grok-cli --version`: passed, output `grok-cli 0.1.0`.
- `grok-cli --help`: passed; public commands include `login`, `status`, `refresh`, `logout`, `state`, `model`, `usage`, `chat`, `search`, `image`, `image-edit`, `video`, `video-edit`, `video-extend`, `tts`, `stt`, and `stt-stream`.
- `grok-cli status --json`: passed, JSON readable, current OAuth state is logged in.
- `skills/grok-cli` copied to the agent / Codex skill directory structure: passed, temporary directory validation confirmed that `SKILL.md` can be copied intact.
- `.tmp/` is ignored in `.gitignore`: passed.

### Issues Found

- The local globally installed `grok-cli 0.1.0` was an older install that lacked `image-edit`, `video-edit`, `video-extend`, and `stt-stream`.
- Because the version number was still `0.1.0`, the skill cannot rely on `grok-cli --version` alone. It must also check that the key commands exist; if they do not, it should instruct a reinstall with `cargo install --git ... --tag v0.1.0 --locked --force`.
