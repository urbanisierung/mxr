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
<!-- Parts 1 & 2 (behavioral + coding quality) live in .github/copilot-instructions.md -->

## Tech Stack

- **Architecture:** Turborepo monorepo
- **Language:** TypeScript (strict mode) — latest stable
- **Runtime:** Node.js — latest LTS
- **Package Manager:** pnpm (workspaces)
- **Build System:** Turborepo
- **Testing:** Vitest
- **Linting & Formatting:** Biome
- **Frontend:** Preact 10 (same API as React, 3KB)
- **State Management:** Zustand (preferred over raw Preact hooks for all state management)
- **Dependencies:** Latest versions only; prefer mature, well-maintained packages
- **Dependency Policy:** Only add external packages when functionality cannot be reasonably implemented in-repo

## Project Structure

Follow Turborepo best practices:

```
apps/
  frontend/        # Preact PWA (Vite + vite-plugin-pwa)
packages/
  ui/              # Shared design system: tokens, components, theme store
turbo.json         # Turborepo pipeline configuration
package.json       # Root package.json (all devDependencies here)
biome.json         # Biome configuration
```

## Adding a New Package

When creating a new published package under `packages/` or `apps/`, complete all of the following before finishing:

1. **`package.json` required fields** — must include: `description`, `keywords` (≥ 3 entries), `license: "MIT"`, `repository.url` (pointing to `github.com/pagome-app/monorepo`), `homepage: "https://pagome.com"`, `bugs.url`, `publishConfig.access: "public"`. Match the field order of existing packages (build config first, then metadata).

2. **`scripts/generate-readmes.mjs`** — add an entry to the `packages` object with `name`, `description`, and `content` (Overview, Features, Installation, Quick Start, API Reference). Also add the package to the `footer()` function's related packages list.

3. **`scripts/sync-license.mjs`** — add the package path to the `PUBLISHED` array.

4. **`scripts/check-packages.mjs`** — add the package path to the `PUBLISHED` array.

5. **Run the scripts** to generate `LICENSE` and `README.md`:
   ```sh
   node scripts/sync-license.mjs
   node scripts/generate-readmes.mjs
   node scripts/check-packages.mjs   # must exit 0
   ```

Never hand-write a `README.md` or `LICENSE` for a published package — they are generated/synced by these scripts and will be overwritten on the next build.

## Dependency Management

- **All `devDependencies` must be declared in the root `package.json`** — never in individual app or package `package.json` files
- Runtime `dependencies` belong in the respective app/package `package.json`

## Build & Check Commands

- Build: `pnpm turbo build`
- Lint & Format: `pnpm turbo check` or `pnpm biome check .`
- Typecheck: `pnpm turbo typecheck` or `pnpm tsc --noEmit`
- Test: `pnpm turbo test`

## Backend Guidelines

- **ESM only** — use `"type": "module"` in `package.json`, use `.js` extensions in imports
- Follow latest Node.js best practices and recommendations
- Use modern APIs: `fetch`, `node:` protocol imports, top-level `await`
- Prefer native Node.js APIs over third-party packages where possible
- Structure code for testability and separation of concerns

## Frontend Guidelines

- **Preact** for all UI components — import from `preact` and `preact/hooks`, not `react`
- **Zustand** for state management — centralize state in stores, avoid scattering `useState`/`useEffect` across components
- Use **discriminated union state machines** in Zustand stores for all multi-step UI flows (routing, forms, async views)
- Only use raw Preact hooks (`useState`, `useEffect`, `useRef`, etc.) when Zustand does not cover the use case
- Optimize for render performance: minimize re-renders, use selectors in Zustand stores

## Brand Tokens

All brand colors and design tokens are defined in **`packages/ui`** (`@pagome/ui`).

- **Single source of truth**: CSS values live in `packages/ui/src/tokens.css`; JS/canvas mirror in `packages/ui/src/tokens.ts` — keep both in sync
- **Token namespace**: all public tokens use `--pagome-*` prefix
- **Usage**: `@import "@pagome/ui/tokens.css"` in app CSS entry points; import `TOKENS` from `@pagome/ui` for canvas/JS contexts
- **Never hardcode brand colors** — always use `var(--pagome-*)` CSS variables
- **Dark mode**: auto via `@media (prefers-color-scheme: dark)`; manual override via `[data-theme]` attribute; user preference persisted in the theme store (`packages/ui/src/theme.ts`)

## TypeScript Practices

- Prefer compile-time (type-level) guarantees over runtime checks
- Write idiomatic TypeScript: use discriminated unions, template literals, and branded types where appropriate
- TypeScript strict mode — zero type errors across the entire monorepo
- Biome — zero warnings, zero errors

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