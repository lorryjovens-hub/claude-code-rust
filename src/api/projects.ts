import { request } from './client';

export interface Project {
  id: string;
  name: string;
  description: string;
  instructions: string;
  workspace_path: string;
  is_archived: number;
  file_count?: number;
  chat_count?: number;
  created_at: string;
  updated_at: string;
}

export interface ProjectFile {
  id: string;
  project_id: string;
  file_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  created_at: string;
}

export async function listProjects(): Promise<Project[]> {
  const res = await request('/projects');
  const data = await res.json();
  return Array.isArray(data) ? data : (Array.isArray(data?.projects) ? data.projects : []);
}

export async function getProjects(): Promise<Project[]> {
  return listProjects();
}

export async function createProject(name: string, description?: string): Promise<Project> {
  const res = await request('/projects', {
    method: 'POST',
    body: JSON.stringify({ name, description: description || '' }),
  });
  return res.json();
}

export async function getProject(id: string) {
  const res = await request(`/projects/${id}`);
  return res.json();
}

export async function updateProject(id: string, data: Partial<Pick<Project, 'name' | 'description' | 'instructions' | 'is_archived'>>) {
  const res = await request(`/projects/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
  return res.json();
}

export async function deleteProject(id: string) {
  const res = await request(`/projects/${id}`, { method: 'DELETE' });
  return res.json();
}

export async function uploadProjectFile(projectId: string, file: File): Promise<ProjectFile> {
  const { waitForApiReady, API_BASE, getToken } = await import('./client');
  await waitForApiReady();
  const formData = new FormData();
  formData.append('file', file);
  const token = getToken();
  const res = await fetch(`${API_BASE}/projects/${projectId}/files`, {
    method: 'POST',
    headers: token ? { 'Authorization': `Bearer ${token}` } : {},
    body: formData,
  });
  if (!res.ok) throw new Error('Upload failed');
  return res.json();
}

export async function deleteProjectFile(projectId: string, fileId: string) {
  const res = await request(`/projects/${projectId}/files/${fileId}`, { method: 'DELETE' });
  return res.json();
}

export async function getProjectConversations(projectId: string) {
  const res = await request(`/projects/${projectId}/conversations`);
  return res.json();
}

export async function createProjectConversation(projectId: string, title?: string, model?: string) {
  const res = await request(`/projects/${projectId}/conversations`, {
    method: 'POST',
    body: JSON.stringify({ title, model }),
  });
  return res.json();
}
