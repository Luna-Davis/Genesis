use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use chrono::Utc;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LockError {
    #[error("Failed to determine data directory")]
    DataDirNotFound,
    #[error("Lock already held by pid {pid} for project {project_id}")]
    AlreadyLocked { pid: u32, project_id: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] toml::de::Error),
    #[error("Serialization error: {0}")]
    SerdeSer(#[from] toml::ser::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockInfo {
    pub pid: u32,
    pub project_id: String,
    pub timestamp: i64,
}

const LOCK_FILE_NAME: &str = "genesis.lock";
const STALE_SECS: u64 = 60 * 60 * 6; // 6 hours

pub struct LockManager;

impl LockManager {
    fn lock_path() -> Result<PathBuf, LockError> {
        let base = data_dir()
            .ok_or(LockError::DataDirNotFound)?
            .join("genesis");
        fs::create_dir_all(&base)?;
        Ok(base.join(LOCK_FILE_NAME))
    }

    /// Acquire the global lock for a given project id.
    pub fn acquire(project_id: &str) -> Result<LockInfo, LockError> {
        let path = Self::lock_path()?;

        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    let info = LockInfo {
                        pid: std::process::id(),
                        project_id: project_id.to_string(),
                        timestamp: Utc::now().timestamp(),
                    };
                    let data = toml::to_string(&info)?;
                    file.write_all(data.as_bytes())?;
                    return Ok(info);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Evaluate stale or active lock
                    let existing = Self::read_lock()?;
                    if Self::is_pid_running(existing.pid) && !Self::is_stale(&path)? {
                        return Err(LockError::AlreadyLocked {
                            pid: existing.pid,
                            project_id: existing.project_id,
                        });
                    }
                    // stale; remove and retry
                    fs::remove_file(&path)?;
                    continue;
                }
                Err(e) => return Err(LockError::Io(e)),
            }
        }
    }

    /// Release the lock if it belongs to the caller's project or is stale.
    pub fn release(expected_project_id: Option<&str>) -> Result<(), LockError> {
        let path = Self::lock_path()?;
        if !path.exists() {
            return Ok(());
        }
        let existing = Self::read_lock()?;

        // If a specific project is expected, only release if it matches or the holder is dead.
        if let Some(proj) = expected_project_id {
            if existing.project_id != proj && Self::is_pid_running(existing.pid) {
                return Ok(());
            }
        } else if existing.pid != std::process::id() && Self::is_pid_running(existing.pid) {
            // Do not remove a live lock held by another process when no project was specified.
            return Ok(());
        }

        // Remove if stale or matches expectation/current pid.
        if !Self::is_pid_running(existing.pid)
            || expected_project_id
                .map(|p| p == existing.project_id)
                .unwrap_or(true)
            || existing.pid == std::process::id()
        {
            let _ = fs::remove_file(path);
        }
        Ok(())
    }

    fn read_lock() -> Result<LockInfo, LockError> {
        let path = Self::lock_path()?;
        let contents = fs::read_to_string(&path)?;
        let info: LockInfo = toml::from_str(&contents)?;
        Ok(info)
    }

    fn is_pid_running(pid: u32) -> bool {
        #[cfg(target_family = "unix")]
        {
            let path = PathBuf::from(format!("/proc/{pid}"));
            path.exists()
        }
        #[cfg(not(target_family = "unix"))]
        {
            // Best-effort fallback: assume stale to avoid permanent lockouts.
            false
        }
    }

    fn is_stale(path: &PathBuf) -> Result<bool, LockError> {
        let meta = fs::metadata(path)?;
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let age = mtime.elapsed().unwrap_or(Duration::from_secs(u64::MAX));
        Ok(age.as_secs() > STALE_SECS)
    }
}
