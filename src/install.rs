use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

use crate::model::Languages;
use crate::run::get_python_bin;
use crate::scaffold::config::GenesisConfig;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Installer failed with status {0}: {1}")]
    Failed(i32, String),
    #[error("Unsupported language for install")]
    Unsupported,
}

pub fn install_package(
    project_dir: &Path,
    lang: &Languages,
    package: &str,
) -> Result<(), InstallError> {
    match lang {
        Languages::Python => install_python(project_dir, package),
        Languages::Rust => install_rust(project_dir, package),
    }
}

fn install_python(project_dir: &Path, package: &str) -> Result<(), InstallError> {
    let python = get_python_bin(project_dir);

    // Prefer uv (rust-based, faster); fall back to pip if uv is absent.
    let mut cmd_uv = Command::new("uv");
    cmd_uv
        .args(["pip", "install", package])
        .current_dir(project_dir)
        .env("UV_PYTHON", &python)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let spinner = start_spinner(format!("Installing {package} (Python)"));
    let output = match cmd_uv.output() {
        Ok(out) => out,
        Err(_) => {
            // uv not available; try pip as a silent fallback
            let mut cmd_pip = Command::new(&python);
            cmd_pip
                .args(["-m", "pip", "install", package])
                .current_dir(project_dir)
                .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            cmd_pip.output()?
        }
    };
    spinner.finish_and_clear();

    let status = output.status.code().unwrap_or(-1);
    if output.status.success() {
        // Best-effort: append to requirements.txt so future installs are reproducible.
        let req_path = project_dir.join("requirements.txt");
        if let Ok(mut existing) = std::fs::read_to_string(&req_path) {
            if !existing.lines().any(|l| l.trim() == package) {
                existing.push('\n');
                existing.push_str(package);
                let _ = std::fs::write(&req_path, existing);
            }
        } else {
            let _ = std::fs::write(&req_path, format!("{package}\n"));
        }
        maybe_add_web_scripts(project_dir, package);
        println!("✔ Installed {package}");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(InstallError::Failed(
            status,
            stderr.lines().take(6).collect::<Vec<_>>().join("\n"),
        ))
    }
}

fn install_rust(project_dir: &Path, package: &str) -> Result<(), InstallError> {
    // For project dependencies, prefer cargo-add if available; fall back to cargo install.
    let mut cmd = Command::new("cargo");
    cmd.args(["add", package])
        .current_dir(project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let spinner = start_spinner(format!("Installing {package} (Rust)"));
    let output = cmd.output()?;
    let mut success = output.status.success();
    let mut stderr_buf = String::from_utf8_lossy(&output.stderr).to_string();

    // Fallback: cargo add may be unavailable; try cargo install for tools.
    if !success {
        let mut fallback = Command::new("cargo");
        fallback
            .args(["install", package])
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let out2 = fallback.output()?;
        success = out2.status.success();
        stderr_buf = String::from_utf8_lossy(&out2.stderr).to_string();
    }

    spinner.finish_and_clear();

    if success {
        println!("✔ Installed {package}");
        Ok(())
    } else {
        let status = output.status.code().unwrap_or(-1);
        Err(InstallError::Failed(
            status,
            stderr_buf.lines().take(6).collect::<Vec<_>>().join("\n"),
        ))
    }
}

fn maybe_add_web_scripts(project_dir: &Path, package: &str) {
    let pkg_lower = package.to_ascii_lowercase();
    let mut cfg = match GenesisConfig::read_genesis(project_dir) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut changed = false;
    if pkg_lower.contains("django") {
        let manage = project_dir.join("manage.py");
        if manage.exists() && !cfg.scripts.contains_key("runserver") {
            cfg.scripts
                .insert("runserver".into(), "python manage.py runserver".into());
            changed = true;
        }
    } else if pkg_lower.contains("flask") {
        if !cfg.scripts.contains_key("runserver") {
            cfg.scripts.insert("runserver".into(), "flask run".into());
            changed = true;
        }
    }

    if changed {
        let _ = GenesisConfig::write_existing(&cfg, project_dir);
    }
}

fn start_spinner(msg: String) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb.set_message(msg);
    pb
}
