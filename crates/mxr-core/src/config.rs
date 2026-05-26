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
        let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
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
        cfg.add("proj".into(), vec!["/home/user/proj".into()])
            .unwrap();
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
