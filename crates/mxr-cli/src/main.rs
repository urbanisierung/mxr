use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "mxr",
    version,
    about = "tmux workspace manager",
    disable_help_subcommand = true
)]
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
    /// Scaffold a new repo with a generated CLAUDE.md and push to GitHub
    New {
        /// Repository name
        name: String,
        /// Create under an existing organization (default: your account)
        #[arg(long)]
        org: Option<String>,
        /// CLAUDE.md template to use (default: "default")
        #[arg(long, short)]
        template: Option<String>,
        /// Make the repository public (default: private)
        #[arg(long)]
        public: bool,
        /// Directory to create the project in (default: ./<name>)
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Also create a deploy target: "cloudflare-pages" or "fly"
        #[arg(long)]
        deploy: Option<String>,
        /// Also seed Claude skills/plugins into the repo (.claude/)
        #[arg(long)]
        claude: bool,
    },
    /// Create a deploy target (Cloudflare Pages/Worker or Fly.io)
    Deploy {
        #[command(subcommand)]
        action: DeployAction,
    },
    /// Set a GitHub Actions secret for the current repo
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },
    /// Seed Claude skills/plugins into a repo (.claude/)
    Claude {
        #[command(subcommand)]
        action: ClaudeAction,
    },
    /// Create a Cloudflare Pages project (alias for deploy pages)
    Pages {
        name: String,
        /// Production branch (default: main)
        #[arg(long, default_value = "main")]
        branch: String,
    },
    /// Scaffold a Cloudflare Worker project (alias for deploy worker)
    Worker { name: String },
    /// Create a Fly.io app (alias for deploy fly)
    Fly { name: Option<String> },
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
    /// Checkout default branch, pull, and create a new numbered branch
    Next,
    /// List configured sessions (alias for session ls)
    Ls,
    /// Add current dir as session (alias for session add)
    Add {
        name: String,
        #[arg(long, short)]
        dir: Option<PathBuf>,
    },
    /// Reattach to the most recently used tmux session
    Back,
    /// Kill a running tmux session
    Kill { name: String },
    /// Rename a session in config
    Rename { old: String, new: String },
    /// Open sessions.toml in $EDITOR
    Edit,
    /// Show command reference
    Help,
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

#[derive(Subcommand)]
enum DeployAction {
    /// Create a Cloudflare Pages project (via wrangler)
    Pages {
        name: String,
        /// Production branch (default: main)
        #[arg(long, default_value = "main")]
        branch: String,
    },
    /// Scaffold a Cloudflare Worker project (via C3 / npm create cloudflare)
    Worker { name: String },
    /// Create a Fly.io app (via flyctl). Generates a name when omitted.
    Fly { name: Option<String> },
}

#[derive(Subcommand)]
enum SecretAction {
    /// Set a single GitHub Actions secret (reads value from stdin if omitted)
    Set { key: String, value: Option<String> },
    /// Set the Cloudflare deploy secrets (token + account id)
    Cloudflare,
    /// Set the Fly.io deploy secret (API token)
    Fly,
}

