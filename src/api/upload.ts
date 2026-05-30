import { API_BASE, getToken, detectBridgePort, request } from './client';

export interface UploadResult {
  fileId: string;
  fileName: string;
  fileType: 'image' | 'document' | 'text';
  mimeType: string;
  size: number;
}

export async function uploadFile(
  file: File,
  onProgress?: (percent: number) => void,
  conversationId?: string
): Promise<UploadResult> {
  const port = await detectBridgePort();
  const uploadUrl = `http://127.0.0.1:${port}/api/upload`;

  return new Promise((resolve, reject) => {
    const token = getToken();
    const xhr = new XMLHttpRequest();
    const formData = new FormData();
    formData.append('file', file);

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress(Math.round((e.loaded / e.total) * 100));
      }
    });

    xhr.addEventListener('load', () => {
      if (xhr.status === 401) {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('user');
        window.location.hash = '#/login'; window.location.reload();
        reject(new Error('认证失效'));
        return;
      }
      const raw = xhr.responseText || '';
      let data: any = null;
      if (raw) {
        try {
          data = JSON.parse(raw);
        } catch {
          data = null;
        }
      }

      if (xhr.status >= 200 && xhr.status < 300) {
        if (data) {
          resolve(data);
          return;
        }
        reject(new Error('上传失败：服务器返回异常'));
        return;
      }

      const serverError = data?.error || data?.message;
      const rawError = !data && raw ? raw.slice(0, 120) : '';
      const detail = serverError || rawError || '上传失败';
      reject(new Error(`${detail} (HTTP ${xhr.status})`));
    });

    xhr.addEventListener('error', (err) => {
      console.error('[API] Upload network error:', err);
      reject(new Error(`网络错误，无法连接到 ${uploadUrl}`));
    });
    xhr.addEventListener('abort', () => reject(new Error('上传已取消')));

    xhr.open('POST', uploadUrl);
    if (token) {
      xhr.setRequestHeader('Authorization', `Bearer ${token}`);
    }
    if (conversationId) {
      xhr.setRequestHeader('x-conversation-id', conversationId);
    }
    xhr.send(formData);
  });
}

export async function deleteAttachment(fileId: string): Promise<void> {
  await request(`/uploads/${fileId}`, { method: 'DELETE' });
}

export function getAttachmentUrl(fileId: string): string {
  return `${API_BASE}/uploads/${fileId}/raw`;
}

export async function getUpload(fileId: string) {
  const res = await request(`/uploads/${fileId}`);
  return res.json();
}

export async function deleteUpload(fileId: string) {
  await request(`/uploads/${fileId}`, { method: 'DELETE' });
}
