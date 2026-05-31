//! Configurable Claude Code skills and plugins, seeded into a repo's `.claude/`
//! so they can be committed. Mirrors the `templates` module: user-editable
//! presets live at `~/.config/mxr/claude.toml`, with built-in defaults written
//! on first run.
//!
//! - Skills are written as `.claude/skills/<name>/SKILL.md`.
//! - Plugins are merged into `.claude/settings.json`. A plugin is either
//!   `kind = "marketplace"` (registers `extraKnownMarketplaces` + enables it via
//!   `enabledPlugins`) or `kind = "settings"` (a raw JSON fragment, e.g. a hook).

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A Claude Code skill, written to `.claude/skills/<name>/SKILL.md`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Skill {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub body: String,
}

/// A Claude Code plugin, merged into `.claude/settings.json`.
///
/// `kind = "marketplace"` uses `marketplace` + `source` (+ `repo` or `url`).
/// `kind = "settings"` uses `settings_json` (a raw JSON object to merge).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Plugin {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// "marketplace" or "settings"
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketplace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ClaudeConfig {
    #[serde(default)]
    pub skill: Vec<Skill>,
    #[serde(default)]
    pub plugin: Vec<Plugin>,
}

/// What `apply` did, for reporting.
#[derive(Debug, Default, PartialEq)]
pub struct ApplyResult {
    pub skills_written: Vec<String>,
    pub skills_skipped: Vec<String>,
    pub plugins_applied: Vec<String>,
}

pub fn claude_config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().ok_or_else(|| anyhow!("cannot resolve config dir"))?;
    Ok(dir.join("mxr").join("claude.toml"))
}

impl ClaudeConfig {
    pub fn load() -> Result<Self> {
        Self::load_from(&claude_config_path()?)
    }

    /// Load presets, writing built-in defaults if the file is missing.
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
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let text = toml::to_string_pretty(self).context("serialize claude.toml")?;
        fs::write(path, text).with_context(|| format!("write {}", path.display()))
    }

    pub fn defaults() -> Self {
        ClaudeConfig {
            skill: vec![
                Skill {
                    name: "caveman".into(),
                    description: "Talk like caveman to save output tokens. Few token do trick."
                        .into(),
                    body: CAVEMAN_BODY.into(),
                },
                Skill {
                    name: "lean-context".into(),
                    description: "Minimize context tokens: search before reading, read narrow \
                                  ranges, never re-read, batch tool calls."
                        .into(),
                    body: LEAN_CONTEXT_BODY.into(),
                },
            ],
            plugin: vec![
                Plugin {
                    name: "rtk".into(),
                    description: "Rust Token Killer — PreToolUse hook compresses Bash output. \
                                  Needs the `rtk` binary installed and `rtk init -g` run once \
                                  (installs ~/.claude/hooks/rtk-rewrite.sh)."
                        .into(),
                    kind: "settings".into(),
                    marketplace: None,
                    source: None,
                    repo: None,
                    url: None,
                    settings_json: Some(RTK_SETTINGS.into()),
                },
                Plugin {
                    name: "ast-grep".into(),
                    description: "Structural code search via AST patterns — find code without \
                                  reading whole files. Needs the `ast-grep` binary installed."
                        .into(),
                    kind: "marketplace".into(),
                    marketplace: Some("ast-grep-marketplace".into()),
                    source: Some("github".into()),
                    repo: Some("ast-grep/agent-skill".into()),
                    url: None,
                    settings_json: None,
                },
                Plugin {
                    name: "superpowers".into(),
                    description: "Agentic skills framework — TDD, planning, and debugging \
                                  workflows that cut expensive redo loops (brainstorm, \
                                  write-plan, execute-plan)."
                        .into(),
                    kind: "marketplace".into(),
                    marketplace: Some("superpowers-marketplace".into()),
                    source: Some("github".into()),
                    repo: Some("obra/superpowers-marketplace".into()),
                    url: None,
                    settings_json: None,
                },
            ],
        }
    }
}

