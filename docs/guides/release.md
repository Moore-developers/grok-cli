# Release and Installation Guide

This document describes the current public release strategy for `grok-cli`.

## 1. Current Strategy

`grok-cli` is distributed SKILL-first.

- macOS (Intel and Apple Silicon) and Linux are source-first: users build or install with Cargo.
- Windows users can download a prebuilt GitHub Release binary.

Recommended user paths:

1. Use the bundled [`grok-cli` skill](../../skills/grok-cli/SKILL.md). See [skills README](../../skills/README.md) for installation notes. The skill checks whether the CLI is installed, installs it from GitHub with Cargo when needed, runs OAuth login, and resumes the user's original Grok task.
2. Install directly with Cargo on macOS or Linux:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

3. Install a tagged version from source on macOS or Linux:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked
```

4. Download the Windows GitHub Release binary:

   - Latest release page: [GitHub Releases](https://github.com/Moore-developers/grok-cli/releases/latest)
   - Asset: `grok-cli-windows-x86_64-pc-windows-msvc.zip`
   - Unzip and run `grok-cli.exe`

The project intentionally keeps macOS and Linux source-first so those platforms do not need prebuilt binary maintenance. Windows gets a prebuilt binary because it gives the clearest no-Rust install path and can be published from GitHub Actions with one dedicated build target.

## 2. Why This Split

Keeping only one release binary path avoids:

- macOS codesigning and notarization work.
- Linux libc / distro compatibility questions.
- Slow or flaky hosted release runners across multiple platforms.
- Releasing binaries that were not exercised on the maintainer's target machines.

Windows gets the binary because it removes the heaviest setup burden for the largest no-Cargo install path.

The tradeoff for macOS and Linux is that users need Rust/Cargo installed. The bundled skill is responsible for detecting that requirement and explaining it clearly.

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

This is the recommended path for macOS (Intel and Apple Silicon) and Linux users who want to build locally.

## 4. Windows Binary Install

If the user is on Windows and wants the prebuilt route:

1. Open the [latest GitHub Release](https://github.com/Moore-developers/grok-cli/releases/latest).
2. Download `grok-cli-windows-x86_64-pc-windows-msvc.zip`.
3. Unzip it and run `grok-cli.exe --version` and `grok-cli.exe --help`.

If the user prefers a source build on Windows and already has Rust/Cargo installed, `cargo install --git` still works.

## 5. Maintainer Release Checklist

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

GitHub Actions builds the Windows release asset from the tag and attaches it to the GitHub Release together with release notes.

## 6. Future Distribution Options

These are intentionally deferred:

- GitHub Release binaries for macOS and Linux
- Homebrew tap
- crates.io
- winget / Scoop
- Windows ARM64 release binary
- Additional installer formats

If demand appears from non-Rust users, add those channels later with explicit platform testing and release ownership.

## 7. User Verification

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
