# Python Streaming

Use streaming when you want text as it arrives instead of waiting for the final message.

## Simple text streaming

```python
from anthropic import Anthropic

client = Anthropic()

with client.messages.stream(
    model="claude-sonnet-4-5",
    max_tokens=1024,
    messages=[{"role": "user", "content": "Write a short release note."}],
) as stream:
    for text in stream.text_stream:
        print(text, end="", flush=True)
```

## Get the final message

If you want streaming transport but still need the final structured `Message`, use the stream helper and then call the SDK method that returns the completed response object for the stream.

## When streaming is worth it

- chat or terminal UIs
- long generations
- progress feedback for users
- tool use flows where early visibility matters

## Practical guidance

- Treat streamed events as incremental UI updates.
- Keep the final assembled message as the authoritative result.
- Long non-streaming requests are more exposed to idle network timeouts than streaming or batches.
- If you do not need live tokens, normal `messages.create(...)` is simpler.

## Official references

- Streaming guide: `https://platform.claude.com/docs/en/build-with-claude/streaming`
- Python SDK: `https://platform.claude.com/docs/en/api/sdks/python`