/// Seed `repo_root`'s `.claude/` from the config. Existing skill files are left
/// alone unless `force`; settings.json is merged (idempotently).
pub fn apply(config: &ClaudeConfig, repo_root: &Path, force: bool) -> Result<ApplyResult> {
    let claude_dir = repo_root.join(".claude");
    let mut result = ApplyResult::default();

    for skill in &config.skill {
        let dir = claude_dir.join("skills").join(&skill.name);
        let file = dir.join("SKILL.md");
        if file.exists() && !force {
            result.skills_skipped.push(skill.name.clone());
            continue;
        }
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        fs::write(&file, skill_file_contents(skill))
            .with_context(|| format!("write {}", file.display()))?;
        result.skills_written.push(skill.name.clone());
    }

    if !config.plugin.is_empty() {
        let settings_path = claude_dir.join("settings.json");
        let mut settings = load_settings(&settings_path)?;
        for plugin in &config.plugin {
            let overlay = plugin_overlay(plugin)?;
            merge_json(&mut settings, &overlay);
            result.plugins_applied.push(plugin.name.clone());
        }
        fs::create_dir_all(&claude_dir)
            .with_context(|| format!("create {}", claude_dir.display()))?;
        let json = serde_json::to_string_pretty(&settings).context("serialize settings.json")?;
        fs::write(&settings_path, format!("{json}\n"))
            .with_context(|| format!("write {}", settings_path.display()))?;
    }

    Ok(result)
}

fn skill_file_contents(s: &Skill) -> String {
    format!(
        "---\nname: {}\ndescription: {}\n---\n\n{}\n",
        s.name,
        yaml_quote(&s.description),
        s.body.trim_end()
    )
}

fn yaml_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn load_settings(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    serde_json::from_str(&text).with_context(|| format!("parse {} (invalid JSON)", path.display()))
}

/// Build the JSON fragment a plugin contributes to settings.json.
fn plugin_overlay(p: &Plugin) -> Result<Value> {
    match p.kind.as_str() {
        "settings" => {
            let raw = p.settings_json.as_deref().ok_or_else(|| {
                anyhow!("plugin '{}' has kind=settings but no settings_json", p.name)
            })?;
            serde_json::from_str(raw)
                .with_context(|| format!("plugin '{}' settings_json is invalid JSON", p.name))
        }
        "marketplace" => {
            let mp = p.marketplace.as_deref().ok_or_else(|| {
                anyhow!(
                    "plugin '{}' has kind=marketplace but no marketplace",
                    p.name
                )
            })?;
            let source = p.source.as_deref().unwrap_or("github");
            let mut src = Map::new();
            src.insert("source".into(), Value::String(source.into()));
            match source {
                "github" => {
                    let repo = p
                        .repo
                        .as_deref()
                        .ok_or_else(|| anyhow!("plugin '{}' (source=github) needs repo", p.name))?;
                    src.insert("repo".into(), Value::String(repo.into()));
                }
                "git" | "url" => {
                    let url = p.url.as_deref().ok_or_else(|| {
                        anyhow!("plugin '{}' (source={}) needs url", p.name, source)
                    })?;
                    src.insert("url".into(), Value::String(url.into()));
                }
                other => anyhow::bail!("plugin '{}' has unsupported source '{}'", p.name, other),
            }
            let mut markets = Map::new();
            markets.insert(mp.to_string(), Value::Object(src));
            let mut enabled = Map::new();
            enabled.insert(format!("{}@{}", p.name, mp), Value::Bool(true));
            let mut root = Map::new();
            root.insert("extraKnownMarketplaces".into(), Value::Object(markets));
            root.insert("enabledPlugins".into(), Value::Object(enabled));
            Ok(Value::Object(root))
        }
        other => anyhow::bail!(
            "plugin '{}' has unknown kind '{}' (expected: settings, marketplace)",
            p.name,
            other
        ),
    }
}

/// Deep-merge `overlay` into `base`: objects merge key-by-key, arrays append
/// without duplicating existing items (so re-applying is idempotent), scalars
/// overwrite.
fn merge_json(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                merge_json(b.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (Value::Array(b), Value::Array(o)) => {
            for item in o {
                if !b.contains(item) {
                    b.push(item.clone());
                }
            }
        }
        (b, o) => *b = o.clone(),
    }
}

const CAVEMAN_BODY: &str = r#"# Caveman Mode

Talk like caveman. Few token do trick. Cut ~75% output tokens, keep technical
accuracy 100%.

## Activate

User say "caveman", "/caveman", or pick level. Stay caveman until user say stop.

## Levels

