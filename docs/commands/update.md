# `grok-cli update`

## Purpose

Check the latest GitHub Release and update the local `grok-cli` binary when a newer version is available.

The CLI also performs a low-frequency passive update check after human-readable commands. Passive notices are written to stderr and are skipped for `--json`, `--raw-stream`, non-interactive output, and the `update` command itself.

Update settings are stored in:

```text
~/.grok-cli/update.json
```

## Common Usage

Check without installing:

```bash
grok-cli update --check
```

Check and install when a newer release exists:

```bash
grok-cli update
```

Disable passive update notices:

```bash
grok-cli update --no-update-check
```

Re-enable passive update notices:

```bash
grok-cli update --enable-update-check
```

Script or skill usage:

```bash
grok-cli update --check --json
```

Temporarily disable passive checks for a single command:

```bash
GROK_CLI_NO_UPDATE_CHECK=1 grok-cli chat "Hello"
```

## Parameters

- `--json`: use the standard JSON envelope.
- `--check`: only check the latest release; do not install anything.
- `--force`: reinstall the latest release even when the current version already matches it.
- `--no-update-check`: persistently disable passive background update notices.
- `--enable-update-check`: persistently re-enable passive background update notices.

`--check`, `--force`, `--no-update-check`, and `--enable-update-check` are mutually exclusive.

## Install Strategy

The active update command follows the same distribution policy as installation:

- macOS Apple Silicon downloads `grok-cli-macos-aarch64-apple-darwin.tar.gz` and verifies the matching `.sha256`.
- Windows x64 downloads `grok-cli-windows-x86_64-pc-windows-msvc.zip`, verifies the matching `.sha256`, and starts a PowerShell updater because Windows cannot reliably overwrite a running executable.
- macOS Intel, Linux, and other source-first platforms run `cargo install --git https://github.com/Moore-developers/grok-cli.git --tag <LATEST_TAG> --locked --force`.

If the selected release asset is missing or checksum verification fails, the command exits with a structured error and does not silently switch install strategies.

## JSON Fields

`update --check --json` returns:

- `current_version`
- `latest_version`
- `latest_tag`
- `update_available`
- `release_url`
- `install_strategy`
- `asset_name`

`update --json` also returns:

- `installed`
- `message`

`update --no-update-check --json` and `update --enable-update-check --json` return:

- `auto_check_enabled`
- `update_config_path`

## Related Docs

- [Release and Installation Guide](../guides/release.md)
- [Troubleshooting](../guides/troubleshooting.md)
