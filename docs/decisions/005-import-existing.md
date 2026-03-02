# 005 — Import existing projects in place

## Context
Users need to bring non-Genesis projects under management without creating a new folder, similar to `cargo init` or `uv init`. Import must be non-destructive, reuse existing structure, and register the project in the database with consistent paths.

## Decision
- Add `genesis import` that operates on the current directory.
- Determine project name from the directory name; language is either provided (`--language`) or auto-detected (Rust: `Cargo.toml`; Python: `pyproject.toml`/`requirements.txt`/`setup.py`). Ambiguity requires an explicit flag.
- If `.genesis/config.toml` exists, trust it as the source of truth; refuse when its name differs from the directory unless `--force`. Register in DB if missing.
- If `.genesis` is absent, create it with a new UUID and canonical path, and `git init` if no repo exists. Do not modify other project files.

## Consequences
- Import is safe by default, never overwriting existing files beyond `.genesis`.
- DB and config stay aligned via canonical paths.
- Ambiguous or mismatched states are surfaced early, with an explicit `--force` escape hatch.
