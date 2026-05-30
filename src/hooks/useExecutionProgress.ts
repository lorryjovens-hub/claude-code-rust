import { useEffect, useRef } from 'react';
import { useExecutionStore } from '../stores/useExecutionStore';

interface ExecutionEventPayload {
  type: string;
  execution_id?: string;
  layer?: number;
  tool_count?: number;
  duration?: { secs: number; nanos: number };
  tool_id?: string;
  tool_name?: string;
  success?: boolean;
  output?: unknown;
  error?: string;
  retry_count?: number;
  completed?: number;
  total?: number;
  total_duration?: { secs: number; nanos: number };
  success_count?: number;
  fail_count?: number;
  plan?: {
    layer_indices: string[][];
    all_tools: Array<{ id: string; tool_name: string }>;
    max_parallelism: number;
  };
}

function msFromDuration(d: { secs: number; nanos: number } | undefined): number {
  if (!d) return 0;
  return d.secs * 1000 + Math.floor(d.nanos / 1_000_000);
}

export function useExecutionProgress(conversationId: string | null) {
  const store = useExecutionStore();
  const subscribedRef = useRef(false);

  useEffect(() => {
    if (!conversationId || subscribedRef.current) return;
    subscribedRef.current = true;

    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

    let unlisten: (() => void) | null = null;

    if (isTauri) {
      import('@tauri-apps/api/event')
        .then(({ listen }) => {
          listen<ExecutionEventPayload>('execution-event', (event) => {
            handleExecutionEvent(store, event.payload);
          }).then(fn => { unlisten = fn; });
        })
        .catch(() => {
          console.warn('Tauri event listener setup failed, using polling fallback');
        });
    }

    return () => {
      unlisten?.();
      subscribedRef.current = false;
    };
  }, [conversationId, store]);

  return {
    progress: conversationId ? store.progress.get(conversationId) : undefined,
    activeExecutionId: store.activeExecutionId,
    clearProgress: () => conversationId && store.clearExecution(conversationId),
  };
}

function handleExecutionEvent(store: ReturnType<typeof useExecutionStore.getState>, payload: ExecutionEventPayload) {
  const execId = payload.execution_id || 'default';

  switch (payload.type) {
    case 'PlanBuilt': {
      if (payload.plan) {
        const toolsTotal = payload.plan.all_tools.length;
        const layersTotal = payload.plan.layer_indices.length;
        store.startExecution(execId, toolsTotal, layersTotal);
      }
      break;
    }
    case 'LayerStarted': {
      store.updateLayerStart(execId, payload.layer ?? 0, payload.tool_count ?? 0);
      break;
    }
    case 'LayerCompleted': {
      store.updateLayerComplete(execId, payload.layer ?? 0);
      break;
    }
    case 'ToolStarted': {
      store.updateToolStart(execId, payload.tool_id ?? 'unknown');
      break;
    }
    case 'ToolCompleted': {
      store.updateToolComplete(execId, {
        id: payload.tool_id ?? 'unknown',
        toolName: payload.tool_name ?? 'unknown',
        success: true,
        output: payload.output,
        error: null,
        durationMs: msFromDuration(payload.duration),
        retryCount: payload.retry_count ?? 0,
      });
      break;
    }
    case 'ToolFailed': {
      store.updateToolFailed(execId, payload.tool_id ?? 'unknown', payload.error ?? 'Unknown error');
      break;
    }
    case 'Progress': {
      store.updateProgress(execId, payload.completed ?? 0, payload.total ?? 0);
      break;
    }
    case 'AllCompleted': {
      store.completeExecution(
        execId,
        msFromDuration(payload.total_duration),
        payload.success_count ?? 0,
        payload.fail_count ?? 0,
      );
      break;
    }
    case 'Cancelled': {
      store.cancelExecution(execId);
      break;
    }
    default:
      break;
  }
}

export { useExecutionStore };