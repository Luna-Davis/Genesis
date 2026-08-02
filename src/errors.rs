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
