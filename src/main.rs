mod blueprint;
mod bootstrap;
mod bundler;
mod ci;
mod cli;
mod db;
mod file_manager;
mod git_automation;
mod install;
mod model;
mod run;
mod scaffold;
mod utilities;
mod watcher;

use crate::cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Cli::cli()?;
    Ok(())
}
