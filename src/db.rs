use std::path::PathBuf;
use std::{fs, str::FromStr};

use chrono::Utc;
use dirs::data_dir;
use rusqlite::{Connection, Result as SqlResult};
use thiserror::Error;

use crate::model::{Languages, Status};

struct Migration {
    version: i64,
    apply: fn(&Connection) -> SqlResult<()>,
}

pub struct Database {
    conn: Connection,
}

#[derive(Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub language: Languages,
    pub location: String,
    pub creation_date: i64,
    pub last_active_date: i64,
    #[allow(dead_code)]
    pub is_lock: bool,
    pub status: Status,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to determine data directory")]
    DataDirNotFound,

    #[error("Database error: {0}")]
    Sql(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Project not found")]
    NotFound,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    apply: |conn| {
        conn.execute(
            r#"
                CREATE TABLE IF NOT EXISTS projects (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    language TEXT NOT NULL,
                    location TEXT NOT NULL,
                    creation_date INTEGER NOT NULL,
                    last_active_date INTEGER NOT NULL,
                    is_lock INTEGER NOT NULL CHECK (is_lock IN (0, 1)),
                    status TEXT NOT NULL CHECK (status IN ('New', 'InProgress', 'Finished'))
                )
                "#,
            [],
        )?;
        Ok(())
    },
}];

impl Database {
    pub fn new() -> Result<Self, DbError> {
        let base_dir = data_dir().ok_or(DbError::DataDirNotFound)?;
        let data_path = base_dir.join("genesis");
        std::fs::create_dir_all(&data_path)?;

        let db_path = data_path.join("database.db");
        let mut conn = Connection::open(db_path)?;

        migrate(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn add_project(
        &self,
        id: &str,
        name: &str,
        language: Languages,
        location: &str,
    ) -> Result<(), DbError> {
        let now = Utc::now().timestamp();

        let language_str = format!("{:?}", language);

        self.conn.execute(
            r#"
            INSERT INTO projects (
                id,
                name,
                language,
                location,
                creation_date,
                last_active_date,
                is_lock,
                status
            )
            VALUES (?, ?, ?, ?, ?, ?, 0, 'New')
            "#,
            (id, name, language_str, location, now, now),
        )?;

        Ok(())
    }

    pub fn delete_project(&self, project: &Project) -> Result<(), DbError> {
        validate_project_dir(project)?;

        let project_dir = resolve_project_dir(project);

        if project_dir.exists() {
            fs::remove_dir_all(&project_dir)?;
        }

        let affected = self
            .conn
            .execute("DELETE FROM projects WHERE id=?", [&project.id])?;

        if affected == 0 {
            return Err(DbError::NotFound);
        }

        Ok(())
    }

    fn row_to_project(row: &rusqlite::Row) -> SqlResult<Project> {
        let language_str: String = row.get(2)?;
        let status_str: String = row.get(7)?;

        let language = Languages::from_str(&language_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let status = Status::from_str(&status_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
        })?;

        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            language,
            location: row.get(3)?,
            creation_date: row.get(4)?,
            last_active_date: row.get(5)?,
            is_lock: row.get::<_, i64>(6)? == 1,
            status,
        })
    }

    pub fn get_project(&self, id_or_name: &str) -> Result<Vec<Project>, DbError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, language, location,
               creation_date, last_active_date,
               is_lock, status
            FROM projects
            WHERE id = ?1 OR name = ?1
            "#,
        )?;

        let rows = stmt.query_map([id_or_name], |row| Database::row_to_project(row))?;

        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }

        if projects.is_empty() {
            return Err(DbError::NotFound);
        }

        Ok(projects)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, DbError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, language, location,
               creation_date, last_active_date,
               is_lock, status
            FROM projects
            "#,
        )?;

        let rows = stmt.query_map([], |row| Database::row_to_project(row))?;

        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }

        Ok(projects)
    }

    pub fn resume_project(&mut self) -> Result<Project, DbError> {
        let now = Utc::now().timestamp();

        let tx = self.conn.transaction()?;

        let id: String = tx
            .query_row(
                "SELECT id FROM projects ORDER BY last_active_date DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .map_err(|_| DbError::NotFound)?;

        tx.execute(
            "UPDATE projects SET is_lock = 1, last_active_date = ? WHERE id = ?",
            (now, &id),
        )?;

        let project = tx.query_row(
            r#"SELECT id, name, language, location, creation_date, last_active_date, is_lock, status
               FROM projects WHERE id = ?"#,
            [&id],
            |row| Database::row_to_project(row),
        )?;

        tx.commit()?;

        Ok(project)
    }

    pub fn stop_project(&mut self) -> Result<(), DbError> {
        let now = Utc::now().timestamp();
        let tx = self.conn.transaction()?;

        let affected = tx.execute(
            "UPDATE projects SET last_active_date = ?, is_lock = 0 WHERE is_lock = 1",
            [now],
        )?;

        // If nothing was locked, treat as a no-op instead of bubbling an error.
        if affected == 0 {
            tx.commit()?;
            return Ok(());
        }

        tx.commit()?;
        Ok(())
    }
}

