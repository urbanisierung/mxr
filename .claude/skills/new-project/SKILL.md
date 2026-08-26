---
name: new-project
description: Scaffold a new GitHub repository from a stack template, seeding CLAUDE.md, README, and .gitignore, then pushing the initial commit. This is the Claude Code web equivalent of `mxr new` — use it to create a brand-new project from a phone/browser session (e.g. "/new-project rust foo", "/new-project ts my-app"). Supported stacks: rust, ts.
---

# new-project

Scaffold a new project repo without a laptop. The web/mobile equivalent of
`mxr new <name> --template <stack>`: create the GitHub repo, seed it with the
matching `templates/<stack>/CLAUDE.md`, a README, and a `.gitignore`, then push.

## Inputs

Invoked as `/new-project <stack> <name> [flags]`.

- `<stack>` — one of `rust`, `ts`. Required.
- `<name>` — the new repository name. Required.
- `--org <org>` — create under an organization instead of the current user.
- `--public` — create a public repo. Default is private.

If `<stack>` or `<name>` is missing, or the stack is unsupported, stop and ask
rather than guessing.

## Stack → template

| `<stack>` | Template file                | `{{stack}}` substitution |
| --------- | ---------------------------- | ------------------------ |
| `rust`    | `templates/rust/CLAUDE.md`   | `Rust`                   |
| `ts`      | `templates/ts/CLAUDE.md`     | `TypeScript monorepo`    |

## Steps

1. **Validate.** Confirm `<stack>` is `rust` or `ts` and `<name>` is present. On
   anything else, stop and ask.
2. **Render CLAUDE.md.** Read `templates/<stack>/CLAUDE.md` from THIS repo.
   Replace every `{{name}}` with the project name and every `{{stack}}` with the
   substitution from the table above. This rendered text is the new repo's
   `CLAUDE.md`.
3. **Confirm the owner.** Call `get_me` to resolve the current GitHub user.
   The repo owner is `--org` if given, otherwise that user.
4. **Create the repo.** Use the GitHub integration's `create_repository`
   (name = `<name>`, private unless `--public`, `autoInit: true` so a default
   branch exists to push onto).
5. **Seed the files.** Push these to the new repo's default branch in a single
   commit ("chore: scaffold <name> from <stack> template"):
   - `CLAUDE.md` — the rendered template from step 2.
   - `README.md` — a minimal `# <name>` heading plus a one-line description.
   - `.gitignore` — appropriate for the stack (see below).
   Prefer `push_files` for one atomic commit; fall back to
   `create_or_update_file` per file if needed.
6. **Report.** Give the user the repo URL and the exact next-step command to
   start building: "Open a Claude Code session on `<owner>/<name>` and go."

## .gitignore per stack

- `rust`:
  ```
  /target
  Cargo.lock
  ```
  (keep `Cargo.lock` ignored only for libraries; for a binary, remove that line —
  ask if unsure.)
- `ts`:
  ```
  node_modules/
  dist/
  .turbo/
  *.log
  .DS_Store
  ```

## Notes

- Do NOT try to run the `mxr` binary or `gh` — neither is available in a web
  session. All GitHub actions go through the GitHub integration tools.
- Keep the scaffold minimal: CLAUDE.md + README + .gitignore. The user drives
  the rest interactively in the new repo's session. Don't invent extra files.
- This skill only creates the repo and seeds it. It does not set up deploy
  targets or CI secrets.