- **lite** — drop filler + pleasantries, keep articles and full sentences.
- **full** (default) — drop articles, fragments OK, short synonyms.
- **ultra** — max compress. Abbreviate common terms (db, auth, cfg). Notes style.

## Rules

- Remove articles (a, an, the), filler words, pleasantries.
- Fragments fine. Pattern: "[thing] [action] [reason]. [next step]."
- Keep exact technical terms, file paths, symbols, commands, identifiers.
- Never compress code blocks, commit messages, PR bodies — write those normal.

## Auto-clarity

Drop to normal prose for: security warnings, irreversible/destructive action
confirmations, multi-step sequences where compression risk confusion. Resume
caveman after.
"#;

const LEAN_CONTEXT_BODY: &str = r#"# Lean Context

Minimize tokens spent loading context. Every file read and tool result persists
across turns, so each one is a recurring cost. Spend the fewest tokens that still
answers the question.

## Reading files

- Search first. Use grep/glob to locate the exact symbol or lines, then read only
  that range (offset/limit) — not the whole file.
- Never re-read a file already in context; trust the prior read.
- For large files, read the relevant function or section, not the entire file.

## Searching

- Prefer precise patterns and narrow scope (specific dirs/extensions) over broad,
  whole-tree reads.
- Use structural search when matching code shape, not just text.

## Tool calls

- Batch independent reads/searches into one turn so they run in parallel.
- Filter noisy command output (head/tail/grep); request only the fields you need.
- Don't dump full build/test logs — grep for the failure line.

## Output

- Answer directly; skip preamble and restating the task.
- Reference code as path:line instead of pasting large blocks.
"#;

