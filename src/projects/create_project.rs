use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use dialoguer::{Confirm, Input, Select};
use dirs;

use crate::errors::CreationErrors::{self, LanguageNotSupported, ProjectCreationError};
use crate::marker::{GenesisMarker, read_manifest_version};

static LANGUAGES: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "Rust".to_string(),
        "Python".to_string(),
        "Flutter".to_string(),
        "JavaScript".to_string(),
    ]
});

struct Project {
    name: String,
    language: String,
    path: PathBuf,
}

impl Project {
    fn new(name: String, language: String, path: &Path) -> Self {
        Self {
            name: name,
            language: language,
            path: path.to_path_buf(),
        }
    }

    fn create_marker(&self) -> Result<(), CreationErrors> {
        // Writes the marker file into the created project directory
        let project_dir = std::env::current_dir()
            .map_err(|e| ProjectCreationError(e.to_string()))?
            .join(self.name.to_lowercase());

        let language = self.language.to_lowercase();
        let manifest = project_dir.join(match language.as_str() {
            "python" => "pyproject.toml",
            "flutter" => "pubspec.yaml",
            "javascript" => "package.json",
            "rust" => "Cargo.toml",
            _ => "",
        });
        let version = read_manifest_version(&manifest);

        let marker = GenesisMarker::new(self.name.clone(), language, version);
        marker
            .create(&project_dir)
            .map_err(|e| ProjectCreationError(e.to_string()))?;

        println!("Genesis marker file created");
        Ok(())
    }

    fn create(&self) -> Result<(), CreationErrors> {
        if self.language.to_lowercase() == "python" {
            let result = Command::new("uv")
                .arg("init")
                .arg(self.name.to_lowercase())
                .output()
                .map_err(|e| ProjectCreationError(e.to_string()))?;

            if result.status.success() {
                println!("Project created successfully");
                self.create_marker()?;
            } else {
                return Err(ProjectCreationError(
                    String::from_utf8_lossy(&result.stderr).to_string(),
                ));
            }
        } else if self.language.to_lowercase() == "rust" {
            let result = Command::new("cargo")
                .arg("new")
                .arg(self.name.to_lowercase())
                .output()
                .map_err(|e| ProjectCreationError(e.to_string()))?;

            if result.status.success() {
                println!("Project created successfully at {:?}", self.path);
                self.create_marker()?;
            } else {
                return Err(ProjectCreationError(
                    String::from_utf8_lossy(&result.stderr).to_string(),
                ));
            }
        } else if self.language.to_lowercase() == "flutter" {
            let result = Command::new("flutter")
                .arg("create")
                .arg(self.name.to_lowercase())
                .output()
                .map_err(|e| ProjectCreationError(e.to_string()))?;

            if result.status.success() {
                println!("Project created successfully at {:?}", self.path);
                self.create_marker()?;
            } else {
                return Err(ProjectCreationError(
                    String::from_utf8_lossy(&result.stderr).to_string(),
                ));
            }
        } else if self.language.to_lowercase() == "javascript" {
            println!("Warning: Using Expo React Framework for project!");
            let confirmation = Confirm::new().with_prompt("Proceed?").interact().unwrap();

            if !confirmation {
                println!("Aborting build with react");
                return Ok(());
            } else {
                let result = Command::new("npx")
                    .arg("create-expo-app@latest")
                    .arg(self.name.to_lowercase())
                    .arg("--yes")
                    .output()
                    .map_err(|e| ProjectCreationError(e.to_string()))?;

                if result.status.success() {
                    println!("Project created successfully at {:?}", self.path);
                    self.create_marker()?;
                } else {
                    return Err(ProjectCreationError(
                        String::from_utf8_lossy(&result.stderr).to_string(),
                    ));
                }
            }
        } else {
            return Err(LanguageNotSupported);
        }
        Ok(())
    }
}

pub fn create_project() -> Result<(), CreationErrors> {
    let project_name: String = Input::new()
        .with_prompt("Project Name")
        .interact_text()
        .map_err(|e| ProjectCreationError(e.to_string()))?;

    let language = Select::new()
        .with_prompt("Select Project Language")
        .items(LANGUAGES.iter().to_owned())
        .interact()
        .map_err(|e| ProjectCreationError(e.to_string()))?;

    let projects_dir = dirs::home_dir().unwrap().as_path().join("projects");
    let path: String = Input::new()
        .with_prompt("Enter Path for {project_name}")
        .default(projects_dir.to_string_lossy().to_string())
        .interact_text()
        .map_err(|e| ProjectCreationError(e.to_string()))?;

    let project = Project::new(
        project_name,
        LANGUAGES[language].to_string(),
        Path::new(&path),
    );

    let confirmation = Confirm::new()
        .with_prompt("Do you want to create project?")
        .interact()
        .map_err(|e| ProjectCreationError(e.to_string()))?;

    if confirmation {
        let _ = project.create(); // Don't require the output
    } else {
        println!("Exiting...");
    }
    Ok(())
}
