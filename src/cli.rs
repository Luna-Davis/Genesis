use std::env;
use std::str::FromStr;

use chrono::DateTime;
use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::file_manager::task_selector;
use crate::model::Languages;

use crate::db::{Database, DbError};
use crate::scaffold;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        name: String,
        language: String,
    },

    Resume,

    Stop,

    List,

    Delete {
        name: String,
        #[arg(long)]
        id: Option<String>,
    },

    New {
        name: String,
    },

    Remove {
        name: String,
    },
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
                let id = Uuid::new_v4().to_string();

                scaffold::selector(&id, name.clone(), &language)?;
                db.add_project(&id, &name, language, location)?;
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

            Commands::Delete { name, id } => {
                if let Some(id) = id {
                    let projects = db.get_project(&id)?;
                    let project = projects.first().ok_or(DbError::NotFound)?;
                    db.delete_project(project)?;
                    return Ok(());
                }

                let matches = db.get_project(&name)?;

                match matches.len() {
                    1 => {
                        db.delete_project(&matches[0])?;
                        Ok(())
                    }
                    n if n > 1 => {
                        eprintln!("Multiple projects named '{name}' found. Re-run with --id <uuid> to delete a specific one:");
                        for p in matches {
                            eprintln!(
                                "- id: {} | language: {} | location: {}",
                                p.id, p.language, p.location
                            );
                        }
                        Ok(())
                    }
                    _ => Err(DbError::NotFound.into()),
                }
            }

            Commands::New { name } => {
                task_selector(crate::file_manager::Task::Create, &name)?;
                Ok(())
            }

            Commands::Remove { name } => {
                task_selector(crate::file_manager::Task::Delete, &name)?;
                Ok(())
            }
        }
    }
}