#[derive(Subcommand)]
enum ClaudeAction {
    /// Write configured skills + plugins into the repo's .claude/ directory
    Init {
        /// Repo to seed (default: current directory)
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Overwrite existing skill files
        #[arg(long)]
        force: bool,
    },
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
        Some(Commands::New {
            name,
            org,
            template,
            public,
            dir,
            deploy,
            claude,
        }) => cmd_new(
            &name,
            org.as_deref(),
            template.as_deref(),
            public,
            dir,
            deploy.as_deref(),
            claude,
        ),
        Some(Commands::Deploy { action }) => match action {
            DeployAction::Pages { name, branch } => cmd_deploy_pages(&name, &branch),
            DeployAction::Worker { name } => cmd_deploy_worker(&name),
            DeployAction::Fly { name } => cmd_deploy_fly(name.as_deref()),
        },
        Some(Commands::Secret { action }) => match action {
            SecretAction::Set { key, value } => cmd_secret_set(&key, value.as_deref()),
            SecretAction::Cloudflare => cmd_secret_preset(mxr_core::deploy::CLOUDFLARE_SECRETS),
            SecretAction::Fly => cmd_secret_preset(mxr_core::deploy::FLY_SECRETS),
        },
        Some(Commands::Claude { action }) => match action {
            ClaudeAction::Init { dir, force } => cmd_claude_init(dir, force),
        },
        Some(Commands::Pages { name, branch }) => cmd_deploy_pages(&name, &branch),
        Some(Commands::Worker { name }) => cmd_deploy_worker(&name),
        Some(Commands::Fly { name }) => cmd_deploy_fly(name.as_deref()),
        Some(Commands::Import { file }) => cmd_import(file),
        Some(Commands::Update { check }) => cmd_update(check),
        Some(Commands::Next) => cmd_next(),
        Some(Commands::Ls) => cmd_session_ls(),
        Some(Commands::Add { name, dir }) => cmd_session_add(name, dir),
        Some(Commands::Back) => cmd_back(),
        Some(Commands::Kill { name }) => cmd_kill(&name),
        Some(Commands::Rename { old, new }) => cmd_rename(&old, &new),
        Some(Commands::Edit) => cmd_edit(),
        Some(Commands::Help) => {
            cmd_help();
            Ok(())
        }
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

fn cmd_new(
    name: &str,
    org: Option<&str>,
    template: Option<&str>,
    public: bool,
    dir: Option<PathBuf>,
    deploy: Option<&str>,
    claude: bool,
) -> Result<()> {
    require_cmd("git")?;
    require_cmd("gh")?;

    let templates = mxr_core::templates::Templates::load()?;
    let tmpl_name = template.unwrap_or("default");
    let tmpl = templates.find(tmpl_name).ok_or_else(|| {
        anyhow::anyhow!(
            "template '{}' not found. Edit {} to add it.",
            tmpl_name,
            mxr_core::templates::templates_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "templates.toml".to_string())
        )
    })?;

    let target = match dir {
        Some(p) => p,
        None => std::env::current_dir()
            .context("get current directory")?
            .join(name),
    };
    if target.exists() {
        anyhow::bail!("directory '{}' already exists", target.display());
    }
    std::fs::create_dir_all(&target).with_context(|| format!("create {}", target.display()))?;

    std::fs::write(target.join("CLAUDE.md"), tmpl.render(name)).context("write CLAUDE.md")?;

    if claude {
        seed_claude(&target, false)?;
    }

    run_cmd_in(&target, "git", &["init", "-q"])?;
    run_cmd_in(&target, "git", &["add", "-A"])?;
    run_cmd_in(&target, "git", &["commit", "-q", "-m", "initial commit"])?;

    let full_name = match org {
        Some(o) => format!("{}/{}", o, name),
        None => name.to_string(),
    };
    let visibility = if public { "--public" } else { "--private" };
    run_cmd_in(
        &target,
        "gh",
        &[
            "repo",
            "create",
            &full_name,
            visibility,
            "--source=.",
            "--remote=origin",
            "--push",
        ],
    )
    .context("gh repo create")?;

    let mut config = mxr_core::config::Config::load()?;
    let path_str = target
        .canonicalize()
        .unwrap_or(target)
        .to_string_lossy()
        .into_owned();
    if config.add(name.to_string(), vec![path_str]).is_ok() {
        config.save()?;
    }

    println!(
        "Created '{}' from template '{}' and pushed to GitHub.",
        full_name, tmpl_name
    );

    if let Some(provider) = deploy {
        match provider {
            "cloudflare-pages" | "pages" | "cloudflare" => cmd_deploy_pages(name, "main")?,
            "fly" | "fly.io" | "flyio" => cmd_deploy_fly(Some(name))?,
            "worker" => anyhow::bail!(
                "--deploy worker scaffolds a fresh project and conflicts with `mxr new`; \
                 run `mxr deploy worker {}` separately instead",
                name
            ),
            other => anyhow::bail!(
                "unknown --deploy target '{}' (expected: cloudflare-pages, fly)",
                other
            ),
        }
    }

