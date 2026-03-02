use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::DateTime;
use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::file_manager::task_selector;
use crate::model::Languages;

use crate::blueprint::BlueprintStore;
use crate::ci;
use crate::db::{Database, DbError};
use crate::git_automation::{GitRepo, generate_commit_message};
use crate::install;
use crate::run;
use crate::scaffold;
use crate::scaffold::config::GenesisConfig;
use crate::scaffold::version::bump_version_str;
use crate::utilities::lock::LockManager;
use crate::utilities::validator::validator;
use crate::watcher::{DebouncedEvents, DebouncedWatcher};
use crossbeam_channel::unbounded;
use rustyline::{DefaultEditor, error::ReadlineError};
use std::io::{self, Read};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        name: String,
        language: String,
        #[arg(long)]
        blueprint: Option<String>,
    },

    Resume,

    Stop,

    Status,

    Run,

    Test,

    List,

    Delete {
        name: String,
        #[arg(long)]
        id: Option<String>,
    },

    New {
        name: String,
    },

    Remove {
        name: String,
    },

    Import {
        #[arg(long)]
        language: Option<String>,
        #[arg(long)]
        force: bool,
    },

    GitCommit {
        #[arg(long)]
        message: Option<String>,
    },

    Blueprint {
        #[command(subcommand)]
        cmd: BlueprintCmd,
    },

    Push,

    Tui,

    Watch {
        #[arg(long)]
        auto_commit: bool,
    },

    /// Launch interactive shell inside a Genesis project
    Shell,

    Build {
        #[arg(long, default_value = "patch")]
        bump: String,
    },

    /// Install a dependency for the current project (pip or cargo)
    Install {
        package: String,
        #[arg(long)]
        language: Option<String>,
    },

    /// Bootstrap project environment (venv/cargo fetch, shared venv reuse)
    Bootstrap,

    /// Emit CI config (currently GitHub Actions)
    Ci {
        #[arg(long, default_value = "github")]
        provider: String,
    },

    /// Show the built-in quickstart guide
    Guide,
}

#[derive(Subcommand)]
enum BlueprintCmd {
    New { name: String },
    List,
    Apply { name: String },
}

impl Cli {
    pub fn cli() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();
        let mut db = Database::new().expect("Failed to initialize database");

        // Default to shell if no subcommand provided
        let command = cli.command.unwrap_or(Commands::Shell);

        // Enforce project context for all commands except Start/Import/Shell (which self-validate)
        match &command {
            Commands::Start { .. } | Commands::Import { .. } | Commands::Shell => {}
            _ => {
                if validator().is_none() {
                    return Err("Not inside a Genesis project (.genesis missing). Run `genesis start ...` or `genesis import ...`, then `cd` into the project and run `genesis`.".into());
                }
            }
        }

