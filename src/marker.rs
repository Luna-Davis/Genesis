use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::MarkerErrors;

const MARKER_FILE: &str = ".genesis.json";
const TOOL_NAME: &str = "genesis";
const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GenesisMarker {
    pub id: Uuid,
    pub name: String,
    pub language: String,
    pub version: String,
    pub schema_version: u32,
    pub tool: String,
    pub tool_version: String,
    pub created_at: DateTime<Utc>,
}

impl GenesisMarker {
    pub fn new(name: String, language: String, version: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            language,
            version,
            schema_version: SCHEMA_VERSION,
            tool: TOOL_NAME.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn create(&self, path: &Path) -> Result<PathBuf, MarkerErrors> {
        let marker_path = path.join(MARKER_FILE);
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&marker_path, contents)?;
        Ok(marker_path)
    }

    pub fn load(path: &Path) -> Result<Self, MarkerErrors> {
        let marker_path = path.join(MARKER_FILE);
        let contents = fs::read_to_string(&marker_path)?;
        let marker: GenesisMarker = serde_json::from_str(&contents)?;

        if marker.tool != TOOL_NAME {
            return Err(MarkerErrors::WrongTool(marker.tool));
        }
        if marker.schema_version != SCHEMA_VERSION {
            return Err(MarkerErrors::UnsupportedSchemaVersion(
                marker.schema_version,
                SCHEMA_VERSION,
            ));
        }
        Ok(marker)
    }

    pub fn is_genesis_project(path: &Path) -> bool {
        path.join(MARKER_FILE).exists()
    }
}

pub(crate) fn read_manifest_version(manifest: &Path) -> String {
    let contents = fs::read_to_string(manifest).unwrap_or_default();
    contents
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("version =")
                .map(|v| v.trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "0.1.0".to_string())
}
