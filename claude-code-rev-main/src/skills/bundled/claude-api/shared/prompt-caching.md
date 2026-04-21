# Prompt Caching

Use this note when the user wants lower repeated-context cost or faster repeated requests.

## When It Helps

- Large static system prompts
- Reused tool definitions
- Long reference documents shared across many requests
- Multi-turn workflows where a stable prefix repeats

## Structuring Advice

- Put reusable content first.
- Keep the dynamic tail as small as possible.
- Cache stable instructions, examples, tools, and reference context before volatile user input.
- Avoid reordering cached sections between requests.

## Reasoning About Hits

- Cache usefulness depends on long shared prefixes, not just similar meaning.
- Small structural changes near the top of the prompt can break reuse.
- Tool definitions count toward the reusable prefix, so changing schemas can reduce hit rate.

## Debugging Low Hit Rates

- Compare request prefixes, not just the final prompt text mentally.
- Check whether timestamps, request IDs, or per-turn metadata are being inserted too early.
- Make sure the application is not rebuilding tool definitions or examples in a different order.

## Guidance For Answers

- Explain caching as an optimization for repeated static prefix content.
- If the user asks for current support matrix or exact API fields, verify against Anthropic’s live prompt caching docs.
