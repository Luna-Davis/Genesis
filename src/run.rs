use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;

use crate::bundler;
use crate::model::Languages;
use crate::scaffold::config::GenesisConfig;
use dotenvy::from_path_iter;

#[derive(Debug, Error)]
pub enum RunError {
    #[error("Not a Genesis project (missing .genesis)")]
    NotGenesis,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed with status {0}")]
    Failed(i32),
    #[error("Unsupported operation for language")]
    Unsupported,
    #[error("Bundling error: {0}")]
    Bundle(String),
}

pub fn run_project(cwd: &Path) -> Result<(), RunError> {
    let cfg = read_config(cwd)?;
    match cfg.language {
        Languages::Rust => run_command(cwd, &["cargo", "run"])?,
        Languages::Python => {
            let python_bin = get_python_bin(cwd);
            // Prefer __main__.py in root, else fallback to src/main.py or src/<name>/main.py
            if cwd.join("__main__.py").exists() {
                run_command(cwd, &[&python_bin, "__main__.py"])?;
            } else {
                let safe_name = cfg.name.replace('-', "_");
                let candidate1 = format!("src/{}/main.py", safe_name);
                let candidate2 = "src/main.py".to_string();

                if cwd.join(&candidate1).exists() {
                    run_command(cwd, &[&python_bin, &candidate1])?;
                } else if cwd.join(&candidate2).exists() {
                    run_command(cwd, &[&python_bin, &candidate2])?;
                } else {
                    return Err(RunError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Could not find Python entry point (__main__.py or src/main.py)",
                    )));
                }
            }
        }
    };
    Ok(())
}

pub fn test_project(cwd: &Path) -> Result<(), RunError> {
    let cfg = read_config(cwd)?;
    match cfg.language {
        Languages::Rust => run_command(cwd, &["cargo", "test"])?,
        Languages::Python => {
            let python_bin = get_python_bin(cwd);
            run_command(
                cwd,
                &[&python_bin, "-m", "unittest", "discover", "-s", "tests"],
            )?;
        }
    };
    Ok(())
}

pub(crate) fn get_python_bin(cwd: &Path) -> String {
    let venv_bin = cwd.join(".venv").join("bin").join("python");
    if venv_bin.exists() {
        venv_bin.to_string_lossy().to_string()
    } else {
        "python3".to_string()
    }
}

/// Run an arbitrary shell script (used by pipelines).
pub fn run_script(cwd: &Path, script: &str) -> Result<(), RunError> {
    let spinner = start_spinner(format!("Running: {}", script));
    let start = Instant::now();
    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    spinner.finish_and_clear();
    let duration = start.elapsed();

    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        println!(
            "{} {} ({}s)",
            style("✔").green(),
            style("Passed").bold(),
            duration.as_secs_f32()
        );
        if !stdout.trim().is_empty() {
            println!("{}", style(stdout.trim()).dim());
        }
        Ok(())
    } else {
        println!(
            "{} {} ({}s)",
            style("✘").red(),
            style("Failed").bold(),
            duration.as_secs_f32()
        );
        if !stdout.trim().is_empty() {
            println!("{}", style(stdout.trim()).yellow());
        }
        if !stderr.trim().is_empty() {
            println!("{}", style(stderr.trim()).red());
        }
        Err(RunError::Failed(status))
    }
}

pub fn build_rust_project(cwd: &Path) -> Result<(), RunError> {
    run_command(cwd, &["cargo", "build", "--release"])
}

pub fn build_project(cwd: &Path) -> Result<(), RunError> {
    let cfg = read_config(cwd)?;
    match cfg.language {
        Languages::Rust => build_rust_project(cwd)?,
        Languages::Python => bundler::build_python_project(cwd, &cfg)
            .map_err(|e: crate::bundler::BundleError| RunError::Bundle(e.to_string()))?,
    };
    Ok(())
}

fn read_config(cwd: &Path) -> Result<crate::scaffold::config::GenesisFile, RunError> {
    let cfg_path = cwd.join(".genesis").join("config.toml");
    if !cfg_path.exists() {
        return Err(RunError::NotGenesis);
    }
    let cfg = GenesisConfig::read_genesis(cwd).map_err(|e| {
        RunError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    Ok(cfg)
}

fn run_command(cwd: &Path, args: &[&str]) -> Result<(), RunError> {
    if args.is_empty() {
        return Err(RunError::Unsupported);
    }
    let cmd = args[0];
    let rest = &args[1..];

    let spinner = start_spinner(format!("Running: {} {}", cmd, rest.join(" ")));
    let start = Instant::now();

    let mut command = Command::new(cmd);
    command
        .args(rest)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    inject_env(&mut command, cwd);
    let output = command.output()?;

    spinner.finish_and_clear();
    let duration = start.elapsed();

    let status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        println!(
            "{} {} ({}s)",
            style("✔").green(),
            style("Success").bold(),
            duration.as_secs_f32()
        );
        if !stdout.trim().is_empty() {
            println!("{}", style(stdout.trim()).dim());
        }
        Ok(())
    } else {
        println!(
            "{} {} ({}s)",
            style("✘").red(),
            style("Failed").bold(),
            duration.as_secs_f32()
        );
        if !stdout.trim().is_empty() {
            println!("{}", style(stdout.trim()).yellow());
        }
        if !stderr.trim().is_empty() {
            println!("{}", style(stderr.trim()).red());
        }
        Err(RunError::Failed(status))
    }
}

pub(crate) fn start_spinner(msg: String) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(msg);
    pb
}

fn inject_env(cmd: &mut Command, cwd: &Path) {
    let env_name = std::env::var("GENESIS_ENV").ok();
    let mut paths = vec![cwd.join(".env")];
    if let Some(env) = &env_name {
        paths.push(cwd.join(format!(".env.{env}")));
    }
    paths.push(cwd.join(".env.local"));

    for p in paths {
        if !p.exists() {
            continue;
        }
        if let Ok(iter) = from_path_iter(&p) {
            for item in iter.flatten() {
                cmd.env(item.0, item.1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn run_script_success() {
        let cwd = env::temp_dir();
        let res = run_script(&cwd, "echo hello");
        assert!(res.is_ok());
    }

    #[test]
    fn run_script_failure_returns_err() {
        let cwd = env::temp_dir();
        let res = run_script(&cwd, "exit 42");
        assert!(matches!(res, Err(RunError::Failed(42))));
    }
}
