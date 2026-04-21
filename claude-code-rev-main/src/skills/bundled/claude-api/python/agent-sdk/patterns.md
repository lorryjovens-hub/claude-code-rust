# Python Agent SDK Patterns

## Pattern 1: one-shot automation

Use `query()` for isolated tasks such as "fix this file", "review this diff", or "generate release notes".

```python
async for message in query(
    prompt="Inspect src/main.py and explain the bug.",
    options=ClaudeAgentOptions(cwd="."),
):
    print(message)
```

Best when conversation history is not needed.

## Pattern 2: conversational workflow

Use `ClaudeSDKClient` when each answer should build on earlier context.

```python
client = ClaudeSDKClient(options=ClaudeAgentOptions(cwd="."))
await client.connect()
await client.query("Read the auth flow.")
await client.query("Now propose a refactor with minimal risk.")
```

Best for REPLs, chat UIs, or multi-step repair loops.

## Pattern 3: custom tools

Expose deterministic local logic as tools and let Claude decide when to call them.

```python
from claude_agent_sdk import tool


@tool("get_build_id", "Return the current build identifier", {})
async def get_build_id(_args):
    return {"content": [{"type": "text", "text": "build-2026-03-31"}]}
```

Keep tools narrow, typed, and side-effect conscious.

## Pattern 4: hooks and policy

Use hooks when you need approval gates, logging, or organization-specific constraints before or after tool use. Put policy in hooks or permission settings, not in the prompt alone.

## Pattern 5: streaming UIs

Enable partial messages when building terminal or web interfaces that should render text and tool calls as they arrive. Treat streamed events as incremental updates; keep the final `AssistantMessage` or `ResultMessage` as the source of truth.

## Pattern 6: robust session control

- Set `cwd` explicitly.
- Set permission mode deliberately.
- Close `ClaudeSDKClient` in `finally`.
- Surface `CLINotFoundError`, connection errors, and permission denials to the caller.

## When not to use the Agent SDK

- Use the Anthropic Python SDK for plain `messages.create(...)` workflows.
- Use the Messages API directly when you need raw API semantics, provider portability, or no Claude Code dependency.
