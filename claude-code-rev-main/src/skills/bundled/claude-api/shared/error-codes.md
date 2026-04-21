# Error Codes

Use this note when the user asks why an Anthropic API call failed or how to harden error handling.

## What To Check First

- HTTP status code
- Anthropic error type and message body
- Whether the failure is transient or request-specific
- Whether streaming failed before completion or before the first event

## Practical Triage

- `400`-class errors usually mean request shape, model choice, auth headers, or unsupported parameters need to be fixed before retrying.
- `401` and `403` usually indicate key, workspace, or permission issues rather than a temporary outage.
- `404` often means the referenced resource, model, batch, or file ID is wrong for the current API surface.
- `429` should be handled with backoff, retry, and concurrency control.
- `5xx` should be treated as transient service failures and retried with jittered exponential backoff.

## Implementation Guidance

- Log the Anthropic request ID when available so failed requests can be correlated.
- Surface response body details in server logs, but avoid leaking full prompts or secrets to users.
- Make retries idempotent where possible, especially for batch polling and file workflows.
- Distinguish validation failures from transport failures in your code paths.

## Recovery Pattern

1. Validate request construction locally.
2. Retry only transient classes such as rate limits and server errors.
3. For streaming, decide whether to resume, restart, or fall back to non-streaming output.
4. If the user asks for exact current error semantics, verify against Anthropic’s live API reference.
