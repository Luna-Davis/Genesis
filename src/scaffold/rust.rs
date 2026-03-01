use std::process::Command;

use crate::{model::Languages, scaffold::config::GenesisConfig};

pub fn scaffold(id: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("cargo").arg("new").arg(name).status()?;

    let language = Languages::Rust;

    match status.success() {
        true => {
            GenesisConfig::write_genesis(id, name, &language)?;
            Ok(())
        }
        false => Err("Failed to run cargo new".into()),
    }
}
