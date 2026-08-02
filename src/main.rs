mod command_handler;
mod commands;
mod errors;
mod marker;
mod projects;
mod shell;

use anyhow::Result;
use clap::Command;

use crate::errors::InitializationErrors;
use crate::errors::InitializationErrors::{NoCommandProvided, ProjectInitializationError};
use crate::projects::{create_project, initialize_project};
use shell::shell;

fn main() -> Result<(), InitializationErrors> {
    let matches = Command::new("Genesis")
        .author("Mr. Lunatic")
        .version("1.0.0")
        .about("Project Manager")
        .subcommand(Command::new("new").about("Create new projects"))
        .subcommand(Command::new("init").about("Initialize an existing project"))
        .subcommand(Command::new("shell").about("Genesis shell"))
        .get_matches();

    let _ = match matches.subcommand() {
        Some(("new", _)) => create_project(),
        Some(("init", _)) => Ok({
            initialize_project().map_err(|e| ProjectInitializationError(e.to_string()))?;
        }),
        Some(("shell", _)) => Ok(shell()),
        _ => return Err(NoCommandProvided.into()),
    };

    Ok(())
}
