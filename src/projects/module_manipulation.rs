use dialoguer::{Confirm, Input, Select};
use walkdir::WalkDir;

use std::ffi::OsStr;
use std::fs::{DirBuilder, remove_dir_all};
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::errors::ModuleManagementErrors;
use crate::errors::ModuleManagementErrors::{ModuleNotFound, OperationNotSupported};

struct ModuleManager {
    module_name: String,
}

impl ModuleManager {
    fn new(module_name: String) -> Self {
        Self { module_name }
    }

    fn create(&self) -> Result<(), ModuleManagementErrors> {
        // Creates a file in the location
        match self.location_handler(Some("create"))? {
            Some(location) => {
                DirBuilder::new().recursive(true).create(location).unwrap();
                println!("{} created successfully!", &self.module_name);
            }
            None => return Err(ModuleNotFound),
        }
        Ok(())
    }

    fn delete(&self) -> Result<(), ModuleManagementErrors> {
        // Deletes a file in the location
        match self.location_handler(Some("delete"))? {
            Some(location) => {
                println!(
                    "Warning: Make sure the {} doesn't have anything important. Process irreversible.",
                    self.module_name
                );
                if Confirm::new().with_prompt("Proceed").interact().unwrap() {
                    let _ = remove_dir_all(location);
                    println!("{} deleted successfully!", &self.module_name);
                } else {
                    println!("Process Aborted");
                }
            }
            None => return Err(ModuleNotFound),
        }
        Ok(())
    }

    fn location_handler(
        &self,
        operation: Option<&str>,
    ) -> Result<Option<PathBuf>, ModuleManagementErrors> {
        // Returns the location of the module
        match operation {
            Some("create") => {
                let project_dir = std::env::current_dir().unwrap();
                let src_dir = project_dir.join("src");
                let module_location = src_dir.join(&self.module_name); // Appends the module folder
                // to source folder
                return Ok(Some(module_location)); // return the module path 
            }
            Some("delete") => {
                // Searches for the module name in the current directory (Assumption that the tool
                // will be run inside a Genesis project)
                let current_dir = std::env::current_dir().unwrap();
                let walker = WalkDir::new(current_dir)
                    .max_depth(3)
                    .into_iter()
                    .filter_entry(|e| {
                        e.file_name() != "target"
                            && e.file_name() != ".git"
                            && e.file_name() != ".venv"
                    });

                // Walks through the directory filtering only directories and compares the names to
                // the module name and returns the path if found
                for entry in walker {
                    let entry = entry.unwrap();
                    let name = if entry.path().is_dir() {
                        entry.file_name()
                    } else {
                        OsStr::new("")
                    };

                    if name.to_string_lossy() == self.module_name.as_str() {
                        return Ok(Some(entry.path().to_path_buf()));
                    } else {
                        continue;
                    }
                }
                return Err(ModuleNotFound);
            }
            Some(_) => return Err(OperationNotSupported),
            None => return Err(OperationNotSupported),
        }
    }
}

static OPTIONS: LazyLock<Vec<String>> =
    LazyLock::new(|| vec!["Create".to_string(), "Delete".to_string()]);

pub fn module_handler() -> Result<(), ModuleManagementErrors> {
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
        _ => return Err(OperationNotSupported),
    }
}
