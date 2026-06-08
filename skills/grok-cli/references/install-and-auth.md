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
8. Retry the original Grok task. If that task reports auth trouble, follow the failure-driven OAuth flow below.

Pinned public install:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.4 --locked --force
```

Latest repository install:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --locked --force
```

After install, rerun:

```bash
grok-cli --version
grok-cli --help
```

Then resume the user's original Grok task.

## Failure-Driven OAuth Flow

Do not run `status`, login checks, refresh checks, entitlement checks, or capability probes before routine user tasks. Run the user's real command first. If it succeeds, render the result. If it fails, recover from the actual error and retry the original command once.

Recovery order after a real command failure:

1. If the JSON error includes `error.recovery_action`, follow that single action instead of reinterpreting the other fields.
2. `refresh_then_retry`: run `grok-cli refresh --json`, then retry the original command once.
3. `login_then_retry`: run `grok-cli login`, then retry the original command once.
4. `wait_then_retry`: wait for `error.retry_after_seconds`, then retry the original command once.
5. `stop_billing`, `stop_quota`, `stop_rate_limit`, or `stop_entitlement`: explain the blocker and stop. Do not reinstall, refresh, or relogin for these blockers.
6. If the local binary is old and lacks `recovery_action`, fall back to message/code matching: refresh credential-validation failures, login relogin-required failures, and stop for pure billing, quota, rate-limit-without-window, or entitlement blockers.

Never replace the user's real command with a probe such as `grok-cli search --json --query "Grok"` or `grok-cli chat --json --prompt "Reply with exactly: ok"`. After recovery, retry the user's original command.

Public OAuth and state flags:

- `login`: `--json`, `--auth-file <PATH>`, `--no-browser`, `--manual-paste`, `--timeout <SECONDS>`, `--port <PORT>`.
- `status`: `--json`, `--auth-file <PATH>`.
- `refresh`: `--json`, `--auth-file <PATH>`.
- `state`: `--json`, `--auth-file <PATH>`.
- `logout`: `--json`, `--auth-file <PATH>`.

If a real command says auth is missing, invalid, credentials are stale or expired, `auth_expired`, `bad-credentials`, or `The OAuth2 access token could not be validated`, try refresh first:

```bash
grok-cli refresh --json
```

If refresh fails because local auth state is missing, refresh cannot recover the session, or relogin is required, run login:

```bash
grok-cli login
```

Do not treat credential errors as installation problems. Refresh or login, then retry the original command once.

Use manual login options when the environment cannot open or receive a browser callback:

```bash
grok-cli login --no-browser --manual-paste --timeout 300 --port 8787
```

Use `--auth-file <PATH>` only when the user explicitly wants an alternate local auth state, for example in isolated validation workspaces:

```bash
grok-cli status --json --auth-file ./tmp/auth.json
```

Use refresh when a real command or explicit status diagnostic says the session exists but the access token is stale or expiring:

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
