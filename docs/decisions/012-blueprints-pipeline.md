# 012 — Blueprints and push pipeline

## Context
Genesis needs reusable CI/CD templates (blueprints) and a `push` command that runs a strict pipeline (lint → build → test → deploy). Placeholders should resolve locally without network dependencies.

## Decision
- Store blueprints at `~/.local/share/genesis/blueprints/<name>.toml` with fields `{ name, scripts }`.
- Provide CLI subcommands:
  - `genesis blueprint new <name>`: create a stub blueprint with lint/build/test/deploy placeholders.
  - `genesis blueprint list`: list available blueprints.
  - `genesis blueprint apply <name>`: apply blueprint to current project by writing scripts into `.genesis/config.toml`, replacing `{{project_name}}` and `{{version}}`.
- Extend `.genesis/config.toml` to include `version` (default `0.1.0`) and `[scripts]` map.
- `genesis push`:
  - Requires scripts `lint`, `build`, `test`, `deploy` in project config.
  - Runs them in order via `sh -c`, showing stage headings; stops on first failure; reports halt stage; success prints completion.
- Script resolution is deterministic and offline; no remote templates.

## Consequences
- Reusable pipeline definitions that stay on disk and can be shared by name.
- Push enforces a consistent gate sequence and halts on first failure.
- Version/scripts now live in project config, enabling placeholder substitution and future pipeline evolution.
