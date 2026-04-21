const API_BASE = 'http://127.0.0.1:30080/api';
const GATEWAY_BASE = 'http://127.0.0.1:30090';
const CHENGDU_API = 'http://127.0.0.1:30090/api';
const isTauriApp = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

// 获取存储的 token
function getToken() {
  return localStorage.getItem('auth_token');
}

// Resolve effective user_mode for a given conversation. If the user has explicitly
// opted to use a cross-mode model in this conversation (via the cross-mode warning
// modal in MainContent), the per-conv override takes precedence over the global
// user_mode. This is what makes "keep using clawparrot opus while in selfhosted
// mode" work — only that one conv switches its endpoint, the rest stay in the
// global mode.
function getUserModeForConversation(conversationId?: string): string {
  if (conversationId) {
    try {
      const raw = localStorage.getItem('cross_mode_overrides');
      if (raw) {
        const map = JSON.parse(raw);
        if (map[conversationId]) return map[conversationId];
      }
    } catch {}
  }
  return localStorage.getItem('user_mode') || 'clawparrot';
}

// Resolve env_token / env_base_url to send to bridge. clawparrot mode must ignore
// CUSTOM_API_KEY/CUSTOM_BASE_URL — those exist only because an old version of the
// app let clawparrot users paste their own relay API key; the UI was removed but
// the localStorage values stick around, and if we fall back to them the user
// silently keeps hitting their old personal relay instead of the clawparrot
// gateway. selfhosted mode still prefers CUSTOM_* since self-deploy users legitimately
// need to bring their own key.
function resolveEnvCreds(mode: string): { env_token?: string; env_base_url?: string } {
  if (mode === 'clawparrot') {
    return {
      env_token: localStorage.getItem('ANTHROPIC_API_KEY') || undefined,
      env_base_url: localStorage.getItem('ANTHROPIC_BASE_URL') || undefined,
    };
  }
  return {
    env_token: localStorage.getItem('CUSTOM_API_KEY') || localStorage.getItem('ANTHROPIC_API_KEY') || undefined,
    env_base_url: localStorage.getItem('CUSTOM_BASE_URL') || localStorage.getItem('ANTHROPIC_BASE_URL') || undefined,
  };
}

// 通用请求方法
async function request(path: string, options: RequestInit = {}) {
  const token = getToken();
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...(options.headers || {}),
  };
  if (token) {
    (headers as any)['Authorization'] = `Bearer ${token}`;
  }
  const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
  if (res.status === 401) {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('user');
    window.location.hash = '#/login'; window.location.reload();
    throw new Error('认证失效');
  }
  if (!res.ok) {
    const errorData = await res.json().catch(() => ({}));
    throw new Error(errorData.error || `Request failed: ${res.status}`);
  }
  return res;
}

// 系统状态（检测 git-bash 等运行时依赖）
export async function getSystemStatus(): Promise<{
  platform: string;
  gitBash: { required: boolean; found: boolean; path: string | null };
}> {
  const res = await fetch(`${API_BASE}/system-status`);
  if (!res.ok) throw new Error('Failed to get system status');
  return res.json();
}

// 认证相关
export async function sendCode(email: string) {
  const res = await request('/auth/send-code', {
    method: 'POST',
    body: JSON.stringify({ email }),
  });
  return res.json();
}

export async function register(email: string, password: string, nickname: string, code: string) {
  const res = await request('/auth/register', {
    method: 'POST',
    body: JSON.stringify({ email, password, nickname, code }),
  });
  return res.json();
}

