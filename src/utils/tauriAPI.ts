import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export interface EngineTextEvent {
  conversation_id: string;
  event: { type: 'text'; text: string };
}

export interface EngineThinkingEvent {
  conversation_id: string;
  event: { type: 'thinking'; text: string };
}

export interface EngineToolUseStartEvent {
  conversation_id: string;
  event: { type: 'tool_use_start'; tool_use_id: string; tool_name: string; tool_input: Record<string, unknown> };
}

export interface EngineToolUseDoneEvent {
  conversation_id: string;
  event: { type: 'tool_use_done'; tool_use_id: string; tool_name: string; tool_input: Record<string, unknown>; output: string; is_error: boolean };
}

export interface EngineMessageStartEvent {
  conversation_id: string;
  event: { type: 'message_start'; model: string };
}

export interface EngineMessageDeltaEvent {
  conversation_id: string;
  event: { type: 'message_delta'; stop_reason: string | null };
}

export interface EngineMessageStopEvent {
  conversation_id: string;
  event: { type: 'message_stop' };
}

export interface EngineErrorEvent {
  conversation_id: string;
  event: { type: 'error'; error: string };
}

export interface EngineUsageEvent {
  conversation_id: string;
  event: { type: 'usage'; usage: { input_tokens: number; output_tokens: number; cache_creation_tokens?: number; cache_read_tokens?: number } };
}

export type EngineEvent =
  | EngineTextEvent
  | EngineThinkingEvent
  | EngineToolUseStartEvent
  | EngineToolUseDoneEvent
  | EngineMessageStartEvent
  | EngineMessageDeltaEvent
  | EngineMessageStopEvent
  | EngineErrorEvent
  | EngineUsageEvent;

export interface NativeEngineState {
  initialized: boolean;
  provider_count: number;
  conversation_count: number;
}

export interface NativeChatRequest {
  conversation_id: string;
  messages: Array<{ role: string; content: string }>;
  model: string;
  system_prompt?: string;
  max_tokens?: number;
}

export interface NativeChatResponse {
  conversation_id: string;
  status: string;
}

export interface CreateConversationRequest {
  model: string;
  title?: string;
  research_mode?: boolean;
}

export interface ConversationInfo {
  id: string;
  title: string | null;
  model: string;
  workspace_path: string;
  created_at: string;
  updated_at: string;
}

export interface ToolCallInfo {
  id: string;
  name: string;
  input: Record<string, unknown>;
  output: string | null;
  is_error: boolean | null;
}

export interface MessageInfo {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  created_at: string;
  tool_calls: ToolCallInfo[] | null;
}

export interface ProviderInfo {
  id: string;
  name: string;
  base_url: string;
  api_format: string;
  enabled: boolean;
  models: Array<{ id: string; name: string; enabled: boolean }>;
}

export interface UpdateProviderRequest {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  api_format: string;
  models: Array<{ id: string; name: string; enabled: boolean }>;
  enabled: boolean;
}

