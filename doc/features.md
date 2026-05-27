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
