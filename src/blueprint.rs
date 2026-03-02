use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use dirs::data_dir;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::scaffold::config::GenesisConfig;

#[derive(Debug, Error)]
pub enum BlueprintError {
    #[error("Failed to determine data directory")]
    DataDirNotFound,
    #[error("Blueprint not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blueprint {
    pub name: String,
    pub scripts: HashMap<String, String>,
}

/// Built-in blueprint presets keyed by name.
fn builtin_blueprints() -> Vec<Blueprint> {
    vec![
        Blueprint {
            name: "django".into(),
            scripts: HashMap::from([
                ("lint".into(), "ruff check .".into()),
                ("test".into(), "python -m pytest".into()),
                ("build".into(), "genesis build".into()),
                (
                    "deploy".into(),
                    "python manage.py migrate && python manage.py collectstatic --noinput".into(),
                ),
                ("runserver".into(), "python manage.py runserver".into()),
            ]),
        },
        Blueprint {
            name: "flask".into(),
            scripts: HashMap::from([
                ("lint".into(), "ruff check .".into()),
                ("test".into(), "python -m pytest".into()),
                ("build".into(), "genesis build".into()),
                ("deploy".into(), "echo \"add your deploy step\"".into()),
                ("runserver".into(), "flask run".into()),
            ]),
        },
        Blueprint {
            name: "fastapi".into(),
            scripts: HashMap::from([
                ("lint".into(), "ruff check .".into()),
                ("test".into(), "python -m pytest".into()),
                ("build".into(), "genesis build".into()),
                ("deploy".into(), "echo \"add your deploy step\"".into()),
                ("runserver".into(), "uvicorn app:app --reload".into()),
            ]),
        },
        Blueprint {
            name: "rust-service".into(),
            scripts: HashMap::from([
                ("lint".into(), "cargo clippy -- -D warnings".into()),
                ("test".into(), "cargo test".into()),
                ("build".into(), "cargo build --release".into()),
                ("deploy".into(), "echo \"deploy binary\"".into()),
            ]),
        },
    ]
}

pub struct BlueprintStore;

impl BlueprintStore {
    fn base_dir() -> Result<PathBuf, BlueprintError> {
        let base = data_dir()
            .ok_or(BlueprintError::DataDirNotFound)?
            .join("genesis")
            .join("blueprints");
        fs::create_dir_all(&base)?;
        Ok(base)
    }

    pub fn list() -> Result<Vec<Blueprint>, BlueprintError> {
        let dir = Self::base_dir()?;
        let mut blueprints = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Ok(bp) = Self::load_from_path(&entry.path()) {
                    blueprints.push(bp);
                }
            }
        }
        Ok(blueprints)
    }

    pub fn create(name: &str) -> Result<PathBuf, BlueprintError> {
        let dir = Self::base_dir()?;
        let path = dir.join(format!("{}.toml", name));
        if path.exists() {
            return Ok(path);
        }

        let blueprint = builtin_blueprints()
            .into_iter()
            .find(|b| b.name == name)
            .unwrap_or_else(|| Blueprint {
                name: name.to_string(),
                scripts: HashMap::from([
                    ("lint".into(), "echo \"lint stub\"".into()),
                    ("build".into(), "echo \"build stub\"".into()),
                    ("test".into(), "echo \"test stub\"".into()),
                    ("deploy".into(), "echo \"deploy stub\"".into()),
                ]),
            });
        let contents = toml::to_string_pretty(&blueprint)?;
        fs::write(&path, contents)?;
        Ok(path)
    }

    pub fn apply_to_project(name: &str, project_dir: &Path) -> Result<(), BlueprintError> {
        let blueprint = Self::load(name)?;
        let mut cfg = GenesisConfig::read_genesis(project_dir).map_err(|e| {
            BlueprintError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        let project_name = cfg.name.clone();
        let version = cfg.version.clone();

        let mut resolved_scripts = HashMap::new();
        for (k, v) in blueprint.scripts.iter() {
            let replaced = v
                .replace("{{project_name}}", &project_name)
                .replace("{{version}}", &version);
            resolved_scripts.insert(k.clone(), replaced);
        }
        cfg.scripts = resolved_scripts;
        GenesisConfig::write_existing(&cfg, project_dir).map_err(|e| {
            BlueprintError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok(())
    }

    pub fn load(name: &str) -> Result<Blueprint, BlueprintError> {
        let dir = Self::base_dir()?;
        let path = dir.join(format!("{}.toml", name));
        if !path.exists() {
            // Seed built-in on demand
            for bp in builtin_blueprints() {
                if bp.name == name {
                    let contents = toml::to_string_pretty(&bp)?;
                    fs::write(&path, contents)?;
                    break;
                }
            }
        }
        Self::load_from_path(&path)
    }

    fn load_from_path(path: &Path) -> Result<Blueprint, BlueprintError> {
        if !path.exists() {
            return Err(BlueprintError::NotFound(path.display().to_string()));
        }
        let contents = fs::read_to_string(path)?;
        let bp: Blueprint = toml::from_str(&contents)?;
        Ok(bp)
    }

    /// Try to auto-apply a blueprint based on project layout or deps.
    pub fn auto_apply(
        project_dir: &Path,
        cfg: &crate::scaffold::config::GenesisFile,
    ) -> Result<Option<String>, BlueprintError> {
        let mut candidates: Vec<&str> = Vec::new();
        match cfg.language {
            crate::model::Languages::Python => {
                if project_dir.join("manage.py").exists() {
                    candidates.push("django");
                } else if project_dir.join("app.py").exists()
                    || project_dir.join("wsgi.py").exists()
                {
                    candidates.push("flask");
                } else if project_dir.join("main.py").exists()
                    && read_contains(project_dir.join("main.py"), "fastapi")
                {
                    candidates.push("fastapi");
                }
                if let Ok(reqs) = std::fs::read_to_string(project_dir.join("requirements.txt")) {
                    let l = reqs.to_ascii_lowercase();
                    if l.contains("django") {
                        candidates.push("django");
                    }
                    if l.contains("flask") {
                        candidates.push("flask");
                    }
                    if l.contains("fastapi") {
                        candidates.push("fastapi");
                    }
                }
            }
            crate::model::Languages::Rust => {
                if let Ok(cargo) = std::fs::read_to_string(project_dir.join("Cargo.toml")) {
                    if cargo.contains("axum")
                        || cargo.contains("actix-web")
                        || cargo.contains("warp")
                    {
                        candidates.push("rust-service");
                    }
                }
            }
        }

        if let Some(name) = candidates.first() {
            let bp_name = name.to_string();
            let _ = Self::create(&bp_name);
            let _ = Self::apply_to_project(&bp_name, project_dir);
            return Ok(Some(bp_name));
        }
        Ok(None)
    }
}

fn read_contains(path: PathBuf, needle: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|c| c.contains(needle))
        .unwrap_or(false)
}
