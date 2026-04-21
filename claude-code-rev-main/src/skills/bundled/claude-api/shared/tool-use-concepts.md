# Tool Use Concepts

Use this note for tool calling, function execution, and agent loops built on Claude.

## Core Model

- You declare tools in the request.
- Claude decides whether to call a tool based on the prompt and tool choice configuration.
- Your application executes the tool and sends the result back in a follow-up message.
- Claude then continues reasoning with the tool result in context.

## Design Guidance

- Keep tool names clear and stable.
- Write descriptions for the model, not for human readers only.
- Use narrow input schemas with concrete field names and required fields.
- Return structured, minimal tool results when possible.

## Common Failure Modes

- Over-broad tools that make selection ambiguous
- Schemas that allow too many invalid states
- Tool results that are too verbose or omit critical fields
- Forgetting to loop until Claude stops requesting tools

## Practical Agent Advice

- Treat tool use as a protocol, not a one-shot helper call.
- Add application-level limits for tool recursion, retries, and external side effects.
- Separate client-side tools from server-side Anthropic tools when explaining implementation responsibilities.
- If the user asks for exact server tool types or current pricing, verify against the live tool-use docs.
