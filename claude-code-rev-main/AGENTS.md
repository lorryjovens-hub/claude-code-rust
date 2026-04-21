# Repository Guidelines

## Project Structure & Module Organization
Core source lives in `src/`. Entry points and CLI wiring are under files such as `src/dev-entry.ts`, `src/main.tsx`, and `src/commands.ts`. Feature code is grouped by area in folders like `src/commands/`, `src/services/`, `src/components/`, `src/tools/`, and `src/utils/`. Restored or compatibility code also appears in `vendor/` and local package shims in `shims/`. There is no dedicated `test/` directory in the restored tree today; treat focused validation near the changed module as the default.

## Build, Test, and Development Commands
Use Bun for local development.

- `bun install`: install dependencies and local shim packages.
- `bun run dev`: start the restored CLI entrypoint interactively.
- `bun run start`: alias for the development entrypoint.
- `bun run version`: verify the CLI boots and prints its version.

If you change TypeScript modules, run the relevant command above and verify the affected flow manually. This repository does not currently expose a first-class `lint` or `test` script in `package.json`.

## Coding Style & Naming Conventions
The codebase is TypeScript-first with ESM imports and `react-jsx`. Match the surrounding file style exactly: many files omit semicolons, use single quotes, and prefer descriptive camelCase for variables and functions, PascalCase for React components and manager classes, and kebab-case for command folders such as `src/commands/install-slack-app/`. Keep imports stable when comments warn against reordering. Prefer small, focused modules over broad utility dumps.

## Testing Guidelines
There is no consolidated automated test suite configured at the repository root yet. For contributor changes, use targeted runtime checks:

- boot the CLI with `bun run dev`
- smoke-test version output with `bun run version`
- exercise the specific command, service, or UI path you changed

When adding tests, place them close to the feature they cover and name them after the module or behavior under test.

## Commit & Pull Request Guidelines
Git history currently starts with a single `first commit`, so no strong conventional pattern is established. Use short, imperative commit subjects, for example `Fix MCP config normalization`. Pull requests should explain the user-visible impact, note restoration-specific tradeoffs, list validation steps, and include screenshots only for TUI/UI changes.

## Restoration Notes
This is a reconstructed source tree, not pristine upstream. Prefer minimal, auditable changes, and document any workaround added because a module was restored with fallbacks or shim behavior.
