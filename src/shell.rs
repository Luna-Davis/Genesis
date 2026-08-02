use std::io::{self, Write};

use crate::command_handler::command_handler;

pub fn shell() {
    loop {
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();
        let _ = command_handler(&command);
    }
}
