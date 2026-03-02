# Genesis

Global CLI project lifecycle manager — opinionated, offline, and beautiful.

## Install

```bash
cargo install --path .
```

Requirements: Rust toolchain, git, python (for Python projects).

## Quick start

```bash
genesis start myproject Rust   # scaffold + register + lock
cd myproject
genesis run                    # run
genesis test                   # test
genesis git-commit             # stage+commit with generated message
genesis push                   # lint -> build -> test -> deploy
genesis install django         # install dependency (uv/pip or cargo) with a spinner
genesis bootstrap              # set up venv/shared cache or cargo fetch
genesis stop                   # release lock
genesis guide                  # print quickstart/usage guide
```

Import an existing project (in-place, like `cargo init`):

```bash
cd existing-project
genesis import --language Rust
```

Blueprints:

```bash
genesis blueprint new rust-service
genesis blueprint list
genesis blueprint apply rust-service
```

## Commands

- `start <name> <language>`: scaffold (Rust/Python), git init, write `.genesis/config.toml`, register in SQLite, acquire lock.
- `import [--language <lang>] [--force]`: initialize current dir; reuse existing `.genesis` if present, otherwise create it and git init.
- `resume` / `stop`: manage active session (single global lock at `~/.local/share/genesis/genesis.lock`).
- `list`: show projects with timestamps/status.
- `delete --id <uuid> | <name>`: safe delete with `.genesis` id match; non-zero on ambiguity.
- `status`: show project in current directory via `.genesis`.
- `run` / `test`: language-aware execution with formatted output.
- `git-commit [--message <msg>]`: stage all and commit with deterministic message.
- `blueprint new|list|apply`: manage CI/CD templates stored at `~/.local/share/genesis/blueprints/`.
- `push`: run pipeline lint→build→test→deploy from project scripts; halts on first failure.
- `watch [--auto-commit]`: debounce file changes, show summaries, optionally stage+commit automatically with generated message.
- `build [--bump <patch|minor|major>]`: run project build (Rust: `cargo build --release`; Python: native bootloader + zip bundle) and bump version in `.genesis/config.toml`.
- `install <package> [--language <Python|Rust>]`: install dependencies via uv (preferred) or pip for Python (respecting venv) and cargo for Rust; installer output is hidden behind a spinner.
- `bootstrap`: set up envs (shared Python venv cloning + uv sync, or cargo fetch).
- `ci --provider github`: emit `.github/workflows/genesis.yml` from your current scripts.
- `guide`: print the built-in quickstart with platform support (Linux x86_64), commands, and tips.

## Guide (also available via `genesis guide`)

- Platform: Linux x86_64. macOS/Windows not supported yet.
- Install: `cargo install --path .` (needs Rust toolchain, git, python/uv for Python projects).
- Start/import: `genesis start <name> <Rust|Python>` or `genesis import --language Python`.
- Auto blueprints: detects Django/Flask/FastAPI (or Rust web crates) and applies matching scripts; override with `--blueprint <name>`.
- Build: Rust → `cargo build --release`; Python → native bootloader + zipapp in `bin/<name>`.
- Install deps: `genesis install <pkg>` (uv/pip or cargo) with spinner; Django/Flask adds `runserver`.
- Bootstrap: shared venv reuse + uv sync, or cargo fetch.
- Env loading: `.env`, `.env.<GENESIS_ENV>`, `.env.local` injected into run/test/push/build.
- CI: `genesis ci --provider github` writes workflow from current scripts.
- Run/test/push: `genesis run | test | push`; `watch --auto-commit` available.

Blueprints live at `~/.local/share/genesis/blueprints/*.toml` and contain reusable `[scripts]` blocks (e.g., lint/build/test/deploy). `genesis blueprint apply <name>` merges them into your project's `.genesis/config.toml` so pipelines stay consistent across projects.

## Safety guarantees

- Single active session enforced by lock file; stale locks auto-recovered.
- Delete validates `.genesis/config.toml` id matches DB before removing directories.
- Import is non-destructive; creates only `.genesis` (and git init if missing).
- Canonical project paths stored in DB and config to avoid drift.

## Project layout

```
~/.local/share/genesis/genesis.db         # SQLite state
~/.local/share/genesis/blueprints/*.toml  # reusable pipeline templates
project/.genesis/config.toml              # project metadata + scripts
```

## Contributing / testing

- WIP: add unit/integration/snapshot tests (see docs/TASKS.md).
- Current tests may fail offline due to crates.io access; rerun with network.

## License

MIT
