use std::env;
use std::path::{Path, PathBuf};

pub fn validator() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;

    let genesis_dir = Path::new(&current_dir).join(".genesis");

    if genesis_dir.exists() && genesis_dir.is_dir() {
        Some(current_dir)
    } else {
        None
    }
}
