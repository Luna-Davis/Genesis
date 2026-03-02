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

    // 2b. Git init if missing
    let git_status = Command::new("git")
        .arg("init")
        .current_dir(&project_dir)
        .status()?;
    if !git_status.success() {
        return Err("Failed to run git init".into());
    }

    // 3. Create .genesis
    let language = Languages::Python;
    GenesisConfig::write_genesis(id, name, &language, &project_dir)?;

    // Add default scripts to config
    let mut cfg = GenesisConfig::read_genesis(&project_dir)?;
    cfg.scripts.insert("lint".into(), "ruff check .".into());
    cfg.scripts.insert("build".into(), "genesis build".into());
    cfg.scripts
        .insert("test".into(), "python -m unittest discover tests".into());
    cfg.scripts
        .insert("deploy".into(), "echo \"no deploy script\"".into());
    GenesisConfig::write_existing(&cfg, &project_dir)?;

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

    // 5. requirements.txt
    let requirements = project_dir.join("requirements.txt");
    let requirements_content = "# Project dependencies\n";
    fs::write(requirements, requirements_content)?;

    // 6. Create Source folder structure
    let safe_name = name.replace('-', "_");
    let src_dir = project_dir.join("src");
    let in_src_project_dir = src_dir.join(&safe_name);
    fs::create_dir_all(&in_src_project_dir)?;

    // 7. Create main.py
    let main = in_src_project_dir.join("main.py");
    let main_content = r#"def main() -> None:
    print("Hello, from main")

if __name__ == "__main__":
    main()
"#;
    fs::write(main, main_content)?;
    fs::write(in_src_project_dir.join("__init__.py"), "")?;

    // 7b. Create root __main__.py for bundler
    let root_main = project_dir.join("__main__.py");
    let root_main_content = format!(
        r#"import sys
import os

# Ensure src is in the python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "src"))

from {}.main import main

if __name__ == "__main__":
    main()
"#,
        safe_name
    );
    fs::write(root_main, root_main_content)?;

    // 8. Create Test folder structure
    let test_dir = project_dir.join("tests");
    fs::create_dir_all(&test_dir)?;
    let main_test = test_dir.join("test_main.py");

    let test_content = format!(
        r#"import unittest
import sys
import os

# Ensure src is in the python path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'src')))

from {}.main import main

class TestMain(unittest.TestCase):
    def test_main(self):
        # Add your tests here
        self.assertTrue(True)

if __name__ == '__main__':
    unittest.main()
"#,
        safe_name
    );
    fs::write(main_test, test_content)?;

    Ok(())
}
