import { useCallback, useRef, useEffect } from 'react';
import { trackEvent } from '../api';

let sessionId: string | null = null;

function getSessionId(): string {
  if (!sessionId) {
    const stored = sessionStorage.getItem('analytics_session_id');
    if (stored) {
      sessionId = stored;
    } else {
      sessionId = `sess-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
      sessionStorage.setItem('analytics_session_id', sessionId);
    }
  }
  return sessionId;
}

export function useAnalytics() {
  const queueRef = useRef<Array<{ eventType: string; properties?: Record<string, any> }>>([]);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flush = useCallback(() => {
    if (queueRef.current.length === 0) return;
    const batch = queueRef.current.splice(0);
    const sid = getSessionId();
    for (const item of batch) {
      trackEvent(item.eventType, item.properties, sid);
    }
  }, []);

  const track = useCallback((eventType: string, properties?: Record<string, any>) => {
    queueRef.current.push({ eventType, properties });
    if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
    flushTimerRef.current = setTimeout(flush, 500);
  }, [flush]);

  useEffect(() => {
    return () => {
      if (flushTimerRef.current) clearTimeout(flushTimerRef.current);
      flush();
    };
  }, [flush]);

  return { track };
}

export function trackMessageSent(model?: string) {
  const sid = getSessionId();
  trackEvent('message_sent', { model }, sid);
}

export function trackConversationCreated() {
  const sid = getSessionId();
  trackEvent('conversation_created', undefined, sid);
}

export function trackTokensUsed(inputTokens: number, outputTokens: number, model?: string) {
  const sid = getSessionId();
  trackEvent('tokens_used', { input_tokens: inputTokens, output_tokens: outputTokens, model }, sid);
}

export function trackToolExecuted(toolName: string) {
  const sid = getSessionId();
  trackEvent('tool_executed', { tool_name: toolName }, sid);
}

export function trackError(errorType: string, message?: string) {
  const sid = getSessionId();
  trackEvent('error', { error_type: errorType, message }, sid);
}

export function trackVoiceInput() {
  const sid = getSessionId();
  trackEvent('voice_input', undefined, sid);
}

export function trackSlashCommand(command: string) {
  const sid = getSessionId();
  trackEvent('slash_command', { command }, sid);
}

export function trackFileUploaded(fileName: string, fileSize?: number) {
  const sid = getSessionId();
  trackEvent('file_uploaded', { file_name: fileName, file_size: fileSize }, sid);
}
