// ============================================================
// API barrel — re-exports EVERYTHING from api.ts so existing
// imports from './api' continue to work without changes.
// ============================================================

// Re-export from client.ts
export {
  API_BASE,
  CHENGDU_API,
  GATEWAY_BASE,
  GATEWAY_ROUTES,
  isTauriApp,
  resetBridgePort,
  detectBridgePort,
  waitForApiReady,
  preloadSecureCredentials,
  getToken,
  request,
  getSystemStatus,
  healthCheck,
  isGatewayLoggedIn,
  gatewayLogout,
  getGatewayUsage,
  getUserModeForConversation,
  resolveEnvCreds,
  ensureNativeEngine,
} from './client';

// Re-export from auth.ts
export {
  sendCode,
  register,
  login,
  gatewayLogin,
  forgotPassword,
  resetPassword,
} from './auth';

// Re-export from user.ts
export {
  getUserProfile,
  updateUserProfile,
  getUserUsage,
  getUserModels,
  getUser,
  logout,
  changePassword,
  deleteAccount,
  getSessions,
  deleteSession,
  logoutOtherSessions,
  getUnreadAnnouncements,
  markAnnouncementRead,
  getPlans,
  createPaymentOrder,
  getPaymentStatus,
  redeemCode,
  getCodeSSO,
  getCodeQuota,
} from './user';

// Re-export from chat.ts
export {
  sendMessage,
  sendMessageNative,
  chatStream,
  chatAsk,
} from './chat';

// Re-export from conversations.ts
export {
  getConversations,
  createConversation,
  getConversation,
  deleteConversation,
  updateConversation,
  exportConversation,
  getGenerationStatus,
  stopGeneration,
  getContextSize,
  compactConversation,
  branchConversation,
  answerUserQuestion,
  respondToolPermission,
  deleteMessagesFrom,
  deleteMessagesTail,
  getStreamStatus,
  reconnectStream,
  persistMessages,
  exportConversationAs,
} from './conversations';
export type { ExportFormat } from './conversations';

// Re-export from projects.ts
export {
  listProjects, getProjects, createProject, getProject, updateProject, deleteProject,
  uploadProjectFile, deleteProjectFile, getProjectConversations, createProjectConversation,
} from './projects';
export type { Project, ProjectFile } from './projects';

// Re-export from config.ts
export {
} from './config';

// Re-export from fs.ts
export {
  fsTree, fsRead, fsWrite, fsCreate, fsDelete,
} from './fs';
export type { FsFileNode, FsTreeResponse, FsReadResponse } from './fs';

// Backward-compatible aliases for renamed fs functions
import { fsTree as _fsTree, fsRead as _fsRead, fsWrite as _fsWrite, fsCreate as _fsCreate, fsDelete as _fsDelete } from './fs';
export const getFileSystemTree = _fsTree;
export const readFileContent = _fsRead;
export const writeFileContent = _fsWrite;
export const createFileOrDir = _fsCreate;
export const deleteFileOrDir = _fsDelete;

// Re-export from terminal.ts
export {
  createTerminal, writeTerminal, resizeTerminal, closeTerminal, listTerminals, streamTerminalOutput,
} from './terminal';
export type { TerminalSession } from './terminal';

// Re-export from mcp.ts
export {
} from './mcp';

// Re-export from git.ts
export {
} from './git';

// Re-export from memory.ts
export {
} from './memory';

// Re-export from tools.ts
export {
} from './tools';

// Re-export from skills.ts
export {
  getSkills,
  getSkill,
  getSkillDetail,
  getSkillFile,
  createSkill,
  updateSkill,
  deleteSkill,
  toggleSkill,
  listSkills,
  executeSkill,
} from './skills';

// Re-export from github.ts
export {
  getGithubStatus,
  getGithubAuthUrl,
  disconnectGithub,
  getGithubRepos,
  getGithubTree,
  getGithubContents,
  materializeGithub,
} from './github';

// Re-export from artifacts.ts
export {
  getUserArtifacts,
  getArtifactContent,
} from './artifacts';

// Re-export from engine.ts
export {
  warmEngine,
  listEngines,
  spawnEngine,
  killEngine,
  listAgents,
  getAgent,
  cancelAgent,
  getIdeStatus,
  startIdeServer,
  stopIdeServer,
  getIdeConnections,
  disconnectIde,
} from './engine';

// Re-export from worktree.ts
export {
  createWorktree,
  listWorktrees,
  getWorktree,
  removeWorktree,
  mergeWorktree,
  syncWorktrees,
} from './worktree';

// Re-export from providers.ts
export {
  testProviderWebSearch, listProviders, getProviders, createProvider, updateProvider, deleteProvider, getProviderModels,
} from './providers';

// Re-export from analytics.ts
export {
  trackEvent,
  getAnalyticsDaily,
  getAnalyticsRange,
  getAnalyticsSummary,
  getAnalyticsEventCounts,
  getAnalyticsRecentEvents,
  getAnalyticsDashboard,
} from './analytics';

// Re-export from caveman.ts
export {
} from './caveman';

// Re-export from research.ts
export {
  multiagentResearch,
  startResearch,
  stopResearch,
  getResearchStatus,
  streamResearch,
} from './research';

// Re-export from workflow.ts
export {
} from './workflow';

// Re-export from upload.ts
export {
  uploadFile, deleteAttachment, getAttachmentUrl, getUpload, deleteUpload,
} from './upload';
export type { UploadResult } from './upload';

// Re-export from notification.ts
export {
} from './notification';

// Re-export from clipboard.ts
export {
} from './clipboard';

// Re-export from im.ts
export {
} from './im';