    Ok(())
}

fn run_cmd_owned(cmd: &str, args: &[String]) -> Result<()> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_cmd(cmd, &refs)
}

fn cmd_deploy_pages(name: &str, branch: &str) -> Result<()> {
    require_cmd("wrangler")?;
    run_cmd_owned(
        "wrangler",
        &mxr_core::deploy::cloudflare_pages_args(name, branch),
    )
    .context("wrangler pages project create")?;
    println!(
        "Created Cloudflare Pages project '{}'. Set CI secrets with `mxr secret cloudflare`.",
        name
    );
    Ok(())
}

fn cmd_deploy_worker(name: &str) -> Result<()> {
    require_cmd("npm")?;
    run_cmd_owned("npm", &mxr_core::deploy::cloudflare_worker_args(name))
        .context("npm create cloudflare")?;
    println!(
        "Scaffolded Cloudflare Worker '{}'. Set CI secrets with `mxr secret cloudflare`.",
        name
    );
    Ok(())
}

fn cmd_deploy_fly(name: Option<&str>) -> Result<()> {
    require_cmd("fly")?;
    run_cmd_owned("fly", &mxr_core::deploy::fly_create_args(name)).context("fly apps create")?;
    println!("Created Fly.io app. Set CI secrets with `mxr secret fly`.");
    Ok(())
}

fn cmd_secret_set(key: &str, value: Option<&str>) -> Result<()> {
    require_cmd("gh")?;
    run_cmd_owned("gh", &mxr_core::deploy::gh_secret_set_args(key, value))
        .with_context(|| format!("gh secret set {}", key))?;
    println!("Set secret '{}'.", key);
    Ok(())
}

fn cmd_secret_preset(keys: &[&str]) -> Result<()> {
    for key in keys {
        cmd_secret_set(key, None)?;
    }
    Ok(())
}

fn cmd_claude_init(dir: Option<PathBuf>, force: bool) -> Result<()> {
    let target = match dir {
        Some(p) => p,
        None => std::env::current_dir().context("get current directory")?,
    };
    seed_claude(&target, force)
}

fn seed_claude(target: &Path, force: bool) -> Result<()> {
    let config = mxr_core::claude::ClaudeConfig::load()?;
    let result = mxr_core::claude::apply(&config, target, force)?;
    if !result.skills_written.is_empty() {
        println!("Skills written: {}.", result.skills_written.join(", "));
    }
    if !result.skills_skipped.is_empty() {
        println!(
            "Skills skipped (exist, use --force): {}.",
            result.skills_skipped.join(", ")
        );
    }
    if !result.plugins_applied.is_empty() {
        println!(
            "Plugins applied to .claude/settings.json: {}.",
            result.plugins_applied.join(", ")
        );
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

fn run_cmd_in(dir: &Path, cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("run '{}'", cmd))?;
    if !status.success() {
        anyhow::bail!("'{}' exited with non-zero status", cmd);
    }
    Ok(())
}

fn cmd_back() -> Result<()> {
    run_cmd("tmux", &["attach"]).map_err(|_| anyhow::anyhow!("no tmux sessions to attach to"))
}

fn cmd_kill(name: &str) -> Result<()> {
    run_cmd("tmux", &["kill-session", "-t", name])
        .with_context(|| format!("kill session '{}'", name))?;
    println!("Killed session '{}'.", name);
    Ok(())
}

fn cmd_rename(old: &str, new: &str) -> Result<()> {
    let mut config = mxr_core::config::Config::load()?;
    config.rename(old, new)?;
    config.save()?;
    println!("Renamed '{}' → '{}'.", old, new);
    Ok(())
}

fn cmd_edit() -> Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());
    let path = mxr_core::config::config_path()?;
    // ensure the file exists before opening
    mxr_core::config::Config::load()?;
    use std::os::unix::process::CommandExt;
    let err = Command::new(&editor).arg(&path).exec();
    Err(anyhow::anyhow!("exec '{}': {}", editor, err))
}

