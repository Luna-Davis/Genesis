use std::path::PathBuf;

#[derive(Debug)]
pub struct AnalysisResult {
    pub entry_vfs: String,
    pub py_files: Vec<FileEntryCandidate>,
}

#[derive(Debug, Clone)]
pub struct FileEntryCandidate {
    pub vfs_path: String,
    pub host_path: PathBuf,
}