        match command {
            Commands::Start {
                name,
                language,
                blueprint,
            } => {
                let cwd = env::current_dir().expect("Failed get current directory");
                let language = Languages::from_str(&language)?;
                let id = Uuid::new_v4().to_string();

                scaffold::selector(&id, name.clone(), &language)?;

                // Acquire global lock for this project
                LockManager::acquire(&id)?;

                let project_root = cwd
                    .join(&name)
                    .canonicalize()
                    .expect("Failed to resolve project directory");
                let location = project_root.to_string_lossy().to_string();

                db.add_project(&id, &name, language, &location)?;

                if let Some(bp) = blueprint {
                    BlueprintStore::apply_to_project(&bp, &project_root)?;
                    println!("Applied blueprint '{}' during start.", bp);
                } else {
                    if let Ok(Some(bp)) = BlueprintStore::auto_apply(
                        &project_root,
                        &GenesisConfig::read_genesis(&project_root)?,
                    ) {
                        println!("Auto-applied blueprint '{}'.", bp);
                    }
                }

                Ok(())
            }
            Commands::Resume => {
                match db.resume_project() {
                    Ok(project) => {
                        LockManager::acquire(&project.id)?;
                        println!(
                            "Resuming: {}\nIn: {}\nStatus: {}",
                            project.name, project.location, project.status
                        );
                    }
                    Err(DbError::NotFound) => eprintln!("No recent projects to resume"),
                    Err(e) => eprintln!("Unexpected error: {}", e),
                }
                Ok(())
            }
            Commands::Stop => {
                // release any existing lock
                let _ = LockManager::release(None);
                db.stop_project()?;
                Ok(())
            }
            Commands::Run => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                run::run_project(&project_dir)?;
                Ok(())
            }
            Commands::Test => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                run::test_project(&project_dir)?;
                Ok(())
            }
            Commands::Status => {
                if let Some(project_dir) = validator() {
                    let cfg = GenesisConfig::read_genesis(&project_dir)?;
                    println!(
                        "Active project detected in cwd:\nName: {}\nId: {}\nLanguage: {}\nLocation: {}",
                        cfg.name, cfg.id, cfg.language, cfg.location
                    );
                } else {
                    println!("No .genesis/config.toml found in current directory.");
                }
                Ok(())
            }
            Commands::List => {
                // Call list projects
                // Handle if empty
                // Loop through projects
                // Format timestaps
                let projects = db.list_projects().expect("Failed to list projects");

                if projects.is_empty() {
                    println!("No projects found.");
                    return Ok(());
                }

                println!("---- Projects ----");
                for project in projects {
                    let creation = DateTime::from_timestamp(project.creation_date, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "Invalid Date".to_string());

                    let last_active = DateTime::from_timestamp(project.last_active_date, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "Invalid Date".to_string());
                    println!(
                        "Name: {}\nLanguage: {}\nLocation: {}\nCreated: {}\nLast Active: {}\nStatus: {}\n",
                        project.name,
                        project.language,
                        project.location,
                        creation,
                        last_active,
                        project.status
                    );
                    println!("----------------");
                }
                Ok(())
            }

            Commands::Delete { name, id } => {
                if let Some(id) = id {
                    let projects = db.get_project(&id)?;
                    let project = projects.first().ok_or(DbError::NotFound)?;
                    db.delete_project(project)?;
                    // if deleting active project, release lock
                    let _ = LockManager::release(Some(&project.id));
                    return Ok(());
                }

                let matches = db.get_project(&name)?;

                match matches.len() {
                    1 => {
                        db.delete_project(&matches[0])?;
                        let _ = LockManager::release(Some(&matches[0].id));
                        Ok(())
                    }
                    n if n > 1 => {
                        eprintln!(
                            "Multiple projects named '{name}' found. Re-run with --id <uuid> to delete a specific one:"
                        );
                        for p in matches {
                            eprintln!(
                                "- id: {} | language: {} | location: {}",
                                p.id, p.language, p.location
                            );
                        }
                        Ok(())
                    }
                    _ => Err(DbError::NotFound.into()),
                }
            }

            Commands::New { name } => {
                task_selector(crate::file_manager::Task::Create, &name)?;
                Ok(())
            }

            Commands::Remove { name } => {
                task_selector(crate::file_manager::Task::Delete, &name)?;
                Ok(())
            }

            Commands::Import { language, force } => {
                let cwd = env::current_dir()?;
                let project_name = cwd
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or("Could not derive project name from current directory")?
                    .to_string();

                let genesis_cfg_path = cwd.join(".genesis").join("config.toml");

                if genesis_cfg_path.exists() {
                    let cfg = GenesisConfig::read_genesis(&cwd)?;

                    if cfg.name != project_name && !force {
                        return Err(format!(
                            "Config name '{}' differs from directory name '{}'. Re-run with --force to accept config as truth.",
                            cfg.name, project_name
                        )
                        .into());
                    }

                    let lang = cfg.language.clone();
                    let id = cfg.id.clone();
                    let location = PathBuf::from(&cfg.location)
                        .canonicalize()
                        .unwrap_or(PathBuf::from(&cfg.location))
                        .to_string_lossy()
                        .to_string();

                    if db.get_project(&id).is_err() {
                        db.add_project(&id, &cfg.name, lang, &location)?;
                        println!("Registered existing Genesis project '{}'", cfg.name);
                    } else {
                        println!("Genesis config already present; no DB changes needed.");
                    }

                    LockManager::acquire(&id)?;

                    return Ok(());
                }

                let language = match language {
                    Some(lang) => Languages::from_str(&lang)?,
                    None => detect_language(&cwd)?,
                };

                let id = Uuid::new_v4().to_string();
                let canonical = cwd.canonicalize()?;

                GenesisConfig::write_genesis(&id, &project_name, &language, &canonical)?;

                if !cwd.join(".git").exists() {
                    let git_status = std::process::Command::new("git")
                        .arg("init")
                        .current_dir(&cwd)
                        .status()?;
                    if !git_status.success() {
                        return Err("Failed to run git init".into());
                    }
                }

                let location = canonical.to_string_lossy().to_string();
                db.add_project(&id, &project_name, language, &location)?;
                LockManager::acquire(&id)?;
                println!(
                    "Initialized existing project '{}' for Genesis",
                    project_name
                );
                Ok(())
            }

            Commands::GitCommit { message } => {
                // Must be inside a Genesis project
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;

                let repo = GitRepo::open_from(&project_dir)?;
                let summary = repo.status_summary(&project_dir)?;
                if summary.added.is_empty()
                    && summary.modified.is_empty()
                    && summary.deleted.is_empty()
                    && summary.untracked.is_empty()
                {
                    println!("No changes to commit.");
                    return Ok(());
                }

                // Stage everything
                repo.stage_all()?;

                let commit_msg = message.unwrap_or_else(|| generate_commit_message(&summary));
                let oid = repo.commit(&commit_msg)?;
                println!("Committed {} as {}", commit_msg, oid);
                Ok(())
            }

            Commands::Blueprint { cmd } => match cmd {
                BlueprintCmd::New { name } => {
                    let path = BlueprintStore::create(&name)?;
                    println!("Created blueprint at {}", path.display());
                    Ok(())
                }
                BlueprintCmd::List => {
                    let bps = BlueprintStore::list()?;
                    if bps.is_empty() {
                        println!("No blueprints found.");
                    } else {
                        for bp in bps {
                            println!("- {}", bp.name);
                        }
                    }
                    Ok(())
                }
                BlueprintCmd::Apply { name } => {
                    let project_dir =
                        validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                    BlueprintStore::apply_to_project(&name, &project_dir)?;
                    println!("Applied blueprint '{}' to project.", name);
                    Ok(())
                }
            },

            Commands::Push => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                let scripts = &cfg.scripts;
                let required = ["lint", "build", "test", "deploy"];
                for r in required {
                    if !scripts.contains_key(r) {
                        return Err(format!("Missing script '{r}' in .genesis/config.toml").into());
                    }
                }

                let mut failed_stage: Option<&str> = None;
                for stage in required {
                    println!("==> {}", stage);
                    let cmd = scripts.get(stage).unwrap();
                    if let Err(e) = run::run_script(&project_dir, cmd) {
                        failed_stage = Some(stage);
                        println!("Stage '{}' failed: {}", stage, e);
                        break;
                    }
                }

                if let Some(stage) = failed_stage {
                    println!("Pipeline halted at '{}'. Fix errors and retry.", stage);
                } else {
                    println!("Pipeline completed successfully.");
                }
                Ok(())
            }

            Commands::Watch { auto_commit } => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                let repo = GitRepo::open_from(&project_dir)?;

                let (tx, rx) = unbounded();
                let _watcher = DebouncedWatcher::watch(project_dir.clone(), tx)?;
                let (hot_tx, hot_rx) = unbounded();
                std::thread::spawn(move || {
                    let stdin = std::io::stdin();
                    for b in stdin.bytes() {
                        if let Ok(c) = b {
                            // Alt+C usually sends ESC (27) then 'c'
                            if c == 27 {
                                let _ = hot_tx.send(());
                                break;
                            }
                        }
                    }
                });
                println!("Watching for changes (debounce ~5s). Press Alt+C to exit.");

                loop {
                    crossbeam_channel::select! {
                        recv(rx) -> msg => {
                            if let Ok(batch) = msg {
                                print_summary_and_maybe_commit(&repo, &project_dir, batch, auto_commit)?;
                            } else {
                                break;
                            }
                        }
                        recv(hot_rx) -> _ => {
                            println!("Watch stopped (Alt+C).");
                            break;
                        }
                    }
                }

                Ok(())
            }

            Commands::Shell => {
                run_shell()?;
                Ok(())
            }

            Commands::Build { bump } => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                run::build_project(&project_dir)?;
                // Bump version in config
                let mut cfg = GenesisConfig::read_genesis(&project_dir)?;
                cfg.version = bump_version_str(&cfg.version, &bump)?;
                GenesisConfig::write_existing(&cfg, &project_dir)?;
                println!("Build complete. Version bumped to {}", cfg.version);
                Ok(())
            }

            Commands::Install { package, language } => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                let lang = match language {
                    Some(l) => Languages::from_str(&l)?,
                    None => cfg.language.clone(),
                };
                install::install_package(&project_dir, &lang, &package)
                    .map_err(|e| format!("Install failed: {e}"))?;
                Ok(())
            }

            Commands::Bootstrap => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                crate::bootstrap::bootstrap(&project_dir, &cfg)
                    .map_err(|e| format!("Bootstrap failed: {e}"))?;
                println!("Bootstrap complete.");
                Ok(())
            }

            Commands::Ci { provider } => {
                let project_dir =
                    validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                match provider.as_str() {
                    "github" => crate::ci::emit_github_actions(&project_dir, &cfg)
                        .map_err(|e| format!("CI emit failed: {e}"))?,
                    _ => return Err("Unsupported CI provider (use --provider github)".into()),
                }
                println!("CI workflow emitted.");
                Ok(())
            }

            Commands::Guide => {
                print_guide();
                Ok(())
            }

            Commands::Tui => unreachable!("TUI command removed"),
        }
    }
}

