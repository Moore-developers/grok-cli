# Reference Index

This directory contains stable contracts, output samples, and internal design notes. These docs are more about what the system is and why it behaves this way than about the fastest way for a new user to get started.

## Document List

1. [Sample state and outputs](./samples.md)
2. [Internal auth recovery entrypoints](./internal-auth.md)
3. [`usage` command spec](./usage-command-spec.md)
4. [SKILL integration contract](./skill-integration.md)
5. [Extension points](./extension-points.md)

## Document Responsibilities

### [Sample state and outputs](./samples.md)

This file keeps JSON and human-readable output examples together so scripts, skills, and readers can compare fields directly.

### [Internal auth recovery entrypoints](./internal-auth.md)

This file records auth recovery capabilities that are not shown in public help or the README:

- `print-authorize-url` was removed from the public surface.
- `exchange-code` remains hidden for exceptional recovery flows.
- Regular users should prefer `grok-cli login`.

### [`usage` command spec](./usage-command-spec.md)

This file captures the deeper design for `usage`:

- local session accounting
- recent rate-limit snapshots
- text/image/video/audio breakdowns
- why it is a top-level command

### [SKILL integration contract](./skill-integration.md)

This file describes how the bundled skill should call `grok-cli`:

- prefer `--json`
- rely on stable output contracts
- restore the original task after login or installation
- do not guess at auth state

### [Extension points](./extension-points.md)

This file records the stable boundaries that future features should reuse instead of reinventing.
