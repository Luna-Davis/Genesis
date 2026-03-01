mod cli;
mod db;
mod file_manager;
mod git_automation;
mod model;
mod scaffold;
mod utilities;

use crate::cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Cli::cli()?;
    Ok(())
}
