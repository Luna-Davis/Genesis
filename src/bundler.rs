#![allow(dead_code)]

#[cfg(feature = "python-bundler")]
mod analyzer;
#[cfg(feature = "python-bundler")]
mod packer;
#[cfg(feature = "python-bundler")]
mod types;

#[cfg(feature = "python-bundler")]
mod enabled {
    use std::fs;
    use std::path::Path;

    use thiserror::Error;

    use super::{analyzer, packer};
    use crate::scaffold::config::GenesisFile;

    #[derive(Debug, Error)]
    pub enum BundleError {
        #[error("Missing entry point: no __main__.py or src/*/main.py found")]
        MissingEntry,
        #[error("Dependency resolution failed: {0}")]
        Resolve(String),
        #[error("Packer error: {0}")]
        Packer(String),
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
    }

    pub fn build_python_project(project_dir: &Path, cfg: &GenesisFile) -> Result<(), BundleError> {
        // Find the entry point
        let entry = find_entry_point(project_dir, cfg).ok_or(BundleError::MissingEntry)?;

        // Analyze imports via tree-sitter
        let analysis = analyzer::analyze(project_dir, &entry)
            .map_err(|e: analyzer::AnalyzeError| BundleError::Resolve(e.to_string()))?;

        // Pack into a zipapp
        let payload = packer::pack_zipapp(&analysis)
            .map_err(|e: packer::PackError| BundleError::Packer(e.to_string()))?;

        // Write to bin/<name>
        let bin_dir = project_dir.join("bin");
        fs::create_dir_all(&bin_dir)?;
        let out_path = bin_dir.join(&cfg.name);
        fs::write(&out_path, &payload.bytes)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&out_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&out_path, perms)?;
        }

        println!("Bundled → {}", out_path.display());
        Ok(())
    }

    fn find_entry_point(project_dir: &Path, cfg: &GenesisFile) -> Option<std::path::PathBuf> {
        // 1. __main__.py at root
        let root_main = project_dir.join("__main__.py");
        if root_main.exists() {
            return Some(root_main);
        }

        // 2. src/<name>/main.py
        let safe_name = cfg.name.replace('-', "_");
        let src_main = project_dir.join("src").join(&safe_name).join("main.py");
        if src_main.exists() {
            return Some(src_main);
        }

        // 3. src/main.py
        let src_flat = project_dir.join("src").join("main.py");
        if src_flat.exists() {
            return Some(src_flat);
        }

        None
    }
}

#[cfg(not(feature = "python-bundler"))]
mod enabled {
    use crate::scaffold::config::GenesisFile;
    use std::path::Path;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum BundleError {
        #[error("Python bundler not compiled (enable feature `python-bundler`)")]
        NotBuilt,
    }

    pub fn build_python_project(
        _project_dir: &Path,
        _cfg: &GenesisFile,
    ) -> Result<(), BundleError> {
        Err(BundleError::NotBuilt)
    }
}

pub use enabled::{BundleError, build_python_project};
