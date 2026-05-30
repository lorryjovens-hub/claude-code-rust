import { request } from './client';

export async function getSkills() {
  const res = await request('/skills');
  return res.json();
}

export async function getSkill(id: string) {
  const res = await request(`/skills/${id}`);
  return res.json();
}

export async function getSkillDetail(id: string) {
  const res = await request(`/skills/${id}`);
  return res.json();
}

export async function getSkillFile(id: string, filePath: string) {
  const res = await request(`/skills/${id}/file?path=${encodeURIComponent(filePath)}`);
  return res.json();
}

export async function createSkill(data: { name: string; description?: string; content?: string }) {
  const res = await request('/skills', {
    method: 'POST',
    body: JSON.stringify(data),
  });
  return res.json();
}

export async function updateSkill(id: string, data: { name?: string; description?: string; content?: string }) {
  const res = await request(`/skills/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
  return res.json();
}

export async function deleteSkill(id: string) {
  const res = await request(`/skills/${id}`, { method: 'DELETE' });
  return res.json();
}

export async function toggleSkill(id: string, enabled: boolean) {
  const res = await request(`/skills/${id}/toggle`, {
    method: 'PATCH',
    body: JSON.stringify({ enabled }),
  });
  return res.json();
}

export async function listSkills() {
  const res = await request('/skills');
  return res.json();
}

export async function executeSkill(id: string, input?: any) {
  const res = await request(`/skills/${id}/execute`, {
    method: 'POST',
    body: JSON.stringify({ input }),
  });
  return res.json();
}
