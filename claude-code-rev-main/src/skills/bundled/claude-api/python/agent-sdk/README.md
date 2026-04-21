# Python Agent SDK

Use the Python Agent SDK when you want Claude Code's agent loop, tools, hooks, and session handling from Python instead of calling the Messages API directly.

## Install

```bash
pip install claude-agent-sdk
```

The SDK talks to a local Claude Code CLI, so the machine running your Python code also needs Claude Code installed and authenticated.

## Choose the right entrypoint

- `query(...)`: one-off tasks. Each call starts a fresh session.
- `ClaudeSDKClient(...)`: multi-turn or long-lived conversations. Reuses session state and supports interrupts.

## Minimal `query()` example

```python
import asyncio
from claude_agent_sdk import query, ClaudeAgentOptions


async def main() -> None:
    async for message in query(
        prompt="Review the repository and suggest the safest fix.",
        options=ClaudeAgentOptions(
            cwd=".",
            permission_mode="default",
        ),
    ):
        print(message)


asyncio.run(main())
```

## Minimal client example

```python
import asyncio
from claude_agent_sdk import ClaudeSDKClient, ClaudeAgentOptions


async def main() -> None:
    client = ClaudeSDKClient(
        options=ClaudeAgentOptions(cwd=".")
    )
    await client.connect()
    try:
        await client.query("Summarize the current branch status.")
        await client.query("Now suggest the next test to run.")
    finally:
        await client.close()


asyncio.run(main())
```

## Practical notes

- Prefer `query()` for scripts, cron jobs, and single task execution.
- Prefer `ClaudeSDKClient` when later prompts depend on earlier tool results.
- Pass `cwd`, permission settings, allowed tools, hooks, and custom tools through `ClaudeAgentOptions`.
- For incremental output, enable partial message streaming and handle `StreamEvent` messages.
- For raw model calls without Claude Code tools, use the Anthropic Python SDK instead of the Agent SDK.

## Official references

- Agent SDK quickstart: `https://platform.claude.com/docs/en/agent-sdk/quickstart`
- Agent SDK Python reference: `https://platform.claude.com/docs/en/agent-sdk/python`
