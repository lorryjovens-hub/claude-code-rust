export interface StreamEvent {
  type: string;
  data: any;
  timestamp: number;
}

interface WorkerMessage {
  action: 'init' | 'parse_chunk' | 'reset';
  buffer?: string;
}

interface WorkerResponse {
  action: 'events' | 'error';
  events?: StreamEvent[];
  error?: string;
}

let pendingBuffer = '';

function parseSSE(buffer: string): StreamEvent[] {
  const events: StreamEvent[] = [];
  const lines = buffer.split('\n');

  let currentEvent: StreamEvent | null = null;
  let dataBuffer = '';

  for (const line of lines) {
    const trimmed = line.trim();

    if (!trimmed) {
      if (currentEvent && dataBuffer) {
        try {
          currentEvent.data = JSON.parse(dataBuffer);
        } catch {
          currentEvent.data = dataBuffer;
        }
        events.push(currentEvent);
        currentEvent = null;
        dataBuffer = '';
      }
      continue;
    }

    if (trimmed.startsWith('event:')) {
      currentEvent = {
        type: trimmed.slice(6).trim(),
        data: null,
        timestamp: Date.now(),
      };
    } else if (trimmed.startsWith('data:')) {
      dataBuffer += trimmed.slice(5).trim();
    }
  }

  return events;
}

self.addEventListener('message', (e: MessageEvent<WorkerMessage>) => {
  const { action } = e.data;

  try {
    if (action === 'init') {
      pendingBuffer = '';
    } else if (action === 'parse_chunk') {
      pendingBuffer += e.data.buffer || '';
      const events = parseSSE(pendingBuffer);

      if (events.length > 0) {
        const lastCompleteEventEnd = pendingBuffer.lastIndexOf('\n\n');
        if (lastCompleteEventEnd >= 0) {
          pendingBuffer = pendingBuffer.slice(lastCompleteEventEnd + 2);
        }
      }

      const response: WorkerResponse = {
        action: 'events',
        events,
      };
      self.postMessage(response);
    } else if (action === 'reset') {
      pendingBuffer = '';
    }
  } catch (err) {
    const response: WorkerResponse = {
      action: 'error',
      error: (err as Error).message,
    };
    self.postMessage(response);
  }
});
