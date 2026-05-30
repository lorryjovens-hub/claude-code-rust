import { useCallback, useEffect, useRef } from 'react';
import { stopGeneration } from '../api';
import { useChatStore } from '../stores/useChatStore';

/**
 * Hook that manages streaming conversation session state:
 * - Tracks the active stream request ID and conversation ID
 * - Provides abort/cleanup for running streams
 * - Cleans up on conversation switch / unmount
 */
export function useStreamSession() {
  const abortControllerRef = useRef<AbortController | null>(null);
  const activeRequestCountRef = useRef(0);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const streamConversationIdRef = useRef<string | null>(null);
  const streamRequestIdRef = useRef(0);
  const isCreatingRef = useRef(false);

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
      stopGeneration(trackedConversationId).catch(e => console.error('[Stop] error:', e));
    }

    useChatStore.getState().setLoading(false);
    isCreatingRef.current = false;
    return true;
  }, [stopPolling]);

  return {
    abortControllerRef,
    activeRequestCountRef,
    pollingRef,
    streamConversationIdRef,
    streamRequestIdRef,
    isCreatingRef,
    stopPolling,
    beginStreamSession,
    isStreamSessionActive,
    clearStreamSession,
    abortStreamSession,
  };
}

/**
 * React effect hook — cleans up polling on conversation switch or unmount.
 */
export function useStreamSessionCleanup(
  activeId: string | null,
  stopPolling: () => void,
  abortStreamSession: (targetConversationId?: string) => boolean,
) {
  useEffect(() => {
    return () => { stopPolling(); };
  }, [activeId, stopPolling]);

  useEffect(() => {
    const handleConversationDeleting = (evt: Event) => {
      const customEvt = evt as CustomEvent<{ id?: string }>;
      const conversationId = customEvt.detail?.id;
      if (!conversationId) return;
      abortStreamSession(conversationId);
    };

    window.addEventListener('conversationDeleting', handleConversationDeleting as EventListener);
    return () => {
      window.removeEventListener('conversationDeleting', handleConversationDeleting as EventListener);
    };
  }, [abortStreamSession]);
}
