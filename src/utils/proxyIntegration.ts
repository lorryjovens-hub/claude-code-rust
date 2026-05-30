import { apiProxy, AnthropicRequest, ProviderConfig } from './apiProxy';

export interface StreamEventHandlers {
  onDelta: (delta: string, full: string) => void;
  onThinking?: (thinking: string, full: string) => void;
  onToolUse?: (event: { type: 'start' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: Record<string, unknown>; output?: string; is_error?: boolean }) => void;
  onSystem?: (event: string, data: Record<string, unknown>) => void;
  onDone: (full: string) => void;
  onError: (err: string) => void;
}

interface ProxyMessage {
  role: 'user' | 'assistant' | 'system';
  content: string | unknown[];
  [key: string]: unknown;
}

function convertAnthropicToProxyRequest(
  conversationId: string,
  messages: ProxyMessage[],
  model: string,
  systemPrompt?: string
): AnthropicRequest {
  const anthropicMessages = messages.map((msg) => ({
    role: msg.role,
    content: msg.content,
  })) as AnthropicRequest['messages'];

  return {
    model,
    messages: anthropicMessages as AnthropicRequest['messages'],
    system: systemPrompt,
    max_tokens: 1000000,
    stream: true,
  };
}

function parseSSEStream(stream: ReadableStream, handlers: StreamEventHandlers): void {
  const reader = stream.getReader();
  const decoder = new TextDecoder();
  let buffer = '';
  let fullText = '';
  let thinkingText = '';
  let currentToolUseId: string | null = null;
  let currentToolName: string | undefined = undefined;
  let inToolInput = false;

  const processLine = (line: string) => {
    if (!line.startsWith('data: ')) return;
    const data = line.slice(6);
    if (data === '[DONE]') return;

    try {
      const event = JSON.parse(data);
      const eventType = event.type || event.event?.type;

      switch (eventType) {
        case 'message_start':
          handlers.onSystem?.('message_start', { model: event.message?.model });
          break;

        case 'content_block_start':
          if (event.content_block?.type === 'tool_use') {
            currentToolUseId = event.content_block.id;
            currentToolName = event.content_block.name || undefined;
            inToolInput = true;
            handlers.onToolUse?.({
              type: 'start',
              tool_use_id: currentToolUseId || '',
              tool_name: currentToolName,
              tool_input: {},
            });
          }
          break;

        case 'content_block_delta':
          if (event.delta?.type === 'text_delta' && event.delta.text) {
            fullText += event.delta.text;
            handlers.onDelta(event.delta.text, fullText);
          } else if (event.delta?.type === 'input_json_delta' && event.delta.partial_json) {
            // Tool input delta - partial JSON
          } else if (event.delta?.type === 'thinking_delta' && event.delta.thinking) {
            thinkingText += event.delta.thinking;
            handlers.onThinking?.(event.delta.thinking, thinkingText);
          }
          break;

        case 'content_block_stop':
          break;

        case 'message_delta':
          if (event.delta?.stop_reason) {
            handlers.onSystem?.('message_delta', { stop_reason: event.delta.stop_reason });
          }
          break;

        case 'message_stop':
          handlers.onDone(fullText);
          break;

        case 'error':
          handlers.onError(event.error || 'Unknown error');
          break;
      }
    } catch (e) {
      // Skip malformed JSON
    }
  };

  const readChunk = async () => {
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) {
          if (buffer) {
            for (const line of buffer.split('\n')) {
              processLine(line);
            }
          }
          if (!fullText && !thinkingText) {
            handlers.onDone('');
          }
          break;
        }

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          processLine(line);
        }
      }
    } catch (e) {
      handlers.onError(`Stream error: ${e}`);
    }
  };

  readChunk();
}

export async function sendMessageWithProxy(
  conversationId: string,
  messages: ProxyMessage[],
  model: string,
  providers: ProviderConfig[],
  handlers: StreamEventHandlers,
  systemPrompt?: string,
  signal?: AbortSignal
): Promise<void> {
  if (providers.length === 0) {
    handlers.onError('No providers configured');
    return;
  }

  apiProxy.setProviders(providers);

  const request = convertAnthropicToProxyRequest(conversationId, messages, model, systemPrompt);

  try {
    const stream = await apiProxy.chatStream(request);
    parseSSEStream(stream, handlers);
  } catch (e: unknown) {
    handlers.onError(e instanceof Error ? e.message : 'Failed to send message');
  }
}

export async function sendMessageDirect(
  provider: ProviderConfig,
  request: AnthropicRequest,
  handlers: StreamEventHandlers
): Promise<void> {
  apiProxy.setProviders([provider]);

  try {
    const stream = await apiProxy.chatStream(request);
    parseSSEStream(stream, handlers);
  } catch (e: unknown) {
    handlers.onError(e instanceof Error ? e.message : 'Failed to send message');
  }
}
