export interface AnthropicMessage {
  role: 'user' | 'assistant' | 'system';
  content: AnthropicContent | string;
}

export interface AnthropicContent {
  type: 'text' | 'image' | 'tool_use' | 'tool_result';
  text?: string;
  source?: {
    type: 'base64';
    media_type: string;
    data: string;
  };
  id?: string;
  name?: string;
  input?: Record<string, any>;
  tool_use_id?: string;
  content?: string;
  is_error?: boolean;
}

export interface AnthropicTool {
  name: string;
  description?: string;
  input_schema: Record<string, any>;
}

export interface AnthropicRequest {
  model: string;
  messages: AnthropicMessage[];
  system?: string;
  max_tokens: number;
  stream?: boolean;
  tools?: AnthropicTool[];
  thinking?: {
    type: 'enabled';
    budget_tokens?: number;
  };
  temperature?: number;
}

export interface OpenAIMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string | OpenAIContent[];
  name?: string;
  tool_call_id?: string;
  tool_calls?: {
    id: string;
    type: 'function';
    function: {
      name: string;
      arguments: string;
    };
  }[];
}

export interface OpenAIContent {
  type: 'text' | 'image_url';
  text?: string;
  image_url?: {
    url: string;
  };
}

export interface OpenAIRequest {
  model: string;
  messages: OpenAIMessage[];
  max_tokens?: number;
  stream?: boolean;
  tools?: {
    type: 'function';
    function: {
      name: string;
      description?: string;
      parameters: Record<string, any>;
    };
  }[];
  enable_thinking?: boolean;
  parallel_tool_calls?: boolean;
  temperature?: number;
}

export interface ProviderConfig {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  api_format: 'anthropic' | 'openai';
  enabled: boolean;
  models: Array<{
    id: string;
    name: string;
    enabled: boolean;
    supports_vision?: boolean;
    supports_web_search?: boolean;
  }>;
  web_search_strategy?: string;
}

function normalizeBaseUrl(url: string): string {
  if (!url) return url;
  let clean = url.replace(/\/+$/, '');
  clean = clean.replace(/\/(chat\/completions|messages)$/, '');
  return clean.replace(/\/+$/, '');
}

export function anthropicToOpenAIMessages(anthropicMessages: AnthropicMessage[], anthropicSystem?: string): OpenAIMessage[] {
  const openaiMessages: OpenAIMessage[] = [];

  if (anthropicSystem) {
    openaiMessages.push({
      role: 'system',
      content: anthropicSystem,
    });
  }

  for (const msg of anthropicMessages) {
    if (msg.role === 'user') {
      const content = msg.content;
      if (typeof content === 'string') {
        openaiMessages.push({ role: 'user', content });
      } else if (Array.isArray(content)) {
        const textParts: string[] = [];
        const imageParts: OpenAIContent[] = [];

        for (const block of content) {
          if (block.type === 'text' && block.text) {
            textParts.push(block.text);
          } else if (block.type === 'image' && block.source) {
            imageParts.push({
              type: 'image_url',
              image_url: {
                url: `data:${block.source.media_type};base64,${block.source.data}`,
              },
            });
          } else if (block.type === 'tool_result') {
            // Tool results are handled separately
            const toolContent = typeof block.content === 'string' 
              ? block.content 
              : JSON.stringify(block.content);
            openaiMessages.push({
              role: 'tool',
              tool_call_id: block.tool_use_id || '',
              content: toolContent,
            });
          }
        }

        if (imageParts.length > 0) {
          const textContent = textParts.join('').trim();
          const combinedContent: OpenAIContent[] = [];
          if (textContent) {
            combinedContent.push({ type: 'text', text: textContent });
          }
          combinedContent.push(...imageParts);
          openaiMessages.push({ role: 'user', content: combinedContent });
        } else if (textParts.length > 0) {
          openaiMessages.push({ role: 'user', content: textParts.join('') });
        }
      }
    } else if (msg.role === 'assistant') {
      const content = msg.content;
      if (typeof content === 'string') {
        openaiMessages.push({ role: 'assistant', content });
      } else if (Array.isArray(content)) {
        const textParts: string[] = [];
        const toolCalls: OpenAIMessage['tool_calls'] = [];

        for (const block of content) {
          if (block.type === 'text' && block.text) {
            textParts.push(block.text);
          } else if (block.type === 'tool_use') {
            toolCalls.push({
              id: block.id || `call_${Date.now()}`,
              type: 'function',
              function: {
                name: block.name || '',
                arguments: JSON.stringify(block.input || {}),
              },
            });
          }
        }

        if (toolCalls.length > 0) {
          openaiMessages.push({
            role: 'assistant',
            content: textParts.join('') || '',
            tool_calls: toolCalls,
          });
        } else if (textParts.length > 0) {
          openaiMessages.push({ role: 'assistant', content: textParts.join('') });
        } else {
          openaiMessages.push({ role: 'assistant', content: '' });
        }
      }
    } else if (msg.role === 'system') {
      // System messages are already handled above
    }
  }

  return openaiMessages;
}

