// Project commands shared between the CLI and the shell
//
// run/test are available for every supported language and dispatch to the
// native tool of the project (cargo / uv / flutter / npm). dev/build are
// Flutter-only: dev runs the app for preview, build asks for a target
// platform and builds the app for it.

use std::process::Command;

use anyhow::{Result, anyhow, bail};
use dialoguer::Select;

use crate::errors::MarkerErrors;
use crate::marker::{GenesisMarker, MARKER_FILE};

// Loads the marker of the project in the current directory
pub fn current_marker() -> Result<GenesisMarker> {
    let cwd = std::env::current_dir()?;
    if !GenesisMarker::is_genesis_project(&cwd) {
        return Err(MarkerErrors::NotAGenesisProject(MARKER_FILE.to_string()).into());
    }
    Ok(GenesisMarker::load(&cwd)?)
}

// Runs the project with the native tool of its language
pub fn run_project(marker: &GenesisMarker) -> Result<()> {
    let (tool, args) = match marker.language.as_str() {
        "rust" => ("cargo", vec!["run"]),
        "python" => ("uv", vec!["run"]),
        "flutter" => ("flutter", vec!["run"]),
        "javascript" => ("npm", vec!["expo", "start"]),
        other => bail!("No run command for language '{other}'"),
    };
    exec(tool, &args)
}

// Runs the test suite with the native tool of its language
pub fn test_project(marker: &GenesisMarker) -> Result<()> {
    let (tool, args) = match marker.language.as_str() {
        "rust" => ("cargo", vec!["test"]),
        "python" => ("uv", vec!["test"]),
        "flutter" => ("flutter", vec!["test"]),
        other => bail!("No test command for language '{other}'"),
    };
    exec(tool, &args)
}

// Runs the Flutter app in dev mode for preview
pub fn dev_project(marker: &GenesisMarker) -> Result<()> {
    if marker.language != "flutter" {
        bail!("'dev' is only supported for Flutter projects");
    }
    exec("flutter", &["run"])
}

// Asks for a platform and builds the Flutter app for it
pub fn build_project(marker: &GenesisMarker) -> Result<()> {
    if marker.language != "flutter" {
        bail!("'build' is only supported for Flutter projects");
    }

    let platforms = [
        "apk",
        "appbundle",
        "web",
        "linux",
        "windows",
        "macos",
        "ios",
    ];
    let selection = Select::new()
        .with_prompt("Select Platform")
        .items(&platforms)
        .interact()?;

    exec("flutter", &["build", platforms[selection]])
}

// Runs a tool, inheriting the terminal, and fails if it exits unsuccessfully
fn exec(tool: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(tool)
        .args(args)
        .status()
        .map_err(|e| anyhow!("Could not run '{tool}': {e}"))?;

    if !status.success() {
        bail!(
            "'{tool} {}' failed with exit code {:?}",
            args.join(" "),
            status.code()
        );
    }
    Ok(())
}
