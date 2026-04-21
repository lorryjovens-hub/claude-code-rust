# TypeScript Claude API

Use `@anthropic-ai/sdk` for direct access to the Messages API from TypeScript.

## Install

```bash
npm install @anthropic-ai/sdk
```

Set `ANTHROPIC_API_KEY` in the environment.

## Basic request

```ts
import Anthropic from '@anthropic-ai/sdk'

const client = new Anthropic({
  apiKey: process.env.ANTHROPIC_API_KEY,
})

const message = await client.messages.create({
  model: 'claude-sonnet-4-6',
  max_tokens: 1024,
  messages: [
    { role: 'user', content: 'Summarize this change in 3 bullets.' },
  ],
})
```

Read text from `message.content` by filtering for `type === 'text'`.

## Recommended request shape

- Always set `model` and `max_tokens` explicitly.
- Put instructions in a stable system prompt when they should apply across turns.
- Keep user messages task-focused and attach only the context needed for the turn.
- Use streaming for responsive UIs and tool use for structured external actions.

## Conversation pattern

Pass prior turns back in `messages` when you need continuity. Keep the conversation compact; summarize or trim old turns instead of replaying large transcripts forever.

## Model choice

- `claude-opus-4-6`: highest reasoning quality
- `claude-sonnet-4-6`: general default for most product workloads
- `claude-haiku-4-5`: lower-latency and lower-cost tasks

Check the current model catalog before hard-coding IDs in production.

## References

- Anthropic docs: Messages API
- Anthropic docs: model overview
