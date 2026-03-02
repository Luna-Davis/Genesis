use std::fs;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::Languages;

pub struct GenesisConfig;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisFile {
    pub id: String,
    pub name: String,
    pub language: Languages,
    pub location: String,
    pub created_at: i64,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
}

impl GenesisConfig {
    /// Create the `.genesis/config.toml` file for an existing project id.
    pub fn write_genesis(
        id: &str,
        name: &str,
        language: &Languages,
        project_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !project_dir.exists() {
            return Err(format!(
                "Project directory '{}' was not found",
                project_dir.display()
            )
            .into());
        }

        let genesis_dir = project_dir.join(".genesis");
        fs::create_dir_all(&genesis_dir)?;

        let location = project_dir.canonicalize()?.to_string_lossy().to_string();

        let config = GenesisFile {
            id: id.to_string(),
            name: name.to_string(),
            language: language.clone(),
            location,
            created_at: Utc::now().timestamp(),
            version: default_version(),
            scripts: HashMap::new(),
        };

        let config_contents = toml::to_string_pretty(&config)?;
        fs::write(genesis_dir.join("config.toml"), config_contents)?;

        Ok(())
    }

    /// Read an existing `.genesis/config.toml` file.
    pub fn read_genesis(project_dir: &Path) -> Result<GenesisFile, Box<dyn std::error::Error>> {
        let config_path = project_dir.join(".genesis").join("config.toml");
        let contents = fs::read_to_string(&config_path)?;
        let cfg: GenesisFile = toml::from_str(&contents)?;
        Ok(cfg)
    }

    /// Write an updated genesis config back to disk.
    pub fn write_existing(
        cfg: &GenesisFile,
        project_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let genesis_dir = project_dir.join(".genesis");
        fs::create_dir_all(&genesis_dir)?;
        let config_contents = toml::to_string_pretty(cfg)?;
        fs::write(genesis_dir.join("config.toml"), config_contents)?;
        Ok(())
    }
}

fn default_version() -> String {
    "0.1.0".to_string()
}
