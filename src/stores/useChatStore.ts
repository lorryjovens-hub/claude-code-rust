import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export interface ModelInfo {
  id: string;
  name: string;
  provider?: string;
  max_tokens?: number;
  supports_vision?: boolean;
  supports_tools?: boolean;
  input_price?: number;
  output_price?: number;
  enabled?: boolean | number;
  tier?: string;
  display_name?: string;
}

interface ModelCatalog {
  common: ModelInfo[];
  all: ModelInfo[];
  fallback_model: string | null;
}

interface CrossModeWarning {
  convId: string;
  originalModel: string;
  otherMode: 'clawparrot' | 'selfhosted';
  fallbackModel: string;
}

export interface CompactStatus {
  state: 'idle' | 'compacting' | 'done' | 'error';
  message?: string;
}

export interface ChatMessage {
  id?: string;
  role?: string;
  content?: string;
  thinking?: string;
  created_at?: string;
  files?: Array<{ id: string; name: string; url: string }>;
  tool_calls?: Array<{
    id: string;
    name: string;
    input: Record<string, unknown>;
    output?: string;
    error?: string;
  }>;
  citations?: Array<{ url: string; title: string; cited_text?: string }>;
  model?: string;
  usage?: { input_tokens: number; output_tokens: number };
  thinking_summary?: string;
  toolTextEndOffset?: number;
  codeExecution?: { language: string; code: string; status?: 'running' | 'done' | 'error'; stdout?: string; stderr?: string; images?: string[]; error?: string };
  research?: { status?: string; query?: string; results?: unknown[]; sub_agents?: unknown[]; sources?: unknown[] };
  isThinking?: boolean;
  thinkingSummary?: string;
  searchLogs?: unknown[];
  document?: { id?: string; title?: string; url?: string; format?: string; content?: string; filename?: string } | null;
  documents?: Array<{ id?: string; title?: string; url?: string; format?: string; content?: string; filename?: string }>;
  documentDrafts?: Array<{ draftId?: string; draft_id?: string; title?: string; format?: string; preview?: string; previewAvailable?: boolean; preview_available?: boolean; done?: boolean; document?: { content?: string } }>;
}

export interface ProviderInfo {
  id: string;
  name: string;
  provider_type?: string;
  api_key?: string;
  apiKey?: string;
  base_url?: string;
  baseUrl?: string;
  format?: string;
  enabled: boolean;
  models?: string[] | Array<{ id: string; name: string; enabled?: boolean }>;
  icon?: string;
  supportsWebSearch?: boolean;
  webSearchStrategy?: string | null;
  webSearchTestedAt?: number;
  webSearchTestReason?: string | null;
}

interface ChatState {
  messages: ChatMessage[];
  loading: boolean;
  inputText: string;
  conversationTitle: string;
  conversationId: string | null;
  modelCatalog: ModelCatalog | null;
  currentModel: string;
  userMode: string;
  researchMode: boolean;
  openedResearchMsgId: string | null;
  compactStatus: CompactStatus;
  compactInstruction: string;
  planMode: boolean;
  crossModeWarning: CrossModeWarning | null;
  providersCache: ProviderInfo[];
  webSearchToast: string | null;
  permissionMode: string;
  reasoningMode: string | null;

  setMessages: (messages: ChatMessage[] | ((prev: ChatMessage[]) => ChatMessage[])) => void;
  appendMessage: (msg: ChatMessage) => void;
  updateLastMessage: (updater: (msg: ChatMessage) => ChatMessage) => void;
  appendToLastMessage: (delta: string) => void;
  appendThinkingToLastMessage: (thinking: string) => void;
  setLoading: (loading: boolean) => void;
  setInputText: (text: string | ((prev: string) => string)) => void;
  setConversationTitle: (title: string) => void;
  setConversationId: (id: string | null) => void;
  setModelCatalog: (catalog: ModelCatalog | null) => void;
  setCurrentModel: (model: string | ((prev: string) => string)) => void;
  setUserMode: (mode: string) => void;
  setResearchMode: (mode: boolean) => void;
  setOpenedResearchMsgId: (id: string | null) => void;
  setCompactStatus: (status: CompactStatus) => void;
  setCompactInstruction: (instruction: string) => void;
  setPlanMode: (mode: boolean) => void;
  setCrossModeWarning: (warning: CrossModeWarning | null) => void;
  setProvidersCache: (providers: ProviderInfo[]) => void;
  setWebSearchToast: (toast: string | null) => void;
  setPermissionMode: (mode: string) => void;
  setReasoningMode: (mode: string | null) => void;
  resetChat: () => void;
}

