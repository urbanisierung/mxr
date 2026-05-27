# mxr Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a self-updating Rust CLI called `mxr` that manages tmux sessions via a TOML config and supports sync, ship, and self-update commands.

**Architecture:** Cargo workspace monorepo with three crates: `mxr-core` (config, session, import, update-check logic), `mxr-update` (GitHub API + self_replace), and `mxr` binary crate (clap routing, ship, sync). No async runtime — blocking only throughout.

**Tech Stack:** Rust stable, clap 4 derive API, serde + toml, reqwest blocking + rustls-tls, semver, self_replace, dirs, anyhow, tempfile (dev)

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | workspace definition |
| `Justfile` | build-release, install, ci-release targets |
| `install.sh` | curl-pipe-sh installer |
| `crates/mxr-core/Cargo.toml` | core lib deps |
| `crates/mxr-core/src/lib.rs` | pub re-exports |
| `crates/mxr-core/src/config.rs` | Config/Session types, load_from/save_to, add/remove |
| `crates/mxr-core/src/session.rs` | tmux open/ls/exists |
| `crates/mxr-core/src/import.rs` | legacy config import |
| `crates/mxr-core/src/update_check.rs` | 24h check timestamp |
| `crates/mxr-update/Cargo.toml` | update lib deps |
| `crates/mxr-update/src/lib.rs` | GitHub API, download, self_replace |
| `crates/mxr-cli/Cargo.toml` | binary deps |
| `crates/mxr-cli/src/main.rs` | clap routing, ship, sync, all subcommands |
| `.github/workflows/release.yml` | musl build + attach assets on tag push |

---

## Task 1: Workspace Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `crates/mxr-core/Cargo.toml`
- Create: `crates/mxr-update/Cargo.toml`
- Create: `crates/mxr-cli/Cargo.toml`
- Create: `crates/mxr-core/src/lib.rs`
- Create: `crates/mxr-update/src/lib.rs`
- Create: `crates/mxr-cli/src/main.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
members = ["crates/*"]
resolver = "2"
```

- [ ] **Step 2: Create mxr-core Cargo.toml**

```toml
[package]
name = "mxr-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "5"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Create mxr-update Cargo.toml**

```toml
[package]
name = "mxr-update"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
reqwest = { version = "0.12", features = ["blocking", "rustls-tls"], default-features = false }
semver = "1"
self_replace = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
```

- [ ] **Step 4: Create mxr-cli Cargo.toml**

```toml
[package]
name = "mxr"
version = "0.1.0"
edition = "2021"

[dependencies]
mxr-core = { path = "../mxr-core" }
mxr-update = { path = "../mxr-update" }
clap = { version = "4", features = ["derive"] }
anyhow = "1"
```

- [ ] **Step 5: Create stub lib and main files**

`crates/mxr-core/src/lib.rs`:
```rust
pub mod config;
pub mod import;
pub mod session;
pub mod update_check;
```

`crates/mxr-update/src/lib.rs`:
```rust
// placeholder
```

`crates/mxr-cli/src/main.rs`:
```rust
fn main() {}
```

- [ ] **Step 6: Verify workspace compiles**

Run: `cargo build`
Expected: success (warnings ok for empty stubs)

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/
git commit -m "chore: scaffold workspace with three crates"
```

---

## Task 2: Config Types and Operations

**Files:**
- Create: `crates/mxr-core/src/config.rs`

- [ ] **Step 1: Write failing tests for config operations**

Add to `crates/mxr-core/src/config.rs`:

