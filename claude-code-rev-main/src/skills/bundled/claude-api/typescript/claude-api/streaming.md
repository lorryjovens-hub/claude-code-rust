# TypeScript Streaming

Use streaming when the user should see output incrementally or when you want earlier visibility into tool use and long generations.

## Basic pattern

```ts
const stream = await client.messages.stream({
  model: 'claude-sonnet-4-6',
  max_tokens: 1024,
  messages: [{ role: 'user', content: 'Draft a release note.' }],
})

for await (const event of stream) {
  // Handle text deltas, message state, and completion events.
}
```

## UI guidance

- Render deltas as they arrive instead of waiting for the final message.
- Keep a final assembled value in state for persistence.
- Handle cancellation explicitly when users close the view or submit a new task.

## Reliability notes

- Expect event-driven parsing, not one final JSON payload.
- Preserve the final completed message if you need usage or stop reason metadata.
- Do not assume every stream yields only text; tool-related events can appear in compatible flows.

## Good fits

- chat UIs
- long summaries
- code generation views
- responsive terminal output

## References

- Anthropic docs: streaming Messages responses
