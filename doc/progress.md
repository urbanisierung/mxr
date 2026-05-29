# Progress

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
