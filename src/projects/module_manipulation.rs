use dialoguer::{Confirm, Input, Select};
use walkdir::WalkDir;

use std::ffi::OsStr;
use std::fs::{DirBuilder, remove_dir_all};
use std::path::{PathBuf, absolute};
use std::sync::LazyLock;

struct ModuleManager {
    module_name: String,
}

impl ModuleManager {
    fn new(module_name: String) -> Self {
        Self { module_name }
    }

    fn create(&self) {
        // Creates a file in the location
        match self.location_handler(Some("create")) {
            Some(location) => DirBuilder::new().recursive(true).create(location).unwrap(),
            None => println!("Internal Error: Could not get file location"),
        }
    }

    fn delete(&self) {
        // Deletes a file in the location
        match self.location_handler(Some("delete")) {
            Some(location) => {
                println!(
                    "Warning: Make sure the {} doesn't have anything important. Process irreversible.",
                    self.module_name
                );
                if Confirm::new().with_prompt("Proceed").interact().unwrap() {
                    let _ = remove_dir_all(location);
                } else {
                    std::process::exit(1);
                }
            }
            None => println!("Internal Error: Could not get file location"),
        }
    }

    fn location_handler(&self, operation: Option<&str>) -> Option<PathBuf> {
        match operation {
            Some("create") => {
                let current_dir = std::env::current_dir().unwrap();
                let parent = current_dir
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("current directory has no parent"))
                    .unwrap();
                let parent = absolute(parent).unwrap();
                let src_dir = parent.join("src");
                let module_location = src_dir.join(&self.module_name);
                return Some(module_location);
            }
            Some("delete") => {
                let current_dir = std::env::current_dir().unwrap();
                let walker = WalkDir::new(current_dir)
                    .max_depth(3)
                    .into_iter()
                    .filter_entry(|e| {
                        e.file_name() != "target"
                            && e.file_name() != ".git"
                            && e.file_name() != ".venv"
                    });

                for entry in walker {
                    let entry = entry.unwrap();
                    let name = if entry.path().is_dir() {
                        entry.file_name()
                    } else {
                        OsStr::new("")
                    };

                    if name.to_string_lossy() == self.module_name.as_str() {
                        return Some(entry.path().to_path_buf());
                    }
                }
                None
            }
            Some(_) => return None,
            None => return None,
        }
    }
}

static OPTIONS: LazyLock<Vec<String>> =
    LazyLock::new(|| vec!["Create".to_string(), "Delete".to_string()]);

pub fn module_handler() {
    let module_name: String = Input::new()
        .with_prompt("Enter Module Name")
        .interact_text()
        .unwrap();

    let option = Select::new()
        .with_prompt("Select Module Operation")
        .items(OPTIONS.iter().to_owned())
        .interact()
        .unwrap();

    let module = ModuleManager::new(module_name);
    match OPTIONS[option].as_str() {
        "Create" => module.create(),
        "Delete" => module.delete(),
        _ => {}
    }
}
