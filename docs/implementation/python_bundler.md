# Genesis Python Bundler (RustPacker) — Implementation Blueprint

Target: Linux x86_64, host Python ≥ 3.12.9, system/venv ABI (no embedded interpreter). Output binary: `bin/<project-name>`. Version source of truth: `.genesis/config.toml`. Payload cached in `$GENESIS_CACHE_DIR/<hash>` or `$XDG_CACHE_HOME/genesis-apps/<hash>` (fallback `/tmp/genesis-apps/<hash>`) with 0700 perms on Unix.

---

## Module layout (inside existing crate)

- `src/bundler/` (submodule, not a separate crate)
  - `analyzer.rs` — dependency graph & asset discovery
  - `packer.rs` — manifest + packing/compression
  - `bootloader.rs` — pyo3 bootloader runtime
  - `build.rs` (top-level) — writes payload to `OUT_DIR/payload.bin` for include_bytes!
  - `mod.rs` — public `build_python_project` entry
  - `types.rs` — shared structs (Manifest, FileEntry, etc.)

CLI wiring stays in `src/cli.rs`/`src/run.rs`.

---

## Data structures (types.rs)

```rust
pub struct Manifest {
    pub entry: String, // VFS path, default "__main__.py"
    pub files: Vec<FileEntry>,          // .py + data
    pub binary_exts: Vec<FileEntry>,    // .so/.pyd
    pub payload_hash: [u8; 32],         // SHA-256 of packed blob
}

pub struct FileEntry {
    pub vfs_path: String,   // e.g., "pkg/module.py"
    pub host_path: String,  // absolute on host FS
    pub offset: u64,        // offset inside packed blob (after header)
    pub len: u64,           // uncompressed length
    pub compressed: bool,   // always true for now
}
```

Manifest serialized with `bincode` and prefixed to payload.

---

## Analyzer (analyzer.rs)

Responsibilities:
1. Validate entry point: `<project>/__main__.py` (fail if missing).
2. Parse Python source using `tree-sitter-python`.
3. Walk CST for:
   - `import_statement`, `import_from_statement`
   - `call_expression` where callee is `__import__` or `importlib.import_module` with string literal arg.
4. Module resolution:
   - Determine Python executable: prefer `VIRTUAL_ENV/bin/python3` else `python3`.
   - Use a helper script (invoked via subprocess) to run `import importlib.util, sys; print(spec.origin)` for discovered modules.
   - Standard library and site-packages resolved via `sys.path` of that interpreter.
   - Prefer venv sys.path if `VIRTUAL_ENV` set.
5. Graph build:
   - BFS from entry; maintain `HashSet<String>` visited modules.
   - Detect cycles; skip revisits.
   - Collect `.py` files, package `__init__.py`, and data files via optional glob list `.genesis/bundle.include`.
6. Outputs:
   - `Vec<FileEntryCandidate { vfs_path, host_path }>` for sources/data.
   - `Vec<FileEntryCandidate>` for `.so/.pyd`.

Failure mode: any unresolved module => hard error with missing module list.

---

## Packer (packer.rs)

Input: candidates from analyzer.

Steps:
1. Sort candidates by `vfs_path` for deterministic builds.
2. Pack format:
   - Header: `bincode::serialize(manifest_without_offsets_and_hash)`.
   - Body: For each file:
     - Read bytes.
     - Compress with zstd (level 6).
     - Write length-prefixed chunk: `[u32 path_len][path bytes][u64 orig_len][u64 comp_len][comp_bytes]`.
     - Record offset/len in manifest.
3. Compute SHA-256 of body; fill `payload_hash` in manifest.
4. Serialize final manifest (with offsets/hash) and prepend to body:
   - `[u32 manifest_len][manifest_bytes][body...]`
5. Write to `OUT_DIR/payload.bin`.

---

## Bootloader (bootloader.rs)

Built by the top-level `build.rs` into `OUT_DIR/bootloader_bin` (build panics if <1 KB to avoid embedding a stub).

Runtime path:
1. Read the ZIP-start offset from the footer (last 8 bytes) and hash the ZIP payload.
2. Extract ZIP to cache if `__main__.py` is missing; otherwise reuse cache.
3. Launch `python3 __main__.py` with `PYTHONPATH=<cache>:<cache>/src`.
4. Cache directory permissions are 0700 on Unix. C-extensions are still planned but not bundled yet.

---

## build.rs

- Builds the native bootloader subcrate and copies it to `OUT_DIR/bootloader_bin`.
- Emits `cargo:rerun-if-changed=src/bundler/bootloader/{src/main.rs,Cargo.toml}` so the bootloader rebuilds when touched.

---

## CLI integration

- `genesis build [--bump <patch|minor|major>]`:
  - Rust: `cargo build --release`.
  - Python: runs analyzer + packer, writes `bin/<project-name>`, bumps version in `.genesis/config.toml`.
- Cache override: `GENESIS_CACHE_DIR=/path genesis build`.

---

## Safety & cleanup

- Temp dir for C-exts under `/tmp`, unique per run, chmod 700, removed on normal exit and SIGINT/SIGTERM.
- Integrity check: payload hash verified before mounting VFS; abort on mismatch.
- No DB writes; version bump only in `.genesis/config.toml`.

---

## Testing (pending)

- Unit: manifest encode/decode; deterministic packing order; version bump helper.
- Integration (ignored by default): bundle a pure-Python sample; bundle with a simple C-ext wheel; run produced binary and assert output, cache reuse.
- Failure cases: missing module → error; wrong Python version → error; missing entry → error.