```rust
use anyhow::{anyhow, Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Session {
    pub name: String,
    pub dirs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub session: Vec<Session>,
}

pub fn config_path() -> Result<PathBuf> {
    let dir = config_dir()
        .ok_or_else(|| anyhow!("cannot resolve config directory"))?
        .join("mxr");
    Ok(dir.join("sessions.toml"))
}

impl Config {
    pub fn load() -> Result<Self> {
        Self::load_from(&config_path()?)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create config dir {}", parent.display()))?;
            }
            return Ok(Self::default());
        }
        let text = fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&config_path()?)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(path, &text).with_context(|| format!("write {}", path.display()))
    }

    pub fn find(&self, name: &str) -> Option<&Session> {
        self.session.iter().find(|s| s.name == name)
    }

    pub fn add(&mut self, name: String, dirs: Vec<String>) -> Result<()> {
        if self.find(&name).is_some() {
            return Err(anyhow!("session '{}' already exists", name));
        }
        self.session.push(Session { name, dirs });
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        let before = self.session.len();
        self.session.retain(|s| s.name != name);
        if self.session.len() == before {
            return Err(anyhow!("session '{}' not found", name));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_config(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn load_empty_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sessions.toml");
        let cfg = Config::load_from(&path).unwrap();
        assert!(cfg.session.is_empty());
    }

    #[test]
    fn round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sessions.toml");
        let mut cfg = Config::default();
        cfg.add("proj".into(), vec!["/home/user/proj".into()]).unwrap();
        cfg.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded.session.len(), 1);
        assert_eq!(loaded.session[0].name, "proj");
        assert_eq!(loaded.session[0].dirs, vec!["/home/user/proj"]);
    }

    #[test]
    fn add_duplicate_errors() {
        let mut cfg = Config::default();
        cfg.add("a".into(), vec!["/x".into()]).unwrap();
        assert!(cfg.add("a".into(), vec!["/y".into()]).is_err());
    }

    #[test]
    fn remove_existing() {
        let mut cfg = Config::default();
        cfg.add("a".into(), vec!["/x".into()]).unwrap();
        cfg.remove("a").unwrap();
        assert!(cfg.session.is_empty());
    }

    #[test]
    fn remove_missing_errors() {
        let mut cfg = Config::default();
        assert!(cfg.remove("nope").is_err());
    }

    #[test]
    fn parse_toml_format() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sessions.toml");
        write_config(
            &path,
            r#"
[[session]]
name = "myproject"
dirs = ["/home/user/myproject"]

[[session]]
name = "infra"
dirs = ["/home/user/infra", "/home/user/infra/terraform"]
"#,
        );
        let cfg = Config::load_from(&path).unwrap();
        assert_eq!(cfg.session.len(), 2);
        assert_eq!(cfg.session[1].dirs.len(), 2);
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p mxr-core`
Expected: all 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/mxr-core/src/config.rs
git commit -m "feat(core): config types with load/save/add/remove and tests"
```

---

## Task 3: Session Logic

**Files:**
- Create: `crates/mxr-core/src/session.rs`

- [ ] **Step 1: Write session.rs**

```rust
use anyhow::Result;
use std::process::Command;
use crate::config::Session;

pub fn session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn list_sessions(sessions: &[Session]) -> Vec<(&Session, bool)> {
    sessions.iter().map(|s| (s, session_exists(&s.name))).collect()
}

pub fn open_session(session: &Session) -> Result<()> {
    let name = &session.name;
    if session_exists(name) {
        exec_tmux(&["attach", "-t", name]);
    }
    let dirs = &session.dirs;
    if dirs.is_empty() {
        anyhow::bail!("session '{}' has no directories configured", name);
    }
    run_tmux(&["new-session", "-d", "-s", name, "-c", &dirs[0]])?;
    // First dir always gets two windows
    run_tmux(&["new-window", "-t", name, "-c", &dirs[0]])?;
    // Each additional dir gets one window
    for dir in dirs.iter().skip(1) {
        run_tmux(&["new-window", "-t", name, "-c", dir])?;
    }
    run_tmux(&["select-window", "-t", &format!("{}:1", name)])?;
    exec_tmux(&["attach", "-t", name]);
}

fn run_tmux(args: &[&str]) -> Result<()> {
    let status = Command::new("tmux").args(args).status()?;
    if !status.success() {
        anyhow::bail!("tmux {} exited non-zero", args.join(" "));
    }
    Ok(())
}

// Replaces the current process with tmux — never returns on success.
fn exec_tmux(args: &[&str]) -> ! {
    use std::os::unix::process::CommandExt;
    let err = Command::new("tmux").args(args).exec();
    eprintln!("mxr: exec tmux: {}", err);
    std::process::exit(1);
}
```

- [ ] **Step 2: Verify compile**

Run: `cargo build -p mxr-core`
Expected: success (no tmux needed at compile time)

- [ ] **Step 3: Commit**

```bash
git add crates/mxr-core/src/session.rs
git commit -m "feat(core): session open/list/exists with exec-based tmux attach"
```

---

## Task 4: Update Check Timestamp

**Files:**
- Create: `crates/mxr-core/src/update_check.rs`

- [ ] **Step 1: Write update_check.rs**

```rust
use anyhow::Result;
use dirs::config_dir;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn check_file_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("mxr").join(".last-update-check"))
}

pub fn should_check_update() -> bool {
    let Some(path) = check_file_path() else { return false };
    let Ok(text) = fs::read_to_string(&path) else { return true };
    let Ok(ts) = text.trim().parse::<u64>() else { return true };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.saturating_sub(ts) > 86_400
}

