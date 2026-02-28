mod cli;
mod db;
mod model;
mod scarfold;

use crate::cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Cli::cli()?;
    Ok(())
}
