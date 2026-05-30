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
                    description: "Portable behavioral + quality guidelines, language-agnostic"
                        .into(),
                    body: SHARED_BASE.into(),
                },
                Template {
                    name: "rust".into(),
                    tech_stack: "Rust".into(),
                    description: "Rust project (cargo, clippy, rustfmt)".into(),
                    body: format!("{SHARED_BASE}\n{RUST_PART3}"),
                },
                Template {
                    name: "kb".into(),
                    tech_stack: "Markdown knowledge base".into(),
                    description: "Markdown KB (markdownlint-cli2, Prettier, cspell)".into(),
                    body: format!("{SHARED_BASE}\n{KB_PART3}"),
                },
                Template {
                    name: "ts-monorepo".into(),
                    tech_stack: "TypeScript monorepo".into(),
                    description:
                        "pnpm + Turborepo + Astro + Vite + Vitest + Biome + Preact + Zustand".into(),
                    body: format!("{SHARED_BASE}\n{TS_MONOREPO_PART3}"),
                },
            ],
        }
    }
}

/// Part 1 + Part 2 of the project CLAUDE.md — portable across stacks. Each stack
/// template appends its own Part 3.
const SHARED_BASE: &str = r#"# {{name}}

## Part 1 — Behavioral Guidelines

### Think Before Coding

Don't assume. Don't hide confusion. Surface tradeoffs.

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### Simplicity First

Minimum code that solves the problem. Nothing speculative.

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### Surgical Changes

Touch only what you must. Clean up only your own mess.

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

### Goal-Driven Execution

Define success criteria. Loop until verified.

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

## Part 2 — General Coding Quality

### Code Correctness

- Zero compiler/type errors. Always.
- Zero linting warnings. Always.
- All existing tests must pass after your changes.
- If you change behavior, update or add tests to cover it.

### Formatting & Linting

- Run the project's formatter and linter before considering any task complete.
- Never submit code that fails formatting or linting checks.
- Match the project's existing formatting configuration — do not override it.

### Testing

- Write tests for new functionality.
- Bug fixes must include a regression test.
- Don't delete or skip existing tests unless explicitly asked.
- Tests must be deterministic — no flaky assertions, no timing dependencies.

### Error Handling

- Handle errors at the appropriate level — don't swallow them silently.
- Provide actionable error messages that help debugging.
- Fail fast on invalid input — don't let bad data propagate.

### Security

- Never commit secrets, tokens, or credentials.
- Validate and sanitize all external input.
- Use parameterized queries for database access.
- Prefer established security libraries over hand-rolled solutions.

### Performance

- Consider performance implications of your changes.
- Avoid unnecessary allocations, copies, or iterations.
- Don't optimize prematurely — but don't write obviously slow code either.

### Documentation

- Update documentation when your changes affect public APIs or user-facing behavior.
- Code comments explain *why*, not *what*. The code itself should explain *what*.
- Don't add comments that merely restate the code.

### Pre-Completion Checklist

Before finishing any task, verify:
1. The project builds with zero warnings and zero errors.
2. Formatting and linting pass.
3. Type checking passes with zero errors.
4. All tests pass.
"#;

const RUST_PART3: &str = r#"## Part 3 — Rust Project

### Tech Stack

- **Language:** Rust (latest stable, edition 2024)
- **Build:** Cargo
- **Testing:** `cargo test` (unit + integration)
- **Lint:** Clippy
- **Format:** rustfmt

### Build & Check Commands

- Build: `cargo build`
- Release build: `cargo build --release`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Format check: `cargo fmt --check`

### Rust Practices

- No `unwrap()` / `expect()` in production paths — propagate errors with `?` and a `Result` type.
- Prefer `anyhow` for application error handling, `thiserror` for library error types.
- Exit with helpful error messages, not panics.
- Prefer the standard library over external crates where reasonable.
- Keep modules focused — one concern per module.

### Testing

- Unit tests live in `#[cfg(test)]` modules next to the code they cover.
- Integration tests live in `tests/`.
- Tests must be deterministic — no reliance on wall-clock time, network, or ordering.
"#;

const KB_PART3: &str = r#"## Part 3 — Markdown Knowledge Base

