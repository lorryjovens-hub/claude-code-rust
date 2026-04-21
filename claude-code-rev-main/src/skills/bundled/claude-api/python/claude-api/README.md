# Python Claude API

Use the official Anthropic Python SDK for direct Claude API access from Python.

## Install

```bash
pip install anthropic
```

Optional extras:

```bash
pip install anthropic[aiohttp]
pip install anthropic[bedrock]
pip install anthropic[vertex]
```

## Basic sync request

```python
import os
from anthropic import Anthropic

client = Anthropic(api_key=os.environ["ANTHROPIC_API_KEY"])

message = client.messages.create(
    model="claude-sonnet-4-5",
    max_tokens=1024,
    messages=[
        {"role": "user", "content": "Summarize the latest changelog."}
    ],
)

print(message.content)
```

## Async request

```python
import os
import asyncio
from anthropic import AsyncAnthropic


async def main() -> None:
    client = AsyncAnthropic(api_key=os.environ["ANTHROPIC_API_KEY"])
    message = await client.messages.create(
        model="claude-sonnet-4-5",
        max_tokens=1024,
        messages=[{"role": "user", "content": "Hello, Claude"}],
    )
    print(message.content)


asyncio.run(main())
```

## Use this SDK when

- you want raw `messages.create(...)` access
- you need sync or async Python clients
- you are implementing streaming, tool use, batches, or Files API directly

## Practical guidance

- Prefer current stable model aliases or exact IDs your application supports.
- Keep long-lived static context at the start of the request so prompt caching can help.
- Use streaming for long outputs or latency-sensitive UIs.
- Use Batches for large asynchronous jobs.
- Use Files API when the same document or image must be referenced across requests.

## Official references

- Python SDK: `https://platform.claude.com/docs/en/api/sdks/python`
- Client SDK overview: `https://platform.claude.com/docs/en/api/client-sdks`
- Messages API reference: `https://platform.claude.com/docs/en/api/python/messages`