fn detect_language(cwd: &PathBuf) -> Result<Languages, Box<dyn std::error::Error>> {
    let mut is_rust = false;
    let mut is_python = false;

    if cwd.join("Cargo.toml").exists() {
        is_rust = true;
    }
    if cwd.join("pyproject.toml").exists()
        || cwd.join("requirements.txt").exists()
        || cwd.join("setup.py").exists()
    {
        is_python = true;
    }

    match (is_rust, is_python) {
        (true, false) => Ok(Languages::Rust),
        (false, true) => Ok(Languages::Python),
        (true, true) => Err(
            "Ambiguous language detection (Rust and Python markers found); please pass --language"
                .into(),
        ),
        _ => Err("Unable to detect language; pass --language <Rust|Python>".into()),
    }
}

fn run_shell() -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = validator().ok_or("Run inside a Genesis project (.genesis missing)")?;
    let cfg = GenesisConfig::read_genesis(&project_dir)?;
    // Acquire lock for this project; release on exit
    let _ = LockManager::acquire(&cfg.id);
    println!(
        "Genesis shell for project '{}' (type 'help' for commands, 'exit' to quit)",
        cfg.name
    );

    let mut rl = DefaultEditor::new()?;

    loop {
        let prompt = format!("genesis({})> ", cfg.name);
        let line = rl.readline(&prompt);
        let input_owned = match line {
            Ok(l) => l,
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Read error: {e}");
                break;
            }
        };
        let input = input_owned.trim();
        if input.is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(input);
        if input == "exit" {
            let _ = LockManager::release(None);
            break;
        }
        if input == "help" {
            println!(
                "Commands: run, test, push, build [--bump <patch|minor|major>], install <package> [--language <Python|Rust>], bootstrap, ci --provider github, guide, git-commit [--message <msg>], blueprint list|new <name>|apply <name>, watch [--auto-commit], status, list, delete <name|--id ID>, stop, exit, help"
            );
            continue;
        }

        // Tokenize crude
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // Map to existing commands
        match parts[0] {
            "run" => run::run_project(&project_dir)?,
            "test" => run::test_project(&project_dir)?,
            "push" => {
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                let scripts = &cfg.scripts;
                let required = ["lint", "build", "test", "deploy"];
                for r in required {
                    if !scripts.contains_key(r) {
                        println!("Missing script '{r}' in .genesis/config.toml");
                        continue;
                    }
                }
                let mut failed_stage: Option<&str> = None;
                for stage in required {
                    println!("==> {}", stage);
                    let cmd = scripts.get(stage).unwrap();
                    if let Err(e) = run::run_script(&project_dir, cmd) {
                        failed_stage = Some(stage);
                        println!("Stage '{}' failed: {}", stage, e);
                        break;
                    }
                }
                if let Some(stage) = failed_stage {
                    println!("Pipeline halted at '{}'. Fix errors and retry.", stage);
                } else {
                    println!("Pipeline completed successfully.");
                }
            }
            "guide" => {
                print_guide();
            }
            "git-commit" => {
                let mut msg: Option<String> = None;
                if parts.len() > 2 && parts[1] == "--message" {
                    msg = Some(parts[2..].join(" "));
                }
                let repo = GitRepo::open_from(&project_dir)?;
                let summary = repo.status_summary(&project_dir)?;
                if summary.added.is_empty()
                    && summary.modified.is_empty()
                    && summary.deleted.is_empty()
                    && summary.untracked.is_empty()
                {
                    println!("No changes to commit.");
                    continue;
                }
                repo.stage_all()?;
                let commit_msg = msg.unwrap_or_else(|| generate_commit_message(&summary));
                let oid = repo.commit(&commit_msg)?;
                println!("Committed {} as {}", commit_msg, oid);
            }
            "blueprint" => {
                if parts.len() < 2 {
                    println!("Usage: blueprint list|new <name>|apply <name>");
                    continue;
                }
                match parts[1] {
                    "list" => {
                        let bps = BlueprintStore::list()?;
                        if bps.is_empty() {
                            println!("No blueprints found.");
                        } else {
                            for bp in bps {
                                println!("- {}", bp.name);
                            }
                        }
                    }
                    "new" if parts.len() >= 3 => {
                        let path = BlueprintStore::create(parts[2])?;
                        println!("Created blueprint at {}", path.display());
                    }
                    "apply" if parts.len() >= 3 => {
                        BlueprintStore::apply_to_project(parts[2], &project_dir)?;
                        println!("Applied blueprint '{}'.", parts[2]);
                    }
                    _ => println!("Usage: blueprint list|new <name>|apply <name>"),
                }
            }
            "watch" => {
                let auto_commit = parts.iter().any(|p| *p == "--auto-commit");
                let repo = GitRepo::open_from(&project_dir)?;
                let (tx, rx) = unbounded();
                let _watcher = DebouncedWatcher::watch(project_dir.clone(), tx)?;
                println!("Watching (debounce ~5s). Ctrl+C to stop.");
                for batch in rx.iter() {
                    let _ = print_summary_and_maybe_commit(&repo, &project_dir, batch, auto_commit);
                }
            }
            "status" => {
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                println!(
                    "Name: {}\nId: {}\nLanguage: {}\nLocation: {}",
                    cfg.name, cfg.id, cfg.language, cfg.location
                );
            }
            "list" => {
                let db = Database::new()?;
                let projects = db.list_projects()?;
                for p in projects {
                    println!("{} ({}) {}", p.name, p.language, p.location);
                }
            }
            "delete" => {
                if parts.len() < 2 {
                    println!("Usage: delete <name|--id ID>");
                    continue;
                }
                let db = Database::new()?;
                if parts[1] == "--id" && parts.len() >= 3 {
                    let projects = db.get_project(parts[2])?;
                    let project = projects.first().ok_or(DbError::NotFound)?;
                    db.delete_project(project)?;
                    let _ = LockManager::release(Some(&project.id));
                } else {
                    let matches = db.get_project(parts[1])?;
                    match matches.len() {
                        1 => {
                            db.delete_project(&matches[0])?;
                            let _ = LockManager::release(Some(&matches[0].id));
                        }
                        _ => println!("Ambiguous delete; use --id."),
                    }
                }
            }
            "build" => {
                let bump = parts.get(1).unwrap_or(&"patch");
                run::build_project(&project_dir)?;
                let mut cfg = GenesisConfig::read_genesis(&project_dir)?;
                cfg.version = bump_version_str(&cfg.version, bump)?;
                GenesisConfig::write_existing(&cfg, &project_dir)?;
                println!("Build complete. Version bumped to {}", cfg.version);
            }
            "install" => {
                if parts.len() < 2 {
                    println!("Usage: install <package> [--language <Python|Rust>]");
                    continue;
                }
                let pkg = parts[1];
                let mut lang_override: Option<Languages> = None;
                if parts.len() >= 4 && parts[2] == "--language" {
                    if let Ok(l) = Languages::from_str(parts[3]) {
                        lang_override = Some(l);
                    }
                }
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                let lang = lang_override.unwrap_or(cfg.language.clone());
                if let Err(e) = install::install_package(&project_dir, &lang, pkg) {
                    println!("Install failed: {e}");
                }
            }
            "bootstrap" => {
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                if let Err(e) = crate::bootstrap::bootstrap(&project_dir, &cfg) {
                    println!("Bootstrap failed: {e}");
                } else {
                    println!("Bootstrap complete.");
                }
            }
            "ci" => {
                let provider = parts.get(1).cloned().unwrap_or("github");
                let cfg = GenesisConfig::read_genesis(&project_dir)?;
                if let Err(e) = match provider {
                    "github" => crate::ci::emit_github_actions(&project_dir, &cfg),
                    _ => Err("Unsupported provider".into()),
                } {
                    println!("CI emit failed: {e}");
                } else {
                    println!("CI workflow emitted.");
                }
            }
            "stop" => {
                let mut db = Database::new()?;
                db.stop_project()?;
                let _ = LockManager::release(None);
                println!("Stopped session.");
                break;
            }
            other => {
                println!("Unknown command: {other}");
            }
        }
    }

    Ok(())
}

