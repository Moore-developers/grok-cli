# `grok-cli model`

## Purpose

Inspect and choose the shared default model for text commands. `chat` and `search` share the same default, so changing it once affects both.

`grok-cli mode` is an alias for `grok-cli model`.

## Common Usage

Interactive selection:

```bash
grok-cli model
grok-cli mode
```

In an interactive terminal, the command shows the following list and supports arrow-key navigation:

```text
grok-4.3
grok-4.20-reasoning
grok-4.20-0309-reasoning
exit
```

After selection, it prints:

```text
Model switched to <MODEL>.
```

Script or skill lookup:

```bash
grok-cli model --json
```

Script or skill direct selection:

```bash
grok-cli model --json --model grok-4.3
```

## Parameters

- `--json`: use the standard JSON envelope; this skips interactive selection.
- `--auth-file <PATH>`: override the OAuth state file path.
- `--model <MODEL>`: save the shared default model for `chat` and `search`.

## Behavior

- `grok-cli model` needs a readable auth state.
- It no longer exposes `show`, `list`, or `set` subcommands.
- Without `--model`, interactive terminals open the arrow-key selection UI; non-interactive environments print the current selection and model catalog.
- With `--model`, the shared text model is written to `auth.json` metadata.
- `--command` and `--task` are preserved only as hidden compatibility parameters. Public behavior always keeps `chat` and `search` on the same model.
- Media commands should pass `--model` directly on their own command line if they need a model override.

## JSON Fields

Lookup mode `data` contains:

- `provider`
- `selected_model`
- `selected`
- `catalog`

Set mode `data` contains:

- `provider`
- `model`
- `selected`
- `catalog`

## Related Docs

- [chat](./chat.md)
- [search](./search.md)
