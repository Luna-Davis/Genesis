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
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        let command = input.trim();
        if command.is_empty() {
            continue;
        }

        match command {
            "exit" | "quit" => break,
            _ => {
                let _ = command_handler(&command);
            }
        }
    }
}
