import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

interface StreamingState {
  streamingIds: Set<string>;
  activeRequestId: string | null;
  abortController: AbortController | null;

  addStreaming: (id: string) => void;
  removeStreaming: (id: string) => void;
  isStreaming: (id: string) => boolean;
  setActiveRequestId: (id: string | null) => void;
  setAbortController: (controller: AbortController | null) => void;
  abort: () => void;
}

export const useStreamingStore = create<StreamingState>()(
  subscribeWithSelector((set, get) => ({
    streamingIds: new Set<string>(),
    activeRequestId: null,
    abortController: null,

    addStreaming: (id) =>
      set((state) => {
        if (state.streamingIds.has(id)) return state;
        const next = new Set(state.streamingIds);
        next.add(id);
        return { streamingIds: next };
      }),

    removeStreaming: (id) =>
      set((state) => {
        if (!state.streamingIds.has(id)) return state;
        const next = new Set(state.streamingIds);
        next.delete(id);
        return { streamingIds: next };
      }),

    isStreaming: (id) => get().streamingIds.has(id),

    setActiveRequestId: (activeRequestId) => set({ activeRequestId }),

    setAbortController: (abortController) => set({ abortController }),

    abort: () => {
      const { abortController } = get();
      if (abortController) {
        abortController.abort();
      }
      set({ abortController: null, activeRequestId: null, streamingIds: new Set() });
    },
  }))
);

let _subscribed = false;

export function ensureStreamingSubscription() {
  if (_subscribed) return;
  _subscribed = true;
  useStreamingStore.subscribe(
    (state) => state.streamingIds,
    () => {
      window.dispatchEvent(new CustomEvent('streaming-change'));
    },
  );
}

if (typeof window !== 'undefined') {
  ensureStreamingSubscription();
}
