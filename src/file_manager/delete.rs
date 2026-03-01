use std::fs;

use crate::utilities::validator::validator;

pub fn deleter(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !validator() {
        return Err("project doesn't contain .genesis folder".into());
    }
    fs::remove_file(name)?;
    Ok(())
}