pub fn record_check_time() -> Result<()> {
    let path = check_file_path()
        .ok_or_else(|| anyhow::anyhow!("cannot resolve config directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    fs::write(&path, ts.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: Verify compile**

Run: `cargo build -p mxr-core`
Expected: success

- [ ] **Step 3: Commit**

```bash
git add crates/mxr-core/src/update_check.rs
git commit -m "feat(core): 24h update check timestamp"
```

---

## Task 5: Legacy Import

**Files:**
- Create: `crates/mxr-core/src/import.rs`

- [ ] **Step 1: Write failing import tests**

```rust
use anyhow::Result;
use std::fs;
use std::path::Path;
use crate::config::Config;

pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

pub fn import_legacy(path: &Path, config: &mut Config) -> Result<ImportResult> {
    let text = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("read {}: {}", path.display(), e))?;
    let mut imported = 0;
    let mut skipped = 0;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, ':');
        let name = match parts.next() {
            Some(n) => n.trim().to_string(),
            None => continue,
        };
        let rest = match parts.next() {
            Some(r) => r,
            None => continue,
        };
        let dirs: Vec<String> = rest.split(':').map(|d| d.trim().to_string()).collect();
        match config.add(name, dirs) {
            Ok(()) => imported += 1,
            Err(_) => skipped += 1,
        }
    }
    Ok(ImportResult { imported, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn import_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".mytmux");
        fs::write(
            &path,
            "# comment\n\nmyproject:/home/user/proj\ninfra:/home/user/infra:/home/user/infra/tf\n",
        )
        .unwrap();
        let mut cfg = Config::default();
        let result = import_legacy(&path, &mut cfg).unwrap();
        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped, 0);
        assert_eq!(cfg.session[1].dirs.len(), 2);
    }

    #[test]
    fn import_skips_duplicates() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".mytmux");
        fs::write(&path, "proj:/home/user/proj\n").unwrap();
        let mut cfg = Config::default();
        cfg.add("proj".into(), vec!["/other".into()]).unwrap();
        let result = import_legacy(&path, &mut cfg).unwrap();
        assert_eq!(result.imported, 0);
        assert_eq!(result.skipped, 1);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p mxr-core import`
Expected: 2 tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/mxr-core/src/import.rs
git commit -m "feat(core): legacy config import with duplicate skip"
```

---

## Task 6: mxr-update Library

**Files:**
- Modify: `crates/mxr-update/src/lib.rs`

- [ ] **Step 1: Write the update library**

```rust
use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;
use std::io::Write;
use std::time::Duration;

#[derive(Debug)]
pub struct UpdateInfo {
    pub current: Version,
    pub latest: Version,
    pub tag_name: String,
    pub download_url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn asset_name() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    { "mxr-linux-x86_64" }
    #[cfg(target_arch = "aarch64")]
    { "mxr-linux-aarch64" }
}

fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("mxr-updater")
        .build()
        .context("build http client")
}

pub fn check_for_update(repo: &str, current_version: &str) -> Result<Option<UpdateInfo>> {
    let current = Version::parse(current_version)
        .with_context(|| format!("parse version '{}'", current_version))?;
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let release: Release = http_client()?
        .get(&url)
        .send()
        .context("fetch latest release")?
        .json()
        .context("parse release JSON")?;
    let tag_str = release.tag_name.trim_start_matches('v');
    let latest = Version::parse(tag_str)
        .with_context(|| format!("parse latest version '{}'", tag_str))?;
    if latest <= current {
        return Ok(None);
    }
    let name = asset_name();
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| anyhow::anyhow!("no asset '{}' in release {}", name, release.tag_name))?;
    Ok(Some(UpdateInfo {
        current,
        latest,
        tag_name: release.tag_name,
        download_url,
    }))
}

pub fn perform_update(url: &str) -> Result<()> {
    let bytes = http_client()?
        .get(url)
        .send()
        .context("download binary")?
        .bytes()
        .context("read binary")?;
    let mut tmp = tempfile::NamedTempFile::new().context("create temp file")?;
    tmp.write_all(&bytes).context("write temp file")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o755))
            .context("chmod temp file")?;
    }
    self_replace::self_replace(tmp.path()).context("replace binary")?;
    Ok(())
}
```

- [ ] **Step 2: Verify compile**

Run: `cargo build -p mxr-update`
Expected: success (may take a moment to fetch deps)

- [ ] **Step 3: Commit**

```bash
git add crates/mxr-update/src/lib.rs
git commit -m "feat(update): GitHub release check and atomic self-replace"
```

---

## Task 7: CLI — Main Entry Point and All Subcommands

**Files:**
- Modify: `crates/mxr-cli/src/main.rs`

- [ ] **Step 1: Write the complete CLI**

```rust
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
            Cli::parse_from(["mxr", "--help"]);
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
    let session = config
        .find(name)
        .ok_or_else(|| anyhow::anyhow!("session '{}' not found. Use `mxr session add` to create it.", name))?;
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
    run_cmd("ssh", &[target, "mkdir -p ~/.config/mxr"])
        .context("ssh mkdir")?;
    let dest = format!("{}:~/.config/mxr/sessions.toml", target);
    run_cmd("scp", &[config_path.to_str().unwrap(), &dest])
        .context("scp config")?;
    println!("Config synced to {}.", target);
    Ok(())
}

