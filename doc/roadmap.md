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
- [x] mxr next (checkout default, pull, new numbered branch)

## v0.2.0 — Scaffolding

- [x] `mxr new <name>` — scaffold a repo, generate CLAUDE.md, push to GitHub
- [x] User-configurable CLAUDE.md templates at `~/.config/mxr/templates.toml`

## v0.3.0 — Deploy targets & secrets

- [x] `mxr deploy pages/worker/fly` — create a deploy target via wrangler / C3 / flyctl
- [x] `mxr secret set` + `mxr secret cloudflare`/`fly` — manage GitHub Actions secrets via `gh`
- [x] `mxr new --deploy <cloudflare-pages|fly>` — create the deploy target while scaffolding

## v0.4.0 — Claude skills & plugins

- [x] `mxr claude init` — seed `.claude/` skills + plugins into a repo, configurable via `~/.config/mxr/claude.toml`
- [x] Built-in defaults: `caveman` + `lean-context` skills, `rtk` + `ast-grep` + `superpowers` plugins (token savers)
- [x] `mxr new --claude` — seed skills/plugins while scaffolding

## v0.5.0 — Claude Code web scaffolding

- [x] `/new-project <stack> <name>` skill — scaffold a repo from a phone/browser Claude Code session (web equivalent of `mxr new`, via the GitHub integration)
- [x] Stack template files under `templates/` (`rust`, `ts`) readable/renderable by a web session
