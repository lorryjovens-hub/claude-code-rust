import { useEffect, useRef, useCallback } from 'react';

interface StreamEvent {
  type: string;
  data: any;
  timestamp: number;
}

interface StreamState {
  events: StreamEvent[];
  isRunning: boolean;
  error: string | null;
  complete: boolean;
}

export function useStreamParser() {
  const workerRef = useRef<Worker | null>(null);
  const stateRef = useRef<StreamState>({
    events: [],
    isRunning: false,
    error: null,
    complete: false,
  });
  const callbacksRef = useRef<{
    onEvent?: (event: StreamEvent) => void;
    onComplete?: () => void;
    onError?: (error: string) => void;
  }>({});

  useEffect(() => {
    try {
      const workerUrl = new URL('../workers/streamParser.worker.ts', import.meta.url);
      const worker = new Worker(workerUrl, { type: 'module' });

      worker.onmessage = (e) => {
        const response = e.data;
        if (response.action === 'events' && response.events) {
          stateRef.current.events.push(...response.events);
          response.events.forEach((event: StreamEvent) => {
            callbacksRef.current.onEvent?.(event);
          });
        } else if (response.action === 'error') {
          stateRef.current.error = response.error || 'Unknown error';
          callbacksRef.current.onError?.(response.error);
        }
      };

      worker.onerror = (error) => {
        stateRef.current.error = error.message;
        callbacksRef.current.onError?.(error.message);
      };

      workerRef.current = worker;
      worker.postMessage({ action: 'init' });

      return () => {
        worker.terminate();
        workerRef.current = null;
      };
    } catch (err) {
      console.warn('Web Worker not available, falling back to main thread parsing', err);
    }
  }, []);

  const parseChunk = useCallback((buffer: string) => {
    if (workerRef.current) {
      workerRef.current.postMessage({ action: 'parse_chunk', buffer });
    } else {
      console.warn('Web Worker not available for stream parsing');
    }
  }, []);

  const reset = useCallback(() => {
    stateRef.current = {
      events: [],
      isRunning: false,
      error: null,
      complete: false,
    };
    if (workerRef.current) {
      workerRef.current.postMessage({ action: 'reset' });
    }
  }, []);

  const getEvents = useCallback(() => stateRef.current.events, []);
  const getError = useCallback(() => stateRef.current.error, []);
  const isComplete = useCallback(() => stateRef.current.complete, []);

  const setCallbacks = useCallback((callbacks: {
    onEvent?: (event: StreamEvent) => void;
    onComplete?: () => void;
    onError?: (error: string) => void;
  }) => {
    callbacksRef.current = callbacks;
  }, []);

  return {
    parseChunk,
    reset,
    getEvents,
    getError,
    isComplete,
    setCallbacks,
  };
}
