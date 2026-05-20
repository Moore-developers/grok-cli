# Security Policy

## Supported Versions

Until the first public release, only the latest `master` branch is supported.

After tagged releases begin, security fixes will target the latest released minor version unless the maintainer announces otherwise.

## Reporting a Vulnerability

Please do not report security vulnerabilities in public GitHub issues.

Send a private report to the repository owner with:

- A concise description of the issue.
- Steps to reproduce or a proof of concept.
- Impact, including whether OAuth tokens, local auth state, or user files can be exposed.
- Affected versions or commit hashes when known.

If GitHub private vulnerability reporting is enabled for the repository, use that first. Otherwise, contact the maintainer through the GitHub profile for `Moore-developers`.

## Security-Sensitive Data

`grok-cli` stores OAuth state locally at `~/.grok-cli/auth.json` by default. Treat this file as sensitive.

Do not share:

- OAuth access tokens, refresh tokens, or ID tokens.
- Local auth state files.
- Session databases containing private usage history.
- Media files or transcripts that contain sensitive content.

## Expected Response

The maintainer will aim to acknowledge valid reports within 7 days and provide an update on fix timing after triage.
