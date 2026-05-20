# Release and Installation Guide

This document describes the current public release strategy for `grok-cli`.

## 1. Current Strategy

`grok-cli` is distributed source-first and SKILL-first.

Recommended user paths:

1. Use the bundled [`grok-cli` skill](../../skills/grok-cli/SKILL.md). See [skills README](../../skills/README.md) for installation notes. The skill checks whether the CLI is installed, installs it from GitHub with Cargo when needed, runs OAuth login, and resumes the user's original Grok task.
2. Install directly with Cargo:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

3. Install a tagged version:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked
```

The project intentionally does not publish prebuilt GitHub Release binaries for the first public version. Users build on their own machine through Cargo, which keeps the release process simple and avoids platform-specific binary maintenance.

## 2. Why Not Prebuilt Binaries Yet

Skipping prebuilt binaries avoids:

- macOS codesigning and notarization work.
- Windows MSVC and antivirus false-positive maintenance.
- Linux libc / distro compatibility questions.
- Slow or flaky hosted release runners.
- Releasing binaries that were not exercised on the maintainer's target machines.

The tradeoff is that users need Rust/Cargo installed. The bundled skill is responsible for detecting that requirement and explaining it clearly.

## 3. Build From Source

Requirements:

- Rust toolchain from `rust-toolchain.toml`
- A C toolchain supported by Rust

Build:

```bash
git clone https://github.com/Moore-developers/grok-cli.git
cd grok-cli
cargo build --release --locked
```

Binary path:

```text
target/release/grok-cli
```

Install into Cargo bin:

```bash
cargo install --path . --force --locked
```

Verify:

```bash
grok-cli --version
grok-cli --help
```

## 4. Maintainer Release Checklist

Before tagging:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
grok-cli --help
grok-cli usage --help
```

Update:

- `Cargo.toml` version
- `CHANGELOG.md`
- `README.md`
- `README.zh-CN.md`
- `docs/guides/release.md`
- `skills/grok-cli/SKILL.md` when installation or command routing changes

Create and push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Create a GitHub Release from the tag with release notes only. Do not upload prebuilt binaries unless the release strategy changes.

## 5. Future Distribution Options

These are intentionally deferred:

- GitHub Release binaries for macOS, Linux, and Windows
- Homebrew tap
- crates.io
- winget / Scoop

If demand appears from non-Rust users, add those channels later with explicit platform testing and release ownership.

## 6. User Verification

After installation, users should verify:

```bash
grok-cli --version
grok-cli --help
grok-cli status --json
```

Then start login:

```bash
grok-cli login
```