export const nativeEngineAPI = {
  async init(): Promise<NativeEngineState> {
    return invoke('native_engine_init');
  },

  async chat(request: NativeChatRequest): Promise<NativeChatResponse> {
    return invoke('native_chat', { request });
  },

  async createConversation(request: CreateConversationRequest): Promise<ConversationInfo> {
    return invoke('native_create_conversation', { request });
  },

  async listConversations(): Promise<ConversationInfo[]> {
    console.log('[TauriAPI] Calling native_list_conversations...');
    const result = await invoke('native_list_conversations');
    console.log('[TauriAPI] native_list_conversations result:', result);
    return result as ConversationInfo[];
  },

  async deleteConversation(conversationId: string): Promise<void> {
    return invoke('native_delete_conversation', { conversationId });
  },

  async getMessages(conversationId: string): Promise<MessageInfo[]> {
    console.log('[TauriAPI] Calling native_get_messages for:', conversationId);
    const result = await invoke('native_get_messages', { conversationId });
    console.log('[TauriAPI] native_get_messages result count:', (result as MessageInfo[]).length);
    return result as MessageInfo[];
  },

  async listProviders(): Promise<ProviderInfo[]> {
    return invoke('native_list_providers');
  },

  async updateProvider(request: UpdateProviderRequest): Promise<void> {
    return invoke('native_update_provider', { request });
  },

  async deleteProvider(id: string): Promise<void> {
    return invoke('native_delete_provider', { id });
  },

  onText(handler: (event: EngineTextEvent) => void): Promise<UnlistenFn> {
    return listen('engine:text', (e) => handler(e.payload as EngineTextEvent));
  },

  onThinking(handler: (event: EngineThinkingEvent) => void): Promise<UnlistenFn> {
    return listen('engine:thinking', (e) => handler(e.payload as EngineThinkingEvent));
  },

  onToolUseStart(handler: (event: EngineToolUseStartEvent) => void): Promise<UnlistenFn> {
    return listen('engine:tool_use_start', (e) => handler(e.payload as EngineToolUseStartEvent));
  },

  onToolUseDone(handler: (event: EngineToolUseDoneEvent) => void): Promise<UnlistenFn> {
    return listen('engine:tool_use_done', (e) => handler(e.payload as EngineToolUseDoneEvent));
  },

  onMessageStart(handler: (event: EngineMessageStartEvent) => void): Promise<UnlistenFn> {
    return listen('engine:message_start', (e) => handler(e.payload as EngineMessageStartEvent));
  },

  onMessageDelta(handler: (event: EngineMessageDeltaEvent) => void): Promise<UnlistenFn> {
    return listen('engine:message_delta', (e) => handler(e.payload as EngineMessageDeltaEvent));
  },

  onMessageStop(handler: (event: EngineMessageStopEvent) => void): Promise<UnlistenFn> {
    return listen('engine:message_stop', (e) => handler(e.payload as EngineMessageStopEvent));
  },

  onError(handler: (event: EngineErrorEvent) => void): Promise<UnlistenFn> {
    return listen('engine:error', (e) => handler(e.payload as EngineErrorEvent));
  },

  onUsage(handler: (event: EngineUsageEvent) => void): Promise<UnlistenFn> {
    return listen('engine:usage', (e) => handler(e.payload as EngineUsageEvent));
  },
};

// ===== Code Diff API =====

export interface CodeDiffInfo {
  id: string;
  conversation_id: string;
  message_id: string;
  file_path: string;
  original_content: string | null;
  modified_content: string | null;
  diff_text: string | null;
  status: string;
  applied_at: string | null;
  created_at: string;
}

export interface DiffHunkInfo {
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: DiffLineInfo[];
}

export interface DiffLineInfo {
  kind: string;
  line_num: number;
  content: string;
}

export interface DiffResultInfo {
  hunks: DiffHunkInfo[];
  additions: number;
  deletions: number;
}

export const diffAPI = {
  async generateDiff(original: string, modified: string): Promise<DiffResultInfo> {
    return invoke('generate_diff', { original, modified });
  },

  async generateDiffForFile(originalPath: string, modifiedPath: string): Promise<DiffResultInfo> {
    return invoke('generate_file_diff', { originalPath, modifiedPath });
  },

  async applyDiff(diffId: string): Promise<void> {
    return invoke('apply_diff', { diffId });
  },

  async rejectDiff(diffId: string): Promise<void> {
    return invoke('reject_diff', { diffId });
  },

  async getCodeDiffs(conversationId: string, messageId?: string): Promise<CodeDiffInfo[]> {
    return invoke('get_code_diffs', { conversationId, messageId });
  },
};