fn cmd_sync_binary(target: &str) -> Result<()> {
    let binary = std::env::current_exe().context("locate current binary")?;
    let dest = format!("{}:~/.local/bin/mxr", target);
    run_cmd("scp", &[binary.to_str().unwrap(), &dest])
        .context("scp binary")?;
    run_cmd("ssh", &[target, "chmod +x ~/.local/bin/mxr"])
        .context("ssh chmod")?;
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
        None => {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("cannot resolve config dir"))?
                .join(".mytmux")
        }
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
                println!(
                    "Update available: v{} → v{}.",
                    info.current, info.latest
                );
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
    Command::new(cmd)
        .arg("--version")
        .output()
        .map_err(|_| anyhow::anyhow!("'{}' not found on PATH", cmd))?;
    Ok(())
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()
        .with_context(|| format!("run '{}'", cmd))?;
    if !status.success() {
        anyhow::bail!("'{}' exited with status {}", cmd, status);
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
```

- [ ] **Step 2: Add `dirs` dependency to mxr-cli**

In `crates/mxr-cli/Cargo.toml`, add:
```toml
dirs = "5"
```

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: success with zero errors

- [ ] **Step 4: Run all tests**

Run: `cargo test --workspace`
Expected: all tests pass

- [ ] **Step 5: Clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings

- [ ] **Step 6: Format**

Run: `cargo fmt --check`
Expected: no diff (run `cargo fmt` first if needed)

- [ ] **Step 7: Commit**

```bash
git add crates/mxr-cli/src/main.rs crates/mxr-cli/Cargo.toml
git commit -m "feat(cli): all subcommands — session, sync, ship, import, update"
```

---

## Task 8: Justfile and install.sh

**Files:**
- Create: `Justfile`
- Create: `install.sh`

- [ ] **Step 1: Write Justfile**

```just
default_target := "x86_64-unknown-linux-musl"

build-release target=default_target:
    cargo build --release --target {{target}}

install: (build-release)
    cp target/{{default_target}}/release/mxr ~/.local/bin/mxr

ci-release:
    just build-release x86_64-unknown-linux-musl
    just build-release aarch64-unknown-linux-musl
```

- [ ] **Step 2: Write install.sh**

```sh
#!/bin/sh
set -euo pipefail
REPO="urbanisierung/mxr"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "$BIN_DIR"
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ASSET="mxr-linux-x86_64" ;;
  aarch64) ASSET="mxr-linux-aarch64" ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
curl -sSfL "$URL" -o "${BIN_DIR}/mxr"
chmod +x "${BIN_DIR}/mxr"
echo "Installed mxr to ${BIN_DIR}/mxr"
```

- [ ] **Step 3: Make install.sh executable**

Run: `chmod +x install.sh`

- [ ] **Step 4: Commit**

```bash
git add Justfile install.sh
git commit -m "chore: add Justfile and install.sh"
```

---

## Task 9: GitHub Actions Release Workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Write release.yml**

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            asset_name: mxr-linux-x86_64
          - target: aarch64-unknown-linux-musl
            asset_name: mxr-linux-aarch64

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install musl tools
        run: |
          sudo apt-get update -q
          sudo apt-get install -y musl-tools gcc-aarch64-linux-gnu

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
        env:
          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER: aarch64-linux-gnu-gcc

      - name: Rename binary
        run: |
          cp target/${{ matrix.target }}/release/mxr ${{ matrix.asset_name }}

      - name: Upload to release
        uses: softprops/action-gh-release@v2
        with:
          files: ${{ matrix.asset_name }}
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: release workflow building musl binaries on tag push"
```

---

## Task 10: Documentation Files

**Files:**
- Create: `README.md`
- Create: `doc/progress.md`
- Create: `doc/features.md`
- Create: `doc/roadmap.md`

- [ ] **Step 1: Write README.md**

