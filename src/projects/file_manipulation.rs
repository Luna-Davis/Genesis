use dialoguer::{Input, Select};
use walkdir::WalkDir;

use std::fs::{File, remove_file};
use std::path::{PathBuf, absolute};
use std::sync::LazyLock;

struct FileManager {
    file_name: String,
}

impl FileManager {
    fn new(file_name: String) -> Self {
        Self { file_name }
    }

    fn create(&self) {
        // Creates a file in the location
        match self.location_handler(Some("create")) {
            Some(location) => {
                let _ = File::create(location);
            }
            None => println!("Internal Error: Could not get file location"),
        }
    }

    fn delete(&self) {
        // Deletes a file in the location
        match self.location_handler(Some("delete")) {
            Some(location) => {
                let _ = remove_file(location);
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
                let file_location = src_dir.join(&self.file_name);
                return Some(file_location);
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
                    let name = entry.file_name();

                    if name.to_string_lossy() == self.file_name.as_str() {
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

pub fn file_handler() {
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
        _ => {}
    }
}