const RTK_SETTINGS: &str = r#"{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": "~/.claude/hooks/rtk-rewrite.sh" }
        ]
      }
    ]
  }
}"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn writes_defaults_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("claude.toml");
        let cfg = ClaudeConfig::load_from(&path).unwrap();
        assert!(path.exists());
        assert!(cfg.skill.iter().any(|s| s.name == "caveman"));
        assert!(cfg.plugin.iter().any(|p| p.name == "rtk"));
    }

    #[test]
    fn round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("claude.toml");
        let cfg = ClaudeConfig::defaults();
        cfg.save_to(&path).unwrap();
        let loaded = ClaudeConfig::load_from(&path).unwrap();
        assert_eq!(loaded.skill.len(), cfg.skill.len());
        assert_eq!(loaded.plugin.len(), cfg.plugin.len());
        assert_eq!(loaded.plugin[0].kind, "settings");
    }

    #[test]
    fn skill_file_has_frontmatter() {
        let s = Skill {
            name: "caveman".into(),
            description: "talk \"terse\"".into(),
            body: "do thing\n".into(),
        };
        let out = skill_file_contents(&s);
        assert!(out.starts_with("---\nname: caveman\n"));
        assert!(out.contains("description: \"talk \\\"terse\\\"\""));
        assert!(out.trim_end().ends_with("do thing"));
    }

    #[test]
    fn settings_plugin_overlay_is_raw_json() {
        let p = Plugin {
            name: "rtk".into(),
            description: String::new(),
            kind: "settings".into(),
            marketplace: None,
            source: None,
            repo: None,
            url: None,
            settings_json: Some(r#"{"hooks":{"PreToolUse":[1]}}"#.into()),
        };
        let v = plugin_overlay(&p).unwrap();
        assert_eq!(v["hooks"]["PreToolUse"][0], serde_json::json!(1));
    }

    #[test]
    fn marketplace_plugin_overlay_uses_object_maps() {
        let p = Plugin {
            name: "caveman".into(),
            description: String::new(),
            kind: "marketplace".into(),
            marketplace: Some("caveman-mp".into()),
            source: Some("github".into()),
            repo: Some("JuliusBrussee/caveman".into()),
            url: None,
            settings_json: None,
        };
        let v = plugin_overlay(&p).unwrap();
        assert_eq!(
            v["extraKnownMarketplaces"]["caveman-mp"],
            serde_json::json!({ "source": "github", "repo": "JuliusBrussee/caveman" })
        );
        assert_eq!(
            v["enabledPlugins"]["caveman@caveman-mp"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn marketplace_plugin_missing_repo_errors() {
        let p = Plugin {
            name: "x".into(),
            description: String::new(),
            kind: "marketplace".into(),
            marketplace: Some("mp".into()),
            source: Some("github".into()),
            repo: None,
            url: None,
            settings_json: None,
        };
        assert!(plugin_overlay(&p).is_err());
    }

    #[test]
    fn unknown_kind_errors() {
        let p = Plugin {
            name: "x".into(),
            description: String::new(),
            kind: "bogus".into(),
            marketplace: None,
            source: None,
            repo: None,
            url: None,
            settings_json: None,
        };
        assert!(plugin_overlay(&p).is_err());
    }

    #[test]
    fn merge_dedups_arrays_and_merges_objects() {
        let mut base = serde_json::json!({ "a": { "x": [1, 2] }, "b": 1 });
        let overlay = serde_json::json!({ "a": { "x": [2, 3], "y": 9 }, "c": 2 });
        merge_json(&mut base, &overlay);
        assert_eq!(base["a"]["x"], serde_json::json!([1, 2, 3]));
        assert_eq!(base["a"]["y"], serde_json::json!(9));
        assert_eq!(base["c"], serde_json::json!(2));
    }

    fn one_skill_one_plugin() -> ClaudeConfig {
        ClaudeConfig {
            skill: vec![Skill {
                name: "demo".into(),
                description: "d".into(),
                body: "body".into(),
            }],
            plugin: vec![Plugin {
                name: "hook".into(),
                description: String::new(),
                kind: "settings".into(),
                marketplace: None,
                source: None,
                repo: None,
                url: None,
                settings_json: Some(RTK_SETTINGS.into()),
            }],
        }
    }

    #[test]
    fn apply_writes_skill_and_settings_idempotently() {
        let dir = tempdir().unwrap();
        let cfg = one_skill_one_plugin();

        let r1 = apply(&cfg, dir.path(), false).unwrap();
        assert_eq!(r1.skills_written, ["demo"]);
        assert_eq!(r1.plugins_applied, ["hook"]);

        assert!(dir.path().join(".claude/skills/demo/SKILL.md").exists());
        let settings_path = dir.path().join(".claude/settings.json");
        let first = fs::read_to_string(&settings_path).unwrap();
        let parsed: Value = serde_json::from_str(&first).unwrap();
        assert_eq!(parsed["hooks"]["PreToolUse"][0]["matcher"], "Bash");

        // Re-apply: skill already exists (skipped), settings unchanged.
        let r2 = apply(&cfg, dir.path(), false).unwrap();
        assert_eq!(r2.skills_skipped, ["demo"]);
        assert!(r2.skills_written.is_empty());
        let second = fs::read_to_string(&settings_path).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn defaults_ship_expected_token_savers() {
        let cfg = ClaudeConfig::defaults();
        let skills: Vec<&str> = cfg.skill.iter().map(|s| s.name.as_str()).collect();
        assert!(skills.contains(&"caveman"));
        assert!(skills.contains(&"lean-context"));

        let ast = cfg.plugin.iter().find(|p| p.name == "ast-grep").unwrap();
        assert_eq!(ast.kind, "marketplace");
        assert_eq!(ast.marketplace.as_deref(), Some("ast-grep-marketplace"));
        assert_eq!(ast.repo.as_deref(), Some("ast-grep/agent-skill"));

        let sp = cfg.plugin.iter().find(|p| p.name == "superpowers").unwrap();
        assert_eq!(sp.marketplace.as_deref(), Some("superpowers-marketplace"));
        assert_eq!(sp.repo.as_deref(), Some("obra/superpowers-marketplace"));

        // Every default plugin must build a valid overlay.
        for p in &cfg.plugin {
            plugin_overlay(p).unwrap();
        }
    }

    #[test]
    fn apply_merges_into_existing_settings() {
        let dir = tempdir().unwrap();
        let claude = dir.path().join(".claude");
        fs::create_dir_all(&claude).unwrap();
        fs::write(
            claude.join("settings.json"),
            r#"{ "model": "opus", "hooks": { "PreToolUse": [] } }"#,
        )
        .unwrap();

        apply(&ClaudeConfig::defaults(), dir.path(), false).unwrap();
        let v: Value =
            serde_json::from_str(&fs::read_to_string(claude.join("settings.json")).unwrap())
                .unwrap();
        // pre-existing key preserved
        assert_eq!(v["model"], "opus");
        // hook merged in
        assert_eq!(v["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    }
}
