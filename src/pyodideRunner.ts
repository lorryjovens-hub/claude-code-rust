/**
 * Pyodide Runner — 封装 Web Worker 通信 + /api/code-result 回调
 */

const API_BASE = '/api';

interface CodeResult {
  stdout: string;
  stderr: string;
  images: string[];
  error: string | null;
}

interface PendingExecution {
  resolve: (result: CodeResult) => void;
  reject: (err: Error) => void;
}

let worker: Worker | null = null;
let workerReady = false;
const pendingExecutions = new Map<string, PendingExecution>();
let onStatusChange: ((status: string) => void) | null = null;

function getWorker(): Worker {
  if (worker) return worker;
  worker = new Worker('/pyodide-worker.js');
  worker.onmessage = (e: MessageEvent) => {
    const { type, id, status, stdout, stderr, images, error } = e.data;
    if (type === 'status') {
      workerReady = status !== 'loading';
      if (onStatusChange) {
        onStatusChange(status === 'loading' ? '正在初始化 Python 环境...' : '');
      }
      return;
    }
    if (type === 'result' && id) {
      const pending = pendingExecutions.get(id);
      if (pending) {
        pendingExecutions.delete(id);
        pending.resolve({ stdout: stdout || '', stderr: stderr || '', images: images || [], error: error || null });
      }
    }
  };
  worker.onerror = (e) => {
    console.error('[PyodideRunner] Worker error:', e);
    // Reject all pending
    for (const [id, pending] of pendingExecutions) {
      pending.reject(new Error('Worker 崩溃'));
      pendingExecutions.delete(id);
    }
    worker = null;
    workerReady = false;
  };
  return worker;
}

export function setStatusCallback(cb: (status: string) => void) {
  onStatusChange = cb;
}

export async function executeCode(
  code: string,
  files: Array<{ name: string; url: string }>,
  executionId: string
): Promise<CodeResult> {
  const w = getWorker();

  return new Promise<CodeResult>((resolve, reject) => {
    // 60 秒超时
    const timer = setTimeout(() => {
      pendingExecutions.delete(executionId);
      resolve({ stdout: '', stderr: '代码执行超时（60秒）', images: [], error: 'timeout' });
    }, 60000);

    pendingExecutions.set(executionId, {
      resolve: (result) => {
        clearTimeout(timer);
        resolve(result);
      },
      reject: (err) => {
        clearTimeout(timer);
        reject(err);
      },
    });

    w.postMessage({ type: 'execute', code, files, id: executionId });
  });
}

export async function sendCodeResult(executionId: string, result: CodeResult): Promise<void> {
  const token = localStorage.getItem('auth_token');
  try {
    const resp = await fetch(`${API_BASE}/code-result`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        executionId,
        stdout: result.stdout,
        stderr: result.stderr,
        images: result.images,
        error: result.error,
      }),
    });
    if (!resp.ok) {
      console.error(`[PyodideRunner] Code result upload failed: HTTP ${resp.status}`);
    }
  } catch (e) {
    console.error('[PyodideRunner] Failed to send code result:', e);
  }
}
