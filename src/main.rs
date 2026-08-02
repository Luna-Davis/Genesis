mod command_handler;
mod commands;
mod errors;
mod marker;
mod projects;
mod shell;

use anyhow::Result;
use clap::Command;

use crate::commands::{build_project, current_marker, dev_project, run_project, test_project};
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
        .subcommand(Command::new("run").about("Run the project"))
        .subcommand(Command::new("test").about("Run the project tests"))
        .subcommand(Command::new("dev").about("Run the Flutter app for preview"))
        .subcommand(Command::new("build").about("Build the Flutter app for a platform"))
        .get_matches();

    let _ = match matches.subcommand() {
        Some(("new", _)) => create_project(),
        Some(("init", _)) => Ok({
            initialize_project().map_err(|e| ProjectInitializationError(e.to_string()))?;
        }),
        Some(("shell", _)) => Ok(shell()),
        Some(("run", _)) => Ok(run_cli(|| run_project(&current_marker()?))),
        Some(("test", _)) => Ok(run_cli(|| test_project(&current_marker()?))),
        Some(("dev", _)) => Ok(run_cli(|| dev_project(&current_marker()?))),
        Some(("build", _)) => Ok(run_cli(|| build_project(&current_marker()?))),
        _ => return Err(NoCommandProvided.into()),
    };

    Ok(())
}

// Runs a CLI command, printing any error to stderr and exiting with code 1
fn run_cli(f: impl FnOnce() -> anyhow::Result<()>) {
    match f() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
