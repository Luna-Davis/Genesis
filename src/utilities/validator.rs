use std::env;
use std::path::Path;

pub fn validator() -> bool {
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return false,
    };

    let genesis_dir = Path::new(&current_dir).join(".genesis");

    genesis_dir.exists() && genesis_dir.is_dir()
}
