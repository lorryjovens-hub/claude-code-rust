# cURL Examples

Use these when you want a raw HTTP baseline before moving to an SDK.

## Basic Message

```bash
curl https://api.anthropic.com/v1/messages \
  --header "x-api-key: $ANTHROPIC_API_KEY" \
  --header "anthropic-version: 2023-06-01" \
  --header "content-type: application/json" \
  --data '{
    "model": "{{SONNET_ID}}",
    "max_tokens": 256,
    "messages": [
      {"role": "user", "content": "Write a two-line release note summary."}
    ]
  }'
```

## With System Prompt

```bash
curl https://api.anthropic.com/v1/messages \
  --header "x-api-key: $ANTHROPIC_API_KEY" \
  --header "anthropic-version: 2023-06-01" \
  --header "content-type: application/json" \
  --data '{
    "model": "{{SONNET_ID}}",
    "max_tokens": 256,
    "system": "You are a terse technical assistant.",
    "messages": [
      {"role": "user", "content": "Explain eventual consistency in one paragraph."}
    ]
  }'
```

## JSON Output Pattern

```bash
curl https://api.anthropic.com/v1/messages \
  --header "x-api-key: $ANTHROPIC_API_KEY" \
  --header "anthropic-version: 2023-06-01" \
  --header "content-type: application/json" \
  --data '{
    "model": "{{SONNET_ID}}",
    "max_tokens": 256,
    "messages": [
      {
        "role": "user",
        "content": "Return JSON only: {\"severity\": string, \"summary\": string} for a database outage."
      }
    ]
  }'
```

## Notes

- Start with cURL when debugging headers, auth, or payload shape.
- Move to a language client once the request body is stable.
- For streaming, batches, and files, use the corresponding skill docs instead of extending these one-off shell commands.
