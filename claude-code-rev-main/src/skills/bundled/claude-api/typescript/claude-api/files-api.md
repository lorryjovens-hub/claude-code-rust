# TypeScript Files API

Use the Files API when the same file needs to be referenced across multiple requests instead of uploading bytes each time.

## Typical flow

1. Upload a file once.
2. Store the returned file ID.
3. Reference that file ID in later requests.

## Upload example

```ts
const file = await client.files.create({
  file: new File(['hello'], 'example.txt', { type: 'text/plain' }),
  purpose: 'user_data',
})
```

## Guidance

- Persist file IDs in your own database; they are the stable handle for reuse.
- Use the Files API for repeated access, not one-off tiny payloads.
- Validate file type and size before upload in your app code.
- Treat uploaded files as user data and apply your normal retention rules.

## Good fits

- multi-turn document workflows
- repeated evaluation inputs
- analysis pipelines that reuse the same source material

## Avoid

- uploading files on every request when the same content is reused
- assuming a local path is meaningful to the API

## References

- Anthropic docs: Files API
