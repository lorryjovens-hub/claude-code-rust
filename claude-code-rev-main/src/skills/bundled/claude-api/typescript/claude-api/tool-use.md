# TypeScript Tool Use

Tool use lets Claude request structured actions from your application. Your code defines the tools, executes them, then sends tool results back into the conversation.

## Define a tool

```ts
const tools = [
  {
    name: 'get_weather',
    description: 'Fetch the current weather for a city',
    input_schema: {
      type: 'object',
      properties: {
        city: { type: 'string' },
      },
      required: ['city'],
    },
  },
]
```

## Request with tools

```ts
const message = await client.messages.create({
  model: 'claude-sonnet-4-6',
  max_tokens: 1024,
  tools,
  messages: [{ role: 'user', content: 'What is the weather in Tokyo?' }],
})
```

If Claude returns a tool-use block, execute the named tool in your application, then append a `tool_result` turn and call the API again.

## Practical guidance

- Keep tool schemas narrow and explicit.
- Validate tool input before execution.
- Return structured, minimal results instead of raw logs when possible.
- Put authorization and side-effect checks in your code, not in the model prompt.

## Avoid

- exposing shell or network primitives unless absolutely necessary
- giving tools vague names or overly broad schemas
- skipping retries and timeout handling around real integrations

## References

- Anthropic docs: tool use
- Anthropic docs: Messages API
