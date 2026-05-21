# Internal Auth Recovery Entrypoints

This file records auth recovery capabilities that are not shown in public help or the README. Regular users should prefer:

```bash
grok-cli login
```

## Removed Public Entry

### `print-authorize-url`

`print-authorize-url` has been removed from the public CLI surface.

Why it was removed:

- It was mainly useful for early OAuth URL validation, not for day-to-day user work.
- `grok-cli login --manual-paste` already prints the authorize URL and writes the pending OAuth session correctly.
- A standalone command made the login flow look more complicated than it is.

If you need to inspect the authorize URL, use:

```bash
grok-cli login --manual-paste
```

## Hidden Recovery Entry

### `exchange-code`

`exchange-code` remains hidden for exceptional recovery and loopback / manual-paste flows.

Why it stays hidden:

- It is an implementation recovery path, not a normal user command.
- It should not appear in the public help surface.
- Public users should continue to think in terms of `login`, `status`, `refresh`, `logout`, and `state`.
