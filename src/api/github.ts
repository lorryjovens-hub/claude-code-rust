import { waitForApiReady, API_BASE, request } from './client';

export async function getGithubStatus() {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/github/status`);
  return res.json();
}

export async function getGithubAuthUrl() {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/github/auth-url`);
  return res.json();
}

export async function disconnectGithub() {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/github/disconnect`, { method: 'POST' });
  return res.json();
}

export async function getGithubRepos(page = 1) {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/github/repos?page=${page}`);
  return res.json();
}

export async function getGithubTree(owner: string, repo: string, ref = '') {
  await waitForApiReady();
  const qs = ref ? `?ref=${encodeURIComponent(ref)}` : '';
  const res = await fetch(`${API_BASE}/github/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/tree${qs}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Failed to fetch tree' }));
    throw new Error(err.error || 'Failed to fetch tree');
  }
  return res.json();
}

export async function getGithubContents(owner: string, repo: string, path = '', ref = '') {
  await waitForApiReady();
  const params = new URLSearchParams();
  if (path) params.set('path', path);
  if (ref) params.set('ref', ref);
  const qs = params.toString();
  const url = `${API_BASE}/github/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/contents${qs ? '?' + qs : ''}`;
  const res = await fetch(url);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Failed to fetch contents' }));
    throw new Error(err.error || 'Failed to fetch contents');
  }
  return res.json();
}

export async function materializeGithub(
  conversationId: string,
  repoFullName: string,
  ref: string,
  selections: Array<{ path: string; isFolder: boolean }>
): Promise<{ ok: boolean; repoFullName: string; ref: string; rootDir: string; fileCount: number; skipped: number }> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/github/materialize`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ conversationId, repoFullName, ref, selections }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Materialize failed' }));
    throw new Error(err.error || 'Materialize failed');
  }
  return res.json();
}
