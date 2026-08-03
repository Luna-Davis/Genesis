use dialoguer::{Input, Select};
use walkdir::WalkDir;

use std::fs::{File, remove_file};
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::errors::FileManagementErrors;
use crate::errors::FileManagementErrors::{FileNotFound, OperationNotSupported};

struct FileManager {
    file_name: String,
}

impl FileManager {
    fn new(file_name: String) -> Self {
        Self { file_name }
    }

    fn create(&self) -> Result<(), FileManagementErrors> {
        // Creates a file in the location
        match self.location_handler(Some("create"))? {
            Some(location) => {
                let _ = File::create(location);
                println!("{} created successfully!", &self.file_name);
            }
            _ => return Err(FileNotFound),
        }
        Ok(())
    }

    fn delete(&self) -> Result<(), FileManagementErrors> {
        // Deletes a file in the location
        match self.location_handler(Some("delete"))? {
            Some(location) => {
                let _ = remove_file(location);
                println!("{} deleted successfully!", &self.file_name);
            }
            _ => return Err(FileNotFound),
        }
        Ok(())
    }

    fn location_handler(
        &self,
        operation: Option<&str>,
    ) -> Result<Option<PathBuf>, FileManagementErrors> {
        // Gets an operation to perform on a file
        match operation {
            // Creates a file
            Some("create") => {
                let project_dir = std::env::current_dir().unwrap();
                let src_dir = project_dir.join("src");
                let file_location = src_dir.join(&self.file_name);
                return Ok(Some(file_location));
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
                            && e.file_name() != "node_modules"
                    });

                for entry in walker {
                    let entry = entry.unwrap();
                    let name = entry.file_name();

                    if name.to_string_lossy() == self.file_name.as_str() {
                        return Ok(Some(entry.path().to_path_buf()));
                    }
                }
                return Ok(None);
            }
            Some(_) => return Err(OperationNotSupported),
            None => return Ok(None),
        }
    }
}

static OPTIONS: LazyLock<Vec<String>> =
    LazyLock::new(|| vec!["Create".to_string(), "Delete".to_string()]);

pub fn file_handler() -> Result<(), FileManagementErrors> {
    let file_name: String = Input::new()
        .with_prompt("Enter File Name")
        .interact_text()
        .unwrap();

    let option = Select::new()
        .with_prompt("Select File Operation")
        .items(OPTIONS.iter().to_owned())
        .interact()
        .unwrap();

    let file = FileManager::new(file_name);
    match OPTIONS[option].as_str() {
        "Create" => file.create(),
        "Delete" => file.delete(),
        _ => return Err(OperationNotSupported),
    }
}
