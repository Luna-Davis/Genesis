# 007 — Detect project from current directory

## Context
Users need a quick way to confirm which Genesis project they are in without mutating state. We already validate `.genesis/config.toml` to gate file operations.

## Decision
- Reuse the validator to check for `.genesis` in the current directory; return the path when present.
- Add a `genesis status` command that reads `.genesis/config.toml` and prints project id/name/language/location.
- Keep auto-acquisition of locks on shell entry out of scope; locking stays on explicit commands (start/resume/import).

## Consequences
- Users can confirm the active project context without side effects.
- Validator now returns the project path instead of a boolean, enabling richer uses later.
