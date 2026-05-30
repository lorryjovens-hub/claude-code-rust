import type { Provider, ProviderModel } from '../api/providers';

const KEYS = {
  providers: 'app_providers',
  conversations: 'app_conversations',
  messages: (convId: string) => `app_messages_${convId}`,
};

export interface LocalConversation {
  id: string;
  title?: string | null;
  model?: string | null;
  provider?: string | null;
  workspace_path?: string | null;
  project_id?: string | null;
  research_mode?: boolean;
  pinned?: boolean;
  archived?: boolean;
  created_at: string;
  updated_at: string;
  message_count: number;
}

export interface LocalMessage {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  thinking?: string | null;
  created_at: string;
  is_compact_boundary?: number;
  sort_order: number;
  toolCalls?: any[];
}

function safeGet<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function safeSet(key: string, value: any): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch (e) {
    console.warn('[localStorageService] Failed to write:', key, e);
  }
}

// ============ Providers ============

export function getLocalProviders(): Provider[] {
  return safeGet<Provider[]>(KEYS.providers, []);
}

export function saveLocalProviders(providers: Provider[]): void {
  safeSet(KEYS.providers, providers);
}

export function getLocalProvider(id: string): Provider | undefined {
  return getLocalProviders().find(p => p.id === id);
}

export function createLocalProvider(p: Partial<Provider> & { name: string; baseUrl: string }): Provider {
  const providers = getLocalProviders();
  const newProvider: Provider = {
    id: crypto.randomUUID(),
    name: p.name,
    baseUrl: p.baseUrl || '',
    apiKey: p.apiKey || '',
    format: p.format || 'openai',
    models: p.models || [],
    enabled: p.enabled !== false,
    supportsWebSearch: p.supportsWebSearch || false,
    webSearchStrategy: p.webSearchStrategy || null,
    webSearchTestedAt: p.webSearchTestedAt,
    webSearchTestReason: p.webSearchTestReason || null,
  };
  providers.push(newProvider);
  saveLocalProviders(providers);
  return newProvider;
}

export function updateLocalProvider(id: string, updates: Partial<Provider>): Provider | null {
  const providers = getLocalProviders();
  const idx = providers.findIndex(p => p.id === id);
  if (idx === -1) return null;
  providers[idx] = { ...providers[idx], ...updates };
  saveLocalProviders(providers);
  return providers[idx];
}

export function deleteLocalProvider(id: string): void {
  const providers = getLocalProviders().filter(p => p.id !== id);
  saveLocalProviders(providers);
}

// ============ Conversations ============

export function getLocalConversations(): LocalConversation[] {
  return safeGet<LocalConversation[]>(KEYS.conversations, []);
}

export function saveLocalConversations(convs: LocalConversation[]): void {
  safeSet(KEYS.conversations, convs);
}

export function getLocalConversation(id: string): LocalConversation | undefined {
  return getLocalConversations().find(c => c.id === id);
}

export function createLocalConversation(
  title?: string | null,
  model?: string | null,
  extras?: { research_mode?: boolean }
): LocalConversation {
  const convs = getLocalConversations();
  const now = new Date().toISOString();
  const newConv: LocalConversation = {
    id: crypto.randomUUID(),
    title: title || null,
    model: model || null,
    provider: null,
    workspace_path: null,
    project_id: null,
    research_mode: extras?.research_mode || false,
    pinned: false,
    archived: false,
    created_at: now,
    updated_at: now,
    message_count: 0,
  };
  convs.unshift(newConv);
  saveLocalConversations(convs);
  return newConv;
}

export function updateLocalConversation(id: string, updates: Partial<LocalConversation>): LocalConversation | null {
  const convs = getLocalConversations();
  const idx = convs.findIndex(c => c.id === id);
  if (idx === -1) return null;
  convs[idx] = { ...convs[idx], ...updates, updated_at: new Date().toISOString() };
  saveLocalConversations(convs);
  return convs[idx];
}

export function deleteLocalConversation(id: string): void {
  const convs = getLocalConversations().filter(c => c.id !== id);
  saveLocalConversations(convs);
  try { localStorage.removeItem(KEYS.messages(id)); } catch {}
}

// ============ Messages ============

export function getLocalMessages(conversationId: string): LocalMessage[] {
  return safeGet<LocalMessage[]>(KEYS.messages(conversationId), []);
}

export function saveLocalMessages(conversationId: string, messages: LocalMessage[]): void {
  safeSet(KEYS.messages(conversationId), messages);
  const convs = getLocalConversations();
  const idx = convs.findIndex(c => c.id === conversationId);
  if (idx !== -1) {
    convs[idx].message_count = messages.length;
    convs[idx].updated_at = new Date().toISOString();
    saveLocalConversations(convs);
  }
}

export function addLocalMessage(conversationId: string, message: LocalMessage): void {
  const messages = getLocalMessages(conversationId);
  messages.push(message);
  saveLocalMessages(conversationId, messages);
}

export function deleteLocalMessagesFrom(conversationId: string, messageId: string): void {
  const messages = getLocalMessages(conversationId);
  const idx = messages.findIndex(m => m.id === messageId);
  if (idx === -1) return;
  saveLocalMessages(conversationId, messages.slice(0, idx));
}

export function deleteLocalMessagesTail(conversationId: string, count: number): void {
  const messages = getLocalMessages(conversationId);
  saveLocalMessages(conversationId, messages.slice(0, Math.max(0, messages.length - count)));
}