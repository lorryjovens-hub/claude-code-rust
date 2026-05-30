import { waitForApiReady, API_BASE } from './client';

export interface TerminalSession {
  id: string;
  shell: string;
  cwd: string;
  pid: number;
}

export async function createTerminal(cwd?: string, shell?: string): Promise<TerminalSession> {
  await waitForApiReady();
  const body: Record<string, string> = {};
  if (cwd) body.cwd = cwd;
  if (shell) body.shell = shell;
  const res = await fetch(`${API_BASE}/terminal/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Failed to create terminal' }));
    throw new Error(err.error || 'Failed to create terminal');
  }
  return res.json();
}

export async function writeTerminal(terminalId: string, data: string): Promise<void> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/terminal/write`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: terminalId, data }),
  });
  if (!res.ok) throw new Error('Failed to write to terminal');
}

export async function resizeTerminal(terminalId: string, cols: number, rows: number): Promise<void> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/terminal/resize`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: terminalId, cols, rows }),
  });
  if (!res.ok) throw new Error('Failed to resize terminal');
}

export async function closeTerminal(terminalId: string): Promise<void> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/terminal/close`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(terminalId),
  });
  if (!res.ok) throw new Error('Failed to close terminal');
}

export async function listTerminals(): Promise<TerminalSession[]> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/terminal/list`);
  if (!res.ok) throw new Error('Failed to list terminals');
  const data = await res.json();
  return data.sessions ?? [];
}

export function streamTerminalOutput(
  terminalId: string,
  onData: (data: string) => void,
  onExit: (code: number | null) => void,
  onError: (err: string) => void,
  signal?: AbortSignal
): () => void {
  let closed = false;

  waitForApiReady().then(() => {
  fetch(`${API_BASE}/terminal/subscribe/${encodeURIComponent(terminalId)}`, { signal })
    .then(async (res) => {
      if (!res.ok || !res.body) {
        onError('Failed to open terminal stream');
        return;
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
          if (line.startsWith('event:exit')) {
            closed = true;
            onExit(null);
            return;
          }
          if (line.startsWith('event:data')) continue;
          if (line.startsWith('data:')) {
            const data = line.startsWith('data: ') ? line.slice(6) : line.slice(5);
            if (data === '[DONE]' || data === '[CLOSED]') {
              closed = true;
              onExit(null);
              return;
            }
            try {
              const parsed = JSON.parse(data);
              if (parsed.type === 'data' && parsed.data) {
                onData(parsed.data);
              } else if (parsed.type === 'exit') {
                onExit(parsed.code ?? null);
                closed = true;
                return;
              } else if (parsed.data) {
                onData(parsed.data);
              }
            } catch {
              onData(line + '\n');
            }
          }
        }
      }
      if (!closed) onExit(null);
    })
    .catch((err) => {
      if (err.name !== 'AbortError') onError(err.message || 'Terminal stream error');
    });
  });

  return () => {
    closed = true;
  };
}
