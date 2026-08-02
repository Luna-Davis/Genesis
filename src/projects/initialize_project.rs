use std::{
    path::{Path, PathBuf, absolute},
    process::Command,
};

use anyhow::Result;
use dialoguer::Input;
use walkdir::WalkDir;

use crate::errors::InitializationErrors::{self, ProjectInitializationError};

pub fn initialize_project() -> Result<(), InitializationErrors> {
    // initializes an already existing project using genesis
    // TODO Add genesis meta file
    let path: String = Input::new()
        .with_prompt("Enter Path")
        .interact_text()
        .unwrap();

    let project_path = if path.as_str() == "." {
        std::env::current_dir()
    } else {
        absolute(Path::new(&path))
    }?;

    let walker = WalkDir::new(&project_path).max_depth(2);

    for entry in walker {
        let entry = entry?;
        let name = entry.file_name();
        match name.to_str() {
            Some("Cargo.toml") => initialize_rust(project_path.clone())?,
            Some("pyproject.toml") => initialize_python(project_path.clone())?,
            _ => {}
        }
    }

    Ok(())
}

fn initialize_rust(path: PathBuf) -> Result<(), InitializationErrors> {
    // Initializes a rust project using cargo to the path provided
    let result = Command::new("cargo")
        .arg("init")
        .arg(&path)
        .output()
        .map_err(|e| ProjectInitializationError(e.to_string()))?;

    if result.status.success() {
        println!("{:?} has been initialized successfully", &path);
    } else {
        return Err(ProjectInitializationError(
            path.to_string_lossy().to_string(),
        ));
    }
    Ok(())
}

fn initialize_python(path: PathBuf) -> Result<(), InitializationErrors> {
    // Initializes a python project using uv to the path provided
    let result = Command::new("uv")
        .arg("init")
        .arg(&path)
        .output()
        .map_err(|e| ProjectInitializationError(e.to_string()))?;

    if result.status.success() {
        println!("{:?} has been initialized successfully", &path);
    } else {
        return Err(ProjectInitializationError(
            path.to_string_lossy().to_string(),
        ));
    }
    Ok(())
}
