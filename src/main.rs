mod command_handler;
mod errors;
mod projects;
mod shell;

use anyhow::Result;
use clap::Command;

use crate::projects::{create_project, initialize_project};
use shell::shell;

fn main() -> Result<()> {
    let matches = Command::new("Genesis")
        .author("Mr. Lunatic")
        .version("1.0.0")
        .about("Project Manager")
        .subcommand(Command::new("new").about("Create new projects"))
        .subcommand(Command::new("init").about("Initialize an existing project"))
        .subcommand(Command::new("shell").about("Genesis shell"))
        .get_matches();

    match matches.subcommand() {
        Some(("new", _)) => create_project(),
        Some(("init", _)) => {
            initialize_project().expect("Internal Error, Could not initialize project")
        }
        Some(("shell", _)) => shell(),
        _ => println!("No command provided. Use --help to display help"),
    }

    Ok(())
}
