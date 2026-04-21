# Python Tool Use

Tool use lets Claude decide when to request a tool, emit a `tool_use` block, and continue after you return a matching `tool_result`.

## Define tools

```python
tools = [
    {
        "name": "get_weather",
        "description": "Return current weather for a city",
        "input_schema": {
            "type": "object",
            "properties": {
                "city": {"type": "string"},
            },
            "required": ["city"],
        },
    }
]
```

## First model call

```python
message = client.messages.create(
    model="claude-sonnet-4-5",
    max_tokens=1024,
    tools=tools,
    messages=[
        {"role": "user", "content": "What is the weather in Tokyo?"}
    ],
)
```

## Handle `tool_use` and continue

Inspect `message.content` for blocks with `type == "tool_use"`. Execute the requested tool, then send a follow-up user message containing a `tool_result` block whose `tool_use_id` matches the tool call.

```python
messages = [
    {"role": "user", "content": "What is the weather in Tokyo?"},
    {"role": "assistant", "content": message.content},
    {
        "role": "user",
        "content": [
            {
                "type": "tool_result",
                "tool_use_id": tool_use.id,
                "content": [{"type": "text", "text": "18C and clear"}],
            }
        ],
    },
]
```

Then call `client.messages.create(...)` again with the updated message history.

## Practical guidance

- Keep tool schemas small and strict.
- Return machine-readable data when possible.
- Match `tool_use_id` exactly.
- Expect multiple tool calls and possibly parallel tool requests.
- If you want higher-level orchestration, consider the Agent SDK instead of manual tool loops.

## Official references

- Tool use overview: `https://platform.claude.com/docs/en/agents-and-tools/tool-use/overview`
- Python Messages API reference: `https://platform.claude.com/docs/en/api/python/messages`