export const tauriAPI = {
  isTauri,

  async getPlatform(): Promise<{ os: string; arch: string; is_electron: boolean }> {
    if (!isTauri) return { os: 'web', arch: 'unknown', is_electron: false };
    return invoke('get_platform');
  },

  async getAppPath(): Promise<string> {
    if (!isTauri) return '';
    return invoke('get_app_path');
  },

  async selectDirectory(): Promise<string | null> {
    if (!isTauri) return null;
    return invoke('select_directory');
  },

  async showItemInFolder(path: string): Promise<void> {
    if (!isTauri) return;
    return invoke('show_item_in_folder', { path });
  },

  async openFolder(path: string): Promise<void> {
    if (!isTauri) return;
    return invoke('open_folder', { path });
  },

  async openExternal(url: string): Promise<void> {
    if (!isTauri) {
      window.open(url, '_blank');
      return;
    }
    return invoke('open_external_url', { url });
  },

  async resizeWindow(width: number, height: number): Promise<void> {
    if (!isTauri) return;
    return invoke('resize_window', { width, height });
  },

  async showMainWindow(): Promise<void> {
    if (!isTauri) return;
    return invoke('show_main_window');
  },

  async exportWorkspace(
    workspaceId: string,
    contextMarkdown: string,
    defaultFilename: string
  ): Promise<string> {
    if (!isTauri) return '';
    return invoke('export_workspace', { workspaceId, contextMarkdown, defaultFilename });
  },

  async getSystemStatus(): Promise<{
    platform: string;
    git_bash: { required: boolean; found: boolean; path: string | null };
  }> {
    if (!isTauri) {
      return {
        platform: 'web',
        git_bash: { required: false, found: false, path: null },
      };
    }
    return invoke('get_system_status');
  },

  async executeTool(
    name: string,
    input: Record<string, unknown>,
    cwd?: string
  ): Promise<Record<string, unknown> | null> {
    if (!isTauri) return null;
    return invoke('execute_tool', { name, input, cwd });
  },

  async checkUpdate(): Promise<{ has_update: boolean }> {
    if (!isTauri) return { has_update: false };
    return invoke('check_update');
  },

  async installUpdate(): Promise<void> {
    if (!isTauri) return;
    return invoke('install_update');
  },
};

export interface H5TokenInfo {
  id: string;
  token: string;
  conversation_id: string;
  expires_at: string;
  is_revoked: boolean;
  created_at: string;
  used_at: string | null;
}

export const h5API = {
  async generateToken(conversationId: string, ttlMinutes: number): Promise<H5TokenInfo> {
    console.log('[h5API] generateToken START, convId:', conversationId, 'ttl:', ttlMinutes);
    if (!isTauri) { console.warn('[h5API] Tauri not available'); throw new Error('Tauri not available'); }
    try {
      const result = await invoke('generate_h5_token', { conversationId, ttlMinutes });
      console.log('[h5API] generateToken SUCCESS:', result);
      return result as H5TokenInfo;
    } catch (err) {
      console.error('[h5API] generateToken FAILED:', err);
      throw err;
    }
  },

  async revokeToken(tokenId: string): Promise<void> {
    console.log('[h5API] revokeToken, tokenId:', tokenId);
    if (!isTauri) { console.warn('[h5API] Tauri not available'); throw new Error('Tauri not available'); }
    try {
      await invoke('revoke_h5_token', { tokenId });
      console.log('[h5API] revokeToken SUCCESS');
    } catch (err) {
      console.error('[h5API] revokeToken FAILED:', err);
      throw err;
    }
  },

  async listTokens(conversationId: string): Promise<H5TokenInfo[]> {
    console.log('[h5API] listTokens START, convId:', conversationId);
    if (!isTauri) { console.warn('[h5API] Tauri not available'); throw new Error('Tauri not available'); }
    try {
      const result = await invoke('list_h5_tokens', { conversationId });
      console.log('[h5API] listTokens SUCCESS, count:', (result as H5TokenInfo[]).length);
      return result as H5TokenInfo[];
    } catch (err) {
      console.error('[h5API] listTokens FAILED:', err);
      throw err;
    }
  },

  async validateToken(token: string): Promise<H5TokenInfo> {
    console.log('[h5API] validateToken, token:', token.substring(0, 20) + '...');
    if (!isTauri) { console.warn('[h5API] Tauri not available'); throw new Error('Tauri not available'); }
    try {
      const result = await invoke('validate_h5_token', { token });
      console.log('[h5API] validateToken SUCCESS:', result);
      return result as H5TokenInfo;
    } catch (err) {
      console.error('[h5API] validateToken FAILED:', err);
      throw err;
    }
  },

  async cleanupExpiredTokens(): Promise<void> {
    console.log('[h5API] cleanupExpiredTokens START');
    if (!isTauri) { console.warn('[h5API] Tauri not available'); throw new Error('Tauri not available'); }
    try {
      await invoke('cleanup_expired_h5_tokens');
      console.log('[h5API] cleanupExpiredTokens SUCCESS');
    } catch (err) {
      console.error('[h5API] cleanupExpiredTokens FAILED:', err);
      throw err;
    }
  },
};

