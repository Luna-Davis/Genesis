# 015 — Testing scope choices

## Context
Adding tests must avoid damaging user data and should run optionally when they touch external tools (cargo/git) or user directories.

## Decision
- Unit tests cover:
  - Path resolution and deletion guard to ensure `.genesis` id matches DB.
  - Git commit message rules.
  - Run script helper success/failure.
- Integration tests:
  - Start/list and import flows using temp directories via the CLI.
  - Marked `#[ignore]` to avoid running by default because they require toolchains and could touch user data_dir. Run with `cargo test -- --ignored`.

## Consequences
- Provides coverage for core logic while keeping default test runs safe.
- Developers can explicitly opt into end-to-end tests when the environment permits.
