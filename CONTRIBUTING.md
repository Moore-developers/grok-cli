# Contributing

Thanks for helping improve `grok-cli`. This project aims to keep the CLI predictable for humans, scripts, and agent skills, so contributions should preserve stable command behavior and JSON output contracts.

## Development Setup

Requirements:

- Rust toolchain from `rust-toolchain.toml`
- A C toolchain supported by Rust

Run the local checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
```

For a release build:

```bash
cargo build --release --locked
```

## Contribution Guidelines

- Keep public commands flat and script-friendly.
- Add or update tests for every new capability, parameter, validation rule, or output field.
- Preserve existing stable JSON fields such as `data.image`, `data.video`, `data.file_path`, and `data.transcript`.
- Update `docs/commands/*.md` for every user-facing CLI change.
- Avoid committing local auth state, media outputs, logs, or files under `.tmp/`.

## Pull Request Checklist

- The change has focused scope and a clear explanation.
- Relevant unit and command-level tests are included.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --locked` pass.
- Documentation and samples are updated when behavior changes.
- No secrets, OAuth tokens, or local state files are included.

## Security Issues

Please do not open public issues for suspected vulnerabilities. Follow [SECURITY.md](SECURITY.md) instead.
