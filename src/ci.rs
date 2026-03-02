use std::fs;
use std::path::Path;

use crate::scaffold::config::GenesisFile;

pub fn emit_github_actions(project_dir: &Path, cfg: &GenesisFile) -> Result<(), String> {
    let workflow_dir = project_dir.join(".github/workflows");
    fs::create_dir_all(&workflow_dir).map_err(|e| e.to_string())?;
    let workflow_path = workflow_dir.join("genesis.yml");
    let steps = cfg
        .scripts
        .iter()
        .map(|(name, cmd)| format!("      - name: {}\n        run: {}", name, cmd))
        .collect::<Vec<_>>()
        .join("\n");

    let contents = format!(
        r#"name: Genesis Pipeline

on:
  push:
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
{}
"#,
        steps
    );
    fs::write(workflow_path, contents).map_err(|e| e.to_string())
}
