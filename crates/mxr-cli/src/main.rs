use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "mxr", version, about = "tmux workspace manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage sessions
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// Sync config or binary to a remote host
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
    /// Quick commit, push, and PR
    Ship {
        /// Commit message (default: "wip <timestamp>")
        message: Option<String>,
    },
    /// Import sessions from legacy ~/.config/.mytmux format
    Import {
        /// Path to legacy config (default: ~/.config/.mytmux)
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Update mxr to the latest release
    Update {
        /// Only check for updates, don't download
        #[arg(long)]
        check: bool,
    },
    #[command(external_subcommand)]
    Name(Vec<String>),
}

#[derive(Subcommand)]
enum SessionAction {
    /// Open a tmux session (create if needed)
    Open { name: String },
    /// Add a session entry
    Add {
        name: String,
        #[arg(long, short)]
        dir: Option<PathBuf>,
    },
    /// List configured sessions
    Ls,
    /// Remove a session entry from config
    Rm { name: String },
}

#[derive(Subcommand)]
enum SyncAction {
    /// Copy sessions.toml to remote host
    Config { target: String },
    /// Copy mxr binary to remote host
    Binary { target: String },
    /// Copy both config and binary
    All { target: String },
}

fn main() {
    maybe_print_update_hint();
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("mxr: {:#}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
        Some(Commands::Name(args)) => {
            let name = args.into_iter().next().unwrap_or_default();
            open_session_by_name(&name)
        }
        Some(Commands::Session { action }) => match action {
            SessionAction::Open { name } => open_session_by_name(&name),
            SessionAction::Add { name, dir } => cmd_session_add(name, dir),
            SessionAction::Ls => cmd_session_ls(),
            SessionAction::Rm { name } => cmd_session_rm(&name),
        },
        Some(Commands::Sync { action }) => match action {
            SyncAction::Config { target } => cmd_sync_config(&target),
            SyncAction::Binary { target } => cmd_sync_binary(&target),
            SyncAction::All { target } => {
                cmd_sync_config(&target)?;
                cmd_sync_binary(&target)
            }
        },
        Some(Commands::Ship { message }) => cmd_ship(message.as_deref()),
        Some(Commands::Import { file }) => cmd_import(file),
        Some(Commands::Update { check }) => cmd_update(check),
    }
}

fn open_session_by_name(name: &str) -> Result<()> {
    let config = mxr_core::config::Config::load()?;
    let session = config.find(name).ok_or_else(|| {
        anyhow::anyhow!(
            "session '{}' not found. Use `mxr session add` to create it.",
            name
        )
    })?;
    mxr_core::session::open_session(session)
}

fn cmd_session_add(name: String, dir: Option<PathBuf>) -> Result<()> {
    let path = match dir {
        Some(p) => p.canonicalize().context("resolve --dir path")?,
        None => std::env::current_dir().context("get current directory")?,
    };
    let path_str = path.to_string_lossy().into_owned();
    let mut config = mxr_core::config::Config::load()?;
    config.add(name.clone(), vec![path_str])?;
    config.save()?;
    println!("Added session '{}'.", name);
    Ok(())
}

fn cmd_session_ls() -> Result<()> {
    let config = mxr_core::config::Config::load()?;
    if config.session.is_empty() {
        println!("No sessions configured.");
        return Ok(());
    }
    let sessions = mxr_core::session::list_sessions(&config.session);
    for (session, active) in sessions {
        if active {
            println!("{} (active)", session.name);
        } else {
            println!("{}", session.name);
        }
    }
    Ok(())
}

fn cmd_session_rm(name: &str) -> Result<()> {
    let mut config = mxr_core::config::Config::load()?;
    config.remove(name)?;
    config.save()?;
    println!("Removed session '{}'.", name);
    Ok(())
}

