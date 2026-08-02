// Marker file management for Genesis projects
//
// The marker file (.genesis.json) is written into the root of a project to
// identify it as a Genesis project and to store its metadata. It is created
// when a project is initialized or created and is required to run the shell.

use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::MarkerErrors;

pub(crate) const MARKER_FILE: &str = ".genesis.json"; // File name of the marker in the project root
const TOOL_NAME: &str = "genesis"; // Name of the tool the marker belongs to
const SCHEMA_VERSION: u32 = 1; // Current marker format version

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GenesisMarker {
    // Metadata stored in the marker file
    pub id: Uuid, // Unique project identifier
    pub name: String, // Project name
    pub language: String, // Project language (rust / python)
    pub version: String, // Project version
    pub schema_version: u32, // Marker format version
    pub tool: String, // Tool that owns the marker
    pub tool_version: String, // Version of the tool that created the marker
    pub created_at: DateTime<Utc>, // Creation timestamp in RFC 3339
}

impl GenesisMarker {
    // Creates a new marker for the given project
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

    // Writes the marker file into the given directory and returns its path
    pub fn create(&self, path: &Path) -> Result<PathBuf, MarkerErrors> {
        let marker_path = path.join(MARKER_FILE);
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&marker_path, contents)?;
        Ok(marker_path)
    }

    // Reads the marker file from the directory and validates the tool ownership
    // and schema version
    pub fn load(path: &Path) -> Result<Self, MarkerErrors> {
        let marker_path = path.join(MARKER_FILE);
        let contents = fs::read_to_string(&marker_path)?;
        let marker: GenesisMarker = serde_json::from_str(&contents)?;

        // Refuses markers created by a different tool
        if marker.tool != TOOL_NAME {
            return Err(MarkerErrors::WrongTool(marker.tool));
        }
        // Refuses markers using a schema version this version cannot read
        if marker.schema_version != SCHEMA_VERSION {
            return Err(MarkerErrors::UnsupportedSchemaVersion(
                marker.schema_version,
                SCHEMA_VERSION,
            ));
        }
        Ok(marker)
    }

    // Checks whether the given directory contains a marker file
    pub fn is_genesis_project(path: &Path) -> bool {
        path.join(MARKER_FILE).exists()
    }
}

// Extracts the version field from a project manifest
// (Cargo.toml / pyproject.toml / pubspec.yaml).
// Falls back to "0.1.0" when the manifest cannot be read or has no version
pub(crate) fn read_manifest_version(manifest: &Path) -> String {
    let contents = fs::read_to_string(manifest).unwrap_or_default();
    contents
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("version =")
                .or_else(|| line.strip_prefix("version:"))
                .map(|v| v.trim().trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "0.1.0".to_string())
}
