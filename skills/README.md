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

Install the skill by copying `skills/grok-cli` into the skills directory used by your agent runtime.

Common local layouts:

```bash
mkdir -p ~/.agents/skills
cp -R skills/grok-cli ~/.agents/skills/grok-cli
```

or:

```bash
mkdir -p ~/.codex/skills
cp -R skills/grok-cli ~/.codex/skills/grok-cli
```

After that, ask your agent to use Grok or xAI. The skill will install the CLI on first use if needed.
