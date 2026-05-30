# Progress

## 2026-05-30

- Added `mxr new <name>` to scaffold a repo: render a CLAUDE.md from a template, `git init` + initial commit, create and push the GitHub repo via `gh` (`--org` targets an existing org, `--public` toggles visibility), and register it as a session.
- Added `mxr-core::templates` module with user-configurable presets at `~/.config/mxr/templates.toml`; built-in defaults written on first run.
- Built-in templates now ship a shared base (CLAUDE.md Part 1 + Part 2, portable across stacks) plus a stack-specific Part 3. Stacks: `default` (base only), `rust`, `kb` (Markdown knowledge base), and `ts-monorepo` (pnpm + Turborepo + Astro + Vite + Vitest + Biome + Preact + Zustand, versions pinned to current latest).

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
