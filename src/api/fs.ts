import { waitForApiReady, API_BASE } from './client';

export interface FsFileNode {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  children?: FsFileNode[];
}

export interface FsTreeResponse {
  success: boolean;
  path: string;
  tree: FsFileNode[];
}

export interface FsReadResponse {
  success: boolean;
  path: string;
  content: string;
}

export async function fsTree(dirPath?: string): Promise<FsTreeResponse> {
  await waitForApiReady();
  const params = dirPath ? `?path=${encodeURIComponent(dirPath)}` : '';
  const res = await fetch(`${API_BASE}/filesystem/tree${params}`);
  if (!res.ok) throw new Error(`Failed to get file tree: ${res.status}`);
  return res.json();
}

export async function fsRead(filePath: string): Promise<FsReadResponse> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/filesystem/read?path=${encodeURIComponent(filePath)}`);
  if (!res.ok) throw new Error(`Failed to read file: ${res.status}`);
  return res.json();
}

export async function fsWrite(filePath: string, content: string): Promise<{ success: boolean; path: string }> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/filesystem/write`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path: filePath, content }),
  });
  if (!res.ok) throw new Error(`Failed to write file: ${res.status}`);
  return res.json();
}

export async function fsCreate(filePath: string, isDir = false): Promise<{ success: boolean; path: string }> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/filesystem/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path: filePath, is_dir: isDir }),
  });
  if (!res.ok) throw new Error(`Failed to create: ${res.status}`);
  return res.json();
}

export async function fsDelete(filePath: string): Promise<{ success: boolean; path: string }> {
  await waitForApiReady();
  const res = await fetch(`${API_BASE}/filesystem/delete`, {
    method: 'DELETE',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path: filePath }),
  });
  if (!res.ok) throw new Error(`Failed to delete: ${res.status}`);
  return res.json();
}
