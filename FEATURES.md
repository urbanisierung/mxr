# mxr — tmux workspace manager

Repo name: `mxr`

Build a Rust CLI tool called `mxr` — a self-updating tmux session manager and workflow accelerator for remote machines.

## Reference: current shell implementation

The tool replaces this zsh script (preserve identical tmux behavior):

```bash
_tmux_open() {
  local session="$1"; shift
  if tmux has-session -t "$session" 2>/dev/null; then
    tmux attach -t "$session"; return
  fi
  tmux new-session -d -s "$session" -c "$1"; shift
  for dir; do
    tmux new-window -t "$session" -c "$dir"
  done
  tmux select-window -t "${session}:1"
  tmux attach -t "$session"
}

# Config format in ~/.config/.mytmux:
# name:dir1[:dir2:...]
# The first directory always gets two windows (opened twice).
```

Quick commit/push/PR function (also to be replicated):

```bash
gcaa() {
  local msg="${1:-wip $(date '+%Y-%m-%d %H:%M:%S')}"
  git add -A && git commit -m "${msg:-wip}" && gpsup && (gh pr view >/dev/null 2>&1 || gh pr create --fill)
}
```

---

## Repo structure

Rust workspace monorepo. Single binary with subcommand groups.

```
mxr/
├── Cargo.toml              # [workspace] members = ["crates/*"]
├── Justfile                # build-release (musl static), install (~/.local/bin), ci
├── install.sh              # curl-pipe-sh installer for remote machines
├── crates/
│   ├── mxr-cli/           # binary crate — clap routing, main.rs
│   │   └── Cargo.toml     # depends on mxr-core, mxr-update
│   ├── mxr-core/          # lib — config parsing, tmux exec, session logic
│   │   └── Cargo.toml
│   └── mxr-update/        # lib — self-update from GitHub releases
│       └── Cargo.toml
└── .github/
    └── workflows/
        └── release.yml     # on tag push: build musl binaries, attach to GH release
```

---

## Config

Location: `~/.config/mxr/sessions.toml`

```toml
[[session]]
name = "myproject"
dirs = ["/home/user/projects/myproject"]

[[session]]
name = "infra"
dirs = ["/home/user/infra", "/home/user/infra/terraform"]
```

Create config dir and file on first use if missing.

---

## Commands

### `mxr <name>` (default action)

- Shortcut for `mxr session open <name>`.
- If the first argument doesn't match a known subcommand, treat it as a session name and open it.

### `mxr session add <name> [--dir <path>]`

- Add current dir (or `--dir` path) as a new session entry.
- Error if name already exists (case-sensitive).
- Appends to sessions.toml.

### `mxr session open <name>`

- If tmux session exists: `tmux attach -t <name>`.
- Otherwise create it:
  - First dir in the list: open **two** windows with that dir as cwd.
  - Each additional dir: one window each.
  - Select window 1, then attach.
- This replicates the `_tmux_open` behavior from the reference script.

### `mxr session ls`

- List all configured sessions.
- Check `tmux has-session -t <name>` for each. Mark active ones with `(active)`.

### `mxr session rm <name>`

- Remove session entry from config. Error if not found.
- Does NOT kill a running tmux session (just removes the config entry).

### `mxr sync config <user@host>`

- `ssh <target> mkdir -p ~/.config/mxr`
- `scp ~/.config/mxr/sessions.toml <target>:~/.config/mxr/sessions.toml`

### `mxr sync binary <user@host>`

- `scp` the mxr binary itself to `<target>:~/.local/bin/mxr`
- `ssh <target> chmod +x ~/.local/bin/mxr`

### `mxr sync all <user@host>`

- Runs both `sync config` and `sync binary`.

### `mxr ship [message]`

- Quick commit, push, and PR — replicates the `gcaa` shell function.
- `git add -A`
- `git commit -m "<message>"` — default message: `wip <timestamp>`
- `git push --set-upstream origin <current-branch>`
- If no PR exists for the branch: `gh pr create --fill`
- Requires `git` and `gh` CLI to be available on PATH.
- Error with helpful message if either is missing.

### `mxr import [--file <path>]`

- Import sessions from the legacy config format (`~/.config/.mytmux` by default, or specify with `--file`).
- Legacy format: `name:dir1[:dir2:...]` (one entry per line, `#` comments, blank lines ignored).
- Converts each line into a `[[session]]` entry in `sessions.toml`.
- Skips entries whose name already exists in the current config.
- Prints a summary: imported N, skipped M (duplicates).

### `mxr update`

- Fetch latest release from GitHub Releases API.
- Compare semver with compiled-in version (`env!("CARGO_PKG_VERSION")`).
- If newer: download asset, atomically replace self.
- Print what happened.

### `mxr update --check`

- Print whether an update is available. Don't download.

---

## Auto-update hint (non-blocking)

On every invocation, if last update check was >24h ago, print a single line to stderr:

```
mxr: update available (v0.3.0 → v0.4.1). Run `mxr update`.
```

Store last-check timestamp in `~/.config/mxr/.last-update-check`.
Never block execution for this check — skip silently on network failure.

---

## Install script (`install.sh`)

```bash
#!/bin/sh
set -euo pipefail
REPO="urbanisierung/mxr"
BIN_DIR="${HOME}/.local/bin"
mkdir -p "$BIN_DIR"
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ASSET="mxr-linux-x86_64" ;;
  aarch64) ASSET="mxr-linux-aarch64" ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
curl -sSfL "$URL" -o "${BIN_DIR}/mxr"
chmod +x "${BIN_DIR}/mxr"
echo "Installed mxr to ${BIN_DIR}/mxr"
```

---

## Technical requirements

- **clap** (derive API) for arg parsing and subcommand routing.
- **serde** + **toml** for config read/write.
- **self_replace** for atomic binary replacement during update.
- **reqwest** (blocking, rustls-tls) for GitHub API calls. No openssl dependency.
- **semver** for version comparison.
- **dirs** for resolving `~/.config/mxr/`.
- **anyhow** for error handling. No `unwrap()` in production paths.
- Exit with helpful error messages, not panics.
- Compile target: `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`.

## Build setup

Justfile with:

```just
default_target := "x86_64-unknown-linux-musl"

build-release target=default_target:
    cargo build --release --target {{target}}

install: (build-release)
    cp target/{{default_target}}/release/mxr ~/.local/bin/mxr

ci-release:
    just build-release x86_64-unknown-linux-musl
    just build-release aarch64-unknown-linux-musl
```

## GitHub Actions release workflow

Trigger on tag push `v*`. Build both musl targets. Attach binaries as release assets named `mxr-linux-x86_64` and `mxr-linux-aarch64`.

## Tests

- Unit tests in `mxr-core` for config parsing (read, write, duplicate detection, removal).
- No tmux required in tests — mock the command execution layer.
- Integration test: write a config, read it back, verify round-trip.

## Constraints

- Don't add features beyond what's listed above.
- Keep it minimal. No async runtime.
- Match the tmux window behavior from the reference script exactly.
- One concern per crate. Don't over-abstract within a crate.
