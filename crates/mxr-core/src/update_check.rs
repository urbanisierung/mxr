use anyhow::Result;
use dirs::config_dir;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn check_file_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("mxr").join(".last-update-check"))
}

pub fn should_check_update() -> bool {
    let Some(path) = check_file_path() else {
        return false;
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return true;
    };
    let Ok(ts) = text.trim().parse::<u64>() else {
        return true;
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.saturating_sub(ts) > 86_400
}

pub fn record_check_time() -> Result<()> {
    let path =
        check_file_path().ok_or_else(|| anyhow::anyhow!("cannot resolve config directory"))?;
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
