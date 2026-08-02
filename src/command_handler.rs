use anyhow::{Result, anyhow};

use crate::projects::{file_handler, module_handler};

const COMMANDS: &[(&str, &str)] = &[
    ("module", "Manage modules"),
    ("file", "Manage files"),
    ("help", "Show this help message"),
];

pub fn command_handler(command: &str) -> Result<()> {
    // Handles a command entered and matches it to the expected function
    match command {
        "module" => module_handler()?,
        "file" => file_handler()?,
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