const initialState = {
  messages: [] as ChatMessage[],
  loading: false,
  inputText: '',
  conversationTitle: '',
  conversationId: null as string | null,
  modelCatalog: null as ModelCatalog | null,
  currentModel: '',
  userMode: '',
  researchMode: false,
  openedResearchMsgId: null as string | null,
  compactStatus: { state: 'idle' as const },
  compactInstruction: '',
  planMode: false,
  crossModeWarning: null as CrossModeWarning | null,
  providersCache: [] as ProviderInfo[],
  webSearchToast: null as string | null,
  permissionMode: (() => {
    try {
      return localStorage.getItem('permission_mode') || 'accept_edits';
    } catch { return 'accept_edits'; }
  })() as string,
  reasoningMode: null as string | null,
};

export const useChatStore = create<ChatState>()(
  subscribeWithSelector((set) => ({
    ...initialState,

    setMessages: (messages) =>
      set((state) => ({
        messages: typeof messages === 'function' ? messages(state.messages) : messages,
      })),

    appendMessage: (msg) =>
      set((state) => ({ messages: [...state.messages, msg] })),

    updateLastMessage: (updater) =>
      set((state) => {
        if (state.messages.length === 0) return state;
        const lastIdx = state.messages.length - 1;
        const updated = updater(state.messages[lastIdx]);
        if (updated === state.messages[lastIdx]) return state;
        const messages = [...state.messages];
        messages[lastIdx] = updated;
        return { messages };
      }),

    appendToLastMessage: (delta) =>
      set((state) => {
        if (state.messages.length === 0) return state;
        const lastIdx = state.messages.length - 1;
        const lastMsg = state.messages[lastIdx];
        // Bug #6 fix: Handle null/undefined content — initialize to empty string
        // For array content (multimodal), append as text to a new string content
        let currentContent: string;
        if (typeof lastMsg.content === 'string') {
          currentContent = lastMsg.content;
        } else if (Array.isArray(lastMsg.content)) {
          // Extract text from multimodal content array
          const contentArr = lastMsg.content as unknown as any[];
          currentContent = contentArr
            .filter((b: any) => b.type === 'text' && typeof b.text === 'string')
            .map((b: any) => b.text)
            .join('');
        } else {
          currentContent = '';
        }
        const newContent = currentContent + delta;
        if (newContent === currentContent) return state;
        const messages = [...state.messages];
        messages[lastIdx] = { ...lastMsg, content: newContent };
        return { messages };
      }),

    appendThinkingToLastMessage: (thinking) =>
      set((state) => {
        if (state.messages.length === 0) return state;
        const lastIdx = state.messages.length - 1;
        const lastMsg = state.messages[lastIdx];
        const newThinking = (lastMsg.thinking ?? '') + thinking;
        if (newThinking === lastMsg.thinking) return state;
        const messages = [...state.messages];
        messages[lastIdx] = { ...lastMsg, thinking: newThinking };
        return { messages };
      }),

    setLoading: (loading) => set({ loading }),

    setInputText: (inputText) =>
      set((state) => ({
        inputText: typeof inputText === 'function' ? inputText(state.inputText) : inputText,
      })),

    setConversationTitle: (conversationTitle) => set({ conversationTitle }),
    setConversationId: (conversationId) => set({ conversationId }),
    setModelCatalog: (modelCatalog) => set({ modelCatalog }),

    setCurrentModel: (currentModel) =>
      set((state) => ({
        currentModel: typeof currentModel === 'function' ? currentModel(state.currentModel) : currentModel,
      })),

    setUserMode: (userMode) => set({ userMode }),
    setResearchMode: (researchMode) => set({ researchMode }),
    setOpenedResearchMsgId: (openedResearchMsgId) => set({ openedResearchMsgId }),
    setCompactStatus: (compactStatus) => set({ compactStatus }),
    setCompactInstruction: (compactInstruction) => set({ compactInstruction }),
    setPlanMode: (planMode) => set({ planMode }),
    setCrossModeWarning: (crossModeWarning) => set({ crossModeWarning }),
    setProvidersCache: (providersCache) => set({ providersCache }),
    setWebSearchToast: (webSearchToast) => set({ webSearchToast }),
    setPermissionMode: (permissionMode: string) => {
      set({ permissionMode });
      try {
        localStorage.setItem('permission_mode', permissionMode);
      } catch {}
    },
    setReasoningMode: (reasoningMode: string | null) => set({ reasoningMode }),
    resetChat: () => {
      // Bug #7 fix: Flush pending deltas before resetting to avoid data loss
      flushAllDeltas();
      set(initialState);
    },
  }))
);

