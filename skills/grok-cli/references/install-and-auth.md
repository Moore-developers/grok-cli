# Install And Auth

Use this reference when `grok-cli` is missing, has an incomplete command surface, needs upgrade, or needs OAuth setup.

## Install Check

Run:

```bash
command -v grok-cli
grok-cli --version
grok-cli --help
```

Treat the local CLI as incomplete if top-level help does not include all required commands:

```text
image-edit
video-edit
video-extend
stt-stream
```

This matters because older pre-release installs can report a valid version while still missing newer commands.

## Install Or Repair

If `grok-cli` is missing or incomplete, first decide whether the platform should use a release binary or a source build:

- macOS Apple Silicon: prefer `grok-cli-macos-aarch64-apple-darwin.tar.gz`
- Windows x64: prefer `grok-cli-windows-x86_64-pc-windows-msvc.zip`
- macOS Intel and Linux: use Cargo source install

Only check Cargo for source installs or when the user explicitly wants to build from source:

```bash
command -v cargo
```

For source installs, also check the compiler version:

```bash
rustc --version
```

`grok-cli` source installs require Rust 1.88 or newer because the crate uses edition 2024 and declares `rust-version = "1.88"`. CI and local development are pinned to Rust 1.92.0 via `rust-toolchain.toml`.

If Cargo is missing on macOS Intel or Linux, explain that Rust/Cargo is required and suggest installing Rust with `rustup`.

If `rustc` is older than 1.88, explain the requirement explicitly. Do not say only "upgrade Rust". Say that source install requires Rust 1.88+ and that the repository toolchain is currently 1.92.0, then suggest `rustup update` before retrying.

If the user's platform has a covered no-Rust release asset, point them to the GitHub Release page:

```text
https://github.com/Moore-developers/grok-cli/releases/latest
```

If a covered release asset is missing, do not silently switch the user to a source build on a product platform. Tell them the binary release is incomplete for that platform and only proceed with Cargo if they explicitly choose a developer source install.

Expected assets:

```text
grok-cli-macos-aarch64-apple-darwin.tar.gz
grok-cli-windows-x86_64-pc-windows-msvc.zip
```

Use the macOS tarball only for Apple Silicon (`arm64` / `aarch64`) Macs. Do not offer it for macOS Intel. Use the Windows zip only for Windows x64. Each covered asset should have a matching `.sha256` checksum file.

Release binary install flow:

1. Confirm the platform is covered by the exact asset name.
2. Download the asset and matching `.sha256`.
3. Verify the checksum when possible.
4. Extract the binary.
5. Put `grok-cli` or `grok-cli.exe` in a directory already on `PATH`; if none is suitable, use `~/.local/bin`, temporarily add it to the current shell `PATH`, and tell the user how to make that permanent.
6. If `command -v grok-cli` still fails but `~/.local/bin/grok-cli` exists and is executable, treat this as a PATH configuration issue, not a failed install. Continue by temporarily exporting `PATH="$HOME/.local/bin:$PATH"` or using `~/.local/bin/grok-cli` directly for verification.
7. Run `grok-cli --version` and `grok-cli --help`.
8. Run `grok-cli status --json`.
9. If status is not usable, complete OAuth handling before any real capability call.
10. Continue the original Grok task.

Pinned public install:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.1 --locked --force
```

Latest repository install:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked --force
```

After install, rerun:

```bash
grok-cli --version
grok-cli --help
grok-cli status --json
```

Then resume the user's original Grok task.

## OAuth Flow

Check status before real Grok calls:

```bash
grok-cli status --json
```

Readiness flow before the user's requested task:

1. Install or repair `grok-cli`.
2. Run `grok-cli --version` and `grok-cli --help`.
3. Run `grok-cli status --json` and inspect `logged_in`, `access_token_expiring`, `relogin_required`, and entitlement fields.
4. If auth is missing, invalid, or relogin is required, run `grok-cli login`, then `grok-cli status --json`.
5. If `access_token_expiring` is true, run `grok-cli refresh --json`, then `grok-cli status --json` before any capability probe.
6. Verify permission with a minimal real capability check before the user's command. For text tasks, use:

```bash
grok-cli chat --json --no-stream --prompt "Reply with exactly: ok" --timeout 120
```

For search tasks, use a real search probe:

```bash
grok-cli search --json --query "Grok" --timeout 120
```

A successful search readiness probe with sparse or empty citations still passes the capability gate; evaluate citation sufficiency only when summarizing the user's real search results.

7. If the permission check returns stale credentials such as `bad-credentials`, run `grok-cli refresh --json`, then `grok-cli status --json`, and retry the permission check once.
8. If the permission check returns `entitlement_denied` or `xai_oauth_tier_denied`, explain the account/tier blocker and stop before running the user's requested command.
9. Only run the user's original Grok command after login and permission are verified.

Public OAuth and state flags:

- `login`: `--json`, `--auth-file <PATH>`, `--no-browser`, `--manual-paste`, `--timeout <SECONDS>`, `--port <PORT>`.
- `status`: `--json`, `--auth-file <PATH>`.
- `refresh`: `--json`, `--auth-file <PATH>`.
- `state`: `--json`, `--auth-file <PATH>`.
- `logout`: `--json`, `--auth-file <PATH>`.

If auth is missing, invalid, or `relogin_required` is true:

```bash
grok-cli login
grok-cli status --json
```

If `access_token_expiring` is true, refresh before the first real capability call:

```bash
grok-cli refresh --json
grok-cli status --json
```

If a real command fails with a credential validation message such as `bad-credentials`, do not treat installation as broken. Refresh once, verify status, then retry the original command:

```bash
grok-cli refresh --json
grok-cli status --json
```

If the retry still fails, explain the returned auth or entitlement code. Ask for login only when the status or error says relogin is required.

Use manual login options when the environment cannot open or receive a browser callback:

```bash
grok-cli login --no-browser --manual-paste --timeout 300 --port 8787
```

Use `--auth-file <PATH>` only when the user explicitly wants an alternate local auth state, for example in isolated validation workspaces:

```bash
grok-cli status --json --auth-file ./tmp/auth.json
```

Use refresh when the session exists but the access token is expiring:

```bash
grok-cli refresh --json
```

Inspect redacted local state only when needed:

```bash
grok-cli state --json
```

`state --json` should be treated as sensitive-adjacent even though token values are redacted. Do not paste it to the user unless they asked for diagnostic details.

## Logout

Only run logout when the user asks to remove local auth:

```bash
grok-cli logout --json
```
