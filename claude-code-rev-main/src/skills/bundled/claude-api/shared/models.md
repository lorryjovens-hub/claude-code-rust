# Models

Use this note when the user asks which Claude model to choose or how model naming works.

## Selection Heuristic

- Use Opus for the hardest reasoning, long-horizon planning, or highest-stakes coding tasks.
- Use Sonnet as the default general-purpose choice for most product and engineering workloads.
- Use Haiku when latency and cost matter more than peak reasoning depth.

## Model Naming Guidance

- Anthropic exposes dated model IDs. Prefer exact IDs in shipped code when reproducibility matters.
- User-facing guidance can mention family names such as Opus, Sonnet, and Haiku, but implementation examples should use concrete model IDs.
- Do not invent date suffixes. Verify current IDs against the live models overview if the user asks for exact names.

## Context And Capability

- Model family choice is only one dimension; prompt design, tool setup, caching, and streaming strategy also affect behavior.
- Some advanced features may be model- or platform-specific. Confirm live availability before promising support.
- If the user is deciding between Anthropic API, Bedrock, and Vertex, check the official model matrix because IDs differ by platform.

## Practical Advice

- Default to the current Sonnet model for most app examples unless the user explicitly asks for premium capability or minimum cost.
- For migration work, separate “API compatibility” from “behavior compatibility”; a model swap that compiles may still change output quality or tool behavior.