fn migrate(conn: &mut Connection) -> SqlResult<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL
        )
        "#,
        [],
    )?;

    let version: i64 =
        match conn.query_row("SELECT version FROM schema_version WHERE id = 1", [], |r| {
            r.get(0)
        }) {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                conn.execute("INSERT INTO schema_version (id, version) VALUES (1, 0)", [])?;
                0
            }
            Err(e) => return Err(e),
        };

    for migration in MIGRATIONS {
        if migration.version > version {
            let tx = conn.transaction()?;
            (migration.apply)(&tx)?;
            tx.execute(
                "UPDATE schema_version SET version = ? WHERE id = 1",
                [migration.version],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}

/// Determine the on-disk directory for a project.
/// Falls back to `<location>/<name>` when `location` points to the parent directory.
fn resolve_project_dir(project: &Project) -> PathBuf {
    let base = PathBuf::from(&project.location);

    match base.file_name() {
        Some(name) if name.to_string_lossy() == project.name => base,
        _ => base.join(&project.name),
    }
}

/// Ensure the project directory looks like a Genesis-managed project and matches the DB record.
fn validate_project_dir(project: &Project) -> Result<(), DbError> {
    let dir = resolve_project_dir(project);
    let config_path = dir.join(".genesis").join("config.toml");

    if !config_path.exists() {
        return Err(DbError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Missing .genesis/config.toml in project directory",
        )));
    }

    let contents = fs::read_to_string(&config_path)?;
    let cfg: crate::scaffold::config::GenesisFile = toml::from_str(&contents)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        .map_err(DbError::Io)?;

    if cfg.id != project.id {
        return Err(DbError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Project ID mismatch between DB and .genesis/config.toml",
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::config::{GenesisConfig, GenesisFile};
    use std::fs;

    fn temp_project(name: &str) -> (Project, PathBuf) {
        let root = std::env::temp_dir().join(format!("genesis-test-{}", name));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".genesis")).unwrap();

        let project = Project {
            id: "id-123".into(),
            name: name.into(),
            language: Languages::Rust,
            location: root.to_string_lossy().to_string(),
            creation_date: 0,
            last_active_date: 0,
            is_lock: false,
            status: Status::New,
        };
        (project, root)
    }

    #[test]
    fn resolve_uses_name_when_location_is_parent() {
        let mut project = Project {
            id: "id-1".into(),
            name: "proj".into(),
            language: Languages::Rust,
            location: "/tmp".into(),
            creation_date: 0,
            last_active_date: 0,
            is_lock: false,
            status: Status::New,
        };
        let path = resolve_project_dir(&project);
        assert!(path.ends_with("proj"));

        project.location = "/tmp/proj".into();
        let path2 = resolve_project_dir(&project);
        assert!(path2.ends_with("proj"));
    }

    #[test]
    fn validate_checks_id_in_config() {
        let (project, root) = temp_project("validate-ok");
        let cfg = GenesisFile {
            id: project.id.clone(),
            name: project.name.clone(),
            language: project.language.clone(),
            location: project.location.clone(),
            created_at: 0,
            version: "0.1.0".into(),
            scripts: std::collections::HashMap::new(),
        };
        GenesisConfig::write_existing(&cfg, &root).unwrap();
        assert!(validate_project_dir(&project).is_ok());
    }

    #[test]
    fn validate_rejects_mismatched_id() {
        let (mut project, root) = temp_project("validate-bad");
        let mut cfg = GenesisFile {
            id: "other".into(),
            name: project.name.clone(),
            language: project.language.clone(),
            location: project.location.clone(),
            created_at: 0,
            version: "0.1.0".into(),
            scripts: std::collections::HashMap::new(),
        };
        GenesisConfig::write_existing(&cfg, &root).unwrap();
        let res = validate_project_dir(&project);
        assert!(res.is_err());

        // fix and validate
        cfg.id = project.id.clone();
        GenesisConfig::write_existing(&cfg, &root).unwrap();
        assert!(validate_project_dir(&project).is_ok());
    }
}
