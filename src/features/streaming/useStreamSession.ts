import { useCallback, useRef } from 'react';
import { useStreamingStore } from '../../stores/useStreamingStore';
import { useChatStore } from '../../stores/useChatStore';
import { stopGeneration } from '../../api';

export function useStreamSession(
  abortControllerRef: React.MutableRefObject<AbortController | null>,
  activeRequestCountRef: React.MutableRefObject<number>,
  isCreatingRef: React.MutableRefObject<boolean>,
) {
  const removeStreaming = useStreamingStore(s => s.removeStreaming);
  const setLoading = useChatStore(s => s.setLoading);

  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const streamConversationIdRef = useRef<string | null>(null);
  const streamRequestIdRef = useRef(0);

  const stopPolling = useCallback(() => {
    if (pollingRef.current) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
  }, []);

  const beginStreamSession = useCallback((conversationId: string) => {
    const nextId = streamRequestIdRef.current + 1;
    streamRequestIdRef.current = nextId;
    streamConversationIdRef.current = conversationId;
    return nextId;
  }, []);

  const isStreamSessionActive = useCallback((conversationId: string, requestId: number) => {
    return streamConversationIdRef.current === conversationId && streamRequestIdRef.current === requestId;
  }, []);

  const clearStreamSession = useCallback((conversationId: string, requestId: number) => {
    if (!isStreamSessionActive(conversationId, requestId)) return false;
    streamConversationIdRef.current = null;
    return true;
  }, [isStreamSessionActive]);

  const abortStreamSession = useCallback((targetConversationId?: string) => {
    const trackedConversationId = streamConversationIdRef.current;
    if (!trackedConversationId) return false;
    if (targetConversationId && trackedConversationId !== targetConversationId) return false;

    streamRequestIdRef.current += 1;
    streamConversationIdRef.current = null;

    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
      activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
    } else if (pollingRef.current) {
      stopPolling();
      stopGeneration(trackedConversationId).catch(() => {});
    }

    removeStreaming(trackedConversationId);
    setLoading(false);
    isCreatingRef.current = false;
    return true;
  }, [stopPolling, abortControllerRef, activeRequestCountRef, isCreatingRef, removeStreaming, setLoading]);

  return {
    pollingRef,
    stopPolling,
    beginStreamSession,
    isStreamSessionActive,
    clearStreamSession,
    abortStreamSession,
  };
}