export async function login(email: string, password: string) {
  const res = await request('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
  return res.json();
}

// Gateway login for Electron app — authenticates via local API proxy, returns API key for Claude Code SDK
export async function gatewayLogin(email: string, password: string) {
  const res = await fetch(`${GATEWAY_BASE}/api/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error || '登录失败');
  }
  const data = await res.json();
  if (data.token) {
    localStorage.setItem('ANTHROPIC_API_KEY', data.token);
    localStorage.setItem('ANTHROPIC_BASE_URL', GATEWAY_BASE);
    localStorage.setItem('gateway_user', JSON.stringify(data.user || {}));
    if (data.apiKey) {
      localStorage.setItem('CUSTOM_API_KEY', data.apiKey);
    }
    localStorage.setItem('auth_token', data.token);
    if (data.user) {
      localStorage.setItem('user', JSON.stringify(data.user));
    }
  }
  return data;
}

// Check if user is logged in via gateway
export function isGatewayLoggedIn(): boolean {
  return !!(localStorage.getItem('ANTHROPIC_API_KEY') && localStorage.getItem('gateway_user'));
}

// Gateway logout
export function gatewayLogout() {
  localStorage.removeItem('ANTHROPIC_API_KEY');
  localStorage.removeItem('ANTHROPIC_BASE_URL');
  localStorage.removeItem('gateway_user');
  localStorage.removeItem('gateway_quota');
}

// Get gateway usage status
export async function getGatewayUsage() {
  const key = localStorage.getItem('ANTHROPIC_API_KEY');
  if (!key) return null;
  const res = await fetch(`${GATEWAY_BASE}/gateway/usage`, {
    headers: { 'x-api-key': key },
  });
  if (!res.ok) return null;
  return res.json();
}

export async function forgotPassword(email: string) {
  const res = await request('/auth/forgot-password', {
    method: 'POST',
    body: JSON.stringify({ email }),
  });
  return res.json();
}

export async function resetPassword(email: string, code: string, password: string) {
  const res = await request('/auth/reset-password', {
    method: 'POST',
    body: JSON.stringify({ email, code, password }),
  });
  return res.json();
}

export function logout() {
  localStorage.removeItem('auth_token');
  localStorage.removeItem('user');
  // Also clear gateway credentials (Electron app)
  localStorage.removeItem('ANTHROPIC_API_KEY');
  localStorage.removeItem('ANTHROPIC_BASE_URL');
  localStorage.removeItem('gateway_user');
  localStorage.removeItem('gateway_quota');
  window.location.hash = '#/login'; window.location.reload();
}

export function getUser() {
  const userStr = localStorage.getItem('user');
  return userStr ? JSON.parse(userStr) : null;
}

// Helper: call Chengdu backend with stored JWT
async function chengduRequest(path: string, options?: RequestInit) {
  const token = localStorage.getItem('auth_token');
  const headers: Record<string, string> = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;
  if (options?.method && options.method !== 'GET') headers['Content-Type'] = 'application/json';
  const url = `${CHENGDU_API}${path}`;
  console.log('[chengduRequest]', url);
  const res = await fetch(url, { ...options, headers: { ...headers, ...(options?.headers as Record<string, string> || {}) } });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    console.error('[chengduRequest] Failed:', res.status, text.slice(0, 200));
    throw new Error(`Chengdu ${path} failed: ${res.status}`);
  }
  return res.json();
}

export async function getUserProfile() {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try {
      const data = await chengduRequest('/user/profile');
      // Update local cache
      if (data.user || data) {
        const user = data.user || data;
        localStorage.setItem('user', JSON.stringify(user));
      }
      return data;
    } catch (e) {
      // Fallback to cached
      const userStr = localStorage.getItem('user');
      return { user: userStr ? JSON.parse(userStr) : {} };
    }
  }
  const userStr = localStorage.getItem('user');
  return { user: userStr ? JSON.parse(userStr) : {} };
}

export async function updateUserProfile(data: Record<string, any>) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    const token = localStorage.getItem('auth_token');
    const res = await fetch(`${CHENGDU_API}/user/profile`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify(data),
    });
    const result = await res.json();
    const userStr = localStorage.getItem('user');
    const user = userStr ? JSON.parse(userStr) : {};
    localStorage.setItem('user', JSON.stringify({ ...user, ...result }));
    return result;
  }
  const userStr = localStorage.getItem('user');
  const user = userStr ? JSON.parse(userStr) : {};
  const updated = { ...user, ...data };
  localStorage.setItem('user', JSON.stringify(updated));
  return updated;
}

export async function getUserUsage() {
  let usage: any = null;

  // Get plan info from Chengdu backend (requires auth_token from session-based login)
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try {
      usage = await chengduRequest('/user/usage');
    } catch (_) {}
  }

  // In Electron mode, overlay gateway usage (the real usage data) onto Chengdu's plan info
  if (isTauriApp) {
    try {
      const gwUsage = await getGatewayUsage();
      if (gwUsage) {
        if (usage && usage.quota) {
          // Both sources available: combine
          if (usage.quota.window) {
            usage.quota.window.used = (usage.quota.window.used || 0) + (gwUsage.window_used || 0);
          }
          if (usage.quota.week) {
            usage.quota.week.used = (usage.quota.week.used || 0) + (gwUsage.week_used || 0);
          }
        } else if (!usage) {
          // No Chengdu auth_token — use gateway usage as primary source.
          // SG gateway's /gateway/usage calls Chengdu internal /user/:id/summary,
          // so it has the real plan+quota data even without a session cookie.
          usage = gwUsage;
        }
      }
    } catch (_) {}
  }

  if (usage) return usage;

  // selfhosted mode (no gateway, no Chengdu) — unlimited placeholder
  return {
    plan: { id: 999, name: 'Self-hosted', status: 'active', price: 0 },
    token_quota: 0,
    token_remaining: 0,
    used: 0,
    reset_date: '',
    is_unlimited: true
  };
}

export async function getUnreadAnnouncements() {
  const res = await request('/user/announcements');
  return res.json();
}

export async function markAnnouncementRead(id: number) {
  const res = await request(`/user/announcements/${id}/read`, {
    method: 'POST',
  });
  return res.json();
}

export async function getUserModels() {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try { return await chengduRequest('/user/models'); } catch (_) {}
  }
  try {
    const res = await request('/user/models');
    return res.json();
  } catch (_) {
    return { all: [] };
  }
}

export async function getSessions() {
  const res = await request('/user/sessions');
  return res.json();
}

export async function deleteSession(id: string) {
  const res = await request(`/user/sessions/${id}`, { method: 'DELETE' });
  return res.json();
}

export async function logoutOtherSessions() {
  const res = await request('/user/sessions/logout-others', { method: 'POST' });
  return res.json();
}

export async function changePassword(currentPassword: string, newPassword: string) {
  const res = await request('/user/change-password', {
    method: 'POST',
    body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
  });
  return res.json();
}

export async function deleteAccount(password: string) {
  const res = await request('/user/delete-account', {
    method: 'POST',
    body: JSON.stringify({ password }),
  });
  return res.json();
}

// 套餐与支付
export async function getPlans() {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try { return await chengduRequest('/payment/plans'); } catch (_) {}
  }
  const res = await request('/payment/plans');
  return res.json();
}

export async function createPaymentOrder(planId: number, paymentMethod: string) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    const token = localStorage.getItem('auth_token');
    const res = await fetch(`${CHENGDU_API}/payment/create`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ plan_id: planId, payment_method: paymentMethod }),
    });
    return res.json();
  }
  const res = await request('/payment/create', {
    method: 'POST',
    body: JSON.stringify({ plan_id: planId, payment_method: paymentMethod }),
  });
  return res.json();
}

export async function getPaymentStatus(orderId: string) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try { return await chengduRequest(`/payment/status/${orderId}`); } catch (_) {}
  }
  const res = await request(`/payment/status/${orderId}`);
  return res.json();
}

// 兑换码
export async function redeemCode(code: string) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    const token = localStorage.getItem('auth_token');
    const res = await fetch(`${CHENGDU_API}/redemption/redeem`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ code }),
    });
    return res.json();
  }
  const res = await request('/redemption/redeem', {
    method: 'POST',
    body: JSON.stringify({ code }),
  });
  return res.json();
}

// ═══ Projects ═══

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

export async function getProjects(): Promise<Project[]> {
  const res = await request('/projects');
  return res.json();
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

// 对话相关
export async function getConversations() {
  const res = await request('/conversations');
  return res.json();
}

export async function getUserArtifacts() {
  const res = await request('/artifacts');
  return res.json();
}

export async function getArtifactContent(filePath: string) {
  const res = await request('/artifacts/content?path=' + encodeURIComponent(filePath));
  return res.json();
}

export async function createConversation(title?: string, model?: string, extras?: { research_mode?: boolean }) {
  const body: any = { model };
  if (title !== undefined) {
    body.title = title;
  }
  if (extras?.research_mode !== undefined) {
    body.research_mode = extras.research_mode;
  }
  const res = await request('/conversations', {
    method: 'POST',
    body: JSON.stringify(body),
  });
  return res.json();
}

export async function getConversation(id: string) {
  const res = await request(`/conversations/${id}`);
  return res.json();
}

export async function exportConversation(id: string): Promise<void> {
  const token = getToken();

  // Desktop (Electron) Logic
  if (typeof window !== 'undefined' && (window as any).electronAPI) {
    try {
      const conv = await getConversation(id);

      // Build a simple markdown snapshot
      const lines = [`# ${conv.title || 'Conversation Snapshot'}\n`];
      if (conv.messages && conv.messages.length > 0) {
        conv.messages.forEach((m: any) => {
          lines.push(`## ${m.role === 'user' ? '用户 (User)' : '助手 (Assistant)'} - ${new Date(m.created_at).toLocaleString()}`);
          lines.push(`${m.content}\n`);
          if (m.toolCalls && m.toolCalls.length > 0) {
            lines.push(`> [Tool Executions] ${m.toolCalls.map((tc: any) => tc.name).join(', ')}\n`);
          }
        });
      }

      const contextMarkdown = lines.join('\n');
      const defaultFilename = `conversation-${id.slice(0, 8)}.zip`;

      const result = await (window as any).electronAPI.exportWorkspace(id, contextMarkdown, defaultFilename);

      if (result && !result.success && result.reason !== 'canceled') {
        throw new Error("Local Export Failed");
      }
      return;
    } catch (err: any) {
      console.warn("Electron native export failed:", err);
      throw new Error(err.message || "工作空间生成导致导出失败");
    }
  }

  // Web Fallback Logic
  const res = await fetch(`${API_BASE}/conversations/${id}/export`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });
  if (res.status === 401) {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('user');
    window.location.hash = '#/login'; window.location.reload();
    throw new Error('认证失效');
  }
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error((err as any).error || '导出失败');
  }
  const blob = await res.blob();
  const disposition = res.headers.get('content-disposition') || '';
  const utf8Match = disposition.match(/filename\*=UTF-8''([^;]+)/i);
  const plainMatch = disposition.match(/filename="?([^"]+)"?/i);
  const filename = utf8Match
    ? decodeURIComponent(utf8Match[1])
    : (plainMatch ? plainMatch[1] : `conversation-${id.slice(0, 8)}.zip`);
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export async function deleteConversation(id: string) {
  // 先广播删除开始，通知前端中止该会话的流式输出，避免“串流到别的会话”
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('conversationDeleting', { detail: { id } }));
  }

  // 最佳努力：先请求后端停止生成（即使失败也不阻塞删除）
  try {
    await request(`/conversations/${id}/stop-generation`, { method: 'POST' });
  } catch { }

  try {
    const res = await request(`/conversations/${id}`, { method: 'DELETE' });
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('conversationDeleted', { detail: { id } }));
    }
    return res.json();
  } catch (err) {
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('conversationDeleteFailed', { detail: { id } }));
    }
    throw err;
  }
}

