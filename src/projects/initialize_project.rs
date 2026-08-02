use std::{
    path::{Path, PathBuf, absolute},
};

use dialoguer::{Input, Select};
use walkdir::WalkDir;

use crate::errors::InitializationErrors::{self, ProjectInitializationError};
use crate::marker::{GenesisMarker, read_manifest_version};

pub fn initialize_project() -> Result<(), InitializationErrors> {
    // initializes an already existing project using genesis
    let path: String = Input::new()
        .with_prompt("Enter Path")
        .interact_text()
        .unwrap();

    let project_path = if path.as_str() == "." {
        std::env::current_dir()
    } else {
        absolute(Path::new(&path))
    }?;

    let mut detected: Option<(&str, PathBuf)> = None;
    // Walks through the project directory looking for a manifest to determine
    // the language of the project
    for entry in WalkDir::new(&project_path).max_depth(2) {
        let entry = entry?;
        match entry.file_name().to_str() {
            Some("Cargo.toml") => detected = Some(("rust", entry.path().to_path_buf())),
            Some("pyproject.toml") => detected = Some(("python", entry.path().to_path_buf())),
            Some("pubspec.yaml") => detected = Some(("flutter", entry.path().to_path_buf())),
            _ => {}
        }
    }

    let (language, manifest) = match detected {
        Some((lang, manifest)) => (lang.to_string(), Some(manifest)),
        None => {
            // No manifest found, ask the user which language the project uses
            let languages = ["Rust", "Python", "Flutter"];
            let selection = Select::new()
                .with_prompt("No project manifest detected. Select project language")
                .items(&languages)
                .interact()
                .unwrap();
            (languages[selection].to_lowercase(), None)
        }
    };

    create_marker(&project_path, &language, manifest.as_deref())
}

fn create_marker(
    project_path: &Path,
    language: &str,
    manifest: Option<&Path>,
) -> Result<(), InitializationErrors> {
    // Writes the marker file into the project root
    let name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let version = match manifest {
        Some(manifest) => read_manifest_version(manifest),
        None => "0.1.0".to_string(),
    };

    let marker = GenesisMarker::new(name, language.to_string(), version);
    marker
        .create(project_path)
        .map_err(|e| ProjectInitializationError(e.to_string()))?;

    println!("Genesis marker file created");
    Ok(())
}