// Bug #5 fix: Per-conversation delta buffers instead of global variables
// to prevent cross-conversation delta contamination
let pendingDeltaMap = new Map<string, string>();
let deltaTimerMap = new Map<string, ReturnType<typeof setTimeout>>();
let currentDeltaConvId: string | null = null;

const flushDelta = (convId: string) => {
  const delta = pendingDeltaMap.get(convId);
  if (delta) {
    useChatStore.getState().appendToLastMessage(delta);
    pendingDeltaMap.delete(convId);
  }
  deltaTimerMap.delete(convId);
};

export function appendDeltaThrottled(delta: string, convId?: string) {
  const id = convId || useChatStore.getState().conversationId || '__default__';
  // If conversation changed, flush previous conversation's pending delta
  if (currentDeltaConvId && currentDeltaConvId !== id) {
    const existingTimer = deltaTimerMap.get(currentDeltaConvId);
    if (existingTimer) {
      clearTimeout(existingTimer);
      flushDelta(currentDeltaConvId);
    }
  }
  currentDeltaConvId = id;
  const existing = pendingDeltaMap.get(id) || '';
  pendingDeltaMap.set(id, existing + delta);
  if (!deltaTimerMap.has(id)) {
    deltaTimerMap.set(id, setTimeout(() => flushDelta(id), 0));
  }
}

let pendingThinkingMap = new Map<string, string>();
let thinkingTimerMap = new Map<string, ReturnType<typeof setTimeout>>();
let currentThinkingConvId: string | null = null;

const flushThinking = (convId: string) => {
  const thinking = pendingThinkingMap.get(convId);
  if (thinking) {
    useChatStore.getState().appendThinkingToLastMessage(thinking);
    pendingThinkingMap.delete(convId);
  }
  thinkingTimerMap.delete(convId);
};

export function appendThinkingThrottled(thinking: string, convId?: string) {
  const id = convId || useChatStore.getState().conversationId || '__default__';
  if (currentThinkingConvId && currentThinkingConvId !== id) {
    const existingTimer = thinkingTimerMap.get(currentThinkingConvId);
    if (existingTimer) {
      clearTimeout(existingTimer);
      flushThinking(currentThinkingConvId);
    }
  }
  currentThinkingConvId = id;
  const existing = pendingThinkingMap.get(id) || '';
  pendingThinkingMap.set(id, existing + thinking);
  if (!thinkingTimerMap.has(id)) {
    thinkingTimerMap.set(id, setTimeout(() => flushThinking(id), 0));
  }
}

export function flushAllDeltas() {
  for (const [convId, timer] of deltaTimerMap) {
    clearTimeout(timer);
    flushDelta(convId);
  }
  deltaTimerMap.clear();
  for (const [convId, timer] of thinkingTimerMap) {
    clearTimeout(timer);
    flushThinking(convId);
  }
  thinkingTimerMap.clear();
  currentDeltaConvId = null;
  currentThinkingConvId = null;
}

// ── Selector hooks — subscribe to individual fields to prevent over-renders ──

export function useMessages() { return useChatStore(s => s.messages); }
export function useLoading() { return useChatStore(s => s.loading); }
export function useConversationId() { return useChatStore(s => s.conversationId); }
export function useCurrentModel() { return useChatStore(s => s.currentModel); }
export function useUserMode() { return useChatStore(s => s.userMode); }
export function usePermissionMode() { return useChatStore(s => s.permissionMode); }
export function usePlanMode() { return useChatStore(s => s.planMode); }
export function useResearchMode() { return useChatStore(s => s.researchMode); }
export function useCompactStatus() { return useChatStore(s => s.compactStatus); }
export function useCrossModeWarning() { return useChatStore(s => s.crossModeWarning); }
