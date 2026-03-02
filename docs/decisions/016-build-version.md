# 016 — Build command and automatic version bump

## Context
Projects need a build command and version management. Rust builds should produce release binaries; Python bundling will be custom later. Versions must live in `.genesis/config.toml`.

## Decision
- Add `genesis build` (and shell `build`) that:
  - Runs `cargo build --release` for Rust projects.
  - For Python projects, returns a clear “custom bundler not implemented yet” error (reserved for future bespoke bundler).
- On successful build, bump the project version in `.genesis/config.toml` (default patch; `--bump minor|major` supported) using semantic version strings.
- Versions remain stored in `.genesis/config.toml`; DB schema is unchanged.
- TUI removed; shell is the interactive interface.

## Consequences
- Rust projects get a one-shot release build with automatic version bump.
- Python build path is explicit TODO, avoiding silent no-ops.
- Version stays the single source of truth in project config, enabling later packaging pipelines.