fn cmd_help() {
    println!("mxr — tmux workspace manager\n");
    println!("USAGE: mxr <command> [args]\n");
    println!("COMMANDS:");
    println!("  <name>                     open session by name");
    println!("  ls                         list configured sessions");
    println!("  add <name>                 add current dir as session");
    println!("  back                       reattach to most recent tmux session");
    println!("  kill <name>                kill a running tmux session");
    println!("  rename <old> <new>         rename a session in config");
    println!("  edit                       open sessions.toml in $EDITOR");
    println!("  session add <name>         add current dir as session");
    println!("  session open <name>        open tmux session (create if needed)");
    println!("  session ls                 list configured sessions");
    println!("  session rm <name>          remove session from config");
    println!("  sync config <host>         copy sessions.toml to remote");
    println!("  sync binary <host>         copy mxr binary to remote");
    println!("  sync all <host>            copy config and binary");
    println!("  ship [msg]                 commit, push, open PR");
    println!("  new <name> [--template T]  scaffold repo + CLAUDE.md, push (--org, --public, --deploy, --claude)");
    println!("  deploy pages <name>        create a Cloudflare Pages project (wrangler)");
    println!("  deploy worker <name>       scaffold a Cloudflare Worker (C3)");
    println!("  deploy fly [name]          create a Fly.io app (flyctl)");
    println!("  secret set <KEY> [VALUE]   set a GitHub Actions secret (gh)");
    println!("  secret cloudflare          set Cloudflare deploy secrets");
    println!("  secret fly                 set Fly.io deploy secret");
    println!("  claude init [--force]      seed .claude/ skills + plugins into repo");
    println!("  next                       checkout default branch, pull, new branch");
    println!("  import [--file <path>]     import legacy ~/.config/.mytmux");
    println!("  update [--check]           self-update from GitHub releases");
    println!("  help                       show this reference");
    println!("\nRun `mxr <command> --help` for details.");
}

fn cmd_next() -> Result<()> {
    require_cmd("git")?;
    let repo_name = git_repo_name()?;
    let default_branch = git_default_branch()?;
    let n = next_branch_number(&repo_name)?;
    let branch_name = format!("{}{}", repo_name, n);
    run_cmd("git", &["checkout", &default_branch])?;
    run_cmd("git", &["pull"])?;
    run_cmd("git", &["checkout", "-b", &branch_name])?;
    println!("Switched to new branch '{}'.", branch_name);
    Ok(())
}

fn git_repo_name() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("git remote get-url origin")?;
    if !output.status.success() {
        // fall back to current directory name
        return Ok(std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "branch".to_string()));
    }
    let url = String::from_utf8(output.stdout)?.trim().to_string();
    let name = url
        .trim_end_matches('/')
        .split(['/', ':'])
        .next_back()
        .unwrap_or(&url)
        .trim_end_matches(".git")
        .to_string();
    if name.is_empty() {
        anyhow::bail!("cannot determine repo name from remote URL: {}", url);
    }
    Ok(name)
}

fn git_default_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let s = String::from_utf8(out.stdout)?;
            if let Some(branch) = s.trim().rsplit('/').next() {
                return Ok(branch.to_string());
            }
        }
    }
    for branch in &["main", "master"] {
        let ok = Command::new("git")
            .args(["show-ref", "--verify", &format!("refs/heads/{}", branch)])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return Ok(branch.to_string());
        }
    }
    anyhow::bail!("cannot determine default branch (tried main, master)");
}

fn next_branch_number(repo_name: &str) -> Result<usize> {
    let output = Command::new("git")
        .args(["branch", "--list", &format!("{}[0-9]*", repo_name)])
        .output()
        .context("git branch --list")?;
    let stdout = String::from_utf8(output.stdout)?;
    let max = stdout
        .lines()
        .filter_map(|line| {
            let name = line.trim().trim_start_matches("* ");
            let suffix = name.strip_prefix(repo_name)?;
            suffix.parse::<usize>().ok()
        })
        .max()
        .unwrap_or(0);
    Ok(max + 1)
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
