# 008 — Safe deletion for Genesis projects

## Context
Deleting a project must not remove arbitrary directories. Previously, deletion trusted the DB path and removed it unconditionally, risking data loss if paths drifted or were mis-recorded.

## Decision
- Before deleting, validate that the target directory contains `.genesis/config.toml` whose `id` matches the DB record. Abort on mismatch or missing config.
- Resolve the project directory consistently via canonical `location` plus project name fallback.
- File-manager delete now resolves relative targets under the current Genesis project and supports both files and directories.
- Ambiguous CLI deletes are already blocked; lock release is attempted after deletion.

## Consequences
- Prevents accidental removal of unrelated directories when DB paths are stale or wrong.
- Removes both files and folders safely within the current project context.
- Gives clear errors when `.genesis` is missing or inconsistent.
