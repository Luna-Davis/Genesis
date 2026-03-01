use crate::model::Languages;
use crate::scaffold::config::GenesisConfig;
use std::env;
use std::fs;
use std::process::Command;

pub fn scaffold(id: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?.to_path_buf();
    let project_dir = current_dir.join(name);

    // 1. Create project root
    fs::create_dir_all(&project_dir)?;

    // 2. Create .venv
    let venv = project_dir.join(".venv");
    fs::create_dir_all(&venv)?;
    let status = Command::new("python")
        .arg("-m")
        .arg("venv")
        .arg(&venv)
        .status()?;

    if status.success() {
        println!("Successfully created python .venv");
    } else {
        return Err("Failed to create python .venv".into());
    }

    // 3. Create .genesis
    let language = Languages::Python;
    GenesisConfig::write_genesis(id, name, &language)?;

    // 4. Create README.md
    let readme = project_dir.join("README.md");
    let readme_content = format!(
        r#"# Title: {}
---
## Overview
Write your project Overview here"#,
        name
    );
    fs::write(readme, readme_content)?;

    // 5. requiremets.txt
    let requirements = project_dir.join("requirements.txt");
    let requirements_content = "# Project dependencies\n";
    fs::write(requirements, requirements_content)?;

    // 6. Create Source folder structure
    let src_dir = project_dir.join("src");
    let in_src_project_dir = src_dir.join(name);
    fs::create_dir_all(&in_src_project_dir)?;

    // 7. Create main.py
    let main = in_src_project_dir.join("main.py");
    let main_content = r#"def main() -> None:
        print("Hello, from main")
"#;
    fs::write(main, main_content)?;
    fs::write(in_src_project_dir.join("__init__.py"), "")?;

    // 8. Create Test folder structure
    let test_dir = project_dir.join("tests");
    fs::create_dir_all(&test_dir)?;
    let main_test = test_dir.join("test_main.py");

    let test_content = format!(
        r#"import unittest
from src.{}.main import main

class TestMain(unittest.TestCase):
    def test_main(self):
        # Add your tests here
        self.assertTrue(True)

if __name__ == '__main__':
    unittest.main()
"#,
        name
    );
    fs::write(main_test, test_content)?;

    Ok(())
}
