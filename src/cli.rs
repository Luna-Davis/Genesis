use std::env;
use std::str::FromStr;

use chrono::DateTime;
use clap::{Parser, Subcommand};

use crate::model::Languages;

use crate::db::{Database, DbError};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start { name: String, language: String },

    Resume,

    Stop,

    List,
}

impl Cli {
    pub fn cli() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();
        let mut db = Database::new().expect("Failed to initialize database");

        match cli.command {
            Commands::Start { name, language } => {
                let location = env::current_dir().expect("Failed get current directory");
                let location = location
                    .to_str()
                    .expect("Failed to convert to string hliteral");
                let language = Languages::from_str(&language)?;
                db.add_project(&name, language, location)?;
                Ok(())
            }
            Commands::Resume => {
                match db.resume_project() {
                    Ok(project) => println!(
                        "Resuming: {}\nIn: {}\nStatus: {}",
                        project.name, project.location, project.status
                    ),
                    Err(DbError::NotFound) => eprintln!("No recent projects to resume"),
                    Err(e) => eprintln!("Unexpected error: {}", e),
                }
                Ok(())
            }
            Commands::Stop => {
                db.stop_project()?;
                Ok(())
            }
            Commands::List => {
                // Call list projects
                // Handle if empty
                // Loop through projects
                // Format timestaps
                let projects = db.list_projects().expect("Failed to list projects");

                if projects.is_empty() {
                    println!("No projects found.");
                    return Ok(());
                }

                println!("---- Projects ----");
                for project in projects {
                    let creation = DateTime::from_timestamp(project.creation_date, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "Invalid Date".to_string());

                    let last_active = DateTime::from_timestamp(project.last_active_date, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "Invalid Date".to_string());
                    println!(
                    "Name: {}\nLanguage: {}\nLocation: {}\nCreated: {}\nLast Active: {}\nStatus: {}\n",
                    project.name,
                    project.language,
                    project.location,
                    creation,
                    last_active,
                    project.status
                    );
                    println!("----------------");
                }
                Ok(())
            }
        }
    }
}
