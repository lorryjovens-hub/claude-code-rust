import { getConversation, getContextSize, getAttachmentUrl } from '../../api';
import { persistMessages } from '../../api/conversations';
import { compactConversation } from '../../api';
import { useTabStore } from '../../stores/useTabStore';
import { executeCode, sendCodeResult } from '../../pyodideRunner';
import { trackTokensUsed } from '../../hooks/useAnalytics';
import {
  mergeDocumentsIntoMessage,
  mergeDocumentDraftIntoMessage,
} from '../../utils/messageHelpers';

/**
 * Dispatch an agent activity event for the Agent Execution View.
 * Components can listen via: window.addEventListener('agent-activity', handler)
 */
function dispatchAgentActivity(detail: {
  type: 'thinking' | 'tool_call' | 'code_gen' | 'file_change' | 'search' | 'complete' | 'error';
  tool_name?: string;
  tool_use_id?: string;
  content?: string;
  input?: any;
  is_error?: boolean;
  status?: 'running' | 'done' | 'error';
}) {
  if (typeof window === 'undefined') return;
  window.dispatchEvent(new CustomEvent('agent-activity', { detail }));
}

interface StreamContext {
  conversationId: string;
  streamRequestId: number;
  isStreamSessionActive: (convId: string, reqId: number) => boolean;
  setMessagesFor: (convId: string, updater: (prev: any[]) => any[]) => void;
  setMessages: (updater: (prev: any[]) => any[]) => void;
  removeStreaming: (convId: string) => void;
  clearStreamSession: (convId: string, reqId: number) => void;
  messagesBufferRef: React.MutableRefObject<Map<string, any[]>>;
  activeRequestCountRef: React.MutableRefObject<number>;
  viewingIdRef: React.MutableRefObject<string | null>;
  abortControllerRef: React.MutableRefObject<AbortController | null>;
  isCreatingRef?: React.MutableRefObject<boolean>;
  setLoading: (loading: boolean) => void;
  setCompactStatus: (status: any) => void;
  setContextInfo: (info: { tokens: number; limit: number }) => void;
  setTokenUsage: (updater: (prev: any) => any) => void;
  setActiveTasks: (updater: (prev: Map<string, any>) => Map<string, any>) => void;
  setAskUserDialog: (dialog: any) => void;
  setToolPermissionDialog: (dialog: any) => void;
  setPermissionApproval: (approval: any) => void;
  setPlanMode: (mode: boolean) => void;
  setConversationTitle: (title: string) => void;
  loadConversation?: (id: string) => Promise<void>;
  activeId?: string | null;
  pollTitle?: boolean;
}

function isSearchStatusMessage(message: string) {
  if (!message) return false;
  return (
    message.startsWith('正在搜索：') ||
    message.startsWith('正在读取网页：') ||
    message.startsWith('正在浏览 GitHub：') ||
    message.startsWith('Searching:') ||
    message.startsWith('Fetching:')
  );
}

function applyResearchEvent(prev: any[], event: string, data: any): any[] {
  const newMsgs = [...prev];
  const lastIdx = newMsgs.length - 1;
  const lastMsg = newMsgs[lastIdx];
  if (!lastMsg || lastMsg.role !== 'assistant') return prev;
  const research = { ...(lastMsg.research || { sub_agents: [], sources: [], phase: null, plan: null, report: null, completed: false }) };
  research.sub_agents = [...(research.sub_agents || [])];
  research.sources = [...(research.sources || [])];
  switch (event) {
    case 'research_phase':
      research.phase = data.phase;
      research.phase_label = data.label;
      break;
    case 'research_plan':
      research.plan = { title: data.title, sub_questions: data.sub_questions };
      break;
    case 'research_subagent_started': {
      const exists = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (!exists) {
        research.sub_agents.push({
          id: data.sub_agent_id, index: data.index,
          sub_question: data.sub_question, status: 'running',
          sources: [], findings: '',
        });
      }
      break;
    }
    case 'research_source': {
      const sub = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (sub) sub.sources = [...sub.sources, data.source];
      const exists = research.sources.find((s: any) => s.url === data.source.url);
      if (!exists) research.sources.push(data.source);
      break;
    }
    case 'research_finding': {
      const sub = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (sub) sub.findings = data.markdown || '';
      break;
    }
    case 'research_subagent_done': {
      const sub = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (sub) { sub.status = data.error ? 'error' : 'done'; if (data.error) sub.error = data.error; }
      break;
    }
    case 'research_report': research.report = data.markdown; break;
    case 'research_done': research.completed = true; research.duration_ms = data.duration_ms; break;
    case 'research_error': research.error = data.error; research.completed = true; break;
  }
  newMsgs[lastIdx] = { ...lastMsg, research };
  return newMsgs;
}

