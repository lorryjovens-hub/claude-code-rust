import { API_BASE, detectBridgePort } from './client';

export async function multiagentResearch(
  query: string,
  model: string,
  onEvent: (event: any) => void
): Promise<void> {
  await detectBridgePort();
  const res = await fetch(`${API_BASE}/multiagent/research`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, model_id: model, user_id: 'default' }),
  });

  if (!res.ok || !res.body) {
    throw new Error('Multiagent research request failed');
  }

  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    for (const line of lines) {
      if (line.startsWith('data:')) {
        const data = line.slice(5).trim();
        if (data === '[DONE]') return;
        try {
          const event = JSON.parse(data);
          onEvent(event);
        } catch {}
      } else if (line.startsWith('event:')) {
        // SSE event type line, consumed with data line
      }
    }
  }
}

export async function startResearch(conversationId: string, query: string) {
  const { request } = await import('./client');
  const res = await request('/research/start', {
    method: 'POST',
    body: JSON.stringify({ conversation_id: conversationId, query }),
  });
  return res.json();
}

export async function stopResearch(conversationId: string) {
  const { request } = await import('./client');
  const res = await request(`/research/${conversationId}/stop`, { method: 'POST' });
  return res.json();
}

export async function getResearchStatus(conversationId: string) {
  const { request } = await import('./client');
  const res = await request(`/research/${conversationId}/status`);
  return res.json();
}

export async function streamResearch(conversationId: string) {
  const { request } = await import('./client');
  const res = await request(`/research/${conversationId}/stream`);
  return res.json();
}
