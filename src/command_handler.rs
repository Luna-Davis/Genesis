use anyhow::{Result, anyhow};

use crate::commands::{build_project, current_marker, dev_project, run_project, test_project};
use crate::projects::{file_handler, module_handler};

const COMMANDS: &[(&str, &str)] = &[
    ("module", "Manage modules"),
    ("file", "Manage files"),
    ("run", "Run the project"),
    ("test", "Run the project tests"),
    ("dev", "Run the Flutter app for preview"),
    ("build", "Build the Flutter app for a platform"),
    ("help", "Show this help message"),
    ("exit", "Exit the shell"),
];

pub fn command_handler(command: &str) -> Result<()> {
    // Handles a command entered and matches it to the expected function
    match command {
        "module" => module_handler()?,
        "file" => file_handler()?,
        "run" => run_project(&current_marker()?)?,
        "test" => test_project(&current_marker()?)?,
        "dev" => dev_project(&current_marker()?)?,
        "build" => build_project(&current_marker()?)?,
        "help" => show_help(),
        _ => return Err(anyhow!("Command not supported")),
    }
    Ok(())
}

fn show_help() {
    println!("Available commands:");
    for (cmd, desc) in COMMANDS {
        println!("  {cmd:<8}{desc}");
    }
}
