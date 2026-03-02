# 011 — Run and Test commands

## Context
Genesis should execute project run and test flows with clean, human-friendly output. It must work offline and respect language differences (Rust vs. Python scaffold).

## Decision
- Add `genesis run` and `genesis test`:
  - Require running inside a Genesis project (`.genesis` present via validator).
  - Rust: `cargo run` / `cargo test`.
  - Python: `python src/main.py` for run; `python -m unittest discover -s tests` for tests (aligns with scaffolded layout).
- Output:
  - Spinner while command runs (indicatif).
  - On completion, show colored success/failure line with elapsed seconds.
  - Print stdout (dim) and stderr (red) trimmed of trailing whitespace; show nothing if empty.
- Errors surface exit codes; no network dependencies beyond cargo/py runtime.

## Consequences
- Consistent, readable run/test UX across languages.
- Minimal assumptions about project structure beyond scaffold defaults.
- Foundation for future TUI reuse of the same run/test helpers.
