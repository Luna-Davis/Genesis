use std::fs;
use std::path::Path;

use crate::utilities::validator::validator;

pub fn deleter(target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = validator().ok_or("project doesn't contain .genesis folder")?;

    let target_path = Path::new(target);
    let resolved = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        project_dir.join(target_path)
    };

    if resolved.is_dir() {
        fs::remove_dir_all(&resolved)?;
    } else if resolved.is_file() {
        fs::remove_file(&resolved)?;
    } else {
        return Err(format!("Target '{}' does not exist", resolved.display()).into());
    }

    Ok(())
}
