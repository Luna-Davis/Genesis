# 006 — Global lock file for active Genesis session

## Context
Genesis should allow only one active project/session at a time, similar to pacman’s lock. Crashes or interrupts must not leave the tool unusable; stale locks need recovery.

## Decision
- Use a single global lock file at `~/.local/share/genesis/genesis.lock`.
- Lock contents: `pid`, `project_id`, `timestamp` (TOML).
- Acquisition is atomic via `create_new`; if the file exists, we treat it as active unless either:
  - the recorded PID is no longer running, or
  - the lock mtime is older than 6 hours (stale), in which case it is removed and retried.
- Releasing removes the lock only if it matches the caller’s project, the caller’s PID, or the holder PID is dead. This avoids stealing a live lock.
- The lock is acquired on `start`, `resume`, and `import`; released on `stop` and when deleting a project.

## Consequences
- Enforces single active session with minimal footprint.
- Stale locks self-heal on next acquisition attempt.
- Behavior is cross-platform (PID liveness via `sysinfo`).

## Open item
- Auto-detecting active project from CWD is handled by `validator` + `status` command, but lock acquisition on shell entry remains out of scope for now.
