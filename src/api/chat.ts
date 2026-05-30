import { getErrorMessage } from '../utils/errorHelpers';
import { API_BASE, getToken, detectBridgePort, isTauriApp, isBridgeReachable, resetBridgeReachableCache, getUserModeForConversation, resolveEnvCreds } from './client';

// 检查模型是否需要使用前端代理（当 Provider 使用 OpenAI 格式时）
async function checkUseProxyForModel(model: string): Promise<boolean> {
  try {
    const { getProviders } = await import('./providers');
    const providers = await getProviders();
    for (const p of providers) {
      if (!p.enabled) continue;
      const hasModel = p.models?.some((m: any) => m.id === model && m.enabled !== false);
      if (hasModel && p.format === 'openai') {
        console.log(`[API] Using frontend proxy for model "${model}" (provider: ${p.name}, format: openai)`);
        return true;
      }
    }
  } catch (e) {
    console.warn('[API] Failed to check providers for proxy:', e);
  }
  return false;
}

// 通过前端代理发送消息（支持 OpenAI 格式的 Provider）
async function sendMessageViaProxy(
  conversationId: string,
  messages: any[],
  model: string,
  onDelta: (delta: string, full: string) => void,
  onDone: (full: string) => void,
  onError: (err: string) => void,
  onThinking?: (thinking: string, full: string) => void,
  onSystem?: (event: string, message: string, data: any) => void,
  onToolUse?: (event: { type: 'start' | 'input' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: any; content?: string; is_error?: boolean; textBefore?: string }) => void,
): Promise<() => void> {
  const t0 = performance.now();
  console.log('[Proxy] sendMessageViaProxy 开始', { conversationId, model, messageCount: messages.length });

  const { apiProxy } = await import('../utils/apiProxy');
  const { getProviders } = await import('./providers');
  const t1 = performance.now();

  const providers = await getProviders();
  const t2 = performance.now();

  const proxyProviders = providers.map((p: any) => ({
    id: p.id,
    name: p.name,
    base_url: p.baseUrl,
    api_key: p.apiKey || '',
    api_format: p.format === 'openai' ? 'openai' as const : 'anthropic' as const,
    enabled: p.enabled,
    models: (p.models || []).map((m: any) => ({
      id: m.id,
      name: m.name,
      enabled: m.enabled,
    })),
  }));

  const activeProvider = proxyProviders.find(p => p.enabled && (p.models || []).some((m: any) => m.id === model && m.enabled !== false));
  console.log('[Proxy] Provider 准备完成', {
    providerLoadMs: (t2 - t1).toFixed(1),
    importMs: (t1 - t0).toFixed(1),
    totalProviders: proxyProviders.length,
    activeProvider: activeProvider ? `${activeProvider.name} (${activeProvider.api_format})` : 'NONE',
    baseUrl: activeProvider?.base_url || 'N/A',
  });

  apiProxy.setProviders(proxyProviders);

  const anthropicMessages = messages.map((msg: any) => {
    if (msg.role === 'user') {
      return { role: 'user', content: msg.content };
    } else if (msg.role === 'assistant') {
      return { role: 'assistant', content: msg.content };
    }
    return msg;
  });

  // 计算 token 估算和消息大小
  const totalInputChars = JSON.stringify(anthropicMessages).length;
  console.log('[Proxy] 请求构建完成', {
    messagesCount: anthropicMessages.length,
    totalInputChars,
    estimatedTokens: Math.round(totalInputChars / 4),
    lastMessage: anthropicMessages[anthropicMessages.length - 1]?.role,
  });

  const request = {
    model,
    messages: anthropicMessages,
    max_tokens: 1000000,
    stream: true,
  };

  let fullText = '';
  let thinkingText = '';
  let currentToolUseId: string | null = null;
  let currentToolName: string | undefined = undefined;

  let firstByteTime = 0;
  let lastChunkTime = 0;
  let totalChunks = 0;
  let totalOutputChars = 0;

  const streamHandlers = {
    onDelta: (delta: string, full: string) => {
      if (firstByteTime === 0) {
        firstByteTime = performance.now();
        console.log(`[Proxy] TTFB (首字节): ${(firstByteTime - t2).toFixed(0)}ms (从provider加载算起)`);
      }
      totalChunks++;
      totalOutputChars += delta.length;
      lastChunkTime = performance.now();
      fullText = full;
      onDelta(delta, fullText);
    },
    onThinking: (delta: string, full: string) => {
      if (firstByteTime === 0) {
        firstByteTime = performance.now();
        console.log(`[Proxy] TTFB (thinking首字节): ${(firstByteTime - t2).toFixed(0)}ms`);
      }
      thinkingText = full;
      onThinking?.(delta, thinkingText);
    },
    onToolUse: (event: { type: 'start' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: any; output?: string; is_error?: boolean }) => {
      if (event.type === 'start') {
        currentToolUseId = event.tool_use_id;
        currentToolName = event.tool_name;
        onToolUse?.({ type: 'start', tool_use_id: event.tool_use_id, tool_name: event.tool_name, tool_input: event.tool_input, content: '', is_error: false });
      } else if (event.type === 'done') {
        onToolUse?.({ type: 'done', tool_use_id: event.tool_use_id, tool_name: currentToolName, tool_input: {}, content: event.output, is_error: event.is_error || false });
        currentToolUseId = null;
        currentToolName = undefined;
      }
    },
    onSystem: (event: string, data: any) => {
      onSystem?.(event, '', data);
    },
    onDone: (full: string) => {
      fullText = full;
      onDone(fullText);
    },
    onError: (err: string) => {
      onError(err);
    },
  };

  const t3 = performance.now();
  console.log(`[Proxy] 准备阶段耗时: ${(t3 - t0).toFixed(1)}ms, 开始请求...`);

  try {
    const streamFetchStart = performance.now();
    const stream = await apiProxy.chatStream(request);
    const ttfStart = performance.now();
    console.log(`[Proxy] chatStream 建立耗时: ${(ttfStart - streamFetchStart).toFixed(0)}ms`);

    const parseStart = performance.now();
    await parseProxyStream(stream, streamHandlers);
    const parseEnd = performance.now();

    const totalElapsed = performance.now() - t0;
    const streamDuration = parseEnd - parseStart;
    const avgChunkInterval = totalChunks > 1 ? (streamDuration / totalChunks).toFixed(1) : '0';
    const tokensPerSec = streamDuration > 0 ? Math.round((totalOutputChars / 4) / (streamDuration / 1000)) : 0;

    console.log(`[Proxy] ✅ 流式响应完成`, {
      总耗时_ms: Math.round(totalElapsed),
      首字节_TTFB_ms: firstByteTime > 0 ? `${Math.round(firstByteTime - t0)}ms` : 'N/A',
      stream处理耗时_ms: Math.round(streamDuration),
      总chunk数: totalChunks,
      输出字符数: totalOutputChars,
      估算输出token: Math.round(totalOutputChars / 4),
      平均chunk间隔_ms: avgChunkInterval,
      估算tokens每秒: tokensPerSec,
      平均速度_kbps: streamDuration > 0 ? (totalOutputChars / streamDuration * 1000 / 1024).toFixed(1) : '0',
    });
  } catch (e: unknown) {
    const errMsg = getErrorMessage(e) || 'Failed to send message via proxy';
    console.log(`[Proxy] ❌ 请求失败 (耗时 ${Math.round(performance.now() - t0)}ms): ${errMsg}`);
    onError(errMsg);
  }

  return () => {};
}

