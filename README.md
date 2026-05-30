# mxr

Rust CLI tool for managing tmux sessions and workspace workflows on remote machines.

## Prerequisites

- tmux
- git + gh (for `mxr ship` and `mxr new`)
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

# Scaffold a new repo: generate CLAUDE.md, init, push to GitHub
mxr new myproject --template rust

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

CLAUDE.md templates for `mxr new` live in `~/.config/mxr/templates.toml`. Defaults are
written on first run; edit them to set up your own. Each body shares a portable base
(behavioral + quality guidelines) and adds a stack-specific section. Built-in templates:

| Template | Stack |
|----------|-------|
| `default` | Portable base only (language-agnostic) |
| `rust` | Rust — cargo, clippy, rustfmt |
| `kb` | Markdown knowledge base — markdownlint-cli2, Prettier, cspell |
| `ts-monorepo` | pnpm + Turborepo + Astro + Vite + Vitest + Biome + Preact + Zustand |

`{{name}}` and `{{stack}}` in the body are substituted when scaffolding:

```toml
[[template]]
name = "rust"
tech_stack = "Rust"
description = "Rust project (cargo, clippy, rustfmt)"
body = "# {{name}}\n\n## Part 1 — Behavioral Guidelines\n..."
```

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
| `mxr new <name> [--org O] [--template T] [--public]` | Scaffold a repo + CLAUDE.md and push to GitHub |
| `mxr import [--file <path>]` | Import legacy format |
| `mxr update` | Self-update |
| `mxr update --check` | Check for updates only |