### Tech Stack

- **Content:** Markdown (CommonMark + GitHub-flavored)
- **Lint:** markdownlint-cli2 `0.22`
- **Format:** Prettier `3.8`
- **Spelling:** cspell `10`
- **Runtime:** Node.js (LTS) with pnpm `11.5`

### Build & Check Commands

- Install tools: `pnpm install`
- Lint: `pnpm markdownlint-cli2 "**/*.md"`
- Format: `pnpm prettier --write "**/*.md"`
- Format check: `pnpm prettier --check "**/*.md"`
- Spellcheck: `pnpm cspell "**/*.md"`

### Conventions

- One H1 (`#`) per document — the title. Use `##`/`###` for structure; never skip a level.
- One sentence per line where practical — keeps diffs reviewable.
- Prefer reference-style links for repeated URLs; collect them at the bottom of the file.
- Fence code blocks with a language tag.
- Use YAML front matter only where the publishing pipeline needs it; keep keys consistent.
- Add new or domain-specific terms to `cspell.json` rather than scattering inline ignores.

### Structure

```
{{name}}/
├── README.md            # entry point / table of contents
├── docs/                # knowledge base content
├── .markdownlint.jsonc  # lint rules
├── cspell.json          # custom dictionary + config
└── .prettierrc          # formatting config
```
"#;

const TS_MONOREPO_PART3: &str = r#"## Part 3 — TypeScript Monorepo

### Tech Stack

- **Package manager:** pnpm `11.5` (workspaces)
- **Task runner:** Turborepo `2.9`
- **Language:** TypeScript `6.0` (strict)
- **Web framework:** Astro `6.4`
- **Bundler / dev server:** Vite `8`
- **Testing:** Vitest `4.1`
- **Lint + format:** Biome `2.4`
- **UI library:** Preact `10.29` (via `@astrojs/preact` `5.1`)
- **State management:** Zustand `5.0`

### Project Structure

```
{{name}}/
├── package.json            # root; "packageManager": "pnpm@11.5.x"
├── pnpm-workspace.yaml      # workspace globs: apps/*, packages/*
├── turbo.json              # task pipeline (build, test, lint, check-types)
├── biome.json              # shared lint + format config
├── tsconfig.json           # base config, extended per package
├── apps/
│   └── web/                # Astro app (Preact islands, Zustand stores)
└── packages/
    ├── ui/                 # shared Preact components
    └── config/             # shared tsconfig / Biome presets
```

### Build & Check Commands

- Install: `pnpm install`
- Dev (all apps): `pnpm turbo dev`
- Build: `pnpm turbo build`
- Test: `pnpm turbo test` (Vitest)
- Test (watch): `pnpm vitest`
- Lint + format check: `pnpm biome check .`
- Apply safe fixes: `pnpm biome check --write .`
- Type-check: `pnpm turbo check-types` (`tsc --noEmit` per package)

### Conventions

- Every package is independently buildable and type-checked. Never import across packages via relative `../../` paths — import by the workspace package name.
- Declare each task's inputs/outputs in `turbo.json` so caching stays correct.
- TypeScript `strict` is on everywhere; no `any` — use `unknown` and narrow.
- Biome is the single source of truth for lint and format. Do not add ESLint or Prettier alongside it.
- Keep interactivity in Preact islands; prefer static Astro output where possible.
- Keep Zustand stores small and colocated with their feature; select narrow slices to avoid needless re-renders.
- Pin tool versions; bump deliberately and run `pnpm turbo build test` after upgrades.
"#;

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

    #[test]
    fn stack_templates_present() {
        let templates = Templates::defaults();
        for name in ["default", "rust", "kb", "ts-monorepo"] {
            assert!(templates.find(name).is_some(), "missing template {name}");
        }
    }

    #[test]
    fn stack_templates_extend_shared_base() {
        let templates = Templates::defaults();
        for name in ["rust", "kb", "ts-monorepo"] {
            let body = &templates.find(name).unwrap().body;
            assert!(
                body.contains("## Part 1 — Behavioral Guidelines"),
                "{name} missing shared base"
            );
            assert!(body.contains("## Part 3"), "{name} missing stack section");
        }
    }
}
