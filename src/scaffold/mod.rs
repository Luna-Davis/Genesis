use crate::model::Languages;

mod config;
mod python;
mod rust;

pub fn selector(
    id: &str,
    name: String,
    language: &Languages,
) -> Result<(), Box<dyn std::error::Error>> {
    match language {
        Languages::Rust => rust::scaffold(id, &name),
        Languages::Python => python::scaffold(id, &name),
    }
}
