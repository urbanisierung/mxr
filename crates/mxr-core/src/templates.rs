use anyhow::{anyhow, Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A user-defined CLAUDE.md preset. `body` may contain `{{name}}` and
/// `{{stack}}` placeholders, filled in when a new project is scaffolded.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Template {
    pub name: String,
    #[serde(default)]
    pub tech_stack: String,
    #[serde(default)]
    pub description: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Templates {
    #[serde(default)]
    pub template: Vec<Template>,
}

pub fn templates_path() -> Result<PathBuf> {
    let dir = config_dir()
        .ok_or_else(|| anyhow!("cannot resolve config directory"))?
        .join("mxr");
    Ok(dir.join("templates.toml"))
}

impl Template {
    pub fn render(&self, project_name: &str) -> String {
        self.body
            .replace("{{name}}", project_name)
            .replace("{{stack}}", &self.tech_stack)
    }
}

impl Templates {
    /// Load templates, writing the built-in defaults on first run so the user
    /// has a file to customize.
    pub fn load() -> Result<Self> {
        Self::load_from(&templates_path()?)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            let defaults = Self::defaults();
            defaults.save_to(path)?;
            return Ok(defaults);
        }
        let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(path, &text).with_context(|| format!("write {}", path.display()))
    }

    pub fn find(&self, name: &str) -> Option<&Template> {
        self.template.iter().find(|t| t.name == name)
    }

    pub fn defaults() -> Self {
        Templates {
            template: vec![
                Template {
                    name: "default".into(),
                    tech_stack: String::new(),
                    description: "Minimal language-agnostic starter".into(),
                    body: DEFAULT_BODY.into(),
                },
                Template {
                    name: "rust".into(),
                    tech_stack: "Rust".into(),
                    description: "Rust project with cargo".into(),
                    body: RUST_BODY.into(),
                },
                Template {
                    name: "node".into(),
                    tech_stack: "TypeScript / Node.js".into(),
                    description: "Node.js project with pnpm".into(),
                    body: NODE_BODY.into(),
                },
                Template {
                    name: "python".into(),
                    tech_stack: "Python".into(),
                    description: "Python project".into(),
                    body: PYTHON_BODY.into(),
                },
            ],
        }
    }
}

const DEFAULT_BODY: &str = "# {{name}}

## Overview

Describe what {{name}} does here.

## Conventions

- State assumptions explicitly; ask when unclear.
- Keep changes surgical — touch only what the task requires.
- Write tests for new behavior; keep them deterministic.

## Build & Test

Document the build, test, and lint commands here.
";

const RUST_BODY: &str = "# {{name}}

## Overview

{{name}} is a {{stack}} project.

## Build & Test

- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`

## Conventions

- No `unwrap()` in production paths — propagate errors.
- Keep changes surgical and tests deterministic.
- Run fmt and clippy before considering a task complete.
";

const NODE_BODY: &str = "# {{name}}

## Overview

{{name}} is a {{stack}} project.

## Build & Test

- Install: `pnpm install`
- Build: `pnpm build`
- Test: `pnpm test`
- Lint: `pnpm lint`

## Conventions

- Prefer explicit types over `any`.
- Keep changes surgical and tests deterministic.
- Run lint and tests before considering a task complete.
";

const PYTHON_BODY: &str = "# {{name}}

## Overview

{{name}} is a {{stack}} project.

## Build & Test

- Test: `pytest`
- Lint: `ruff check`
- Format: `ruff format`

## Conventions

- Type-annotate public functions.
- Keep changes surgical and tests deterministic.
- Run lint and tests before considering a task complete.
";

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn writes_defaults_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("templates.toml");
        let templates = Templates::load_from(&path).unwrap();
        assert!(path.exists());
        assert!(templates.find("default").is_some());
        assert!(templates.find("rust").is_some());
    }

    #[test]
    fn round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("templates.toml");
        Templates::load_from(&path).unwrap();
        let loaded = Templates::load_from(&path).unwrap();
        assert_eq!(loaded.template.len(), Templates::defaults().template.len());
    }

    #[test]
    fn render_replaces_placeholders() {
        let tmpl = Template {
            name: "rust".into(),
            tech_stack: "Rust".into(),
            description: String::new(),
            body: "# {{name}}\nStack: {{stack}}".into(),
        };
        let out = tmpl.render("acme");
        assert_eq!(out, "# acme\nStack: Rust");
    }

    #[test]
    fn find_missing_returns_none() {
        let templates = Templates::defaults();
        assert!(templates.find("nope").is_none());
    }
}
