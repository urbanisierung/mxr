# Instructions

<!-- Part 1 & 2: Portable across repos. Do NOT add repo-specific rules here. -->
<!-- Repo-specific instructions go in .github/instructions/repo.instructions.md -->

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

<!-- Part 3: Everything below is specific to THIS repository. -->

## Tech Stack

- **Language:** Rust (latest stable)
- **Architecture:** Cargo workspace monorepo
- **Build:** `cargo build` via Justfile, musl static binaries
- **Compile targets:** `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
- **Testing:** `cargo test` (unit + integration)
- **CI:** GitHub Actions — release on tag push `v*`
- **No async runtime** — blocking only

## Key Dependencies

- **clap** (derive API) — arg parsing, subcommand routing
- **serde** + **toml** — config read/write
- **self_replace** — atomic binary replacement during update
- **reqwest** (blocking, rustls-tls) — GitHub API calls (no openssl)
- **semver** — version comparison
- **dirs** — resolving `~/.config/mxr/`
- **anyhow** — error handling

## Project Structure

```
mxr/
├── Cargo.toml              # [workspace] members = ["crates/*"]
├── Justfile                # build-release, install, ci-release
├── install.sh              # curl-pipe-sh installer
├── crates/
│   ├── mxr-cli/           # binary crate — clap routing, main.rs
│   ├── mxr-core/          # lib — config parsing, tmux exec, session logic
│   └── mxr-update/        # lib — self-update from GitHub releases
└── .github/
    └── workflows/
        └── release.yml     # build musl binaries, attach to GH release
```

## Build & Check Commands

- Build (release, musl): `just build-release`
- Install to `~/.local/bin`: `just install`
- Test: `cargo test --workspace`
- Clippy: `cargo clippy --workspace -- -D warnings`
- Format check: `cargo fmt --check`

## Rust Practices

- No `unwrap()` in production paths — use `anyhow` for error propagation
- Exit with helpful error messages, not panics
- One concern per crate — don't over-abstract within a crate
- No async runtime — keep it simple and blocking
- Prefer standard library over external crates where reasonable

## Testing

- Unit tests in `mxr-core` for config parsing (read, write, duplicate detection, removal)
- No tmux required in tests — mock the command execution layer
- Integration test: write a config, read it back, verify round-trip
- Tests must be deterministic

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

## Documentation Requirements

| File              | Purpose                                            | Update Frequency           |
| ----------------- | -------------------------------------------------- | -------------------------- |
| `README.md`       | Brief intro, motivation, prerequisites, quickstart | On significant changes     |
| `doc/progress.md` | Historical changelog                               | **Every change**           |
| `doc/features.md` | High-level feature list with timestamps            | When features are added    |
| `doc/roadmap.md`  | Implementation roadmap with action items           | Check items when completed |

### Roadmap Tracking

When completing action items from `doc/roadmap.md`:

- Mark completed items with `[x]` instead of `[ ]`
- Keep the roadmap up-to-date as features are implemented