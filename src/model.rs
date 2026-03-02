use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Invalid Status")]
    InvalidStatus,

    #[error("Invalid language")]
    InvalidLanguage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    New,
    InProgress,
    Finished,
}

impl FromStr for Status {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "New" => Ok(Status::New),
            "InProgress" => Ok(Status::InProgress),
            "Finished" => Ok(Status::Finished),
            _ => Err(ModelError::InvalidStatus),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::New => write!(f, "New"),
            Status::InProgress => write!(f, "In Progress"),
            Status::Finished => write!(f, "Finished"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Languages {
    Python,
    Rust,
}

impl FromStr for Languages {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Python" => Ok(Languages::Python),
            "Rust" => Ok(Languages::Rust),
            _ => Err(ModelError::InvalidLanguage),
        }
    }
}

impl std::fmt::Display for Languages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Languages::Python => write!(f, "Python"),
            Languages::Rust => write!(f, "Rust"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub location: PathBuf,
    pub language: Languages,
    pub creation_date: DateTime<Utc>,
    pub last_active_date: DateTime<Utc>,
    #[allow(dead_code)]
    pub is_lock: bool,
    pub status: Status,
}