export async function updateConversation(id: string, data: any) {
  const res = await request(`/conversations/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
  return res.json();
}


// 查询对话的活跃生成状态
export async function getGenerationStatus(conversationId: string) {
  const res = await request(`/conversations/${conversationId}/generation-status`);
  return res.json();
}

// 主动停止后台生成
export async function stopGeneration(conversationId: string) {
  const res = await request(`/conversations/${conversationId}/stop-generation`, { method: 'POST' });
  return res.json();
}

// 获取对话上下文大小
export async function getContextSize(conversationId: string): Promise<{ tokens: number; limit: number }> {
  const res = await request(`/conversations/${conversationId}/context-size`);
  return res.json();
}

// 手动压缩对话 — delegates to engine's /compact command
export async function compactConversation(
  id: string,
  instruction?: string
): Promise<{ summary: string; tokensSaved: number; messagesCompacted: number }> {
  const res = await request(`/conversations/${id}/compact`, {
    method: 'POST',
    body: JSON.stringify({
      instruction,
      ...resolveEnvCreds(getUserModeForConversation(id)),
    }),
  });
  return res.json();
}

// 回答 AskUserQuestion — write control_response to engine stdin
export async function answerUserQuestion(
  conversationId: string,
  requestId: string,
  toolUseId: string,
  answers: Record<string, string>
): Promise<{ ok: boolean }> {
  const res = await request(`/conversations/${conversationId}/answer`, {
    method: 'POST',
    body: JSON.stringify({ request_id: requestId, tool_use_id: toolUseId, answers }),
  });
  return res.json();
}

// Pre-warm engine for a conversation (spawn in background before user sends first message)
export function warmEngine(conversationId: string): void {
  const userMode = getUserModeForConversation(conversationId);
  let userProfile: any;
  try {
    const p = JSON.parse(localStorage.getItem('user_profile') || localStorage.getItem('user') || '{}');
    const wf = p.work_function; const pp = p.personal_preferences;
    userProfile = (wf || pp) ? { work_function: wf, personal_preferences: pp } : undefined;
  } catch { userProfile = undefined; }
  // Fire-and-forget — don't block UI
  request(`/conversations/${conversationId}/warm`, {
    method: 'POST',
    body: JSON.stringify({
      ...resolveEnvCreds(userMode),
      user_mode: userMode,
      user_profile: userProfile,
    }),
  }).catch(() => {}); // ignore errors
}

// ===== Provider Management =====
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
  const res = await fetch(`${API_BASE}/providers/${id}/test-websearch`, { method: 'POST' });
  if (!res.ok) return { ok: false, reason: 'HTTP ' + res.status };
  return res.json();
}

export async function getProviders(): Promise<Provider[]> {
  const res = await fetch(`${API_BASE}/providers`);
  return res.json();
}
export async function createProvider(p: Partial<Provider>): Promise<Provider> {
  const res = await fetch(`${API_BASE}/providers`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(p) });
  return res.json();
}
export async function updateProvider(id: string, p: Partial<Provider>): Promise<Provider> {
  const res = await fetch(`${API_BASE}/providers/${id}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(p) });
  return res.json();
}
export async function deleteProvider(id: string): Promise<void> {
  await fetch(`${API_BASE}/providers/${id}`, { method: 'DELETE' });
}
export async function getProviderModels(): Promise<Array<{ id: string; name: string; providerId: string; providerName: string }>> {
  const res = await fetch(`${API_BASE}/providers/models`);
  return res.json();
}

