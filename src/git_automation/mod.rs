use std::path::Path;

use git2::{IndexAddOption, Repository, Status, StatusOptions};
use pathdiff::diff_paths;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git repository not found at or above {0}")]
    NotFound(String),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
}

#[derive(Debug)]
pub struct ChangeSummary {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub renamed: Vec<String>,
    pub untracked: Vec<String>,
}

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open_from(path: &Path) -> Result<Self, GitError> {
        let repo = Repository::discover(path)
            .map_err(|_| GitError::NotFound(path.display().to_string()))?;
        Ok(Self { repo })
    }

    #[allow(dead_code)]
    pub fn ensure_initialized(path: &Path) -> Result<Self, GitError> {
        if let Ok(repo) = Repository::discover(path) {
            return Ok(Self { repo });
        }
        let repo = Repository::init(path)?;
        Ok(Self { repo })
    }

    pub fn status_summary(&self, workdir: &Path) -> Result<ChangeSummary, GitError> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let mut summary = ChangeSummary {
            added: vec![],
            modified: vec![],
            deleted: vec![],
            renamed: vec![],
            untracked: vec![],
        };

        let statuses = self.repo.statuses(Some(&mut opts))?;
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry
                .head_to_index()
                .and_then(|d| d.new_file().path())
                .or_else(|| entry.index_to_workdir().and_then(|d| d.new_file().path()))
                .or_else(|| entry.index_to_workdir().and_then(|d| d.old_file().path()))
                .and_then(|p| self.to_relative(workdir, p))
                .unwrap_or_else(|| "<unknown>".to_string());

            if status.contains(Status::WT_NEW) {
                summary.untracked.push(path.clone());
            }
            if status.contains(Status::WT_MODIFIED) || status.contains(Status::INDEX_MODIFIED) {
                summary.modified.push(path.clone());
            }
            if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED) {
                summary.deleted.push(path.clone());
            }
            if status.contains(Status::INDEX_NEW) {
                summary.added.push(path.clone());
            }
            if status.contains(Status::INDEX_RENAMED) || status.contains(Status::WT_RENAMED) {
                summary.renamed.push(path.clone());
            }
        }

        Ok(summary)
    }

    pub fn stage_all(&self) -> Result<(), GitError> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<String, GitError> {
        let sig = self.repo.signature()?;
        let oid = {
            let mut index = self.repo.index()?;
            let tree_id = index.write_tree()?;
            let tree = self.repo.find_tree(tree_id)?;

            let head = self.repo.head().ok();
            let parent = head
                .and_then(|h| h.target())
                .and_then(|oid| self.repo.find_commit(oid).ok());

            match parent {
                Some(p) => self
                    .repo
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&p])?,
                None => self
                    .repo
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?,
            }
        };
        Ok(oid.to_string())
    }

    fn to_relative(&self, workdir: &Path, path: &Path) -> Option<String> {
        let repo_workdir = self.repo.workdir()?;
        let absolute = repo_workdir.join(path);
        diff_paths(&absolute, workdir).map(|p| p.to_string_lossy().to_string())
    }
}

/// Generate a deterministic commit message based on file changes.
pub fn generate_commit_message(summary: &ChangeSummary) -> String {
    if !summary.deleted.is_empty() && summary.added.is_empty() && summary.modified.is_empty() {
        if summary.deleted.len() == 1 {
            return format!("chore: remove {}", summary.deleted[0]);
        }
        return "chore: remove files".into();
    }

    if !summary.added.is_empty() && summary.modified.is_empty() && summary.deleted.is_empty() {
        if summary.added.len() == 1 {
            return format!("feat: add {}", summary.added[0]);
        }
        return "feat: add files".into();
    }

    if !summary.modified.is_empty() && summary.deleted.is_empty() && summary.added.is_empty() {
        // Detect test focus
        let tests_only = summary
            .modified
            .iter()
            .all(|p| p.contains("test") || p.contains("tests"));
        if tests_only {
            return "test: update tests".into();
        }
        if summary.modified.len() == 1 {
            return format!("chore: update {}", summary.modified[0]);
        }
        return "chore: update files".into();
    }

    // Mixed changes
    "chore: update project".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary_with(
        added: &[&str],
        modified: &[&str],
        deleted: &[&str],
        untracked: &[&str],
    ) -> ChangeSummary {
        ChangeSummary {
            added: added.iter().map(|s| s.to_string()).collect(),
            modified: modified.iter().map(|s| s.to_string()).collect(),
            deleted: deleted.iter().map(|s| s.to_string()).collect(),
            renamed: vec![],
            untracked: untracked.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn message_for_single_add() {
        let s = summary_with(&["foo.rs"], &[], &[], &[]);
        assert_eq!(generate_commit_message(&s), "feat: add foo.rs");
    }

    #[test]
    fn message_for_single_delete() {
        let s = summary_with(&[], &[], &["foo.rs"], &[]);
        assert_eq!(generate_commit_message(&s), "chore: remove foo.rs");
    }

    #[test]
    fn message_for_tests_only() {
        let s = summary_with(&[], &["tests/foo.rs"], &[], &[]);
        assert_eq!(generate_commit_message(&s), "test: update tests");
    }

    #[test]
    fn message_for_mixed_changes() {
        let s = summary_with(&["a"], &["b"], &[], &["c"]);
        assert_eq!(generate_commit_message(&s), "chore: update project");
    }
}
