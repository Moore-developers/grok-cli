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

If Cargo is missing, explain that Rust/Cargo is required and suggest installing Rust with `rustup`. Do not invent a prebuilt binary path.

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

If auth is missing, expired, invalid, or `relogin_required` is true:

```bash
grok-cli login
grok-cli status --json
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