export interface PermissionApprovalInfo {
  id: string;
  conversation_id: string;
  message_id: string;
  tool_name: string;
  action: string;
  risk_level: string;
  status: string;
  user_decision: string | null;
  decision_reason: string | null;
  created_at: string;
  decided_at: string | null;
}

export interface DangerousTool {
  tool_name: string;
  action: string;
  risk_level: string;
  description: string;
}

export interface AlwaysAllowRule {
  id: string;
  pattern: string;
  rule_type: string;
  created_at: string;
}

export const permissionApprovalAPI = {
  async requestApproval(
    conversationId: string,
    messageId: string,
    toolName: string,
    action: string,
    riskLevel: string
  ): Promise<PermissionApprovalInfo> {
    return invoke('request_permission_approval', {
      conversationId,
      messageId,
      toolName,
      action,
      riskLevel,
    });
  },

  async approvePermission(
    approvalId: string,
    userDecision: string,
    decisionReason?: string
  ): Promise<void> {
    return invoke('approve_permission', {
      approvalId,
      userDecision,
      decisionReason,
    });
  },

  async rejectPermission(
    approvalId: string,
    decisionReason?: string
  ): Promise<void> {
    return invoke('reject_permission', {
      approvalId,
      decisionReason,
    });
  },

  async getPendingApprovals(conversationId: string): Promise<PermissionApprovalInfo[]> {
    return invoke('get_pending_approvals', { conversationId });
  },

  async alwaysAllowPermission(
    approvalId: string,
    toolName: string,
    action: string
  ): Promise<void> {
    return invoke('always_allow_permission', {
      approvalId,
      toolName,
      action,
    });
  },

  async getDangerousTools(): Promise<DangerousTool[]> {
    return invoke('get_dangerous_tools_list');
  },

  async addAlwaysAllowRule(pattern: string, ruleType: string): Promise<void> {
    return invoke('add_always_allow_rule', { pattern, ruleType });
  },

  async removeAlwaysAllowRule(id: string): Promise<void> {
    return invoke('remove_always_allow_rule', { id });
  },

  async getAlwaysAllowRules(): Promise<AlwaysAllowRule[]> {
    return invoke('get_always_allow_rules');
  },
};

