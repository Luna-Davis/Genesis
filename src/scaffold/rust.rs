use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::{model::Languages, scaffold::config::GenesisConfig};

pub fn scaffold(id: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let project_dir: PathBuf = current_dir.join(name);

    let status = Command::new("cargo").arg("new").arg(name).status()?;

    let language = Languages::Rust;

    match status.success() {
        true => {
            GenesisConfig::write_genesis(id, name, &language, &project_dir)?;

            // Add default scripts to config
            let mut cfg = GenesisConfig::read_genesis(&project_dir)?;
            cfg.scripts.insert("lint".into(), "cargo check".into());
            cfg.scripts
                .insert("build".into(), "cargo build --release".into());
            cfg.scripts.insert("test".into(), "cargo test".into());
            cfg.scripts
                .insert("deploy".into(), "echo \"no deploy script\"".into());
            GenesisConfig::write_existing(&cfg, &project_dir)?;

            Ok(())
        }
        false => Err("Failed to run cargo new".into()),
    }
}
