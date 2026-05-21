# Skills

This directory contains agent skills that make `grok-cli` easier to use from an AI assistant or automation runtime.

## `grok-cli`

The [`grok-cli`](./grok-cli/SKILL.md) skill is the recommended user-facing entry point. It:

- Checks whether `grok-cli` is installed.
- Installs `grok-cli` from GitHub with Cargo when missing.
- Checks OAuth status.
- Runs `grok-cli login` when needed.
- Resumes the user's original Grok / xAI task.
- Uses JSON-mode CLI commands for reliable automation.

## Install The Skill

Install the skill with `npx skills`:

```bash
npx --yes skills add Moore-developers/grok-cli --skill grok-cli --global --yes
```

If you want to inspect or manage the installed skill later:

```bash
npx --yes skills list
npx --yes skills remove grok-cli --global --yes
```

After that, ask your agent to use Grok or xAI. The skill will install the CLI on first use if needed.
