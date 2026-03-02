# ADR 017: Python Bundler + Native Bootloader

## Context
- Genesis needs `genesis build` to emit a self-contained executable for Python projects.
- Earlier iterations produced a zipapp without a native loader, leading to runtime errors like `can't find '__main__' module`.
- The bootloader must work offline, cache extracted files safely, and stay invisible to users (spinner-only UX).

## Decision
- Embed a Rust bootloader binary into the bundle (built in `build.rs` and copied to `OUT_DIR/bootloader_bin` with a sanity-size check).
- `packer` prepends the bootloader, then appends a ZIP archive of Python sources and a footer with the ZIP offset.
- Bootloader behavior:
  - Reads ZIP offset from the last 8 bytes.
  - Hashes the ZIP payload, extracts to `$GENESIS_CACHE_DIR/<hash>` (or `$XDG_CACHE_HOME/genesis-apps`, else `/tmp/genesis-apps`), and re-extracts if `__main__.py` is missing.
  - Runs `python3 __main__.py` with `PYTHONPATH` pointing at cache and `src/`.
  - Extraction dirs are 0700 on Unix.
- Bundler writes output to `bin/<project-name>` and marks it executable.

## Consequences
- Builds are deterministic: files sorted; stub `__main__.py` synthesized when needed.
- Broken or missing bootloader binaries abort the build early (size check).
- Cached payloads avoid re-extracting unless incomplete.
- No install noise leaks to stdout/stderr; UX stays spinner-only.

## Status
Accepted – implemented in `src/bundler/*`, bootloader in `src/bundler/bootloader`, glue in `build.rs`.