fn cmd_sync_config(target: &str) -> Result<()> {
    let config_path = mxr_core::config::config_path()?;
    run_cmd("ssh", &[target, "mkdir -p ~/.config/mxr"]).context("ssh mkdir")?;
    let dest = format!("{}:~/.config/mxr/sessions.toml", target);
    run_cmd(
        "scp",
        &[config_path.to_string_lossy().as_ref(), dest.as_str()],
    )
    .context("scp config")?;
    println!("Config synced to {}.", target);
    Ok(())
}

fn cmd_sync_binary(target: &str) -> Result<()> {
    let binary = std::env::current_exe().context("locate current binary")?;
    let dest = format!("{}:~/.local/bin/mxr", target);
    run_cmd("scp", &[binary.to_string_lossy().as_ref(), dest.as_str()]).context("scp binary")?;
    run_cmd("ssh", &[target, "chmod +x ~/.local/bin/mxr"]).context("ssh chmod")?;
    println!("Binary synced to {}.", target);
    Ok(())
}

fn cmd_ship(message: Option<&str>) -> Result<()> {
    require_cmd("git")?;
    require_cmd("gh")?;
    let msg = match message {
        Some(m) => m.to_string(),
        None => {
            let ts = Command::new("date")
                .arg("+%Y-%m-%d %H:%M:%S")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "wip".to_string());
            format!("wip {}", ts)
        }
    };
    run_cmd("git", &["add", "-A"])?;
    run_cmd("git", &["commit", "-m", &msg])?;
    let branch = git_current_branch()?;
    run_cmd("git", &["push", "--set-upstream", "origin", &branch])?;
    let pr_exists = Command::new("gh")
        .args(["pr", "view"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !pr_exists {
        run_cmd("gh", &["pr", "create", "--fill"])?;
    }
    Ok(())
}

fn cmd_import(file: Option<PathBuf>) -> Result<()> {
    let path = match file {
        Some(p) => p,
        None => dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot resolve config dir"))?
            .join(".mytmux"),
    };
    let mut config = mxr_core::config::Config::load()?;
    let result = mxr_core::import::import_legacy(&path, &mut config)?;
    config.save()?;
    println!(
        "Import complete: {} imported, {} skipped (duplicates).",
        result.imported, result.skipped
    );
    Ok(())
}

fn cmd_update(check_only: bool) -> Result<()> {
    let _ = mxr_core::update_check::record_check_time();
    match mxr_update::check_for_update("urbanisierung/mxr", env!("CARGO_PKG_VERSION"))? {
        None => println!("mxr is up to date (v{}).", env!("CARGO_PKG_VERSION")),
        Some(info) => {
            if check_only {
                println!("Update available: v{} → v{}.", info.current, info.latest);
            } else {
                println!("Updating v{} → v{}...", info.current, info.latest);
                mxr_update::perform_update(&info.download_url)?;
                println!("Updated to {}. Restart mxr.", info.tag_name);
            }
        }
    }
    Ok(())
}

fn maybe_print_update_hint() {
    if !mxr_core::update_check::should_check_update() {
        return;
    }
    let _ = mxr_core::update_check::record_check_time();
    if let Ok(Some(info)) =
        mxr_update::check_for_update("urbanisierung/mxr", env!("CARGO_PKG_VERSION"))
    {
        eprintln!(
            "mxr: update available (v{} → v{}). Run `mxr update`.",
            info.current, info.latest
        );
    }
}

fn require_cmd(cmd: &str) -> Result<()> {
    Command::new(cmd).arg("--version").output().map_err(|_| {
        anyhow::anyhow!(
            "'{}' not found on PATH. Install it to use this command.",
            cmd
        )
    })?;
    Ok(())
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("run '{}'", cmd))?;
    if !status.success() {
        anyhow::bail!("'{}' exited with non-zero status", cmd);
    }
    Ok(())
}

fn git_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("git rev-parse")?;
    if !output.status.success() {
        anyhow::bail!("not in a git repository");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
