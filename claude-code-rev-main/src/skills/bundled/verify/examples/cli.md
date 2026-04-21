# Verify CLI Example

Use this pattern when the change affects a command, flag, formatter, or local workflow.

## Example

Change: update `claude doctor` output or bootstrap behavior.

Verification:

1. Run the narrowest command that hits the changed code path.
2. Re-run with a nearby flag if the change touched parsing or help text.
3. Check both output and exit behavior.

```bash
bun run version
bun run dev --help
```

## What to Look For

- Command exits successfully.
- Output includes the expected text, option, or command.
- The restored entrypoint reaches the real CLI path instead of a stub.

## Good Result Summary

- `Verified: bun run version printed the restored Claude Code version.`
- `Verified: bun run dev --help showed the full CLI command tree.`
- `Risk: interactive raw-mode flows were not exercised in a non-TTY shell.`
