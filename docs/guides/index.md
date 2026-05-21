# Guides Index

This directory contains the most common how-to guides for users and release maintainers. These docs focus on workflows rather than exhaustive parameter specs.

## Reading Order

1. [Quickstart](./quickstart.md): install, log in, and run the first real commands.
2. [Troubleshooting](./troubleshooting.md): diagnose auth, entitlement, media, and streaming issues.
3. [Release and Installation Guide](./release.md): SKILL-first distribution, macOS Apple Silicon local release uploads, macOS Intel/Linux source-first installs, Windows GitHub Release binaries, Cargo installs, tags, and future distribution options.

## Doc Responsibilities

### [Quickstart](./quickstart.md)

For first-time `grok-cli` users. It keeps the shortest useful path:

- Install or build the CLI.
- Check auth status.
- Start browser login.
- Run chat, search, media, audio, and usage commands.
- Jump to the full command reference.

### [Troubleshooting](./troubleshooting.md)

For quick diagnosis when a command fails. It currently covers:

- `state_file_missing`
- `auth_relogin_required`
- `xai_oauth_tier_denied`
- Browser authorization succeeds but the CLI later fails.
- `stt` file and parameter issues.
- `chat` / `search` streaming issues.

### [Release and Installation Guide](./release.md)

For publishing the project or helping users install it. It covers:

- macOS Apple Silicon release tarballs or source installs.
- macOS Intel and Linux source installs.
- Windows GitHub Release binaries built by GitHub Actions.
- The bundled `grok-cli` skill as the preferred agent-facing entry point.
- Why prebuilt binaries are limited to platforms the maintainer can own.
- Deferred channels such as Homebrew, crates.io, winget, and Scoop.
- Pre-release checklists.

## Related Entrypoints

- [CLI Command Index](../commands/index.md)
- [Reference Index](../reference/index.md)
- [Project Index](../project/index.md)
- [Documentation Index](../index.md)
