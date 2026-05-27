use crate::config::Session;
use anyhow::Result;
use std::process::Command;

pub fn session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn list_sessions(sessions: &[Session]) -> Vec<(&Session, bool)> {
    sessions
        .iter()
        .map(|s| (s, session_exists(&s.name)))
        .collect()
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
    // First dir always gets two windows (matching reference script behavior)
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
