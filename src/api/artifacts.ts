import { request } from './client';

export async function getUserArtifacts() {
  const res = await request('/artifacts');
  return res.json();
}

export async function getArtifactContent(filePath: string) {
  const res = await request('/artifacts/content?path=' + encodeURIComponent(filePath));
  return res.json();
}