export interface TaskInfo {
  id: string;
  name: string;
  description: string | null;
  cron_expression: string;
  task_type: string;
  task_config: string;
  conversation_id: string | null;
  is_enabled: boolean;
  last_run_at: string | null;
  last_run_status: string | null;
  last_run_output: string | null;
  next_run_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTaskRequest {
  name: string;
  description?: string;
  cron_expression: string;
  task_type: string;
  task_config: string;
  conversation_id?: string;
}

export interface UpdateTaskRequest {
  id: string;
  name?: string;
  description?: string;
  cron_expression?: string;
  task_type?: string;
  task_config?: string;
  is_enabled?: boolean;
}

export interface TaskRunInfo {
  id: string;
  task_id: string;
  started_at: string;
  finished_at: string | null;
  status: string;
  output: string | null;
  error_message: string | null;
  duration_ms: number | null;
}

export const taskAPI = {
  async createTask(request: CreateTaskRequest): Promise<TaskInfo> {
    return invoke('create_scheduled_task', { request });
  },

  async listTasks(): Promise<TaskInfo[]> {
    return invoke('list_scheduled_tasks');
  },

  async updateTask(request: UpdateTaskRequest): Promise<TaskInfo> {
    return invoke('update_scheduled_task', { request });
  },

  async deleteTask(id: string): Promise<void> {
    return invoke('delete_scheduled_task', { id });
  },

  async executeNow(id: string): Promise<void> {
    return invoke('execute_task_now', { id });
  },

  async getTaskRuns(taskId?: string): Promise<TaskRunInfo[]> {
    return invoke('get_task_runs', { taskId });
  },
};

export interface ImPlatformConfig {
  webhook_url: string;
  token: string;
  extra?: Record<string, string>;
}

export interface ImConnectionInfo {
  id: string;
  platform: string;
  status: string;
  config: ImPlatformConfig;
  created_at: string;
  updated_at: string;
}

export interface ImConnectionStatus {
  platform: string;
  status: 'connected' | 'connecting' | 'disconnected' | 'error';
  last_connected_at: string | null;
  error_message: string | null;
}

export interface ImMessageStats {
  platform: string;
  today_messages: number;
  active_users: number;
  total_messages: number;
  updated_at: string;
}

export type ImPermissionMode = 'open' | 'whitelist' | 'pairing_code';

export interface ImPairingRequest {
  id: string;
  user_id: string;
  user_name: string;
  platform: string;
  status: 'pending' | 'approved' | 'rejected';
  created_at: string;
}

export interface ImErrorLog {
  id: string;
  platform: string;
  error: string;
  details: string | null;
  created_at: string;
}

export const imAPI = {
  async connectPlatform(
    platform: string,
    config: ImPlatformConfig
  ): Promise<ImConnectionInfo> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_connect_platform', { platform, config });
  },

  async disconnectPlatform(platform: string): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_disconnect_platform', { platform });
  },

  async listConnections(): Promise<ImConnectionInfo[]> {
    if (!isTauri) return [];
    return invoke('im_list_connections');
  },

  async sendMessage(
    platform: string,
    chatId: string,
    message: string
  ): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_send_message', { platform, chatId, message });
  },

  async getConfig(platform: string): Promise<ImConnectionInfo | null> {
    if (!isTauri) return null;
    return invoke('im_get_config', { platform });
  },

  async updateConfig(
    platform: string,
    config: ImPlatformConfig
  ): Promise<ImConnectionInfo> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_update_config', { platform, config });
  },

  async generateQrCode(platform: string): Promise<{ qr_url: string; expires_at: string }> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_generate_qr_code', { platform });
  },

  async checkAuthStatus(platform: string): Promise<{ authorized: boolean; user_info?: { id: string; name: string } }> {
    if (!isTauri) return { authorized: false };
    return invoke('im_check_auth_status', { platform });
  },

  async getConnectionStatus(platform: string): Promise<ImConnectionStatus> {
    if (!isTauri) return { platform, status: 'disconnected', last_connected_at: null, error_message: null };
    return invoke('im_get_connection_status', { platform });
  },

  async getMessageStats(platform?: string): Promise<ImMessageStats[]> {
    if (!isTauri) return [];
    return invoke('im_get_message_stats', { platform });
  },

  async setPermissionMode(platform: string, mode: ImPermissionMode): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_set_permission_mode', { platform, mode });
  },

  async getPermissionMode(platform: string): Promise<{ mode: ImPermissionMode }> {
    if (!isTauri) return { mode: 'open' };
    return invoke('im_get_permission_mode', { platform });
  },

  async generatePairingCode(platform: string, userId: string): Promise<{ code: string; expires_at: string }> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_generate_pairing_code', { platform, userId });
  },

  async getPendingPairingRequests(platform: string): Promise<ImPairingRequest[]> {
    if (!isTauri) return [];
    return invoke('im_get_pending_pairing_requests', { platform });
  },

  async approvePairingRequest(platform: string, userId: string): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_approve_pairing_request', { platform, userId });
  },

  async rejectPairingRequest(platform: string, userId: string): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('im_reject_pairing_request', { platform, userId });
  },

  async getErrorLogs(platform?: string): Promise<ImErrorLog[]> {
    if (!isTauri) return [];
    return invoke('im_get_error_logs', { platform });
  },
};

