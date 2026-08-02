use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreationErrors {
    #[error("Internal Error: Could not create {0}")]
    ProjectCreationError(String),

    #[error("Internal Error: Language Not Supported")]
    LanguageNotSupported,
}

#[derive(Error, Debug)]
pub enum InitializationErrors {
    #[error("Internal Error: Could not initialize project at {0}")]
    ProjectInitializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Walkdir error: {0}")]
    WalkDirError(#[from] walkdir::Error),

    #[error("No command provided. Use --help to display help")]
    NoCommandProvided,
}

#[derive(Error, Debug)]
pub enum FileManagementErrors {
    #[error("Could Not Find File")]
    FileNotFound,

    #[error("Operation Not supported")]
    OperationNotSupported,
}

#[derive(Error, Debug)]
pub enum ModuleManagementErrors {
    #[error("Could Not Find Module")]
    ModuleNotFound,

    #[error("Operation Not supported")]
    OperationNotSupported,
}

#[derive(Error, Debug)]
pub enum MarkerErrors {
    #[error("Directory is not a Genesis project: missing {0}")]
    NotAGenesisProject(String),

    #[error("Marker file belongs to a different tool: expected '{0}'")]
    WrongTool(String),

    #[error("Unsupported schema version {0}. Expected {1}")]
    UnsupportedSchemaVersion(u32, u32),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}
