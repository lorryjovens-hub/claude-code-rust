# Python Batches

Use Message Batches when you have many independent requests and do not need interactive latency.

## Good fit

- nightly evaluations
- backfills and migrations
- large classification or summarization jobs
- bulk tasks where polling is easier than holding open many live connections

## Create a batch

```python
from anthropic import Anthropic

client = Anthropic()

batch = client.beta.messages.batches.create(
    requests=[
        {
            "custom_id": "ticket-1",
            "params": {
                "model": "claude-sonnet-4-5",
                "max_tokens": 512,
                "messages": [
                    {"role": "user", "content": "Summarize ticket #1"}
                ],
            },
        },
        {
            "custom_id": "ticket-2",
            "params": {
                "model": "claude-sonnet-4-5",
                "max_tokens": 512,
                "messages": [
                    {"role": "user", "content": "Summarize ticket #2"}
                ],
            },
        },
    ],
)
```

## Poll for completion

```python
batch = client.beta.messages.batches.retrieve(batch.id)
print(batch.processing_status)
print(batch.request_counts)
```

## Read results

Results are not guaranteed to come back in request order. Match on `custom_id`, not array position. Once processing finishes, fetch or iterate the batch results and join them back to your application records by `custom_id`.

## Practical guidance

- Use batches only for independent requests.
- Store your own mapping from `custom_id` to source record.
- Expect partial failures; some requests may succeed while others error or expire.
- If you need immediate user feedback, use normal Messages API calls instead.

## Official references

- Batch guide/reference: `https://platform.claude.com/docs/en/api/messages/batches`
- Python batch endpoints: `https://platform.claude.com/docs/en/api/python/beta/messages/batches/list`
