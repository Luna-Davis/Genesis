use crate::projects::{create_project, file_handler, initialize_project, module_handler};

const COMMANDS: &[(&str, &str)] = &[
    ("new", "Create a new project"),
    ("init", "Initialize an existing project"),
    ("module", "Manage modules"),
    ("file", "Manage files"),
    ("help", "Show this help message"),
];

pub fn command_handler(command: &str) {
    // Handles a command entered and matches it to the expected function
    match command {
        "new" => create_project(),
        "init" => initialize_project().expect("Could not initialize project"),
        "module" => module_handler(),
        "file" => file_handler(),
        "help" => show_help(),
        _ => println!("Command not supported"),
    }
}

fn show_help() {
    println!("Available commands:");
    for (cmd, desc) in COMMANDS {
        println!("  {cmd:<8}{desc}");
    }
}
