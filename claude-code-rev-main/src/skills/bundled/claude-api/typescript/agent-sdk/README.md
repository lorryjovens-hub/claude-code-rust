# TypeScript Agent SDK

Use the Claude Agent SDK when you want Claude Code style agents from Node or Bun, not just raw model calls. Install the package, create an agent, then run tasks through the harness.

## Install

```bash
npm install @anthropic-ai/claude-agent-sdk
```

Set `ANTHROPIC_API_KEY` unless your runtime provides Claude Code style auth separately.

## Minimal flow

```ts
import { Agent } from '@anthropic-ai/claude-agent-sdk'

const agent = new Agent({
  model: 'claude-sonnet-4-6',
  systemPrompt: 'You are a precise engineering assistant.',
})

const result = await agent.run('Summarize the repository layout.')
console.log(result.outputText)
```

## When to choose Agent SDK

- Use Agent SDK for multi-step tasks, tool use, file edits, shell execution, or MCP access.
- Use `@anthropic-ai/sdk` instead when you only need direct Messages API calls.

## Practical guidance

- Keep the system prompt short and role-specific.
- Limit enabled tools to the minimum needed for the task.
- Prefer explicit task boundaries such as “analyze only” or “edit only these files”.
- Capture structured output in your application instead of parsing long prose after the fact.

## Operational notes

- Reuse agent instances for related work when you want consistent behavior.
- Create fresh agents when permissions, tool sets, or task roles change materially.
- Treat tool access as part of your security model; do not expose shell or file tools by default.

## References

- Anthropic docs: Claude Code SDK overview and API reference
- GitHub: `anthropics/claude-agent-sdk-typescript`
