use std::fs;

use crate::utilities::validator::validator;

pub fn creator(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !validator() {
        return Err("project doesn't contain .genesis folder".into());
    }
    fs::write(name, "")?;
    Ok(())
}
