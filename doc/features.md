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

## 2026-05-31 — Claude skills & plugins

- **mxr claude init**: seed a repo's `.claude/` with configurable Claude Code skills (`.claude/skills/<name>/SKILL.md`) and plugins (merged into `.claude/settings.json`) so they can be committed. `--force` overwrites existing skill files
- **Configurable presets**: skills/plugins live in `~/.config/mxr/claude.toml`, defaults written on first run. Plugins are either `kind = "marketplace"` (extraKnownMarketplaces + enabledPlugins) or `kind = "settings"` (a raw JSON fragment, e.g. a hook). Defaults ship five token-savers: the `caveman` and `lean-context` skills, and the `rtk` (hook), `ast-grep` (marketplace), and `superpowers` (marketplace) plugins
- **mxr new --claude**: seed skills/plugins as part of scaffolding a new repo

## 2026-05-31 — Deploy targets & secrets

- **mxr deploy pages/worker/fly**: create a deploy target via the providers' official CLIs (wrangler, C3, flyctl). Prerequisite step before a CI workflow deploys; also available as top-level `mxr pages`/`mxr worker`/`mxr fly`
- **mxr secret**: set GitHub Actions secrets via `gh` — generic `secret set <KEY> [VALUE]` plus `secret cloudflare` and `secret fly` presets for the known deploy credentials
- **mxr new --deploy**: create a Cloudflare Pages or Fly.io target as part of scaffolding

## 2026-05-30 — Scaffolding

- **mxr new <name>**: scaffold a new project — generate a CLAUDE.md from a template, `git init`, create the GitHub repo (use `--org` for an existing organization, `--public` for visibility), push, and register it as an mxr session
- **Configurable templates**: CLAUDE.md presets live in `~/.config/mxr/templates.toml`. Defaults (`default`, `rust`, `node`, `python`) are written on first run; each user edits them to taste. Bodies support `{{name}}` and `{{stack}}` placeholders
