use std::env;
use std::fs;

use chrono::Utc;
use serde::Serialize;

use crate::model::Languages;

pub struct GenesisConfig;

#[derive(Serialize)]
struct GenesisFile {
    id: String,
    name: String,
    language: Languages,
    location: String,
    created_at: i64,
}

impl GenesisConfig {
    /// Create the `.genesis/config.toml` file for an existing project id.
    pub fn write_genesis(
        id: &str,
        name: &str,
        language: &Languages,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cwd = env::current_dir()?;
        let project_dir = cwd.join(name);

        if !project_dir.exists() {
            return Err(format!(
                "Project directory '{}' was not found",
                project_dir.display()
            )
            .into());
        }

        let genesis_dir = project_dir.join(".genesis");
        fs::create_dir_all(&genesis_dir)?;

        let location = project_dir
            .canonicalize()
            .unwrap_or(project_dir.clone())
            .to_string_lossy()
            .to_string();

        let config = GenesisFile {
            id: id.to_string(),
            name: name.to_string(),
            language: language.clone(),
            location,
            created_at: Utc::now().timestamp(),
        };

        let config_contents = toml::to_string_pretty(&config)?;
        fs::write(genesis_dir.join("config.toml"), config_contents)?;

        Ok(())
    }
}
