---
description: Verify a code change by running the app, the relevant command, or a focused server flow and reporting concrete evidence.
---

# Verify

Use this skill when a task is not finished until the change is exercised.

## Goal

Produce a short verification result grounded in execution, not inference. Prefer the narrowest check that proves the changed behavior works.

## Workflow

1. Identify the changed surface area.
2. Pick the smallest realistic verification path.
3. Run the relevant command or request flow.
4. Capture the observable result: exit status, key output, HTTP status, or changed behavior.
5. Report what passed, what was not verified, and any remaining risk.

## Rules

- Do not claim success without running something.
- Prefer focused checks over broad smoke tests.
- If the repo has no formal test target, use the nearest runnable workflow.
- If a check is blocked by environment limits, state that explicitly.
- Include exact commands when they are useful to repeat the verification.

## Verification Patterns

### CLI changes

- Run the exact command path affected by the edit.
- Check help text, flags, output formatting, exit codes, and side effects.
- For interactive flows, prefer the most scriptable subcommand first.

See `examples/cli.md`.

### Server changes

- Start only the needed service.
- Exercise the changed route, handler, or background path.
- Validate status code, response shape, logs, and failure handling.

See `examples/server.md`.

## Reporting Format

- `Verified:` what you ran and what passed.
- `Not verified:` anything you could not run.
- `Risk:` the main remaining uncertainty, if any.
