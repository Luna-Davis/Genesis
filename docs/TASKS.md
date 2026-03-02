# Genesis Backlog

## Scaffolding & Project Init
- [x] Wire UUID flow end-to-end: CLI → DB → .genesis/config.toml with canonical project root.
- [x] `genesis start`: Rust uses `cargo new`; Python builds src/tests/README/requirements/venv; git init; write .genesis/config.toml.
- [x] Blueprint prompt during start: if chosen, copy scripts into .genesis/config.toml with placeholders resolved.

## Import Existing Project (init in place)
- [x] Add `genesis import` to initialize the current directory (no new folder), similar to `cargo init`/`uv init`.
- [x] Detect language (or accept flag), create `.genesis/config.toml` with id/name/lang/location/version, do not overwrite existing files.
- [x] Git init if repo absent; register project in DB with canonical path; keep existing structure intact.
- [x] Validate `.genesis` presence on re-import; refuse if config name mismatch unless `--force`.
- [x] Lock handling on import (see Locking section).

## Session Management & Locking
- [x] Pacman-style lock: create a single lock file (e.g., `~/.local/share/genesis/genesis.lock`) when a session is active; opening a new session waits/fails with clear message.
- [x] Ensure lock is released on `stop` and on crash recovery (stale lock detection by PID/mtime).
- [x] Auto-detect `.genesis/config.toml` in CWD to set active project (via `status` command); enforce single active session.

## Safety & Deletion
- [x] Delete command: require exact id or single match; non-zero exit on ambiguity; print matches with id/lang/location.
- [x] Before deleting dirs, validate `.genesis/config.toml` matches project id; abort on mismatch.
- [x] File-manager delete: support directories (recursive) safely; validate target project root.

## Git Automation
- [x] Implement git2 helpers: init, status, stage changed files, commit, push (local only).
- [x] Deterministic commit message rules (feat/add, chore/remove, test/update, fix/bug).
- [x] `genesis git commit` (or auto flow) stages+commits with generated message.
- [x] Integrate with watcher to suggest/perform commits after debounce.

## File Watching
- [x] Add notify-based watcher with ~5s debounce; collect changed paths.
- [x] On settle, display concise change summary; trigger git automation suggestions.

## Run / Test / Build
- [x] `genesis run`: execute project entrypoint (per language), capture stdout/stderr, format errors cleanly.
- [x] `genesis test`: run test suite with clear pass/fail per test and timing summary.
- [x] Colorized, noise-stripped output using indicatif/console.

## Blueprint & Pipeline System
- [x] Blueprint storage at `~/.local/share/genesis/blueprints/*.toml`; commands: blueprint new/list/apply.
- [x] Placeholder resolution: `{{project_name}}`, `{{version}}`, etc., copied into project .genesis config under [scripts].
- [x] `genesis push`: run lint → build → test → deploy in order; halt on first failure; pretty pipeline summary.

## Configuration & Paths
- [x] Store canonical absolute project root in DB; keep .genesis/config.toml and DB in sync (id/name/language/location/version/scripts).

## Documentation & DX
- [x] Update README with install, commands, safety guarantees, import vs start distinction.
- [x] Add CONTRIBUTING/testing instructions.
- [x] Add blueprint examples.

## Pending (Phase 2)
- [ ] Custom Python bundler (single executable) and Python build path for `genesis build`.

## Testing
- [x] Unit tests for path resolution and deletion guard; git commit message rules.
- [x] Integration tests for start/import/list/delete flows and lock contention (marked ignored; run with `--ignored`).
- [x] Snapshot/CLI tests for run/test/push output formatting (run_script success/failure).