export function anthropicToOpenAITools(anthropicTools?: AnthropicTool[]): OpenAIRequest['tools'] {
  if (!anthropicTools || anthropicTools.length === 0) return undefined;

  return anthropicTools.map(tool => ({
    type: 'function',
    function: {
      name: tool.name,
      description: tool.description || '',
      parameters: tool.input_schema || { type: 'object', properties: {} },
    },
  }));
}

export async function sendOpenAIRequest(
  provider: ProviderConfig,
  request: AnthropicRequest
): Promise<Response> {
  const baseUrl = normalizeBaseUrl(provider.base_url);
  const endpoint = baseUrl.endsWith('/v1') 
    ? `${baseUrl}/chat/completions`
    : `${baseUrl}/v1/chat/completions`;

  const openaiMessages = anthropicToOpenAIMessages(request.messages, request.system);
  
  const isQwenOrGLM = /qwen|glm|deepseek|minimax/i.test(request.model);
  
  const openaiBody: OpenAIRequest = {
    model: request.model,
    messages: openaiMessages,
    max_tokens: Math.min(request.max_tokens || 1000000, 2000000),
    stream: request.stream || false,
    tools: anthropicToOpenAITools(request.tools),
    temperature: request.temperature,
  };

  if (isQwenOrGLM) {
    openaiBody.parallel_tool_calls = false;
  }

  if (request.thinking && request.thinking.type === 'enabled' && (!request.tools || request.tools.length === 0)) {
    openaiBody.enable_thinking = true;
  }

  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${provider.api_key}`,
    },
    body: JSON.stringify(openaiBody),
  });

  return response;
}

export function parseOpenAIStreamResponse(
  reader: ReadableStreamDefaultReader<Uint8Array>,
  model: string
): ReadableStream {
  const decoder = new TextDecoder();
  let buffer = '';
  let contentBlockIndex = 0;
  let textBlockIndex: number | null = null;
  let isFirstToken = true;

  return new ReadableStream({
    async start(controller) {
      const processChunk = async (chunk: Uint8Array): Promise<void> => {
        const text = decoder.decode(chunk, { stream: true });
        buffer += text;
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (!line.startsWith('data: ')) continue;
          const data = line.slice(6);
          if (data === '[DONE]') {
            controller.enqueue(`event: message_delta\ndata: ${JSON.stringify({ type: 'message_delta', delta: { stop_reason: 'end_turn' }, usage: { output_tokens: 1 } })}\n\n`);
            controller.enqueue(`event: message_stop\ndata: ${JSON.stringify({ type: 'message_stop' })}\n\n`);
            return;
          }

          try {
            const parsed = JSON.parse(data);
            const choice = parsed.choices?.[0];
            if (!choice) continue;

            if (isFirstToken) {
              isFirstToken = false;
              controller.enqueue(`event: message_start\ndata: ${JSON.stringify({
                type: 'message_start',
                message: { id: parsed.id || `msg_${Date.now()}`, type: 'message', role: 'assistant', content: [], model, usage: { input_tokens: 0, output_tokens: 0 } }
              })}\n\n`);
              controller.enqueue(`event: content_block_start\ndata: ${JSON.stringify({
                type: 'content_block_start',
                index: 0,
                content_block: { type: 'text', text: '' }
              })}\n\n`);
              textBlockIndex = 0;
              contentBlockIndex = 1;
            }

            const delta = choice.delta;
            if (delta.content) {
              for (const content of delta.content) {
                if (content.type === 'text' && content.text) {
                  controller.enqueue(`event: content_block_delta\ndata: ${JSON.stringify({
                    type: 'content_block_delta',
                    index: textBlockIndex ?? 0,
                    delta: { type: 'text_delta', text: content.text }
                  })}\n\n`);
                } else if (content.type === 'tool_call') {
                  const toolCall = content;
                  controller.enqueue(`event: content_block_start\ndata: ${JSON.stringify({
                    type: 'content_block_start',
                    index: contentBlockIndex,
                    content_block: {
                      type: 'tool_use',
                      id: toolCall.id || `toolu_${Date.now()}`,
                      name: toolCall.function?.name || '',
                      input: {}
                    }
                  })}\n\n`);
                  
                  if (toolCall.function?.arguments) {
                    controller.enqueue(`event: content_block_delta\ndata: ${JSON.stringify({
                      type: 'content_block_delta',
                      index: contentBlockIndex,
                      delta: { type: 'input_json_delta', partial_json: toolCall.function.arguments }
                    })}\n\n`);
                  }
                  
                  controller.enqueue(`event: content_block_stop\ndata: ${JSON.stringify({
                    type: 'content_block_stop',
                    index: contentBlockIndex
                  })}\n\n`);
                  contentBlockIndex++;
                }
              }
            }

            if (choice.finish_reason === 'stop' || choice.finish_reason === 'length') {
              controller.enqueue(`event: message_delta\ndata: ${JSON.stringify({
                type: 'message_delta',
                delta: { stop_reason: choice.finish_reason === 'stop' ? 'end_turn' : 'max_tokens' },
                usage: parsed.usage ? {
                  input_tokens: parsed.usage.prompt_tokens || 0,
                  output_tokens: parsed.usage.completion_tokens || 0
                } : { output_tokens: 1 }
              })}\n\n`);
              controller.enqueue(`event: message_stop\ndata: ${JSON.stringify({ type: 'message_stop' })}\n\n`);
            }
          } catch (e) {
            // Skip malformed JSON
          }
        }
      };

      try {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          await processChunk(value);
        }
        if (buffer) {
          const lines = buffer.split('\n');
          for (const line of lines) {
            if (line.startsWith('data: ') && line.slice(6) !== '[DONE]') {
              try {
                const parsed = JSON.parse(line.slice(6));
                if (parsed.choices?.[0]?.finish_reason) {
                  controller.enqueue(`event: message_delta\ndata: ${JSON.stringify({
                    type: 'message_delta',
                    delta: { stop_reason: 'end_turn' },
                    usage: { output_tokens: 1 }
                  })}\n\n`);
                  controller.enqueue(`event: message_stop\ndata: ${JSON.stringify({ type: 'message_stop' })}\n\n`);
                }
              } catch {}
            }
          }
        }
      } catch (e) {
        controller.error(e);
      }
    },
  });
}

export async function sendAnthropicRequest(
  provider: ProviderConfig,
  request: AnthropicRequest
): Promise<Response> {
  const baseUrl = normalizeBaseUrl(provider.base_url);
  const endpoint = baseUrl.endsWith('/v1') || baseUrl.endsWith('/v1/messages')
    ? `${baseUrl}/messages`
    : `${baseUrl}/v1/messages`;

  const response = await fetch(endpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': provider.api_key,
      'anthropic-version': '2023-06-01',
    },
    body: JSON.stringify(request),
  });

  return response;
}

export function resolveProviderForModel(
  providers: ProviderConfig[],
  modelId: string
): ProviderConfig | null {
  let match: ProviderConfig | null = null;

  for (const p of providers) {
    if (!p.enabled) continue;
    const hasModel = p.models?.some(m => m.id === modelId && m.enabled !== false);
    if (hasModel) {
      if (!match) {
        match = p;
      } else {
        console.warn(`[API Proxy] Model "${modelId}" exists in multiple providers. Using first match: "${match.name}"`);
      }
    }
  }

  if (match) {
    console.log(`[API Proxy] Resolved "${modelId}" → "${match.name}" (${match.base_url})`);
  } else {
    console.warn(`[API Proxy] No provider found for "${modelId}"`);
  }

  return match;
}

export class APIProxy {
  private providers: ProviderConfig[] = [];
  private cache: Map<string, ProviderConfig> = new Map();

  setProviders(providers: ProviderConfig[]) {
    this.providers = providers;
    this.cache.clear();
  }

  getProviders(): ProviderConfig[] {
    return this.providers;
  }

  async chat(request: AnthropicRequest): Promise<Response> {
    const provider = resolveProviderForModel(this.providers, request.model);
    
    if (!provider) {
      throw new Error(`No provider found for model: ${request.model}`);
    }

    if (provider.api_format === 'openai') {
      return sendOpenAIRequest(provider, request);
    } else {
      return sendAnthropicRequest(provider, request);
    }
  }

  async chatStream(request: AnthropicRequest): Promise<ReadableStream> {
    const provider = resolveProviderForModel(this.providers, request.model);
    
    if (!provider) {
      throw new Error(`No provider found for model: ${request.model}`);
    }

    if (provider.api_format === 'openai') {
      const response = await sendOpenAIRequest(provider, { ...request, stream: true });
      if (!response.ok) {
        const error = await response.text();
        throw new Error(`API error ${response.status}: ${error}`);
      }
      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error('No response body');
      }
      return parseOpenAIStreamResponse(reader, request.model);
    } else {
      const response = await sendAnthropicRequest(provider, { ...request, stream: true });
      if (!response.ok) {
        const error = await response.text();
        throw new Error(`API error ${response.status}: ${error}`);
      }
      return response.body!;
    }
  }
}

export const apiProxy = new APIProxy();
