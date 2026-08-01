use std::io::{self, Write};

use crate::command_handler::command_handler;

pub fn shell() -> String {
    loop {
        println!(">");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();
        command_handler(&command);
    }
}