fn print_summary_and_maybe_commit(
    repo: &GitRepo,
    project_dir: &PathBuf,
    batch: DebouncedEvents,
    auto_commit: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = repo.status_summary(project_dir)?;

    let total_changes = summary.added.len()
        + summary.modified.len()
        + summary.deleted.len()
        + summary.untracked.len()
        + summary.renamed.len();

    if total_changes == 0 {
        println!("No changes to commit.");
        return Ok(());
    }

    println!("Changes settled at {:?}:", batch.settled_at);
    if !summary.added.is_empty() {
        println!("  Added: {}", summary.added.join(", "));
    }
    if !summary.modified.is_empty() {
        println!("  Modified: {}", summary.modified.join(", "));
    }
    if !summary.deleted.is_empty() {
        println!("  Deleted: {}", summary.deleted.join(", "));
    }
    if !summary.renamed.is_empty() {
        println!("  Renamed: {}", summary.renamed.join(", "));
    }
    if !summary.untracked.is_empty() {
        println!("  Untracked: {}", summary.untracked.join(", "));
    }

    let msg = generate_commit_message(&summary);
    println!("Suggested commit: {}", msg);

    if auto_commit {
        repo.stage_all()?;
        let oid = repo.commit(&msg)?;
        println!("Auto-committed as {}", oid);
    }

    Ok(())
}

fn print_guide() {
    println!(
        "\nGenesis Guide (Linux x86_64)\n\
        - Install: cargo install --path .\n\
        - Start/import: genesis start <name> <Rust|Python> | genesis import --language Python\n\
        - Auto blueprints: Django/Flask/FastAPI/Rust-web detected and applied on start\n\
        - Build: Rust -> cargo build --release; Python -> native bundle in bin/<name>\n\
        - Install deps: genesis install <pkg> (uv->pip or cargo) with spinner\n\
        - Bootstrap: genesis bootstrap (shared venv reuse or cargo fetch)\n\
        - Env loading: .env, .env.$GENESIS_ENV, .env.local applied to run/test/push/build\n\
        - CI: genesis ci --provider github writes .github/workflows/genesis.yml\n\
        - Guide anytime: genesis guide\n"
    );
}
