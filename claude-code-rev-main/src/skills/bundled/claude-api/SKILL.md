# Claude API

Use this skill when the user is building against Anthropic APIs or SDKs, including `@anthropic-ai/sdk`, `anthropic`, or Agent SDK integrations.

## What This Skill Covers

- Messages API basics across supported languages
- Streaming responses and incremental rendering
- Prompt caching for repeated context
- Tool use and agent-style orchestration
- Batches and Files API workflows
- Model selection and error handling

## Working Rules

- Prefer Anthropic official docs and SDK idioms over generic LLM advice.
- Keep examples aligned with the user’s detected language when possible.
- Use the language-specific `README.md` for standard request flow, auth, and request shape.
- Use the shared docs for topics that cut across all SDKs, such as models, caching, tool-use concepts, and error codes.
- If the user asks for exact current model IDs, feature availability, or pricing, verify against Anthropic’s live docs before answering.

## Reading Guide

- Basic request/response flow: `{lang}/claude-api/README.md`
- Streaming output: `{lang}/claude-api/streaming.md`
- Tool use: `shared/tool-use-concepts.md` and `{lang}/claude-api/tool-use.md`
- Prompt caching: `shared/prompt-caching.md`
- Batch processing: `{lang}/claude-api/batches.md`
- File upload flows: `{lang}/claude-api/files-api.md`
- Model choice or naming: `shared/models.md`
- API and SDK failures: `shared/error-codes.md`
- Live sources for fresh answers: `shared/live-sources.md`

## Response Style

- Give production-usable examples, not pseudocode, when the user asks for implementation help.
- Call out when you are making an inference from the docs rather than repeating an explicit guarantee.
- If the user’s request depends on fast-changing details such as model names or pricing, browse Anthropic docs and cite the relevant page.
