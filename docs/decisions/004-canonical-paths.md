# 004 — Canonical project paths for Genesis records

## Context
`genesis start` previously stored the parent working directory in the DB while `.genesis/config.toml` stored a canonicalized project path. This mismatch risked deleting or resuming the wrong directory, especially after moving a project.

## Decision
- Compute the project root as `cwd / <project-name>` and canonicalize it once.
- Pass that canonical path into both the DB record (`location`) and `.genesis/config.toml`.
- Keep path derivation outside the config writer so callers ensure consistency.

## Consequences
- DB and on-disk config now agree on the absolute project root, reducing deletion/resume risk.
- Scaffolds for Rust and Python share the same path strategy.
- Future import and safety checks can rely on a single canonical source of truth.
