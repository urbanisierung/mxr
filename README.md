# mxr

Rust CLI tool for managing tmux sessions and workspace workflows on remote machines.

## Prerequisites

- tmux
- git + gh (for `mxr ship`, `mxr new`, and `mxr secret`)
- ssh + scp (for `mxr sync`)
- wrangler / npm / flyctl (only for the matching `mxr deploy` target)

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

# Scaffold a new repo: generate CLAUDE.md, init, push to GitHub
mxr new myproject --template rust

# Seed Claude skills + plugins into a repo (.claude/), ready to commit
mxr claude init

# Quick commit + push + PR
mxr ship "feat: add something"

# Create a deploy target and wire up CI secrets
mxr deploy pages myapp        # Cloudflare Pages project (via wrangler)
mxr deploy fly myapp          # Fly.io app (via flyctl)
mxr secret cloudflare         # set CLOUDFLARE_API_TOKEN + CLOUDFLARE_ACCOUNT_ID

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

CLAUDE.md templates for `mxr new` live in `~/.config/mxr/templates.toml`. Defaults are
written on first run; edit them to set up your own. Each body shares a portable base
(behavioral + quality guidelines) and adds a stack-specific section. Built-in templates:

| Template | Stack |
|----------|-------|
| `default` | Portable base only (language-agnostic) |
| `rust` | Rust — cargo, clippy, rustfmt |
| `kb` | Markdown knowledge base — markdownlint-cli2, Prettier, cspell |
| `ts-monorepo` | pnpm + Turborepo + Astro + Vite + Vitest + Biome + Preact + Zustand |
| `python` | Python — uv, Ruff, ty, pytest |
| `go-service` | Go backend service — net/http, golangci-lint, go test |

`{{name}}` and `{{stack}}` in the body are substituted when scaffolding:

```toml
[[template]]
name = "rust"
tech_stack = "Rust"
description = "Rust project (cargo, clippy, rustfmt)"
body = "# {{name}}\n\n## Part 1 — Behavioral Guidelines\n..."
```

## Deploy targets & secrets

`mxr deploy` shells out to the providers' official CLIs to create a target to
deploy to — it does not deploy itself. A GitHub Actions workflow (such as
`.github/workflows/deploy-web.yml`) does the deploying, reading credentials from
repo secrets you set with `mxr secret`.

```sh
mxr deploy pages mxrlp          # wrangler pages project create
mxr deploy worker my-worker     # npm create cloudflare@latest (C3 scaffold)
mxr deploy fly                  # fly apps create --generate-name

# Wire up the CI secrets the deploy workflow reads (delegates entry to gh):
mxr secret cloudflare           # CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID
mxr secret fly                  # FLY_API_TOKEN
mxr secret set MY_KEY           # any secret; reads the value from stdin
mxr secret set MY_KEY value     # or pass it inline

# Or fold target creation into scaffolding:
mxr new myapp --deploy cloudflare-pages
mxr new myapp --deploy fly
```

`mxr secret` requires `gh` and runs against the current repo. Secret values are
never handled by mxr — `gh` reads them from stdin so they stay out of your
shell history and argv when omitted.

## Claude skills & plugins

`mxr claude init` seeds the current repo's `.claude/` with configurable Claude
Code skills and plugins so they can be committed and shared with anyone who
clones it:

```sh
mxr claude init            # write skills + plugins into ./.claude/
mxr claude init --force     # overwrite existing skill files
mxr new myapp --claude      # seed during scaffolding (before the initial commit)
```

- **Skills** are written as `.claude/skills/<name>/SKILL.md`.
- **Plugins** are merged into `.claude/settings.json`. Existing files are left
  alone (skills) or deep-merged idempotently (settings.json).

Presets live in `~/.config/mxr/claude.toml` (defaults written on first run, same
as templates). Built-in defaults are two token-savers: the **caveman** skill and
the **rtk** plugin. A plugin is one of two kinds:

```toml
[[skill]]
name = "caveman"
description = "Talk like caveman to save output tokens."
body = "# Caveman Mode\n..."

# kind = "settings": a raw JSON fragment merged into settings.json (e.g. a hook)
[[plugin]]
name = "rtk"
kind = "settings"
settings_json = '{ "hooks": { "PreToolUse": [ ... ] } }'

# kind = "marketplace": register a marketplace and enable the plugin from it
[[plugin]]
name = "my-plugin"
kind = "marketplace"
marketplace = "my-mp"
source = "github"        # or "git"/"url" with a `url` field
repo = "owner/repo"
```

The `rtk` default needs the [`rtk`](https://github.com/rtk-ai/rtk) binary
installed and a one-time `rtk init -g` (it installs the hook script the committed
settings reference).

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
| `mxr new <name> [--org O] [--template T] [--public] [--deploy P] [--claude]` | Scaffold a repo + CLAUDE.md, push to GitHub, optionally create a deploy target and seed `.claude/` |
| `mxr claude init [--dir D] [--force]` | Seed `.claude/` skills + plugins into a repo |
| `mxr deploy pages <name> [--branch B]` | Create a Cloudflare Pages project (wrangler) |
| `mxr deploy worker <name>` | Scaffold a Cloudflare Worker (C3) |
| `mxr deploy fly [name]` | Create a Fly.io app (flyctl) |
| `mxr secret set <KEY> [VALUE]` | Set a GitHub Actions secret (gh) |
| `mxr secret cloudflare` | Set Cloudflare deploy secrets |
| `mxr secret fly` | Set Fly.io deploy secret |
| `mxr import [--file <path>]` | Import legacy format |
| `mxr update` | Self-update |
| `mxr update --check` | Check for updates only |