```markdown
# mxr

Rust CLI tool for managing tmux sessions and workspace workflows on remote machines.

## Prerequisites

- tmux
- git + gh (for `mxr ship`)
- ssh + scp (for `mxr sync`)

## Install

```sh
curl -sSfL https://github.com/urbanisierung/mxr/releases/latest/download/mxr-linux-x86_64 -o ~/.local/bin/mxr
chmod +x ~/.local/bin/mxr
```

Or use the install script:

```sh
curl -sSfL https://raw.githubusercontent.com/urbanisierung/mxr/main/install.sh | sh
```

## Quickstart

```sh
# Add current directory as a session
mxr session add myproject

# Open the session (create + attach, or just attach if running)
mxr myproject

# List sessions (marks active ones)
mxr session ls

# Sync to a remote machine
mxr sync all user@host

# Quick commit + push + PR
mxr ship "feat: add something"

# Self-update
mxr update
```

## Config

`~/.config/mxr/sessions.toml`:

```toml
[[session]]
name = "myproject"
dirs = ["/home/user/projects/myproject"]

[[session]]
name = "infra"
dirs = ["/home/user/infra", "/home/user/infra/terraform"]
```

The first directory always gets two windows. Each additional directory gets one window.

## Commands

| Command | Description |
|---------|-------------|
| `mxr <name>` | Open session (shortcut) |
| `mxr session add <name> [--dir <path>]` | Add session |
| `mxr session open <name>` | Open session |
| `mxr session ls` | List sessions |
| `mxr session rm <name>` | Remove session |
| `mxr sync config <user@host>` | Copy sessions.toml to remote |
| `mxr sync binary <user@host>` | Copy binary to remote |
| `mxr sync all <user@host>` | Copy both |
| `mxr ship [message]` | Commit, push, create PR |
| `mxr import [--file <path>]` | Import legacy format |
| `mxr update` | Self-update |
| `mxr update --check` | Check for updates only |
```

- [ ] **Step 2: Write doc/progress.md**

```markdown
# Progress

## 2026-05-26

- Initial implementation: workspace scaffold, config parsing, session management
- Session open/ls/add/rm commands
- Sync config and binary to remote hosts
- Ship command (git add -A + commit + push + pr create)
- Legacy config import
- Self-update from GitHub releases
- Auto-update hint (24h check)
- Justfile, install.sh, GitHub Actions release workflow
```

- [ ] **Step 3: Write doc/features.md**

```markdown
# Features

## 2026-05-26 — Initial Release

- **Session management**: TOML config at `~/.config/mxr/sessions.toml`
- **mxr <name>**: shortcut to open a session
- **mxr session add/open/ls/rm**: full session lifecycle
- **mxr sync config/binary/all**: push config or binary to remote over SSH
- **mxr ship**: git add-A + commit + push + gh pr create
- **mxr import**: migrate legacy `:` delimited config format
- **mxr update / --check**: self-update from GitHub releases
- **Auto-update hint**: non-blocking stderr notice when update available (>24h check interval)
- **Static musl binaries**: x86_64 and aarch64 Linux
```

- [ ] **Step 4: Write doc/roadmap.md**

```markdown
# Roadmap

## v0.1.0

- [x] Cargo workspace with mxr-core, mxr-update, mxr-cli
- [x] TOML session config at ~/.config/mxr/sessions.toml
- [x] mxr <name> shortcut
- [x] mxr session add/open/ls/rm
- [x] mxr sync config/binary/all
- [x] mxr ship
- [x] mxr import from legacy format
- [x] mxr update / --check
- [x] Auto-update hint (stderr, non-blocking, 24h cooldown)
- [x] install.sh curl-pipe installer
- [x] GitHub Actions musl release workflow
```

- [ ] **Step 5: Commit**

```bash
git add README.md doc/
git commit -m "docs: README, progress, features, roadmap"
```

---

## Task 11: Final Verification

- [ ] **Step 1: Full build**

Run: `cargo build --workspace`
Expected: zero errors, zero warnings

- [ ] **Step 2: All tests pass**

Run: `cargo test --workspace`
Expected: all tests pass

- [ ] **Step 3: Clippy clean**

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings

- [ ] **Step 4: Format clean**

Run: `cargo fmt --check`
Expected: no output (all formatted)

- [ ] **Step 5: Check FEATURES.md coverage**

Verify each command in FEATURES.md has a corresponding handler in `crates/mxr-cli/src/main.rs`:
- `mxr <name>` → `Commands::Name`
- `mxr session add/open/ls/rm` → `SessionAction`
- `mxr sync config/binary/all` → `SyncAction`
- `mxr ship` → `cmd_ship`
- `mxr import` → `cmd_import`
- `mxr update [--check]` → `cmd_update`
- Auto-update hint → `maybe_print_update_hint`

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "chore: final verification pass"
```
