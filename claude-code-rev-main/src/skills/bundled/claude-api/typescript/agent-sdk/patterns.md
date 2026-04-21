# TypeScript Agent SDK Patterns

## Narrow tool access

Start with the smallest tool surface that can finish the task.

```ts
const agent = new Agent({
  model: 'claude-sonnet-4-6',
  systemPrompt: 'Review code and report findings only.',
  allowedTools: ['Read', 'Glob', 'Grep'],
})
```

Use write-capable tools only when the task truly requires edits.

## Separate planning from execution

For longer workflows, run an analysis pass first, then a second pass that can edit or execute.

```ts
const review = await reviewAgent.run('Find the highest-risk regression.')
const fix = await fixAgent.run(`Implement this change:\n\n${review.outputText}`)
```

This reduces accidental edits and makes logs easier to audit.

## Constrain the workspace

- Pass explicit directories or file lists when the task is repo-scoped.
- Prefer short user prompts with exact success criteria.
- If the task is high risk, force structured output such as JSON or a checklist.

## Use agents for orchestration, not hidden business logic

- Keep validation, persistence, and authorization in application code.
- Let the agent decide how to solve a task within boundaries you define.
- Re-check outputs before applying them to production systems.

## Good use cases

- Repository review bots
- Migration assistants
- Incident triage helpers
- Controlled code editing workflows

## Avoid

- Giving one agent unrestricted shell, network, and file access by default
- Relying on implicit context instead of attaching the exact files or instructions
- Treating agent text as trusted structured data without validation
