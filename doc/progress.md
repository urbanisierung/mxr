# Progress

## 2026-06-04

- `mxr help`: refreshed the `new` line, which was stale — it only showed
  `[--org O]` and never surfaced `--template` (the flag for picking one of the
  built-in CLAUDE.md presets) or `--public`/`--deploy`/`--claude`. The `new`
  subcommand itself already existed; only the hand-written help summary lagged.

- Fixed the install command 404. The landing page (`apps/web`) and README pointed
  at `releases/latest/download/mxr-linux-x86_64`, but GitHub's `latest` endpoint
  only resolves full releases — mxr's `0.x` builds are all prereleases, so that
  URL 404s. Both now use the existing `install.sh` one-liner, which resolves the
  newest release (prereleases included) via the API and auto-detects the arch.

## 2026-05-31 (later)

- Added `mxr claude init` to seed a repo's `.claude/` with configurable Claude
  Code skills and plugins so they can be committed. Skills are written as
  `.claude/skills/<name>/SKILL.md`; plugins are merged into
  `.claude/settings.json`. Existing skill files are left alone unless `--force`;
  settings.json is deep-merged idempotently (arrays dedup, objects merge).
- Added `mxr-core::claude` with user-configurable presets at
  `~/.config/mxr/claude.toml` (built-in defaults written on first run, same as
  templates). A plugin is `kind = "marketplace"` (registers
  `extraKnownMarketplaces` + `enabledPlugins`) or `kind = "settings"` (a raw JSON
  fragment merged in, e.g. a hook).
- Default presets ship five token-savers: `caveman` (terse-prose skill) and
  `lean-context` (search-before-read / narrow-read skill); `rtk` (Rust Token
  Killer PreToolUse hook; needs the `rtk` binary + a one-time `rtk init -g`),
  `ast-grep` (marketplace plugin for structural search; needs the `ast-grep`
  binary), and `superpowers` (marketplace plugin; TDD/planning/debugging
  workflows that cut expensive redo loops).
- `mxr new --claude` seeds the skills/plugins into the new repo before the
  initial commit.

## 2026-05-31

- Added `mxr deploy` to create a deploy target via the providers' official CLIs:
  `pages` (`wrangler pages project create`), `worker` (C3 / `npm create
  cloudflare@latest`), and `fly` (`flyctl apps create`, `--generate-name` when no
  name is given). Each prints a hint pointing at the matching `mxr secret`
  command. Available both as `mxr deploy <target>` and as top-level aliases
  `mxr pages`/`mxr worker`/`mxr fly`.
- Added `mxr secret` to manage GitHub Actions secrets via `gh`: `secret set <KEY>
  [VALUE]` (value read from stdin by `gh` when omitted, so it stays out of argv),
  plus `secret cloudflare` (CLOUDFLARE_API_TOKEN + CLOUDFLARE_ACCOUNT_ID) and
  `secret fly` (FLY_API_TOKEN) presets matching the deploy workflows.
- `mxr new --deploy <cloudflare-pages|fly>` now creates the deploy target after
  pushing the repo (worker is rejected with a hint, since C3 scaffolds a fresh
  project that conflicts with the new repo).
- Added `mxr-core::deploy` with pure argument-builders for the above, covered by
  unit tests; the CLI crate spawns the commands.

## 2026-05-30

- Added `mxr new <name>` to scaffold a repo: render a CLAUDE.md from a template, `git init` + initial commit, create and push the GitHub repo via `gh` (`--org` targets an existing org, `--public` toggles visibility), and register it as a session.
- Added `mxr-core::templates` module with user-configurable presets at `~/.config/mxr/templates.toml`; built-in defaults written on first run.
- Built-in templates now ship a shared base (CLAUDE.md Part 1 + Part 2, portable across stacks) plus a stack-specific Part 3. Stacks: `default` (base only), `rust`, `kb` (Markdown knowledge base), and `ts-monorepo` (pnpm + Turborepo + Astro + Vite + Vitest + Biome + Preact + Zustand, versions pinned to current latest).
- Added `python` (uv + Ruff + ty + pytest) and `go-service` (Go 1.26 net/http + golangci-lint + go test) templates, versions pinned to current latest.

## 2026-05-29

- Fix install.sh 404: it downloaded from `releases/latest/download/...`, but
  `releases/latest` only resolves full releases — all current releases are
  prereleases, so it 404'd (the binaries themselves were attached fine).
  install.sh now resolves the newest release (prereleases included) via the
  GitHub API and downloads that tag's asset.
- release.yml: publish releases as full (`prerelease: false`, `make_latest:
  true`) so `releases/latest` resolves going forward.
- deploy-web.yml: deploy the Astro web app (`apps/web`) to Cloudflare Pages on
  push to main via wrangler-action. Needs `CLOUDFLARE_API_TOKEN` and
  `CLOUDFLARE_ACCOUNT_ID` secrets.
- deploy-web.yml: run wrangler-action with `workingDirectory: apps/web` so its
  `pnpm add wrangler` installs into the workspace member instead of the
  workspace root, which pnpm rejects with ERR_PNPM_ADDING_TO_ROOT.

## 2026-05-27

- `mxr next`: checkout default branch, pull, create next numbered branch from repo name

## 2026-05-26

- Initial implementation: workspace scaffold, config parsing, session management
- Session open/ls/add/rm commands
- Sync config and binary to remote hosts
- Ship command (git add -A + commit + push + pr create)
- Legacy config import
- Self-update from GitHub releases
- Auto-update hint (24h check)
- Justfile, install.sh, GitHub Actions release workflow
