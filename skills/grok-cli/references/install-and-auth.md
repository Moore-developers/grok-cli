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

This matters because a local binary can report `grok-cli 0.1.0` while still being installed before the public `v0.1.0` tag was finalized.

## Install Or Repair

If `grok-cli` is missing or incomplete, check Cargo:

```bash
command -v cargo
```

If Cargo is missing on macOS Intel or Linux, explain that Rust/Cargo is required and suggest installing Rust with `rustup`.

If Cargo is missing and the user's platform has a covered no-Rust release asset, point them to the GitHub Release page:

```text
https://github.com/Moore-developers/grok-cli/releases/latest
```

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
5. Put `grok-cli` or `grok-cli.exe` in a directory already on `PATH`; if none is suitable, use `~/.local/bin` and tell the user to add it to `PATH`.
6. Run `grok-cli --version` and `grok-cli --help`.
7. Continue the original Grok task.

Pinned public install:

```bash
cargo install --git https://github.com/Moore-developers/grok-cli.git --tag v0.1.0 --locked --force
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

## OAuth Flow

Check status before real Grok calls:

```bash
grok-cli status --json
```

Public OAuth and state flags:

- `login`: `--json`, `--auth-file <PATH>`, `--no-browser`, `--manual-paste`, `--timeout <SECONDS>`, `--port <PORT>`.
- `status`: `--json`, `--auth-file <PATH>`.
- `refresh`: `--json`, `--auth-file <PATH>`.
- `state`: `--json`, `--auth-file <PATH>`.
- `logout`: `--json`, `--auth-file <PATH>`.

If auth is missing, expired, invalid, or `relogin_required` is true:

```bash
grok-cli login
grok-cli status --json
```

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
