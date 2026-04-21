# TypeScript Batches

Use message batches when latency is not interactive and you want to process many independent requests efficiently.

## When batches fit

- large backfills
- offline classification
- nightly summarization jobs
- document queues where each task is independent

## Create a batch

```ts
const batch = await client.messages.batches.create({
  requests: [
    {
      custom_id: 'item-1',
      params: {
        model: 'claude-sonnet-4-6',
        max_tokens: 512,
        messages: [{ role: 'user', content: 'Classify this ticket.' }],
      },
    },
  ],
})
```

## Operational guidance

- Set a stable `custom_id` for each request so you can join results back to your job records.
- Keep each request self-contained; batches are not conversational threads.
- Poll batch status and then fetch results once processing completes.
- Design for partial failure. Some requests may succeed while others error.

## Avoid

- using batches for chat UIs
- assuming result order is the same as input order
- sending dependent tasks that require previous model output

## References

- Anthropic docs: Message Batches API
