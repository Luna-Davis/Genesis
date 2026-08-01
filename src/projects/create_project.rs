use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use dialoguer::{Confirm, Input, Select};
use dirs;

static LANGUAGES: LazyLock<Vec<String>> =
    LazyLock::new(|| vec!["Rust".to_string(), "Python".to_string()]);

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

    fn create(&self) {
        if self.language.to_lowercase() == "python" {
            let result = Command::new("uv")
                .arg("init")
                .arg(self.name.to_lowercase())
                .output()
                .unwrap();

            if result.status.success() {
                println!("Project created successfully");
            } else {
                println!("Project not created!");
                std::process::exit(1);
            }
        } else if self.language.to_lowercase() == "Rust" {
            let result = Command::new("cargo")
                .arg("new")
                .arg(self.name.to_lowercase())
                .output()
                .expect("Could not execute the creation command");

            if result.status.success() {
                println!("Project created successfully");
            } else {
                println!("Project not created!");
                std::process::exit(1);
            }
        } else {
            println!("Language not supported (for now)!");
        }
    }
}

pub fn create_project() {
    let project_name: String = Input::new()
        .with_prompt("Project Name")
        .interact_text()
        .unwrap();

    let language = Select::new()
        .with_prompt("Select Project Language")
        .items(LANGUAGES.iter().to_owned())
        .interact()
        .unwrap();

    let projects_dir = dirs::home_dir().unwrap().as_path().join("projects");
    let path: String = Input::new()
        .with_prompt("Enter Path for {project_name}")
        .default(projects_dir.to_string_lossy().to_string())
        .interact_text()
        .unwrap();

    let project = Project::new(project_name, language.to_string(), Path::new(&path));

    let confirmation = Confirm::new()
        .with_prompt("Do you want to create project?")
        .interact()
        .unwrap();

    if confirmation {
        project.create();
    } else {
        println!("Exiting...");
        std::process::exit(0);
    }
}