// Check if a conversation has an active engine stream
export async function getStreamStatus(conversationId: string): Promise<{ active: boolean; eventCount: number }> {
  const res = await request(`/conversations/${conversationId}/stream-status`);
  return res.json();
}

// Reconnect to an active stream — receives buffered + live SSE events
export function reconnectStream(
  conversationId: string,
  onDelta: (delta: string, full: string) => void,
  onDone: (full: string) => void,
  onError: (err: string) => void,
  onThinking?: (thinking: string, full: string) => void,
  onSystem?: (event: string, message: string, data: any) => void,
  onToolUse?: (event: { type: 'start' | 'input' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: any; content?: string; is_error?: boolean; textBefore?: string }) => void,
  signal?: AbortSignal
): void {
  let fullText = '';
  let thinkingText = '';

  fetch(`${API_BASE}/conversations/${conversationId}/reconnect`, { signal })
    .then(async (res) => {
      if (!res.ok || !res.body) { onError('Reconnect failed'); return; }
      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (!line.startsWith('data: ')) continue;
          const data = line.slice(6);
          if (data.trim() === '[DONE]') { onDone(fullText); return; }

          try {
            const parsed = JSON.parse(data);

            if (parsed.type === 'content_block_delta' && parsed.delta) {
              if (parsed.delta.type === 'text_delta' && parsed.delta.text) {
                fullText += parsed.delta.text;
                onDelta(parsed.delta.text, fullText);
              }
              if (parsed.delta.type === 'thinking_delta' && parsed.delta.thinking && onThinking) {
                thinkingText += parsed.delta.thinking;
                onThinking(parsed.delta.thinking, thinkingText);
              }
            }
            if (parsed.type === 'tool_use_start' && onToolUse) {
              onToolUse({ type: 'start', tool_use_id: parsed.tool_use_id, tool_name: parsed.tool_name, tool_input: parsed.tool_input, textBefore: parsed.textBefore || '' });
            }
            if (parsed.type === 'tool_use_input' && onToolUse) {
              onToolUse({ type: 'input', tool_use_id: parsed.tool_use_id, tool_input: parsed.tool_input });
            }
            if (parsed.type === 'tool_use_done' && onToolUse) {
              onToolUse({ type: 'done', tool_use_id: parsed.tool_use_id, content: parsed.content, is_error: parsed.is_error });
            }
            if (parsed.type === 'ask_user' && onSystem) {
              onSystem('ask_user', '', parsed);
            }
            if (parsed.type === 'task_event' && onSystem) {
              onSystem('task_event', '', parsed);
            }
            if (parsed.type === 'compact_boundary' && onSystem) {
              onSystem('compact_boundary', '', parsed);
            }
            // Research mode events on reconnect path
            if (parsed.type && parsed.type.startsWith('research_') && onSystem) {
              onSystem(parsed.type, '', parsed);
              if (parsed.type === 'research_report_delta' && parsed.text) {
                fullText += parsed.text;
                onDelta(parsed.text, fullText);
              }
            }
            if (parsed.type === 'message_stop') {
              if (fullText) { onDone(fullText); return; }
            }
            if (parsed.type === 'error') {
              onError(parsed.error || 'Stream error');
              return;
            }
          } catch (_) {}
        }
      }
    })
    .catch((err) => {
      if (err.name !== 'AbortError') onError(err.message || 'Reconnect failed');
    });
}

