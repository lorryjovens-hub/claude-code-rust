import { request } from './client';

export async function createWorktree(data: { branch_prefix?: string; agent_name?: string; task?: string; model?: string }) {
  const res = await request('/worktrees', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return res.json();
}

export async function listWorktrees() {
  const res = await request('/worktrees');
  return res.json();
}

export async function getWorktree(id: string) {
  const res = await request(`/worktrees/${id}`);
  return res.json();
}

export async function removeWorktree(id: string) {
  const res = await request(`/worktrees/${id}`, { method: 'DELETE' });
  return res.json();
}

export async function mergeWorktree(worktreeId: string, strategy?: string) {
  const res = await request('/worktrees/merge', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ worktree_id: worktreeId, strategy }),
  });
  return res.json();
}

export async function syncWorktrees() {
  const res = await request('/worktrees/sync', { method: 'POST' });
  return res.json();
}
