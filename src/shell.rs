use std::io::{self, Write};

use crate::command_handler::command_handler;
use crate::marker::GenesisMarker;

pub fn shell() {
    // Only allows the shell to run inside a Genesis project
    match std::env::current_dir() {
        Ok(cwd) if GenesisMarker::is_genesis_project(&cwd) => {}
        _ => {
            eprintln!(
                "This directory is not a Genesis project. Run 'gen init' to initialize it first."
            );
            return;
        }
    }

    loop {
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Could not read input.");

        let command = input.trim();
        let _ = command_handler(&command);
    }
}