export function createStreamCallbacks(ctx: StreamContext) {
  const {
    conversationId, streamRequestId, isStreamSessionActive,
    setMessagesFor, setMessages, removeStreaming, clearStreamSession,
    messagesBufferRef, activeRequestCountRef, viewingIdRef, abortControllerRef,
    isCreatingRef, setLoading, setCompactStatus, setContextInfo, setTokenUsage,
    setActiveTasks, setAskUserDialog, setToolPermissionDialog, setPermissionApproval,
    setPlanMode, setConversationTitle, loadConversation, activeId, pollTitle,
  } = ctx;

  function cleanup() {
    removeStreaming(conversationId);
    messagesBufferRef.current.delete(conversationId);
    activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
  }

  const onDelta = (_delta: string, full: string) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    dispatchAgentActivity({ type: 'code_gen', content: full.slice(-200), status: 'running' });

    // Detect HTML code blocks for live preview
    const htmlMatch = full.match(/```html?\n([\s\S]*?)```/);
    if (htmlMatch && htmlMatch[1] && htmlMatch[1].trim().length > 100) {
      const html = htmlMatch[1].trim();
      if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('live-preview', {
          detail: { html, full, conversationId },
        }));
        // Auto-open live preview panel
        window.dispatchEvent(new CustomEvent('request-open-panel', {
          detail: { panel: 'preview' },
        }));
      }
    }
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastMsg = newMsgs[newMsgs.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        lastMsg.content = full;
        lastMsg.isThinking = false;
      }
      return newMsgs;
    });
  };

  const onError = (err: string) => {
    dispatchAgentActivity({ type: 'error', content: err, is_error: true, status: 'error' });
    cleanup();
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    if (viewingIdRef.current === conversationId) setLoading(false);
    abortControllerRef.current = null;
    if (isCreatingRef) isCreatingRef.current = false;
    clearStreamSession(conversationId, streamRequestId);
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      if (newMsgs[newMsgs.length - 1] && newMsgs[newMsgs.length - 1].role === 'assistant') {
        newMsgs[newMsgs.length - 1].content = 'Error: ' + err;
        newMsgs[newMsgs.length - 1].isThinking = false;
      }
      persistMessages(conversationId, newMsgs);
      return newMsgs;
    });
  };

  const onDone = (full: string) => {
    dispatchAgentActivity({ type: 'complete', content: 'Response complete', status: 'done' });
    cleanup();
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    if (viewingIdRef.current === conversationId) setLoading(false);
    abortControllerRef.current = null;
    if (isCreatingRef) isCreatingRef.current = false;
    clearStreamSession(conversationId, streamRequestId);
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastMsg = newMsgs[newMsgs.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        lastMsg.content = full;
        lastMsg.isThinking = false;
      }
      persistMessages(conversationId, newMsgs);
      return newMsgs;
    });

    if (pollTitle && conversationId) {
      let pollCount = 0;
      const maxPolls = 10;
      const refreshTitle = async () => {
        try {
          const data = await getConversation(conversationId);
          if (data && data.title && data.title !== 'New Chat') {
            setConversationTitle(data.title);
            const { openTabs } = useTabStore.getState();
            const tab = openTabs.find(t => t.conversationId === conversationId);
            if (tab) useTabStore.getState().renameTab(tab.id, data.title);
            window.dispatchEvent(new CustomEvent('conversationTitleUpdated'));
            return true;
          }
          pollCount++;
          if (pollCount < maxPolls) setTimeout(refreshTitle, 2000);
          return false;
        } catch {
          pollCount++;
          if (pollCount < maxPolls) setTimeout(refreshTitle, 2000);
          return false;
        }
      };
      setTimeout(refreshTitle, 1500);
    }
  };

  const onThinking = (_thinkingDelta: string, thinkingFull: string) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    dispatchAgentActivity({ type: 'thinking', content: thinkingFull.slice(-300), status: 'running' });
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastMsg = newMsgs[newMsgs.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        lastMsg.thinking = thinkingFull;
        lastMsg.isThinking = true;
        delete lastMsg.searchStatus;
      }
      return newMsgs;
    });
  };

  const onSystemEvents = (event: string, message: string, data: any) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;

    if (event === 'metadata' && data?.user_message_id) {
      setMessages(prev => {
        const newMsgs = [...prev];
        const userIdx = newMsgs.length - 2;
        if (userIdx >= 0 && newMsgs[userIdx].role === 'user') {
          newMsgs[userIdx] = { ...newMsgs[userIdx], id: data.user_message_id };
        }
        return newMsgs;
      });
    }
    if (event === 'status' && message && isSearchStatusMessage(message)) {
      dispatchAgentActivity({ type: 'search', content: message, status: 'running' });
      setMessagesFor(conversationId, prev => {
        const newMsgs = [...prev];
        const lastMsg = newMsgs[newMsgs.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.searchStatus = message;
          lastMsg._contentLenBeforeSearch = (lastMsg.content || '').length;
        }
        return newMsgs;
      });
    }
    if (event === 'thinking_summary' && message) {
      setMessages(prev => {
        const newMsgs = [...prev];
        const lastMsg = newMsgs[newMsgs.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') lastMsg.thinking_summary = message;
        return newMsgs;
      });
    }
    if (event === 'compaction_start') setCompactStatus({ state: 'compacting' });
    if (event === 'compaction_done') {
      if (data?.messagesCompacted > 0) {
        setCompactStatus({ state: 'done', message: `Compacted ${data.messagesCompacted} messages, saved ~${data.tokensSaved} tokens` });
        setTimeout(() => setCompactStatus({ state: 'idle' }), 4000);
      } else {
        setCompactStatus({ state: 'idle' });
      }
    }
    if (event === 'compact_boundary') {
      const meta = data?.compact_metadata || {};
      const preTokens = meta.pre_tokens || 0;
      const saved = preTokens ? Math.round(preTokens * 0.7) : 0;
      setCompactStatus({ state: 'done', message: saved > 0 ? `Auto-compacted, saved ~${saved} tokens` : 'Context auto-compacted' });
      setTimeout(() => setCompactStatus({ state: 'idle' }), 4000);
      if (activeId && loadConversation) {
        loadConversation(activeId);
        getContextSize(activeId).then(setContextInfo).catch(() => {});
      }
    }
    if (event === 'context_size' && data) setContextInfo({ tokens: data.tokens, limit: data.limit });
    if (event === 'tool_text_offset' && data?.offset != null) {
      setMessages(prev => {
        const newMsgs = [...prev];
        const lastMsg = newMsgs[newMsgs.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') lastMsg.toolTextEndOffset = data.offset;
        return newMsgs;
      });
    }
    if (event?.startsWith('research_')) setMessages(prev => applyResearchEvent(prev, event, data));
    if (event === 'ask_user' && data) {
      setAskUserDialog({ request_id: data.request_id, tool_use_id: data.tool_use_id, questions: data.questions || [], answers: {} });
    }
    if (event === 'tool_permission' && data) {
      setToolPermissionDialog({ request_id: data.request_id, tool_use_id: data.tool_use_id, tool_name: data.tool_name, input: data.input });
      setPermissionApproval({
        id: data.request_id, tool_name: data.tool_name,
        action: data.action || data.tool_name, risk_level: data.risk_level || 'medium',
        description: data.description || `Tool "${data.tool_name}" requires permission to execute.`,
      });
    }
    if (event === 'message_start' && data?.usage) {
      const u = data.usage;
      trackTokensUsed(u.input_tokens || 0, u.output_tokens || 0, data.message?.model);
      setTokenUsage((prev: any) => ({ input_tokens: (prev?.input_tokens || 0) + (u.input_tokens || 0), output_tokens: (prev?.output_tokens || 0) + (u.output_tokens || 0) }));
    }
    if (event === 'message_delta' && data?.usage) {
      const u = data.usage;
      trackTokensUsed(0, u.output_tokens || 0);
      setTokenUsage((prev: any) => ({ input_tokens: prev?.input_tokens || 0, output_tokens: (prev?.output_tokens || 0) + (u.output_tokens || 0) }));
    }
    if (event === 'task_event' && data) {
      if (data.subtype === 'task_started') {
        dispatchAgentActivity({ type: 'tool_call', tool_name: data.description || 'Task', status: 'running' });
      } else if (data.subtype === 'task_progress') {
        dispatchAgentActivity({ type: 'code_gen', tool_name: data.last_tool_name, content: data.summary, status: 'running' });
      }
      setActiveTasks(prev => {
        const next = new Map(prev);
        if (data.subtype === 'task_started') next.set(data.task_id, { description: data.description || 'Running task...' });
        else if (data.subtype === 'task_progress') {
          const existing = next.get(data.task_id);
          if (existing) next.set(data.task_id, { ...existing, last_tool_name: data.last_tool_name, summary: data.summary });
        } else if (data.subtype === 'task_notification') next.delete(data.task_id);
        return next;
      });
    }
  };

  const onCitations = (sources: any[], query?: string, tokens?: number) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastMsg = newMsgs[newMsgs.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        const existing = lastMsg.citations || [];
        const existingUrls = new Set(existing.map((s: any) => s.url));
        const newSources = sources.filter((s: any) => !existingUrls.has(s.url));
        lastMsg.citations = [...existing, ...newSources];
        if (query) {
          const logs = lastMsg.searchLogs || [];
          const existingLogIndex = logs.findIndex((log: any) => log.query === query);
          if (existingLogIndex !== -1) {
            const existingLog = logs[existingLogIndex];
            const currentResults = existingLog.results || [];
            const currentUrls = new Set(currentResults.map((r: any) => r.url));
            existingLog.results = [...currentResults, ...sources.filter((s: any) => !currentUrls.has(s.url))];
            if (tokens !== undefined) existingLog.tokens = tokens;
          } else {
            logs.push({ query, results: sources, tokens });
          }
          lastMsg.searchLogs = logs;
        }
      }
      return newMsgs;
    });
  };

  const onDocument = (doc: any) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastIdx = newMsgs.length - 1;
      if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
        newMsgs[lastIdx] = mergeDocumentsIntoMessage(newMsgs[lastIdx], doc);
      }
      return newMsgs;
    });
  };

  const onDocumentDraft = (draft: any) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastIdx = newMsgs.length - 1;
      if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
        newMsgs[lastIdx] = mergeDocumentDraftIntoMessage(newMsgs[lastIdx], draft);
      }
      return newMsgs;
    });
  };

  const onCodeExecution = async (data: any) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    if (data.type === 'code_execution') {
      setMessages(prev => {
        const newMsgs = [...prev];
        const lastMsg = newMsgs[newMsgs.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          lastMsg.codeExecution = {
            language: data.language || 'python', code: data.code || '',
            status: 'running' as const, stdout: '', stderr: '', images: [], error: undefined,
          };
        }
        return newMsgs;
      });
      const authToken = localStorage.getItem('auth_token') || '';
      const files = (data.files || []).map((f: any) => ({
        name: f.name,
        url: (() => {
          const baseUrl = getAttachmentUrl(f.id);
          if (!authToken) return baseUrl;
          return `${baseUrl}${baseUrl.includes('?') ? '&' : '?'}token=${encodeURIComponent(authToken)}`;
        })(),
      }));
      try {
        const result = await executeCode(data.code || '', files, data.executionId);
        await sendCodeResult(data.executionId, result);
      } catch (e: unknown) {
        await sendCodeResult(data.executionId, { stdout: '', stderr: '', images: [], error: e instanceof Error ? e.message : 'Pyodide 执行失败' });
      }
    }
    if (data.type === 'code_result') {
      setMessages(prev => {
        const newMsgs = [...prev];
        const lastMsg = newMsgs[newMsgs.length - 1];
        if (lastMsg && lastMsg.role === 'assistant' && lastMsg.codeExecution) {
          lastMsg.codeExecution = {
            ...lastMsg.codeExecution,
            status: data.error ? 'error' as const : 'done' as const,
            stdout: data.stdout || '', stderr: data.stderr || '',
            images: data.images || [], error: data.error || undefined,
          };
        }
        return newMsgs;
      });
    }
  };

  const onToolUse = (toolEvent: any) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    if (toolEvent.type === 'start') {
      dispatchAgentActivity({
        type: 'tool_call', tool_name: toolEvent.tool_name, tool_use_id: toolEvent.tool_use_id,
        input: toolEvent.tool_input, status: 'running',
      });
    }
    if (toolEvent.type === 'done') {
      dispatchAgentActivity({
        type: 'tool_call', tool_name: toolEvent.tool_name, tool_use_id: toolEvent.tool_use_id,
        content: toolEvent.content, is_error: toolEvent.is_error, status: toolEvent.is_error ? 'error' : 'done',
      });
    }
    if (toolEvent.type === 'done' && toolEvent.tool_name === 'EnterPlanMode') setPlanMode(true);
    if (toolEvent.type === 'done' && toolEvent.tool_name === 'ExitPlanMode') setPlanMode(false);
    const INTERNAL_TOOLS = new Set(['EnterPlanMode', 'ExitPlanMode', 'TaskCreate', 'TaskUpdate', 'TaskGet', 'TaskList', 'TaskOutput', 'TaskStop']);
    if (INTERNAL_TOOLS.has(toolEvent.tool_name || '')) return;

    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastMsg = newMsgs[newMsgs.length - 1];
      if (!lastMsg || lastMsg.role !== 'assistant') return prev;
      const toolCalls = lastMsg.toolCalls || [];
      if (toolEvent.type === 'start') {
        let existing = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
        if (existing) {
          existing.name = toolEvent.tool_name || existing.name;
          if (toolEvent.tool_input && Object.keys(toolEvent.tool_input).length > 0) existing.input = toolEvent.tool_input;
          if (toolEvent.textBefore) existing.textBefore = toolEvent.textBefore;
        } else {
          toolCalls.push({ id: toolEvent.tool_use_id, name: toolEvent.tool_name || 'unknown', input: toolEvent.tool_input || {}, status: 'running' as const, textBefore: toolEvent.textBefore || '' });
        }
      } else if (toolEvent.type === 'input') {
        const tc = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
        if (tc) tc.input = toolEvent.tool_input || {};
      } else if (toolEvent.type === 'done') {
        let tc = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
        if (!tc) {
          tc = { id: toolEvent.tool_use_id, name: toolEvent.tool_name || 'unknown', input: {}, status: 'done' as const, result: toolEvent.content };
          toolCalls.push(tc);
        } else {
          tc.status = toolEvent.is_error ? 'error' as const : 'done' as const;
          tc.result = toolEvent.content;
        }
      }
      lastMsg.toolCalls = toolCalls;
      persistMessages(conversationId, newMsgs);
      return newMsgs;
    });
  };

  const onDoneSimple = (full: string) => {
    cleanup();
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    if (viewingIdRef.current === conversationId) setLoading(false);
    abortControllerRef.current = null;
    if (isCreatingRef) isCreatingRef.current = false;
    clearStreamSession(conversationId, streamRequestId);
    setMessagesFor(conversationId, prev => {
      const newMsgs = [...prev];
      const lastMsg = newMsgs[newMsgs.length - 1];
      if (lastMsg && lastMsg.role === 'assistant') {
        lastMsg.content = full;
        lastMsg.isThinking = false;
      }
      persistMessages(conversationId, newMsgs);
      return newMsgs;
    });
  };

  const onSystemEventsLight = (event: string, message: string, data: any) => {
    if (!isStreamSessionActive(conversationId, streamRequestId)) return;
    if (event === 'metadata' && data?.user_message_id) {
      setMessagesFor(conversationId, prev => {
        const newMsgs = [...prev];
        const userIdx = newMsgs.length - 2;
        if (userIdx >= 0 && newMsgs[userIdx].role === 'user') {
          newMsgs[userIdx] = { ...newMsgs[userIdx], id: data.user_message_id };
        }
        return newMsgs;
      });
    }
    if (event === 'thinking_summary' && message) {
      setMessagesFor(conversationId, prev => {
        const newMsgs = [...prev];
        const lastMsg = newMsgs[newMsgs.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') lastMsg.thinking_summary = message;
        return newMsgs;
      });
    }
    if (event === 'context_size' && data) setContextInfo({ tokens: data.tokens, limit: data.limit });
    if (event === 'tool_text_offset' && data?.offset != null) {
      setMessages(prev => {
        const newMsgs = [...prev];
        const lastMsg = newMsgs[newMsgs.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') lastMsg.toolTextEndOffset = data.offset;
        return newMsgs;
      });
    }
    if (event?.startsWith('research_')) setMessages(prev => applyResearchEvent(prev, event, data));
  };

  return {
    onDelta, onDone, onError, onThinking, onSystemEvents,
    onCitations, onDocument, onDocumentDraft, onCodeExecution, onToolUse,
    onDoneSimple, onSystemEventsLight,
  };
}
