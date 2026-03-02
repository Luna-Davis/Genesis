use std::path::Path;
use std::process::{Command, Stdio};

use crate::model::Languages;
use crate::run::start_spinner;
use crate::scaffold::config::GenesisFile;
use md5;
#[cfg(unix)]

pub fn bootstrap(project_dir: &Path, cfg: &GenesisFile) -> Result<(), String> {
    match cfg.language {
        Languages::Python => bootstrap_python(project_dir),
        Languages::Rust => bootstrap_rust(project_dir),
    }
}

fn bootstrap_python(project_dir: &Path) -> Result<(), String> {
    let py_bin = crate::run::get_python_bin(project_dir);
    let shared = shared_venv_path(&py_bin);
    if !shared.exists() {
        create_shared_venv(&shared, &py_bin)?;
    }

    let project_venv = project_dir.join(".venv");
    if !project_venv.exists() {
        clone_venv(&shared, &project_venv)?;
    }

    // Install project deps quietly via uv if requirements.txt exists
    let req = project_dir.join("requirements.txt");
    if req.exists() {
        let spinner = start_spinner("Syncing requirements via uv".into());
        let status = Command::new("uv")
            .args(["pip", "install", "-r"])
            .arg(&req)
            .env("UV_PYTHON", &py_bin)
            .current_dir(project_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| "uv not installed?")?;
        spinner.finish_and_clear();
        if !status.success() {
            return Err("uv install failed (is uv installed?)".into());
        }
    }

    Ok(())
}

fn bootstrap_rust(project_dir: &Path) -> Result<(), String> {
    // Warm cargo registry and fetch deps
    let spinner = start_spinner("Fetching cargo deps".into());
    let status = Command::new("cargo")
        .args(["fetch"])
        .current_dir(project_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;
    spinner.finish_and_clear();
    if !status.success() {
        return Err("cargo fetch failed".into());
    }
    Ok(())
}

fn shared_venv_path(py_bin: &str) -> std::path::PathBuf {
    let hash = format!("{:x}", md5::compute(py_bin.as_bytes()));
    dirs::cache_dir()
        .unwrap_or(std::path::PathBuf::from("/tmp"))
        .join("genesis-shared-venv")
        .join(hash)
}

fn create_shared_venv(path: &std::path::Path, py_bin: &str) -> Result<(), String> {
    std::fs::create_dir_all(path.parent().unwrap_or_else(|| std::path::Path::new(".")))
        .map_err(|e| e.to_string())?;
    let spinner = start_spinner("Creating shared Python venv (uv)".into());
    let status = Command::new("uv")
        .args(["venv", "--python"])
        .arg(py_bin)
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .or_else(|_| {
            // fallback to python -m venv
            Command::new(py_bin)
                .args(["-m", "venv"])
                .arg(path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        })
        .map_err(|e| e.to_string())?;
    spinner.finish_and_clear();
    if !status.success() {
        return Err("failed to create shared venv".into());
    }
    Ok(())
}

fn clone_venv(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if dst.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(dst.parent().unwrap_or_else(|| std::path::Path::new(".")))
        .map_err(|e| e.to_string())?;
    // Use cp -a for speed; fallback to std copy if unavailable.
    let status = Command::new("cp")
        .args([
            "-a",
            src.to_str().unwrap_or_default(),
            dst.to_str().unwrap_or_default(),
        ])
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            // naive copy
            fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
                std::fs::create_dir_all(dst)?;
                for entry in std::fs::read_dir(src)? {
                    let entry = entry?;
                    let file_type = entry.file_type()?;
                    let src_path = entry.path();
                    let dst_path = dst.join(entry.file_name());
                    if file_type.is_dir() {
                        copy_dir(&src_path, &dst_path)?;
                    } else if file_type.is_symlink() {
                        let target = std::fs::read_link(&src_path)?;
                        #[cfg(unix)]
                        {
                            std::os::unix::fs::symlink(target, dst_path)?;
                        }
                        #[cfg(not(unix))]
                        {
                            std::fs::copy(&src_path, &dst_path)?;
                        }
                    } else {
                        std::fs::copy(&src_path, &dst_path)?;
                    }
                }
                Ok(())
            }
            copy_dir(src, dst).map_err(|e| e.to_string())
        }
    }
}
