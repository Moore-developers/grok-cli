# Release and Installation Guide

This document describes the current public release strategy for `grok-cli`.

## 1. Current Strategy

`grok-cli` is distributed SKILL-first.

- macOS Apple Silicon users should prefer the maintainer-uploaded GitHub Release tarball.
- macOS Intel and Linux users are source-first: users build or install with Cargo.
- Windows x64 users should prefer the GitHub Actions-built GitHub Release binary.

Recommended user paths:

1. Use the bundled [`grok-cli` skill](../../skills/grok-cli/SKILL.md). Install it with `npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes`. See [skills README](../../skills/README.md) for setup notes. The skill checks whether the CLI is installed, prefers the GitHub Release binary on macOS Apple Silicon and Windows x64, falls back to Cargo on source-first platforms, runs OAuth login, and resumes the user's original Grok task.
2. Install directly with Cargo on macOS, Linux, or Windows when you want a source build:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

Source installs require Rust 1.88 or newer because the crate uses edition 2024 and declares `rust-version = "1.88"`. The repository toolchain is pinned to Rust 1.92.0 in `rust-toolchain.toml`.

3. Install a tagged version from source on macOS, Linux, or Windows:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.3 --locked
```

4. Download a GitHub Release binary when the platform is covered:

   - Latest release page: [GitHub Releases](https://github.com/Moore-developers/grok-cli/releases/latest)
   - macOS Apple Silicon asset: `grok-cli-macos-aarch64-apple-darwin.tar.gz`
   - Asset: `grok-cli-windows-x86_64-pc-windows-msvc.zip`
   - Checksum files use the same asset name with `.sha256`
   - macOS: extract the tarball and run `grok-cli`
   - Windows: unzip and run `grok-cli.exe`

The project intentionally keeps hosted CI release builds narrow. Windows gets a GitHub Actions binary because the maintainer cannot build it locally on macOS. macOS Apple Silicon can be built locally by the maintainer, then uploaded as a Release asset. macOS Intel and Linux remain source-first until there is enough demand to justify dedicated release ownership.

## 2. Why This Split

Keeping only the platforms we can own avoids:

- macOS universal binary, codesigning, and notarization work.
- Linux libc / distro compatibility questions.
- Slow or flaky hosted release runners across every platform.
- Releasing binaries that were not exercised on the maintainer's target machines.

Windows gets a CI binary because it removes the heaviest setup burden for the largest no-Cargo install path. macOS Apple Silicon gets a local maintainer-built tarball because it is fast to build and directly test on the maintainer machine.

The tradeoff for macOS Intel and Linux is that users need Rust/Cargo installed. The bundled skill is responsible for detecting that requirement and explaining it clearly.

## 3. Build From Source

Requirements:

- Rust 1.88+ minimum for source install
- Rust toolchain from `rust-toolchain.toml` for repository development and CI parity
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

This is the recommended path for macOS Intel, Linux, and any user who prefers building locally.

## 4. Release Binary Install

If the user is on macOS Apple Silicon and a local maintainer-built asset is attached:

1. Open the [latest GitHub Release](https://github.com/Moore-developers/grok-cli/releases/latest).
2. Download `grok-cli-macos-aarch64-apple-darwin.tar.gz`.
3. Optionally verify the matching `.sha256` file.
4. Extract it and run `grok-cli --version` and `grok-cli --help`.

If the user is on Windows and wants the prebuilt route:

1. Open the [latest GitHub Release](https://github.com/Moore-developers/grok-cli/releases/latest).
2. Download `grok-cli-windows-x86_64-pc-windows-msvc.zip`.
3. Optionally verify the matching `.sha256` file.
4. Unzip it and run `grok-cli.exe --version` and `grok-cli.exe --help`.

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
git tag v0.1.3
git push origin v0.1.3
```

GitHub Actions builds the Windows release asset from the tag and attaches it to the GitHub Release with its checksum:

```text
grok-cli-windows-x86_64-pc-windows-msvc.zip
grok-cli-windows-x86_64-pc-windows-msvc.zip.sha256
```

After the GitHub Release exists, package and upload the local macOS Apple Silicon asset from the same tagged commit:

```bash
scripts/package-local-macos-release.sh v0.1.3 --upload
```

This uploads:

```text
grok-cli-macos-aarch64-apple-darwin.tar.gz
grok-cli-macos-aarch64-apple-darwin.tar.gz.sha256
```

If `--upload` fails because the release does not exist yet, wait for the Windows workflow to finish or create the release first, then rerun the same command. The script uses `--clobber`, so rerunning it replaces the same macOS assets.

Final release assets to verify:

- `grok-cli-macos-aarch64-apple-darwin.tar.gz`
- `grok-cli-macos-aarch64-apple-darwin.tar.gz.sha256`
- `grok-cli-windows-x86_64-pc-windows-msvc.zip`
- `grok-cli-windows-x86_64-pc-windows-msvc.zip.sha256`

## 6. Future Distribution Options

These are intentionally deferred:

- macOS Intel GitHub Release binary
- Linux GitHub Release binaries
- macOS universal binary, codesigning, and notarization
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
