# Verify Server Example

Use this pattern when the change affects an API handler, transport, worker, or server-side integration.

## Example

Change: update a request handler or startup bootstrap for a local service.

Verification:

1. Start only the service or subprocess needed for the changed path.
2. Send a minimal request that reaches the edited code.
3. Confirm status, payload shape, and any important side effect.

```bash
# Example pattern only; adapt to the repo's actual server entrypoint
bun run dev -- --some-server-mode
curl -i http://127.0.0.1:PORT/health
```

## What to Look For

- Process starts without crashing.
- Request returns the expected status code.
- Response shape matches the intended contract.
- Logs do not show obvious runtime errors on the touched path.

## Good Result Summary

- `Verified: server booted and accepted a health-check request.`
- `Verified: the changed handler returned the expected status and payload.`
- `Not verified: unrelated routes and production-only integrations.`
