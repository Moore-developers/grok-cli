# Release and Installation Guide

This document describes practical ways to publish and install `grok-cli` from GitHub.

## 1. Current Release Strategy

Recommended first public strategy:

1. Keep source install available with `cargo install --git`.
2. Use GitHub Releases for prebuilt macOS and Linux binaries.
3. Keep `publish = false` in `Cargo.toml`; do not publish to crates.io yet.
4. Add Homebrew only after release names and upgrade cadence settle.

The repository includes:

- `.github/workflows/ci.yml`: runs formatting, clippy, and tests on pushes and pull requests.
- `.github/workflows/release.yml`: builds release archives when a `v*.*.*` tag is pushed.
- `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, issue templates, and a PR template.

## 2. Release Options

### Option A: Source-first release

Best for early releases.

Users install through Cargo:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked
```

Tagged install:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked
```

Pros:

- Very easy to publish
- No binary hosting work
- Works well for Rust users

Cons:

- Users need Rust installed
- First install compiles dependencies locally

### Option B: GitHub Release binaries

Best default for normal CLI users.

The current release workflow builds these assets:

```text
grok-cli-aarch64-apple-darwin.tar.gz
grok-cli-x86_64-apple-darwin.tar.gz
grok-cli-x86_64-unknown-linux-gnu.tar.gz
checksums.txt
```

Users install with:

```bash
curl -L https://github.com/Moore-developers/grok-cli/releases/download/v0.1.0/grok-cli-aarch64-apple-darwin.tar.gz -o grok-cli.tar.gz
tar -xzf grok-cli.tar.gz
chmod +x grok-cli
sudo mv grok-cli /usr/local/bin/grok-cli
```

Pros:

- Users do not need Rust
- Fast install
- Familiar GitHub CLI distribution model

Cons:

- Maintainers must build and upload per-platform binaries
- Codesigning/notarization may matter for polished macOS distribution

### Option C: Homebrew tap

Best for macOS users after the project stabilizes.

User-facing install:

```bash
brew tap Moore-developers/grok-cli
brew install grok-cli
```

Typical formula shape:

```ruby
class GrokCli < Formula
  desc "OAuth-first CLI for Grok and xAI capabilities"
  homepage "https://github.com/Moore-developers/grok-cli"
  url "https://github.com/Moore-developers/grok-cli/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "<sha256>"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/grok-cli", "--help"
  end
end
```

Pros:

- Very natural for macOS users
- Supports upgrades with `brew upgrade`

Cons:

- Requires a tap repository or upstream Homebrew formula
- Formula checksums must be maintained

### Option D: crates.io

Not currently enabled because `Cargo.toml` has:

```toml
publish = false
```

To publish to crates.io later:

1. Remove or change `publish = false`
2. Add complete package metadata
3. Confirm name availability
4. Run:

```bash
cargo publish --dry-run
cargo publish
```

User install:

```bash
cargo install grok-cli --locked
```

Pros:

- Standard Rust install path
- Versioned package registry

Cons:

- Public package name and release metadata need more care
- Still requires Rust on user machines

## 3. Build From Source

Requirements:

- Rust `1.88` or newer
- A C toolchain supported by Rust

Build:

```bash
git clone https://github.com/Moore-developers/grok-cli.git
cd grok-cli
cargo build --release
```

Binary path:

```text
target/release/grok-cli
```

Install into Cargo bin:

```bash
cargo install --path . --force
```

Verify:

```bash
grok-cli --help
```

## 4. Maintainer Release Checklist

Before tagging:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --release --locked
grok-cli --help
grok-cli usage --help
```

Update:

- `Cargo.toml` version
- `CHANGELOG.md`
- `README.md`
- `docs/commands/index.md`
- `docs/guides/release.md`

Create and push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The `Release` workflow will create the GitHub Release and upload archives plus `checksums.txt`.

Manual fallback:

```bash
rm -rf dist
mkdir -p dist
cargo build --release --locked
cp target/release/grok-cli dist/grok-cli
tar -C dist -czf grok-cli-aarch64-apple-darwin.tar.gz grok-cli
shasum -a 256 grok-cli-aarch64-apple-darwin.tar.gz > checksums.txt
```

Upload the archive and `checksums.txt` to GitHub Releases.

## 5. Versioning And Branches

- Release tags use `vMAJOR.MINOR.PATCH`, for example `v0.1.0`.
- `master` is the default development branch.
- `Cargo.toml` currently has `publish = false`; crates.io is intentionally deferred.
- Repository URLs currently use `Moore-developers/grok-cli`. The GitHub username `Moore` is already occupied by another account, so do not change repository metadata to `Moore/grok-cli` unless GitHub owner migration becomes possible.

## 6. User Verification

After installation, users should verify:

```bash
grok-cli --version
grok-cli --help
grok-cli status --json
```

Then start login:

```bash
grok-cli login --json
```
