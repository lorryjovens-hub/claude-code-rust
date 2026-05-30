import { request } from './client';

export async function warmEngine(conversationId: string): Promise<void> {
  const { getUserModeForConversation, resolveEnvCreds } = await import('./client');
  const userMode = getUserModeForConversation(conversationId);
  let userProfile: any;
  try {
    const p = JSON.parse(localStorage.getItem('user_profile') || localStorage.getItem('user') || '{}');
    const wf = p.work_function; const pp = p.personal_preferences;
    userProfile = (wf || pp) ? { work_function: wf, personal_preferences: pp } : undefined;
  } catch { userProfile = undefined; }

  let permissionMode: string | undefined;
  try {
    if (typeof window !== 'undefined' && (window as any).__chatStore) {
      permissionMode = (window as any).__chatStore.getState().permissionMode;
    } else {
      permissionMode = localStorage.getItem('permission_mode') || undefined;
    }
  } catch {}

  request(`/conversations/${conversationId}/warm`, {
    method: 'POST',
    body: JSON.stringify({
      ...resolveEnvCreds(userMode),
      user_mode: userMode,
      user_profile: userProfile,
      permission_mode: permissionMode,
    }),
  }).catch(() => {});
}

export async function listEngines() {
  const res = await request('/engines');
  return res.json();
}

export async function spawnEngine(config?: any) {
  const res = await request('/engines/spawn', {
    method: 'POST',
    body: JSON.stringify(config || {}),
  });
  return res.json();
}

export async function killEngine(id: string) {
  const res = await request(`/engines/${id}/kill`, { method: 'POST' });
  return res.json();
}

export async function listAgents() {
  const res = await request('/agents');
  return res.json();
}

export async function getAgent(id: string) {
  const res = await request(`/agents/${id}`);
  return res.json();
}

export async function cancelAgent(id: string) {
  const res = await request(`/agents/${id}/cancel`, { method: 'POST' });
  return res.json();
}

export async function getIdeStatus() {
  const res = await request('/ide/status');
  return res.json();
}

export async function startIdeServer() {
  const res = await request('/ide/start', { method: 'POST' });
  return res.json();
}

export async function stopIdeServer() {
  const res = await request('/ide/stop', { method: 'POST' });
  return res.json();
}

export async function getIdeConnections() {
  const res = await request('/ide/connections');
  return res.json();
}

export async function disconnectIde(id: string) {
  const res = await request(`/ide/connections/${id}`, { method: 'DELETE' });
  return res.json();
}