// 解析代理返回的 SSE 流
async function parseProxyStream(stream: ReadableStream, handlers: {
  onDelta: (delta: string, full: string) => void;
  onThinking?: (delta: string, full: string) => void;
  onToolUse?: (event: { type: 'start' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: any; output?: string; is_error?: boolean }) => void;
  onSystem?: (event: string, data: any) => void;
  onDone: (full: string) => void;
  onError: (err: string) => void;
}): Promise<void> {
  const reader = stream.getReader();
  const decoder = new TextDecoder();
  let buffer = '';
  let fullText = '';
  let thinkingText = '';
  let currentToolUseId: string | null = null;
  let currentToolName: string | undefined = undefined;

  const processLine = (line: string) => {
    if (!line.startsWith('data:')) return;
    const data = line.startsWith('data: ') ? line.slice(6) : line.slice(5);
    if (data === '[DONE]') {
      handlers.onDone(fullText);
      return;
    }

    try {
      const event = JSON.parse(data);
      const eventType = event.type;

      switch (eventType) {
        case 'message_start':
          handlers.onSystem?.('message_start', { model: event.message?.model });
          break;
        case 'content_block_start':
          if (event.content_block?.type === 'tool_use') {
            currentToolUseId = event.content_block.id;
            currentToolName = event.content_block.name || undefined;
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
  } catch (e: unknown) {
    handlers.onError(`Stream error: ${getErrorMessage(e)}`);
  }
}

// 流式对话（核心 - Tauri 版本，直接使用 bridge-server HTTP API）
export async function sendMessageNative(
  conversationId: string,
  messages: any[],
  model: string,
  onDelta: (delta: string, full: string) => void,
  onDone: (full: string) => void,
  onError: (err: string) => void,
  onThinking?: (thinking: string, full: string) => void,
  onSystem?: (event: string, message: string, data: any) => void,
  onToolUse?: (event: { type: 'start' | 'input' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: any; content?: string; is_error?: boolean; textBefore?: string }) => void,
  reasoningMode?: string | null,
): Promise<() => void> {
  if (!isTauriApp) {
    console.log('[API] Web mode (native): using direct provider API call via proxy');
    try {
      return await sendMessageViaProxy(
        conversationId, messages, model,
        onDelta, onDone, onError, onThinking, onSystem, onToolUse,
      );
    } catch (e) {
      onError(getErrorMessage(e) || 'Failed to send message');
      return () => {};
    }
  }

  const token = getToken();
  let fullText = '';
  let thinkingText = '';
  let deltaCount = 0;

  console.log(`[API] Sending message (native): model=${model}, messages=${messages.length}, stream=true`);
  console.log(`[API] Request URL: ${API_BASE}/chat`);
  console.log(`[API] Establishing SSE connection to ${API_BASE}/chat`);

  let permissionMode: string | undefined;
  try {
    permissionMode = localStorage.getItem('permission_mode') || undefined;
  } catch {}

  try {
    await detectBridgePort();
    const res = await fetch(`${API_BASE}/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        conversation_id: conversationId,
        messages,
        model,
        ...resolveEnvCreds(getUserModeForConversation(conversationId)),
        user_mode: getUserModeForConversation(conversationId),
        permission_mode: permissionMode,
        reasoning_mode: reasoningMode || undefined,
      }),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: '请求失败' }));
      onError(err.error || '请求失败');
      return () => {};
    }

    if (!res.body) return () => {};

    const reader = res.body.getReader();
    console.log(`[API] SSE connection established, reading stream...`);
    const decoder = new TextDecoder();
    let buffer = '';
    let pendingTextDelta = '';
    let pendingThinkingDelta = '';
    let flushScheduled = false;

    const flushPending = () => {
      flushScheduled = false;
      if (pendingThinkingDelta && onThinking) {
        const delta = pendingThinkingDelta;
        pendingThinkingDelta = '';
        onThinking(delta, thinkingText);
      }
      if (pendingTextDelta) {
        const delta = pendingTextDelta;
        pendingTextDelta = '';
        fullText += delta;
        onDelta(delta, fullText);
      }
    };

    const scheduleFlush = () => {
      if (!flushScheduled) {
        flushScheduled = true;
        setTimeout(flushPending, 0);
      }
    };

    const processLine = (line: string) => {
      if (!line.startsWith('event:')) {
        if (line.startsWith('data:')) {
          const data = line.startsWith('data: ') ? line.slice(6) : line.slice(5);
          if (data === '[DONE]') return;
          try {
            const event = JSON.parse(data);
            if (event.type === 'text' && event.text) {
              deltaCount++;
              if (deltaCount % 50 === 0) {
                console.log(`[API] Stream progress: ${deltaCount} deltas received, ${fullText.length} chars`);
              }
              pendingTextDelta += event.text;
              scheduleFlush();
            } else if (event.type === 'thinking' && event.thinking) {
              thinkingText += event.thinking;
              pendingThinkingDelta += event.thinking;
              scheduleFlush();
            } else if (event.type === 'content_block_start') {
              if (event.content_block?.type === 'tool_use') {
                onToolUse?.({ type: 'start', tool_use_id: event.content_block.id || '', tool_name: event.content_block.name || '', tool_input: event.content_block.input || {}, content: '', is_error: false });
              }
            } else if (event.type === 'content_block_delta') {
              if (event.delta?.type === 'tool_use_delta') {
              } else if (event.delta?.type === 'text_delta' && event.delta.text) {
                pendingTextDelta += event.delta.text;
                scheduleFlush();
              }
            } else if (event.type === 'content_block_stop') {
            } else if (event.type === 'tool_use_start') {
              onToolUse?.({ type: 'start', tool_use_id: event.tool_use_id || '', tool_name: event.tool_name || '', tool_input: event.tool_input || {}, content: '', is_error: false });
            } else if (event.type === 'tool_use_done') {
              onToolUse?.({ type: 'done', tool_use_id: event.tool_use_id || '', tool_name: event.tool_name || 'unknown', tool_input: event.tool_input || {}, content: event.output || event.content || '', is_error: event.is_error === true });
            } else if (event.type === 'tool_arg_delta') {
            } else if (event.type === 'message_start') {
              onSystem?.('message_start', '', { model: event.message?.model });
            } else if (event.type === 'message_delta') {
              if (event.delta?.stop_reason) {
                onSystem?.('message_delta', '', { stop_reason: event.delta.stop_reason });
              }
            } else if (event.type === 'message_stop') {
              flushPending();
              console.log(`[API] Stream complete: ${deltaCount} deltas, ${fullText.length} chars total`);
              console.log(`[API] SSE connection closed`);
              onDone(fullText);
            } else if (event.type === 'error') {
              console.log(`[API] SSE connection closed`);
              onError(event.error || 'Unknown error');
            }
          } catch (e) {
            // Skip malformed JSON
          }
        }
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
            flushPending();
            if (!fullText && !thinkingText) {
              onDone('');
            }
            console.log(`[API] SSE connection closed`);
            break;
          }
          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';
          for (const line of lines) {
            processLine(line);
          }
        }
      } catch (e: unknown) {
        console.log(`[API] SSE connection closed`);
        onError(`Stream error: ${getErrorMessage(e)}`);
      }
    };

    readChunk();
  } catch (e: unknown) {
    if (e instanceof DOMException && e.name === 'AbortError') {
      onDone(fullText);
      return () => {};
    }
    const isNetworkError = e instanceof TypeError && (e.message === 'Failed to fetch' || e.message.includes('NetworkError') || e.message.includes('fetch'));
    if (isNetworkError && model && messages && messages.length > 0) {
      console.log('[API] Bridge unreachable, falling back to frontend proxy (native)...');
      resetBridgeReachableCache();
      try {
        return await sendMessageViaProxy(
          conversationId, messages, model,
          onDelta, onDone, onError,
          onThinking, onSystem, onToolUse
        );
      } catch (proxyErr: any) {
        console.error('[API] Proxy fallback also failed:', proxyErr);
      }
    }
    console.log(`[API] SSE connection closed`);
    onError(getErrorMessage(e) || 'Failed to send message');
  }

  return () => {};
}

// 流式对话（核心 - HTTP 版本）
export async function sendMessage(
  conversationId: string,
  message: string,
  attachments: any[] | null,
  onDelta: (delta: string, full: string) => void,
  onDone: (full: string) => void,
  onError: (err: string) => void,
  onThinking?: (thinking: string, full: string) => void,
  onSystem?: (event: string, message: string, data: any) => void,
  onCitations?: (citations: Array<{ url: string; title: string; cited_text?: string }>, query?: string, tokens?: number) => void,
  onDocument?: (document: { id: string; title: string; filename: string; url: string; content?: string; format?: 'markdown' | 'docx' | 'pptx'; slides?: Array<{ title: string; content: string; notes?: string }> }) => void,
  onDocumentDraft?: (draft: { draft_id: string; title?: string; format?: string; preview?: string; preview_available?: boolean; done?: boolean; document?: any }) => void,
  onCodeExecution?: (data: { type: string; executionId: string; code?: string; language?: string; files?: Array<{ id: string; name: string }>; stdout?: string; stderr?: string; images?: string[]; error?: string | null }) => void,
  onToolUse?: (event: { type: 'start' | 'input' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: any; content?: string; is_error?: boolean; textBefore?: string }) => void,
  signal?: AbortSignal,
  model?: string,
  messages?: any[],
  reasoningMode?: string | null,
) {
  if (!isTauriApp) {
    let effectiveModel = model || '';
    if (!effectiveModel && messages && messages.length > 0) {
      try {
        const { getProviders } = await import('./providers');
        const providers = await getProviders();
        for (const p of providers) {
          if (!p.enabled) continue;
          const firstEnabled = (p.models || []).find((m: any) => m.enabled !== false);
          if (firstEnabled) {
            effectiveModel = firstEnabled.id;
            console.log(`[API] Web mode: auto-resolved model "${effectiveModel}" from provider "${p.name}"`);
            break;
          }
        }
      } catch {}
    }
    if (messages && effectiveModel) {
      console.log('[API] Web mode: using direct provider API call via proxy');
      try {
        return await sendMessageViaProxy(
          conversationId,
          messages,
          effectiveModel,
          onDelta,
          onDone,
          onError,
          onThinking,
          onSystem,
          onToolUse,
        );
      } catch (e) {
        onError(getErrorMessage(e) || 'Failed to send message');
        return;
      }
    }
    onError('No model configured. Please add a model in Settings > Models.');
    return;
  }

  const token = getToken();
  let fullText = '';
  let deltaCount = 0;
  console.log(`[API] Sending message: model=${model}, messages=${messages?.length || 0}, stream=true`);
  console.log(`[API] Request URL: ${API_BASE}/chat`);
  console.log(`[API] Establishing SSE connection to ${API_BASE}/chat`);

  const MAX_RETRIES = 3;
  const RETRY_DELAYS = [500, 1500, 3000];

  const doFetch = async (attempt: number): Promise<boolean> => {
    try {
      if (attempt > 0) {
        console.log(`[API] Retry attempt ${attempt + 1}/${MAX_RETRIES}...`);
        await new Promise(resolve => setTimeout(resolve, RETRY_DELAYS[attempt - 1] || RETRY_DELAYS[RETRY_DELAYS.length - 1]));
      }
      await detectBridgePort();
    } catch (e) {
      console.warn(`[API] Bridge port detection failed (attempt ${attempt + 1}):`, e);
    }

    let permissionMode: string | undefined;
    try {
      if (typeof window !== 'undefined' && (window as any).__chatStore) {
        permissionMode = (window as any).__chatStore.getState().permissionMode;
      }
    } catch {}

    let res: Response;
    try {
      res = await fetch(`${API_BASE}/chat`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify({
          conversation_id: conversationId,
          message,
          model: model || undefined,
          messages: messages && messages.length > 0 ? messages : undefined,
          attachments: attachments || undefined,
          ...resolveEnvCreds(getUserModeForConversation(conversationId)),
          user_mode: getUserModeForConversation(conversationId),
          permission_mode: permissionMode,
          reasoning_mode: reasoningMode || undefined,
          user_profile: (() => {
            try {
              const p = JSON.parse(localStorage.getItem('user_profile') || localStorage.getItem('user') || '{}');
              const wf = p.work_function;
              const pp = p.personal_preferences;
              return (wf || pp) ? { work_function: wf, personal_preferences: pp } : undefined;
            } catch { return undefined; }
          })(),
        }),
        signal,
      });
    } catch (fetchErr: any) {
      const isNetworkError = fetchErr instanceof TypeError && (fetchErr.message === 'Failed to fetch' || fetchErr.message.includes('NetworkError') || fetchErr.message.includes('fetch'));
      if (isNetworkError && attempt < MAX_RETRIES - 1) {
        console.warn(`[API] Network error, retrying... (${attempt + 1}/${MAX_RETRIES})`);
        return false;
      }
      throw fetchErr;
    }

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: '请求失败' }));
      onError(err.error || '请求失败');
      return true;
    }

    if (!res.body) return true;

    const reader = res.body.getReader();
    console.log(`[API] SSE connection established, reading stream...`);
    const decoder = new TextDecoder();
    let buffer = '';
    let thinkingText = '';
    let pendingTextDelta = '';
    let pendingThinkingDelta = '';
    let flushScheduled = false;
    const INLINE_ARTIFACT_OPEN = '<cp_artifact';
    const INLINE_ARTIFACT_CLOSE = '</cp_artifact>';
    let inlineArtifactBuffer = '';
    let inlineArtifactSeq = 0;
    let activeInlineArtifact: null | {
      draft_id: string;
      title: string;
      format: string;
      preview: string;
    } = null;

    const flushPending = () => {
      flushScheduled = false;
      try {
        if (pendingThinkingDelta && onThinking) {
          const delta = pendingThinkingDelta;
          pendingThinkingDelta = '';
          onThinking(delta, thinkingText);
        }
      } catch (err) {
        console.error('[API] onThinking callback error:', err);
      }
      try {
        if (pendingTextDelta) {
          const delta = pendingTextDelta;
          pendingTextDelta = '';
          onDelta(delta, fullText);
        }
      } catch (err) {
        console.error('[API] onDelta callback error:', err);
      }
    };

    const scheduleFlush = () => {
      if (flushScheduled) return;
      flushScheduled = true;
      setTimeout(flushPending, 0);
    };

    const appendVisibleText = (text: string) => {
      if (!text) return;
      fullText += text;
      pendingTextDelta += text;
      try {
        scheduleFlush();
      } catch (err) {
        console.error('[API] scheduleFlush error:', err);
      }
    };

    const emitInlineArtifactDraft = (done = false) => {
      if (!activeInlineArtifact || !onDocumentDraft) return;
      try {
        onDocumentDraft({
          draft_id: activeInlineArtifact.draft_id,
          title: activeInlineArtifact.title,
          format: activeInlineArtifact.format,
          preview: activeInlineArtifact.preview,
          preview_available: activeInlineArtifact.preview.length > 0,
          done,
        });
      } catch (err) {
        console.error('[API] onDocumentDraft callback error:', err);
      }
    };

    const appendInlineArtifactPreview = (text: string) => {
      if (!text || !activeInlineArtifact) return;
      activeInlineArtifact.preview += text;
      emitInlineArtifactDraft(false);
    };

    const parseInlineArtifactAttrs = (tagText: string) => {
      const titleMatch = tagText.match(/title="([^"]*)"/i);
      const formatMatch = tagText.match(/format="([^"]*)"/i);
      return {
        title: (titleMatch?.[1] || '').trim() || 'Untitled document',
        format: (formatMatch?.[1] || 'markdown').trim() || 'markdown',
      };
    };

    const processInlineArtifactText = (chunk: string, flushAll = false) => {
      if (!chunk && !flushAll) return;
      inlineArtifactBuffer += chunk;

      while (inlineArtifactBuffer) {
        if (!activeInlineArtifact) {
          const startIdx = inlineArtifactBuffer.indexOf(INLINE_ARTIFACT_OPEN);
          if (startIdx === -1) {
            if (flushAll) {
              appendVisibleText(inlineArtifactBuffer);
              inlineArtifactBuffer = '';
            } else {
              const keep = Math.min(inlineArtifactBuffer.length, INLINE_ARTIFACT_OPEN.length - 1);
              const emit = inlineArtifactBuffer.slice(0, inlineArtifactBuffer.length - keep);
              if (emit) appendVisibleText(emit);
              inlineArtifactBuffer = inlineArtifactBuffer.slice(inlineArtifactBuffer.length - keep);
            }
            break;
          }

          if (startIdx > 0) {
            appendVisibleText(inlineArtifactBuffer.slice(0, startIdx));
            inlineArtifactBuffer = inlineArtifactBuffer.slice(startIdx);
          }

          const tagEndIdx = inlineArtifactBuffer.indexOf('>');
          if (tagEndIdx === -1) {
            if (flushAll) {
              appendVisibleText(inlineArtifactBuffer);
              inlineArtifactBuffer = '';
            }
            break;
          }

          const tagText = inlineArtifactBuffer.slice(0, tagEndIdx + 1);
          const attrs = parseInlineArtifactAttrs(tagText);
          inlineArtifactSeq += 1;
          activeInlineArtifact = {
            draft_id: `inline-artifact-${inlineArtifactSeq}`,
            title: attrs.title,
            format: attrs.format,
            preview: '',
          };
          emitInlineArtifactDraft(false);
          inlineArtifactBuffer = inlineArtifactBuffer.slice(tagEndIdx + 1);
          continue;
        }

        const closeIdx = inlineArtifactBuffer.indexOf(INLINE_ARTIFACT_CLOSE);
        if (closeIdx === -1) {
          if (flushAll) {
            appendInlineArtifactPreview(inlineArtifactBuffer);
            inlineArtifactBuffer = '';
            emitInlineArtifactDraft(true);
            activeInlineArtifact = null;
          } else {
            const keep = Math.min(inlineArtifactBuffer.length, INLINE_ARTIFACT_CLOSE.length - 1);
            const emit = inlineArtifactBuffer.slice(0, inlineArtifactBuffer.length - keep);
            if (emit) appendInlineArtifactPreview(emit);
            inlineArtifactBuffer = inlineArtifactBuffer.slice(inlineArtifactBuffer.length - keep);
          }
          break;
        }

        if (closeIdx > 0) {
          appendInlineArtifactPreview(inlineArtifactBuffer.slice(0, closeIdx));
        }
        inlineArtifactBuffer = inlineArtifactBuffer.slice(closeIdx + INLINE_ARTIFACT_CLOSE.length);
        emitInlineArtifactDraft(true);
        activeInlineArtifact = null;
      }
    };

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (!line.startsWith('data:')) continue;
          const data = line.startsWith('data: ') ? line.slice(6) : line.slice(5);
          if (data.trim() === '[DONE]') {
            processInlineArtifactText('', true);
            flushPending();
            console.log(`[API] Stream complete: ${deltaCount} deltas, ${fullText.length} chars total`);
            console.log(`[API] SSE connection closed`);
            try {
              onDone(fullText);
            } catch (err) {
              console.error('[API] onDone callback error:', err);
            }
            return true;
          }

          try {
            const parsed = JSON.parse(data);

            if (parsed.type === 'system') {
              if (onSystem) {
                try { onSystem(parsed.event, parsed.message, parsed); } catch (err) { console.error('[API] onSystem callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'status') {
              if (onSystem) {
                try { onSystem('status', parsed.message, parsed); } catch (err) { console.error('[API] onSystem status callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'thinking_summary' && parsed.summary) {
              if (onSystem) {
                try { onSystem('thinking_summary', parsed.summary, parsed); } catch (err) { console.error('[API] onSystem thinking_summary callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'search_sources') {
              if (onCitations && Array.isArray(parsed.sources)) {
                try { onCitations(parsed.sources, parsed.query, parsed.tokens); } catch (err) { console.error('[API] onCitations callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'document_created') {
              if (onDocument && parsed.document) {
                try { onDocument(parsed.document); } catch (err) { console.error('[API] onDocument callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'document_updated') {
              if (onDocument && parsed.document) {
                try { onDocument(parsed.document); } catch (err) { console.error('[API] onDocument update callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'document_draft') {
              if (onDocumentDraft) {
                try { onDocumentDraft(parsed); } catch (err) { console.error('[API] onDocumentDraft event callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'code_execution') {
              if (onCodeExecution) {
                try { onCodeExecution(parsed); } catch (err) { console.error('[API] onCodeExecution callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'code_result') {
              if (onCodeExecution) {
                try { onCodeExecution(parsed); } catch (err) { console.error('[API] onCodeExecution result callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'content_block_delta' && parsed.delta) {
              if (parsed.delta.type === 'text_delta' && parsed.delta.text) {
                deltaCount++;
                if (deltaCount % 50 === 0) {
                  console.log(`[API] Stream progress: ${deltaCount} deltas received, ${fullText.length} chars`);
                }
                const textChunk = parsed.delta.text;
                if (textChunk.includes('<thinking>') || textChunk.includes('</thinking>')) {
                  const thinkRegex = /<thinking>([\s\S]*?)<\/thinking>/g;
                  let match;
                  let cleaned = textChunk;
                  while ((match = thinkRegex.exec(textChunk)) !== null) {
                    if (onThinking) {
                      try {
                        thinkingText += match[1];
                        pendingThinkingDelta += match[1];
                        scheduleFlush();
                      } catch (err) {
                        console.error('[API] onThinking thinking extraction error:', err);
                      }
                    }
                  }
                  cleaned = textChunk.replace(/<thinking>[\s\S]*?<\/thinking>\s*/g, '');
                  if (cleaned) {
                    processInlineArtifactText(cleaned);
                  }
                } else {
                  processInlineArtifactText(textChunk);
                }
              }
              if (parsed.delta.type === 'thinking_delta' && parsed.delta.thinking) {
                thinkingText += parsed.delta.thinking;
                if (onThinking) {
                  try {
                    pendingThinkingDelta += parsed.delta.thinking;
                    scheduleFlush();
                  } catch (err) {
                    console.error('[API] onThinking thinking_delta error:', err);
                  }
                }
              }
            }

            if (parsed.type === 'content_block_start' && parsed.content_block) {
              if (parsed.content_block.type === 'thinking' && onThinking) {
                thinkingText = '';
              }
            }

            if (parsed.type === 'compact_boundary') {
              if (onSystem) {
                try { onSystem('compact_boundary', '', parsed); } catch (err) { console.error('[API] onSystem compact_boundary callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'ask_user') {
              if (onSystem) {
                try { onSystem('ask_user', '', parsed); } catch (err) { console.error('[API] onSystem ask_user callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'tool_permission') {
              if (onSystem) {
                try { onSystem('tool_permission', '', parsed); } catch (err) { console.error('[API] onSystem tool_permission callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'message_start') {
              if (onSystem) {
                try { onSystem('message_start', '', parsed); } catch (err) { console.error('[API] onSystem message_start callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'message_delta') {
              if (onSystem) {
                try { onSystem('message_delta', '', parsed); } catch (err) { console.error('[API] onSystem message_delta callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'task_event') {
              if (onSystem) {
                try { onSystem('task_event', '', parsed); } catch (err) { console.error('[API] onSystem task_event callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'tool_use_start' && onToolUse) {
              try { onToolUse({ type: 'start', tool_use_id: parsed.tool_use_id, tool_name: parsed.tool_name, tool_input: parsed.tool_input, textBefore: parsed.textBefore || '' }); } catch (err) { console.error('[API] onToolUse start callback error:', err); }
            }
            if (parsed.type === 'tool_use_input' && onToolUse) {
              try { onToolUse({ type: 'input', tool_use_id: parsed.tool_use_id, tool_input: parsed.tool_input }); } catch (err) { console.error('[API] onToolUse input callback error:', err); }
            }
            if (parsed.type === 'tool_use_done' && onToolUse) {
              try { onToolUse({ type: 'done', tool_use_id: parsed.tool_use_id, content: parsed.content || parsed.output, is_error: parsed.is_error }); } catch (err) { console.error('[API] onToolUse done callback error:', err); }
            }

            if (parsed.type && parsed.type.startsWith('research_') && onSystem) {
              try { onSystem(parsed.type, '', parsed); } catch (err) { console.error('[API] onSystem research callback error:', err); }
              if (parsed.type === 'research_report_delta' && parsed.text) {
                fullText += parsed.text;
                try { onDelta(parsed.text, fullText); } catch (err) { console.error('[API] onDelta research callback error:', err); }
              }
              continue;
            }

            if (parsed.type === 'tool_text_offset' && onSystem) {
              try { onSystem('tool_text_offset', '', parsed); } catch (err) { console.error('[API] onSystem tool_text_offset callback error:', err); }
            }

            if (parsed.type === 'message_stop') {
              processInlineArtifactText('', true);
              if (fullText) {
                flushPending();
                console.log(`[API] Stream complete: ${deltaCount} deltas, ${fullText.length} chars total`);
                console.log(`[API] SSE connection closed`);
                try { onDone(fullText); } catch (err) { console.error('[API] onDone message_stop callback error:', err); }
                return true;
              }
              continue;
            }

            if (parsed.type === 'error') {
              const detail = parsed.detail ? `\n${parsed.detail}` : '';
              processInlineArtifactText('', true);
              flushPending();
              console.log(`[API] SSE connection closed`);
              try { onError((parsed.error || '未知错误') + detail); } catch (err) { console.error('[API] onError callback error:', err); }
              return true;
            }
          } catch (e) {
            // 忽略非JSON行
          }
        }
      }
    } catch (readErr: any) {
      if (readErr.name === 'AbortError') {
        try { onDone(fullText); } catch (cbErr) { console.error('[API] onDone abort callback error:', cbErr); }
        return true;
      }
      throw readErr;
    }

    processInlineArtifactText('', true);
    if (fullText) {
      flushPending();
      console.log(`[API] SSE connection closed`);
      try { onDone(fullText); } catch (err) { console.error('[API] onDone stream end callback error:', err); }
    } else {
      flushPending();
      console.log(`[API] SSE connection closed`);
      try { onDone(''); } catch (err) { console.error('[API] onDone empty stream end callback error:', err); }
    }
    return true;
  };

  try {
    for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
      const success = await doFetch(attempt);
      if (success) return;
    }
    onError('网络连接失败，请检查网络后重试');
  } catch (err: any) {
    if (err.name === 'AbortError') {
      try { onDone(fullText); } catch (cbErr) { console.error('[API] onDone abort callback error:', cbErr); }
      return;
    }
    console.error('[API] Stream processing error:', err);
    const isNetworkError = err instanceof TypeError && (err.message === 'Failed to fetch' || err.message.includes('NetworkError') || err.message.includes('fetch'));
    if (isNetworkError && messages && messages.length > 0) {
      let effectiveModel = model || '';
      if (!effectiveModel) {
        try {
          const { getProviders } = await import('./providers');
          const providers = await getProviders();
          for (const p of providers) {
            if (!p.enabled) continue;
            const firstEnabled = (p.models || []).find((m: any) => m.enabled !== false);
            if (firstEnabled) { effectiveModel = firstEnabled.id; break; }
          }
        } catch {}
      }
      if (effectiveModel) {
        console.log('[API] Bridge unreachable after all retries, falling back to frontend proxy...');
        resetBridgeReachableCache();
        try {
          sendMessageViaProxy(
            conversationId, messages, effectiveModel,
            onDelta, onDone, onError,
            onThinking, onSystem, onToolUse
          );
          return;
        } catch (proxyErr: any) {
          console.error('[API] Proxy fallback also failed:', proxyErr);
        }
      }
    }
    console.log(`[API] SSE connection closed`);
    try { onError(err.message || 'Network error'); } catch (cbErr) { console.error('[API] onError stream error callback error:', cbErr); }
  }
}

export async function chatStream(conversationId: string, message: string, attachments: any[] | null, onDelta: (delta: string, full: string) => void, onDone: (full: string) => void, onError: (err: string) => void, onThinking?: (thinking: string, full: string) => void, signal?: AbortSignal) {
  return sendMessage(conversationId, message, attachments, onDelta, onDone, onError, onThinking, undefined, undefined, undefined, undefined, undefined, undefined, signal);
}

export async function chatAsk(conversationId: string, messages: any[], model: string, onDelta: (delta: string, full: string) => void, onDone: (full: string) => void, onError: (err: string) => void) {
  return sendMessageNative(conversationId, messages, model, onDelta, onDone, onError);
}
