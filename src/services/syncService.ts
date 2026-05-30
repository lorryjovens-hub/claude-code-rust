import type { Provider } from '../api/providers';
import {
  LocalConversation,
  LocalMessage,
  getLocalProviders,
  saveLocalProviders,
  getLocalConversations,
  saveLocalConversations,
  getLocalMessages,
  saveLocalMessages,
  deleteLocalConversation,
  deleteLocalProvider,
} from './localStorageService';

const CHENGDU_API = 'http://127.0.0.1:30090/api';

const SYNC_KEYS = {
  lastPushAt: 'sync_lastPushAt',
  lastPullAt: 'sync_lastPullAt',
  deviceId: 'sync_deviceId',
  deletedProviders: 'sync_deletedProviders',
  deletedConversations: 'sync_deletedConversations',
  syncError: 'sync_lastError',
};

function getAuthToken(): string | null {
  return localStorage.getItem('auth_token');
}

function getDeviceId(): string {
  let deviceId = localStorage.getItem(SYNC_KEYS.deviceId);
  if (!deviceId) {
    deviceId = crypto.randomUUID();
    localStorage.setItem(SYNC_KEYS.deviceId, deviceId);
  }
  return deviceId;
}

function getDeletedProviders(): string[] {
  try {
    const raw = localStorage.getItem(SYNC_KEYS.deletedProviders);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function addDeletedProvider(id: string): void {
  const ids = getDeletedProviders();
  if (!ids.includes(id)) {
    ids.push(id);
    localStorage.setItem(SYNC_KEYS.deletedProviders, JSON.stringify(ids));
  }
}

function clearDeletedProviders(): void {
  localStorage.removeItem(SYNC_KEYS.deletedProviders);
}

function getDeletedConversations(): string[] {
  try {
    const raw = localStorage.getItem(SYNC_KEYS.deletedConversations);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function addDeletedConversation(id: string): void {
  const ids = getDeletedConversations();
  if (!ids.includes(id)) {
    ids.push(id);
    localStorage.setItem(SYNC_KEYS.deletedConversations, JSON.stringify(ids));
  }
}

function clearDeletedConversations(): void {
  localStorage.removeItem(SYNC_KEYS.deletedConversations);
}

function getLastPushAt(): string | null {
  return localStorage.getItem(SYNC_KEYS.lastPushAt);
}

function setLastPushAt(ts: string): void {
  localStorage.setItem(SYNC_KEYS.lastPushAt, ts);
}

function getLastPullAt(): string | null {
  return localStorage.getItem(SYNC_KEYS.lastPullAt);
}

function setLastPullAt(ts: string): void {
  localStorage.setItem(SYNC_KEYS.lastPullAt, ts);
}

function getSyncError(): string | null {
  return localStorage.getItem(SYNC_KEYS.syncError);
}

function setSyncError(err: string | null): void {
  if (err) {
    localStorage.setItem(SYNC_KEYS.syncError, err);
  } else {
    localStorage.removeItem(SYNC_KEYS.syncError);
  }
}

export interface SyncPayload {
  deviceId: string;
  providers: Provider[];
  conversations: LocalConversation[];
  messagesPerConversation: Record<string, LocalMessage[]>;
  deletedProviderIds: string[];
  deletedConversationIds: string[];
  lastPushAt: string | null;
}

export interface SyncResponse {
  providers: Provider[];
  conversations: LocalConversation[];
  messagesPerConversation: Record<string, LocalMessage[]>;
  deletedProviderIds: string[];
  deletedConversationIds: string[];
  serverTimestamp: string;
}

export interface SyncStatus {
  lastPushAt: string | null;
  lastPullAt: string | null;
  localProviderCount: number;
  localConversationCount: number;
  localMessageCount: number;
  lastError: string | null;
}

export function getSyncStatus(): SyncStatus {
  const providers = getLocalProviders();
  const conversations = getLocalConversations();
  let totalMessages = 0;
  for (const c of conversations) {
    totalMessages += c.message_count || getLocalMessages(c.id).length;
  }
  return {
    lastPushAt: getLastPushAt(),
    lastPullAt: getLastPullAt(),
    localProviderCount: providers.length,
    localConversationCount: conversations.length,
    localMessageCount: totalMessages,
    lastError: getSyncError(),
  };
}

function buildSyncPayload(): SyncPayload {
  const providers = getLocalProviders();
  const conversations = getLocalConversations();
  const messagesPerConversation: Record<string, LocalMessage[]> = {};

  console.log('[Sync:BuildPayload] 开始构建同步负载', {
    localProviderCount: providers.length,
    localConversationCount: conversations.length,
    localProviderIds: providers.map(p => p.id),
    localConversationIds: conversations.map(c => c.id),
    localConversationTimestamps: conversations.map(c => ({ id: c.id, updated_at: c.updated_at })),
  });

  let totalMessages = 0;
  for (const c of conversations) {
    const msgs = getLocalMessages(c.id);
    if (msgs.length > 0) {
      messagesPerConversation[c.id] = msgs;
      totalMessages += msgs.length;
      const lastMsg = msgs[msgs.length - 1];
      console.log(`[Sync:BuildPayload] 对话 ${c.id} (${c.title || '无标题'}): ${msgs.length} 条消息, 最新消息时间=${lastMsg.created_at}`);
    } else {
      console.log(`[Sync:BuildPayload] 对话 ${c.id} (${c.title || '无标题'}): 0 条消息 (跳过)`);
    }
  }

  const deletedProviders = getDeletedProviders();
  const deletedConversations = getDeletedConversations();

  console.log('[Sync:BuildPayload] 待删除实体', {
    deletedProviderIds: deletedProviders,
    deletedConversationIds: deletedConversations,
  });

  const payload = {
    deviceId: getDeviceId(),
    providers,
    conversations,
    messagesPerConversation,
    deletedProviderIds: deletedProviders,
    deletedConversationIds: deletedConversations,
    lastPushAt: getLastPushAt(),
  };

  const payloadSize = JSON.stringify(payload).length;
  console.log(`[Sync:BuildPayload] 负载构建完成, 大小=${(payloadSize / 1024).toFixed(1)}KB`, {
    providers: providers.length,
    conversations: conversations.length,
    totalMessages,
    deletedProviders: deletedProviders.length,
    deletedConversations: deletedConversations.length,
  });

  return payload;
}

async function chengduAuthFetch(path: string, options: RequestInit = {}): Promise<Response> {
  const token = getAuthToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  const url = `${CHENGDU_API}${path}`;
  const res = await fetch(url, { ...options, headers });
  if (res.status === 401) {
    throw new Error('认证已过期，请重新登录');
  }
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`同步失败 (HTTP ${res.status}): ${text.slice(0, 200)}`);
  }
  return res;
}

export async function pushToCloud(): Promise<{ ok: boolean; serverTimestamp: string }> {
  const token = getAuthToken();
  if (!token) throw new Error('未登录，无法同步');

  setSyncError(null);
  const deviceId = getDeviceId();
  console.log(`[Sync:Push] ===== 开始推送, deviceId=${deviceId} =====`);

  const payload = buildSyncPayload();

  const totalMessages = Object.values(payload.messagesPerConversation)
    .reduce((sum, msgs) => sum + msgs.length, 0);
  console.log('[Sync:Push] 推送统计:', {
    deviceId: payload.deviceId,
    providers: payload.providers.length,
    conversations: payload.conversations.length,
    totalMessages,
    deletedProviders: payload.deletedProviderIds.length,
    deletedConversations: payload.deletedConversationIds.length,
    lastPushAt: payload.lastPushAt || '(首次同步)',
  });

  const startTime = performance.now();
  const res = await chengduAuthFetch('/data/sync/push', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
  const elapsed = (performance.now() - startTime).toFixed(0);

  const data = await res.json();
  const serverTimestamp = data.serverTimestamp || new Date().toISOString();
  setLastPushAt(serverTimestamp);

  if (data.deletedProviderIds && data.deletedProviderIds.length > 0) {
    console.log(`[Sync:Push] 服务器确认删除 ${data.deletedProviderIds.length} 个Provider:`, data.deletedProviderIds);
    clearDeletedProviders();
  }
  if (data.deletedConversationIds && data.deletedConversationIds.length > 0) {
    console.log(`[Sync:Push] 服务器确认删除 ${data.deletedConversationIds.length} 个对话:`, data.deletedConversationIds);
    clearDeletedConversations();
  }

  console.log(`[Sync:Push] 推送成功, 耗时=${elapsed}ms, 服务器时间=${serverTimestamp}`);
  return { ok: true, serverTimestamp };
}

function mergeByUpdatedAt<T extends { id: string; updated_at?: string; created_at?: string }>(
  local: T[],
  remote: T[],
  entityType: string = 'entity'
): T[] {
  const getTs = (item: T): string => item.updated_at || item.created_at || '';
  const map = new Map<string, T>();
  let localWins = 0;
  let remoteWins = 0;
  let remoteNew = 0;

  for (const item of local) {
    map.set(item.id, item);
  }
  for (const item of remote) {
    const existing = map.get(item.id);
    if (!existing) {
      map.set(item.id, item);
      remoteNew++;
      console.log(`[Sync:Merge] [${entityType}] 新增远端实体: id=${item.id}, ts=${getTs(item)}`);
    } else {
      const existingTs = getTs(existing);
      const itemTs = getTs(item);
      if (itemTs > existingTs) {
        map.set(item.id, item);
        remoteWins++;
        console.log(`[Sync:Merge] [${entityType}] 远端覆盖本地: id=${item.id}, 远端ts=${itemTs} > 本地ts=${existingTs}`);
      } else if (itemTs < existingTs) {
        localWins++;
        console.log(`[Sync:Merge] [${entityType}] 本地保留(更新): id=${item.id}, 本地ts=${existingTs} > 远端ts=${itemTs}`);
      } else {
        console.log(`[Sync:Merge] [${entityType}] 时间戳相同,保留本地: id=${item.id}, ts=${existingTs}`);
      }
    }
  }

  console.log(`[Sync:Merge] [${entityType}] 合并完成: 本地=${local.length}, 远端=${remote.length}, 结果=${map.size} | 远端新增=${remoteNew}, 远端覆盖=${remoteWins}, 本地保留=${localWins}`);

  return Array.from(map.values()).sort(
    (a, b) => new Date(getTs(b)).getTime() - new Date(getTs(a)).getTime()
  );
}

export async function pullFromCloud(): Promise<{
  ok: boolean;
  serverTimestamp: string;
  providersMerged: number;
  conversationsMerged: number;
}> {
  const token = getAuthToken();
  if (!token) throw new Error('未登录，无法同步');

  setSyncError(null);
  const deviceId = getDeviceId();
  const lastPull = getLastPullAt();
  console.log(`[Sync:Pull] ===== 开始拉取, deviceId=${deviceId}, lastPull=${lastPull || '(首次)'} =====`);

  const startTime = performance.now();
  const res = await chengduAuthFetch('/data/sync/pull', {
    method: 'POST',
    body: JSON.stringify({
      deviceId,
      lastPullAt: lastPull,
    }),
  });
  const networkTime = (performance.now() - startTime).toFixed(0);

  const data: SyncResponse = await res.json();
  const serverTimestamp = data.serverTimestamp || new Date().toISOString();

  const totalRemoteMessages = Object.values(data.messagesPerConversation || {})
    .reduce((sum, msgs) => sum + msgs.length, 0);

  console.log(`[Sync:Pull] 拉取完成, 网络耗时=${networkTime}ms`, {
    remoteProviders: data.providers?.length || 0,
    remoteConversations: data.conversations?.length || 0,
    remoteMessages: totalRemoteMessages,
    deletedProviders: data.deletedProviderIds?.length || 0,
    deletedConversations: data.deletedConversationIds?.length || 0,
    serverTimestamp,
  });

  const mergeStart = performance.now();
  let providersMerged = 0;
  let conversationsMerged = 0;

  if (data.providers && data.providers.length > 0) {
    const localProviders = getLocalProviders();
    console.log(`[Sync:Pull] 合并Provider: 本地=${localProviders.length}, 远端=${data.providers.length}`);
    const merged = mergeByUpdatedAt(localProviders, data.providers, 'Provider');
    saveLocalProviders(merged);
    providersMerged = merged.length;
  }

  if (data.conversations && data.conversations.length > 0) {
    const localConvs = getLocalConversations();
    console.log(`[Sync:Pull] 合并Conversation: 本地=${localConvs.length}, 远端=${data.conversations.length}`);
    const merged = mergeByUpdatedAt(localConvs, data.conversations, 'Conversation');
    saveLocalConversations(merged);
    conversationsMerged = merged.length;
  }

  if (data.messagesPerConversation) {
    const convIds = Object.keys(data.messagesPerConversation);
    console.log(`[Sync:Pull] 合并Message: ${convIds.length} 个对话`);
    for (const [convId, remoteMessages] of Object.entries(data.messagesPerConversation)) {
      const localMessages = getLocalMessages(convId);
      console.log(`[Sync:Pull]   对话 ${convId}: 本地=${localMessages.length} 条, 远端=${remoteMessages.length} 条`);
      const merged = mergeByUpdatedAt(localMessages, remoteMessages, `Message[${convId.slice(0, 8)}]`);
      saveLocalMessages(convId, merged);
    }
  }

  if (data.deletedProviderIds && data.deletedProviderIds.length > 0) {
    for (const id of data.deletedProviderIds) {
      deleteLocalProvider(id);
    }
  }

  if (data.deletedConversationIds && data.deletedConversationIds.length > 0) {
    for (const id of data.deletedConversationIds) {
      deleteLocalConversation(id);
    }
  }

  setLastPullAt(serverTimestamp);

  const mergeTime = (performance.now() - mergeStart).toFixed(0);
  console.log(`[Sync:Pull] 拉取完成, 合并耗时=${mergeTime}ms`, { providersMerged, conversationsMerged });
  return { ok: true, serverTimestamp, providersMerged, conversationsMerged };
}

export async function fullSync(): Promise<{
  ok: boolean;
  pushResult: { serverTimestamp: string };
  pullResult: { serverTimestamp: string; providersMerged: number; conversationsMerged: number };
}> {
  const token = getAuthToken();
  if (!token) throw new Error('未登录，无法同步');

  setSyncError(null);

  const totalStart = performance.now();
  console.log('[Sync:FullSync] ===== 开始全量同步 (push → pull) =====');

  const pushResult = await pushToCloud();
  const pullResult = await pullFromCloud();

  const totalTime = (performance.now() - totalStart).toFixed(0);
  const localData = getSyncStatus();
  console.log(`[Sync:FullSync] 全量同步完成, 总耗时=${totalTime}ms`, {
    pushTime: pushResult.serverTimestamp,
    pullTime: pullResult.serverTimestamp,
    providersMerged: pullResult.providersMerged,
    conversationsMerged: pullResult.conversationsMerged,
    localProviders: localData.localProviderCount,
    localConversations: localData.localConversationCount,
    localMessages: localData.localMessageCount,
  });

  return { ok: true, pushResult, pullResult };
}

export function trackProviderDeletion(id: string): void {
  addDeletedProvider(id);
  console.log(`[Sync:Track] 记录Provider删除: ${id}`);
}

export function trackConversationDeletion(id: string): void {
  addDeletedConversation(id);
  console.log(`[Sync:Track] 记录对话删除: ${id}`);
}

export function clearSyncState(): void {
  console.log('[Sync:Clear] 清除所有同步状态');
  const keys = [SYNC_KEYS.lastPushAt, SYNC_KEYS.lastPullAt, SYNC_KEYS.syncError];
  for (const key of keys) {
    if (localStorage.getItem(key)) {
      console.log(`[Sync:Clear] 清除: ${key}`);
    }
  }
  localStorage.removeItem(SYNC_KEYS.lastPushAt);
  localStorage.removeItem(SYNC_KEYS.lastPullAt);
  localStorage.removeItem(SYNC_KEYS.syncError);
  clearDeletedProviders();
  clearDeletedConversations();
}