// 删除指定消息及其后续消息
export async function deleteMessagesFrom(
  conversationId: string,
  messageId: string,
  preserveAttachmentIds?: string[]
) {
  const res = await request(`/conversations/${conversationId}/messages/${messageId}`, {
    method: 'DELETE',
    body: preserveAttachmentIds && preserveAttachmentIds.length > 0
      ? JSON.stringify({ preserve_attachment_ids: preserveAttachmentIds })
      : undefined,
  });
  return res.json();
}

// 删除对话末尾 N 条消息（编辑时 msg.id 不可用的回退方案）
export async function deleteMessagesTail(
  conversationId: string,
  count: number,
  preserveAttachmentIds?: string[]
) {
  const res = await request(`/conversations/${conversationId}/messages-tail/${count}`, {
    method: 'DELETE',
    body: preserveAttachmentIds && preserveAttachmentIds.length > 0
      ? JSON.stringify({ preserve_attachment_ids: preserveAttachmentIds })
      : undefined,
  });
  return res.json();
}

// 文件上传相关
export interface UploadResult {
  fileId: string;
  fileName: string;
  fileType: 'image' | 'document' | 'text';
  mimeType: string;
  size: number;
}

export function uploadFile(
  file: File,
  onProgress?: (percent: number) => void,
  conversationId?: string
): Promise<UploadResult> {
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

    xhr.addEventListener('error', () => reject(new Error('网络错误')));
    xhr.addEventListener('abort', () => reject(new Error('上传已取消')));

    xhr.open('POST', `${API_BASE}/upload`);
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

// Skills 相关
export async function getSkills() {
  const res = await request('/skills');
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

// GitHub Connector
export async function getGithubStatus() {
  const res = await fetch(`${API_BASE}/github/status`);
  return res.json();
}

export async function getGithubAuthUrl() {
  const res = await fetch(`${API_BASE}/github/auth-url`);
  return res.json();
}

export async function disconnectGithub() {
  const res = await fetch(`${API_BASE}/github/disconnect`, { method: 'POST' });
  return res.json();
}

export async function getGithubRepos(page = 1) {
  const res = await fetch(`${API_BASE}/github/repos?page=${page}`);
  return res.json();
}

export async function getGithubTree(owner: string, repo: string, ref = '') {
  const qs = ref ? `?ref=${encodeURIComponent(ref)}` : '';
  const res = await fetch(`${API_BASE}/github/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/tree${qs}`);
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: 'Failed to fetch tree' }));
    throw new Error(err.error || 'Failed to fetch tree');
  }
  return res.json();
}

