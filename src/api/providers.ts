import { waitForApiReady, API_BASE, isTauriApp } from './client';
import {
  getLocalProviders,
  createLocalProvider,
  updateLocalProvider,
  deleteLocalProvider,
} from '../services/localStorageService';

export interface ProviderModel { id: string; name: string; enabled?: boolean; }
export interface Provider {
  id: string; name: string; apiKey: string; baseUrl: string;
  format: 'anthropic' | 'openai'; models: ProviderModel[]; enabled: boolean;
  icon?: string;
  supportsWebSearch?: boolean;
  webSearchStrategy?: 'dashscope' | 'bigmodel' | 'anthropic_native' | null;
  webSearchTestedAt?: number;
  webSearchTestReason?: string | null;
}

export interface WebSearchTestResult {
  ok: boolean;
  strategy?: 'dashscope' | 'bigmodel' | 'anthropic_native' | null;
  hitCount?: number;
  reason?: string;
}

export async function testProviderWebSearch(id: string): Promise<WebSearchTestResult> {
  if (!isTauriApp) return { ok: false, reason: 'Web search test requires Tauri bridge' };
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/providers/${id}/test-websearch`, { method: 'POST' });
  if (!res.ok) return { ok: false, reason: 'HTTP ' + res.status };
  return res.json();
}

async function bridgeListProviders(): Promise<Provider[]> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/providers`);
  const data = await res.json();
  return Array.isArray(data) ? data : (Array.isArray(data?.providers) ? data.providers : []);
}

export async function getProviders(): Promise<Provider[]> {
  if (isTauriApp) {
    try { return await bridgeListProviders(); } catch { return []; }
  }
  return getLocalProviders();
}

export async function listProviders(): Promise<Provider[]> {
  return getProviders();
}

export async function createProvider(p: Partial<Provider>): Promise<Provider> {
  if (isTauriApp) {
    await waitForApiReady();
    const res = await fetch(`${API_BASE}/providers`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(p) });
    return res.json();
  }
  return createLocalProvider(p as any);
}

export async function updateProvider(id: string, p: Partial<Provider>): Promise<Provider> {
  if (isTauriApp) {
    await waitForApiReady();
    const res = await fetch(`${API_BASE}/providers/${id}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(p) });
    return res.json();
  }
  return updateLocalProvider(id, p) || (() => { throw new Error('Provider not found'); })();
}

export async function deleteProvider(id: string): Promise<void> {
  if (isTauriApp) {
    await waitForApiReady();
    await fetch(`${API_BASE}/providers/${id}`, { method: 'DELETE' });
    return;
  }
  deleteLocalProvider(id);
}

export async function getProviderModels(): Promise<Array<{ id: string; name: string; providerId: string; providerName: string }>> {
  if (isTauriApp) {
    await waitForApiReady();
    const res = await fetch(`${API_BASE}/providers/models`);
    return res.json();
  }
  const providers = getLocalProviders();
  const models: Array<{ id: string; name: string; providerId: string; providerName: string }> = [];
  for (const p of providers) {
    if (!p.enabled) continue;
    for (const m of (p.models || [])) {
      if (m.enabled === false) continue;
      models.push({ id: m.id, name: m.name || m.id, providerId: p.id, providerName: p.name });
    }
  }
  return models;
}