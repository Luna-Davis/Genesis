# 014 — README scope and contents

## Context
The project needs a GitHub-ready README summarizing install, commands, safety, and key flows while remaining concise.

## Decision
- README now includes:
  - Install instructions (`cargo install --path .`).
  - Quick start for start/run/test/git-commit/push/stop.
  - Import flow (in-place init).
  - Blueprint usage (new/list/apply).
  - TUI entrypoint.
  - Command reference (one-line behaviors).
  - Safety guarantees (lock, delete validation, non-destructive import, canonical paths).
  - Project layout overview.
  - Contributing/testing note (tests pending; network caveat).
  - License.

## Consequences
- Sets expectations for users landing on GitHub without needing deeper docs.
- Highlights safety and offline guarantees.
- Points contributors to TODOs in docs/TASKS.md.