export async function getGithubContents(owner: string, repo: string, path = '', ref = '') {
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

// 流式对话（核心）
export async function sendMessage(
  conversationId: string,
  message: string,
  attachments: any[] | null,
  onDelta: (delta: string, full: string) => void,
  onDone: (full: string) => void,
  onError: (err: string) => void,
  onThinking?: (thinking: string, full: string) => void,
  onSystem?: (event: string, message: string, data: any) => void,
  onCitations?: (citations: Array<{ url: string; title: string; cited_text?: string }>, query?: string, tokens?: number) => void,
  onDocument?: (document: { id: string; title: string; filename: string; url: string; content?: string; format?: 'markdown' | 'docx' | 'pptx'; slides?: Array<{ title: string; content: string; notes?: string }> }) => void,
  onDocumentDraft?: (draft: { draft_id: string; title?: string; format?: string; preview?: string; preview_available?: boolean; done?: boolean; document?: any }) => void,
  onCodeExecution?: (data: { type: string; executionId: string; code?: string; language?: string; files?: Array<{ id: string; name: string }>; stdout?: string; stderr?: string; images?: string[]; error?: string | null }) => void,
  onToolUse?: (event: { type: 'start' | 'input' | 'done'; tool_use_id: string; tool_name?: string; tool_input?: any; content?: string; is_error?: boolean; textBefore?: string }) => void,
  signal?: AbortSignal
) {
  const token = getToken();
  let fullText = '';
  try {
    const res = await fetch(`${API_BASE}/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({
        conversation_id: conversationId,
        message,
        attachments: attachments || undefined,
        ...resolveEnvCreds(getUserModeForConversation(conversationId)),
        user_mode: getUserModeForConversation(conversationId),
        user_profile: (() => {
          try {
            const p = JSON.parse(localStorage.getItem('user_profile') || localStorage.getItem('user') || '{}');
            const wf = p.work_function;
            const pp = p.personal_preferences;
            return (wf || pp) ? { work_function: wf, personal_preferences: pp } : undefined;
          } catch { return undefined; }
        })(),
      }),
      signal,
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: '请求失败' }));
      onError(err.error || '请求失败');
      return;
    }

    if (!res.body) return;

    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    let thinkingText = '';
    let pendingTextDelta = '';
    let pendingThinkingDelta = '';
    let flushScheduled = false;
    const INLINE_ARTIFACT_OPEN = '<cp_artifact';
    const INLINE_ARTIFACT_CLOSE = '</cp_artifact>';
    let inlineArtifactBuffer = '';
    let inlineArtifactSeq = 0;
    let activeInlineArtifact: null | {
      draft_id: string;
      title: string;
      format: string;
      preview: string;
    } = null;

    const flushPending = () => {
      flushScheduled = false;
      if (pendingThinkingDelta && onThinking) {
        const delta = pendingThinkingDelta;
        pendingThinkingDelta = '';
        onThinking(delta, thinkingText);
      }
      if (pendingTextDelta) {
        const delta = pendingTextDelta;
        pendingTextDelta = '';
        onDelta(delta, fullText);
      }
    };

    const scheduleFlush = () => {
      if (flushScheduled) return;
      flushScheduled = true;
      if (typeof window !== 'undefined' && typeof window.requestAnimationFrame === 'function') {
        window.requestAnimationFrame(() => flushPending());
      } else {
        setTimeout(flushPending, 16);
      }
    };

    const appendVisibleText = (text: string) => {
      if (!text) return;
      fullText += text;
      pendingTextDelta += text;
      scheduleFlush();
    };

    const emitInlineArtifactDraft = (done = false) => {
      if (!activeInlineArtifact || !onDocumentDraft) return;
      onDocumentDraft({
        draft_id: activeInlineArtifact.draft_id,
        title: activeInlineArtifact.title,
        format: activeInlineArtifact.format,
        preview: activeInlineArtifact.preview,
        preview_available: activeInlineArtifact.preview.length > 0,
        done,
      });
    };

    const appendInlineArtifactPreview = (text: string) => {
      if (!text || !activeInlineArtifact) return;
      activeInlineArtifact.preview += text;
      emitInlineArtifactDraft(false);
    };

    const parseInlineArtifactAttrs = (tagText: string) => {
      const titleMatch = tagText.match(/title="([^"]*)"/i);
      const formatMatch = tagText.match(/format="([^"]*)"/i);
      return {
        title: (titleMatch?.[1] || '').trim() || 'Untitled document',
        format: (formatMatch?.[1] || 'markdown').trim() || 'markdown',
      };
    };

    const processInlineArtifactText = (chunk: string, flushAll = false) => {
      if (!chunk && !flushAll) return;
      inlineArtifactBuffer += chunk;

      while (inlineArtifactBuffer) {
        if (!activeInlineArtifact) {
          const startIdx = inlineArtifactBuffer.indexOf(INLINE_ARTIFACT_OPEN);
          if (startIdx === -1) {
            if (flushAll) {
              appendVisibleText(inlineArtifactBuffer);
              inlineArtifactBuffer = '';
            } else {
              const keep = Math.min(inlineArtifactBuffer.length, INLINE_ARTIFACT_OPEN.length - 1);
              const emit = inlineArtifactBuffer.slice(0, inlineArtifactBuffer.length - keep);
              if (emit) appendVisibleText(emit);
              inlineArtifactBuffer = inlineArtifactBuffer.slice(inlineArtifactBuffer.length - keep);
            }
            break;
          }

          if (startIdx > 0) {
            appendVisibleText(inlineArtifactBuffer.slice(0, startIdx));
            inlineArtifactBuffer = inlineArtifactBuffer.slice(startIdx);
          }

          const tagEndIdx = inlineArtifactBuffer.indexOf('>');
          if (tagEndIdx === -1) {
            if (flushAll) {
              appendVisibleText(inlineArtifactBuffer);
              inlineArtifactBuffer = '';
            }
            break;
          }

          const tagText = inlineArtifactBuffer.slice(0, tagEndIdx + 1);
          const attrs = parseInlineArtifactAttrs(tagText);
          inlineArtifactSeq += 1;
          activeInlineArtifact = {
            draft_id: `inline-artifact-${inlineArtifactSeq}`,
            title: attrs.title,
            format: attrs.format,
            preview: '',
          };
          emitInlineArtifactDraft(false);
          inlineArtifactBuffer = inlineArtifactBuffer.slice(tagEndIdx + 1);
          continue;
        }

        const closeIdx = inlineArtifactBuffer.indexOf(INLINE_ARTIFACT_CLOSE);
        if (closeIdx === -1) {
          if (flushAll) {
            appendInlineArtifactPreview(inlineArtifactBuffer);
            inlineArtifactBuffer = '';
            emitInlineArtifactDraft(true);
            activeInlineArtifact = null;
          } else {
            const keep = Math.min(inlineArtifactBuffer.length, INLINE_ARTIFACT_CLOSE.length - 1);
            const emit = inlineArtifactBuffer.slice(0, inlineArtifactBuffer.length - keep);
            if (emit) appendInlineArtifactPreview(emit);
            inlineArtifactBuffer = inlineArtifactBuffer.slice(inlineArtifactBuffer.length - keep);
          }
          break;
        }

        if (closeIdx > 0) {
          appendInlineArtifactPreview(inlineArtifactBuffer.slice(0, closeIdx));
        }
        inlineArtifactBuffer = inlineArtifactBuffer.slice(closeIdx + INLINE_ARTIFACT_CLOSE.length);
        emitInlineArtifactDraft(true);
        activeInlineArtifact = null;
      }
    };

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() || ''; // 保留不完整的最后一行

      for (const line of lines) {
        if (!line.startsWith('data: ')) continue;
        const data = line.slice(6);
        if (data.trim() === '[DONE]') {
          processInlineArtifactText('', true);
          flushPending();
          onDone(fullText);
          return;
        }

        try {
          const parsed = JSON.parse(data);

          // 处理 system 事件（如 compaction 通知）
          if (parsed.type === 'system') {
            if (onSystem) {
              onSystem(parsed.event, parsed.message, parsed);
            }
            continue;
          }

          // 处理 status 事件（如搜索状态通知）
          if (parsed.type === 'status') {
            if (onSystem) {
              onSystem('status', parsed.message, parsed);
            }
            continue;
          }

          if (parsed.type === 'thinking_summary' && parsed.summary) {
            if (onSystem) {
              onSystem('thinking_summary', parsed.summary, parsed);
            }
            continue;
          }

          // 处理搜索来源事件
          if (parsed.type === 'search_sources') {
            if (onCitations && Array.isArray(parsed.sources)) {
              onCitations(parsed.sources, parsed.query, parsed.tokens);
            }
            continue;
          }

          // 处理文档创建事件
          if (parsed.type === 'document_created') {
            if (onDocument && parsed.document) {
              onDocument(parsed.document);
            }
            continue;
          }

          // 处理文档更新事件
          if (parsed.type === 'document_updated') {
            if (onDocument && parsed.document) {
              onDocument(parsed.document);
            }
            continue;
          }

          if (parsed.type === 'document_draft') {
            if (onDocumentDraft) {
              onDocumentDraft(parsed);
            }
            continue;
          }

          // 处理代码执行事件
          if (parsed.type === 'code_execution') {
            if (onCodeExecution) {
              onCodeExecution(parsed);
            }
            continue;
          }

          // 处理代码执行结果事件
          if (parsed.type === 'code_result') {
            if (onCodeExecution) {
              onCodeExecution(parsed);
            }
            continue;
          }

          // 处理 thinking 内容
          if (parsed.type === 'content_block_delta' && parsed.delta) {
            if (parsed.delta.type === 'text_delta' && parsed.delta.text) {
              const textChunk = parsed.delta.text;
              // 处理中转 API 将 <thinking> 标签嵌入 text 的情况
              if (textChunk.includes('<thinking>') || textChunk.includes('</thinking>')) {
                const thinkRegex = /<thinking>([\s\S]*?)<\/thinking>/g;
                let match;
                let cleaned = textChunk;
                while ((match = thinkRegex.exec(textChunk)) !== null) {
                  if (onThinking) {
                    thinkingText += match[1];
                    pendingThinkingDelta += match[1];
                    scheduleFlush();
                  }
                }
                cleaned = textChunk.replace(/<thinking>[\s\S]*?<\/thinking>\s*/g, '');
                if (cleaned) {
                  processInlineArtifactText(cleaned);
                }
              } else {
                processInlineArtifactText(textChunk);
              }
            }
            if (parsed.delta.type === 'thinking_delta' && parsed.delta.thinking) {
              thinkingText += parsed.delta.thinking;
              if (onThinking) {
                pendingThinkingDelta += parsed.delta.thinking;
                scheduleFlush();
              }
            }
          }

          // 处理 content_block_start 来识别 thinking block
          if (parsed.type === 'content_block_start' && parsed.content_block) {
            if (parsed.content_block.type === 'thinking' && onThinking) {
              // 新的 thinking block 开始
              thinkingText = '';
            }
          }

          // Handle compact_boundary from engine auto-compact
          if (parsed.type === 'compact_boundary') {
            if (onSystem) {
              onSystem('compact_boundary', '', parsed);
            }
            continue;
          }

          // Handle AskUserQuestion from engine
          if (parsed.type === 'ask_user') {
            if (onSystem) {
              onSystem('ask_user', '', parsed);
            }
            continue;
          }

          // Handle task/agent progress events
          if (parsed.type === 'task_event') {
            if (onSystem) {
              onSystem('task_event', '', parsed);
            }
            continue;
          }

          // Handle tool use events
          if (parsed.type === 'tool_use_start' && onToolUse) {
            onToolUse({ type: 'start', tool_use_id: parsed.tool_use_id, tool_name: parsed.tool_name, tool_input: parsed.tool_input, textBefore: parsed.textBefore || '' });
          }
          if (parsed.type === 'tool_use_input' && onToolUse) {
            onToolUse({ type: 'input', tool_use_id: parsed.tool_use_id, tool_input: parsed.tool_input });
          }
          if (parsed.type === 'tool_use_done' && onToolUse) {
            onToolUse({ type: 'done', tool_use_id: parsed.tool_use_id, content: parsed.content, is_error: parsed.is_error });
          }

          // Research mode events — forward as system events for MainContent to handle
          if (parsed.type && parsed.type.startsWith('research_') && onSystem) {
            onSystem(parsed.type, '', parsed);
            // research_report_delta also feeds into the streaming text so the
            // final report appears as the assistant message body
            if (parsed.type === 'research_report_delta' && parsed.text) {
              fullText += parsed.text;
              onDelta(parsed.text, fullText);
            }
            continue;
          }

          // Track text offset where tool work ends and final response begins
          if (parsed.type === 'tool_text_offset' && onSystem) {
            onSystem('tool_text_offset', '', parsed);
          }

          if (parsed.type === 'message_stop') {
            processInlineArtifactText('', true);
            // 如果有文本内容才结束，否则可能是服务端工具中间的 message_stop
            if (fullText) {
              flushPending();
              onDone(fullText);
              return;
            }
            // 没有文本内容时继续等待后续事件
            continue;
          }

          if (parsed.type === 'error') {
            const detail = parsed.detail ? `\n${parsed.detail}` : '';
            processInlineArtifactText('', true);
            flushPending();
            onError((parsed.error || '未知错误') + detail);
            return;
          }
        } catch (e) {
          // 忽略非JSON行
        }
      }
    }

    processInlineArtifactText('', true);
    if (fullText) {
      flushPending();
      onDone(fullText);
    } else {
      // 无文本回复（如纯工具事件），也要触发完成回调
      flushPending();
      onDone('');
    }
  } catch (err: any) {
    // 用户主动中断不算错误
    if (err.name === 'AbortError') {
      // 主动中断时也先把已积累的内容刷到界面
      onDone(fullText);
      return;
    }
    onError(err.message || 'Network error');
  }
}

// Code API 相关
export async function getCodeSSO() {
  const res = await request('/code/sso');
  return res.json();
}

export async function getCodeQuota() {
  const res = await request('/code/quota');
  return res.json();
}

export async function getCodePlans() {
  const res = await request('/code/plans');
  return res.json();
}
