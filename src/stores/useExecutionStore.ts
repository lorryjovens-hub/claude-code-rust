import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export interface ToolExecutionResult {
  id: string;
  toolName: string;
  success: boolean;
  output: unknown;
  error: string | null;
  durationMs: number;
  retryCount: number;
}

export interface ExecutionProgress {
  executionId: string;
  layersTotal: number;
  layersCompleted: number;
  toolsTotal: number;
  toolsCompleted: number;
  currentLayer: number;
  currentTools: string[];
  results: Record<string, ToolExecutionResult>;
  status: 'idle' | 'running' | 'completed' | 'failed' | 'cancelled';
  totalDurationMs: number;
  successCount: number;
  failCount: number;
}

interface ExecutionState {
  progress: Map<string, ExecutionProgress>;
  activeExecutionId: string | null;

  startExecution: (executionId: string, toolsTotal: number, layersTotal: number) => void;
  updateLayerStart: (executionId: string, layer: number, toolCount: number) => void;
  updateLayerComplete: (executionId: string, layer: number) => void;
  updateToolStart: (executionId: string, toolId: string) => void;
  updateToolComplete: (executionId: string, result: ToolExecutionResult) => void;
  updateToolFailed: (executionId: string, toolId: string, error: string) => void;
  updateProgress: (executionId: string, completed: number, total: number) => void;
  completeExecution: (executionId: string, totalDurationMs: number, successCount: number, failCount: number) => void;
  cancelExecution: (executionId: string) => void;
  clearExecution: (executionId: string) => void;
  clearAll: () => void;
}

export const useExecutionStore = create<ExecutionState>()(
  subscribeWithSelector((set, get) => ({
    progress: new Map(),
    activeExecutionId: null,

    startExecution: (executionId, toolsTotal, layersTotal) =>
      set((state) => {
        const next = new Map(state.progress);
        next.set(executionId, {
          executionId,
          layersTotal,
          layersCompleted: 0,
          toolsTotal,
          toolsCompleted: 0,
          currentLayer: 0,
          currentTools: [],
          results: {},
          status: 'running',
          totalDurationMs: 0,
          successCount: 0,
          failCount: 0,
        });
        return { progress: next, activeExecutionId: executionId };
      }),

    updateLayerStart: (executionId, layer, toolCount) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          next.set(executionId, { ...p, currentLayer: layer, currentTools: Array(toolCount).fill('pending') });
        }
        return { progress: next };
      }),

    updateLayerComplete: (executionId, _layer) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          next.set(executionId, { ...p, layersCompleted: p.layersCompleted + 1 });
        }
        return { progress: next };
      }),

    updateToolStart: (executionId, toolId) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          const tools = [...p.currentTools];
          const idx = tools.indexOf('pending');
          if (idx >= 0) tools[idx] = toolId;
          next.set(executionId, { ...p, currentTools: tools });
        }
        return { progress: next };
      }),

    updateToolComplete: (executionId, result) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          const results = { ...p.results, [result.id]: result };
          const tools = p.currentTools.filter(t => t !== result.id);
          next.set(executionId, {
            ...p,
            toolsCompleted: p.toolsCompleted + 1,
            results,
            currentTools: tools,
          });
        }
        return { progress: next };
      }),

    updateToolFailed: (executionId, toolId, error) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          const results = {
            ...p.results,
            [toolId]: {
              id: toolId,
              toolName: toolId,
              success: false,
              output: null,
              error,
              durationMs: 0,
              retryCount: 0,
            },
          };
          const tools = p.currentTools.filter(t => t !== toolId);
          next.set(executionId, {
            ...p,
            toolsCompleted: p.toolsCompleted + 1,
            results,
            currentTools: tools,
          });
        }
        return { progress: next };
      }),

    updateProgress: (executionId, completed, total) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          next.set(executionId, { ...p, toolsCompleted: completed, toolsTotal: total });
        }
        return { progress: next };
      }),

    completeExecution: (executionId, totalDurationMs, successCount, failCount) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          next.set(executionId, {
            ...p,
            status: failCount > 0 ? 'failed' : 'completed',
            totalDurationMs,
            successCount,
            failCount,
          });
        }
        return { progress: next };
      }),

    cancelExecution: (executionId) =>
      set((state) => {
        const next = new Map(state.progress);
        const p = next.get(executionId);
        if (p) {
          next.set(executionId, { ...p, status: 'cancelled' });
        }
        return { progress: next };
      }),

    clearExecution: (executionId) =>
      set((state) => {
        const next = new Map(state.progress);
        next.delete(executionId);
        return {
          progress: next,
          activeExecutionId: state.activeExecutionId === executionId ? null : state.activeExecutionId,
        };
      }),

    clearAll: () => set({ progress: new Map(), activeExecutionId: null }),
  }))
);