/* ── Lark Channel Bridge API ── */
export interface BridgeInstanceInfo {
  running: boolean;
  pid: number | null;
  bot_name: string | null;
  app_id: string | null;
  version: string | null;
  started_at: string | null;
  config_path: string | null;
}

export interface FeishuCredentials {
  app_id: string;
  app_secret: string;
  tenant: string;
  admin_open_id?: string;
}

export const bridgeAPI = {
  async detect(): Promise<BridgeInstanceInfo[]> {
    if (!isTauri) return [];
    return invoke('bridge_detect');
  },

  async getStatus(id?: string): Promise<BridgeInstanceInfo> {
    if (!isTauri) return { running: false, pid: null, bot_name: null, app_id: null, version: null, started_at: null, config_path: null };
    return invoke('bridge_get_status', { id: id || null });
  },

  async start(): Promise<string> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('bridge_start');
  },

  async stop(id?: string): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('bridge_stop', { id: id || null });
  },

  async readConfig(): Promise<string> {
    if (!isTauri) return '';
    return invoke('bridge_read_config');
  },

  /** Read bridge's Feishu credentials (app_id + app_secret) */
  async getCredentials(): Promise<FeishuCredentials | null> {
    if (!isTauri) return null;
    return invoke('bridge_get_credentials');
  },

  /** Start QR code auth flow, returns verification URL */
  async startAuth(): Promise<string> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('bridge_start_auth');
  },

  /** Poll auth status */
  async pollAuth(): Promise<{ status: string; verification_url?: string }> {
    if (!isTauri) return { status: 'no_auth_in_progress' };
    return invoke('bridge_poll_auth');
  },

  /** Complete auth and get credentials */
  async completeAuth(): Promise<FeishuCredentials | null> {
    if (!isTauri) return null;
    return invoke('bridge_complete_auth');
  },
};

/* ── Feishu Multi-Window Chat API ── */
export interface FeishuChatMapping {
  chat_id: string;
  conversation_id: string;
  title: string;
  created_at: string;
  last_active_at: string;
  message_count: number;
}

export const feishuChatAPI = {
  async getOrCreateConversation(chatId: string, title?: string): Promise<FeishuChatMapping> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('feishu_get_or_create_conversation', { chatId, title: title || null });
  },

  async listConversations(): Promise<FeishuChatMapping[]> {
    if (!isTauri) return [];
    return invoke('feishu_list_conversations');
  },

  async deleteConversation(chatId: string): Promise<void> {
    if (!isTauri) return;
    return invoke('feishu_delete_conversation', { chatId });
  },
};

export interface ComputerUseScreenInfo {
  width: number;
  height: number;
  scale_factor: number;
}

export interface ComputerUseScreenshotResult {
  base64: string;
  width: number;
  height: number;
}

export const computerUseAPI = {
  async screenshot(): Promise<ComputerUseScreenshotResult> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('computer_use_screenshot');
  },

  async mouseClick(x: number, y: number, button: string = 'left'): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('computer_use_mouse_click', { x, y, button });
  },

  async keyboardType(text: string): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('computer_use_keyboard_type', { text });
  },

  async keyboardKey(key: string): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('computer_use_keyboard_key', { key });
  },

  async mouseScroll(scrollX: number, scrollY: number): Promise<void> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('computer_use_mouse_scroll', { scrollX, scrollY });
  },

  async getScreenInfo(): Promise<ComputerUseScreenInfo> {
    if (!isTauri) throw new Error('Tauri not available');
    return invoke('computer_use_get_screen_info');
  },
};
