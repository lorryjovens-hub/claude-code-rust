import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import { ChevronDown, FileText, ArrowUp, RotateCcw, Pencil, Copy, Check, Paperclip, ListCollapse, Globe, Clock, Info, Github, Plus, X, Loader2 } from 'lucide-react';
import { useParams, useNavigate, useLocation } from 'react-router-dom';
import { IconPlus, IconVoice, IconPencil, IconProjects, IconResearch, IconWebSearch } from './Icons';
import ClaudeLogo from './ClaudeLogo';
import { getConversation, sendMessage, createConversation, getUser, updateConversation, deleteMessagesFrom, deleteMessagesTail, uploadFile, deleteAttachment, compactConversation, answerUserQuestion, getUserUsage, getAttachmentUrl, getGenerationStatus, stopGeneration, getContextSize, getUserModels, getStreamStatus, reconnectStream, getProviderModels, getSkills, warmEngine, getProjects, createProject, Project, materializeGithub, getProviders, Provider } from '../api';
import { addStreaming, removeStreaming, isStreaming } from '../streamingState';
import MarkdownRenderer from './MarkdownRenderer';
import ResearchPanel from './ResearchPanel';
import ModelSelector, { SelectableModel } from './ModelSelector';
import FileUploadPreview, { PendingFile } from './FileUploadPreview';
import AddFromGithubModal, { GithubAddPayload } from './AddFromGithubModal';
import MessageAttachments from './MessageAttachments';
import DocumentCard, { DocumentInfo } from './DocumentCard';
import { copyToClipboard } from '../utils/clipboard';
import SearchProcess from './SearchProcess';
import DocumentCreationProcess, { DocumentDraftInfo } from './DocumentCreationProcess';
import CodeExecution from './CodeExecution';
import ToolDiffView, { shouldUseDiffView, hasExpandableContent, getToolStats } from './ToolDiffView';
import { executeCode, sendCodeResult, setStatusCallback } from '../pyodideRunner';

function formatChatError(err: string): string {
  const lower = (err || '').toLowerCase();
  if (lower.includes('quota_exceeded') || lower.includes('额度已用完') || lower.includes('额度已用尽') || lower.includes('时段额度') || lower.includes('周期额度')) {
    return '⚠️ 当前额度已用完，请等待额度重置后再试。你可以在设置页查看额度详情。';
  }
  if (lower.includes('订阅已过期') || lower.includes('未激活') || lower.includes('inactive') || lower.includes('expired')) {
    return '⚠️ 你的订阅已过期或未激活，请续费后继续使用。';
  }
  if (lower.includes('invalid api key') || lower.includes('authentication')) {
    return '⚠️ API 认证失败，请重新登录。';
  }
  if (lower.includes('overloaded') || lower.includes('rate limit') || lower.includes('529')) {
    return '⚠️ 服务暂时繁忙，请稍后再试。';
  }
  return 'Error: ' + err;
}

// Blue skill tag shown in chat messages (hover shows tooltip)
const SkillTag: React.FC<{ slug: string; description?: string }> = ({ slug, description }) => {
  const [hover, setHover] = useState(false);
  return (
    <span className="relative inline" onMouseEnter={() => setHover(true)} onMouseLeave={() => setHover(false)}>
      <span className={`text-[#4B9EFA] font-medium cursor-default transition-colors ${hover ? 'bg-[#4B9EFA]/10 rounded px-0.5 -mx-0.5' : ''}`}>
        /{slug}
      </span>
      {hover && description && (
        <div className="absolute left-0 top-full mt-2 w-[240px] p-3 bg-claude-input border border-claude-border rounded-xl shadow-lg z-[100] pointer-events-none">
          <div className="text-[12px] text-claude-textSecondary leading-snug mb-1.5">{description.length > 150 ? description.slice(0, 150) + '...' : description}</div>
          <div className="text-[11px] text-claude-textSecondary/60">Skill</div>
        </div>
      )}
    </span>
  );
};

// Overlay that mirrors textarea text: /skill-name in blue, rest in normal color
const SkillInputOverlay: React.FC<{ text: string; className?: string; style?: React.CSSProperties }> = ({ text, className, style }) => {
  const match = text.match(/^(\/[a-zA-Z0-9_-]+)([\s\S]*)$/);
  if (!match) return null;
  return (
    <div className={className} style={{ ...style, pointerEvents: 'none', position: 'absolute', top: 0, left: 0, right: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-word' }} aria-hidden>
      <span className="text-[#4B9EFA]">{match[1]}</span>
      <span className="text-claude-text">{match[2] || ''}</span>
    </div>
  );
};

const CompactingStatus = () => {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    // Fake progress animation
    const interval = setInterval(() => {
      setProgress(prev => {
        if (prev >= 95) return prev;
        // Logarithmic-like slowdown
        const remaining = 95 - prev;
        const inc = Math.max(0.2, remaining * 0.05);
        return Math.min(95, prev + inc);
      });
    }, 100);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex flex-col justify-center ml-2">
      <div className="text-[#404040] dark:text-[#d1d5db] font-serif italic text-[17px] leading-relaxed mb-1">
        Compacting our conversation so we can keep chatting...
      </div>
      <div className="flex items-center gap-3">
        <div className="w-48 h-1.5 bg-[#EAE8E1] dark:bg-white/10 rounded-full overflow-hidden">
          <div
            className="h-full bg-[#404040] dark:bg-[#d1d5db] rounded-full transition-all duration-100 ease-out"
            style={{ width: `${progress}%` }}
          />
        </div>
        <span className="text-[13px] text-[#707070] dark:text-[#9ca3af] font-medium font-mono">
          {Math.round(progress)}%
        </span>
      </div>
    </div>
  );
};

// 时间戳格式化
function formatMessageTime(dateStr: string): string {
  if (!dateStr) return '';

  let timeStr = dateStr;
  // Handle SQLite format (space instead of T)
  if (timeStr.includes(' ') && !timeStr.includes('T')) {
    timeStr = timeStr.replace(' ', 'T');
  }
  // Handle missing timezone (assume UTC if no Z or offset at end)
  if (!/Z$|[+-]\d{2}:?\d{2}$/.test(timeStr)) {
    timeStr += 'Z';
  }

  const date = new Date(timeStr);
  if (isNaN(date.getTime())) return '';

  const now = new Date();
  const isToday = date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();
  if (isToday) {
    return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  }
  const isSameYear = date.getFullYear() === now.getFullYear();
  if (isSameYear) {
    return `${date.getMonth() + 1}月${date.getDate()}日`;
  }
  return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
}

function stripThinking(model: string) {
  return (model || '').replace(/-thinking$/, '');
}

function withThinking(base: string, thinking: boolean) {
  return thinking ? `${base}-thinking` : base;
}

function isThinkingModel(model: string) {
  return typeof model === 'string' && model.endsWith('-thinking');
}

// ─── Cross-mode override helpers ──────────────────────────────────────────────
// When a conversation's model belongs to a different mode than the user is
// currently in, we let the user opt to "keep using the cross-mode model".
// The choice is persisted per-conversation in localStorage so the next message
// in the same conversation continues to use the override mode without re-prompting.
function getCrossModeOverride(convId: string): 'clawparrot' | 'selfhosted' | null {
  try {
    const raw = localStorage.getItem('cross_mode_overrides');
    if (!raw) return null;
    const map = JSON.parse(raw);
    return map[convId] || null;
  } catch { return null; }
}

function setCrossModeOverride(convId: string, mode: 'clawparrot' | 'selfhosted') {
  try {
    const raw = localStorage.getItem('cross_mode_overrides');
    const map = raw ? JSON.parse(raw) : {};
    map[convId] = mode;
    localStorage.setItem('cross_mode_overrides', JSON.stringify(map));
  } catch {}
}

function clearCrossModeOverride(convId: string) {
  try {
    const raw = localStorage.getItem('cross_mode_overrides');
    if (!raw) return;
    const map = JSON.parse(raw);
    delete map[convId];
    localStorage.setItem('cross_mode_overrides', JSON.stringify(map));
  } catch {}
}

function isSearchStatusMessage(message: string) {
  if (!message) return false;
  return (
    message.startsWith('正在搜索：') ||
    message.startsWith('正在读取网页：') ||
    message.startsWith('正在浏览 GitHub：') ||
    message.startsWith('Searching:') ||
    message.startsWith('Fetching:')
  );
}

// Extract display text from content that may be a plain string or a JSON-stringified content array
function extractTextContent(content: any): string {
  if (!content) return '';
  if (typeof content !== 'string') return String(content);
  // Try to parse as JSON array (Anthropic API content format)
  if (content.startsWith('[')) {
    try {
      const parsed = JSON.parse(content);
      if (Array.isArray(parsed)) {
        return parsed
          .filter((block: any) => block && block.type === 'text' && block.text)
          .map((block: any) => block.text)
          .join('\n');
      }
    } catch {
      // Not valid JSON, treat as plain text
    }
  }
  return content;
}

function withAuthToken(url: string) {
  if (!url || url.startsWith('data:') || /[?&]token=/.test(url)) return url;
  if (typeof window === 'undefined') return url;
  const token = localStorage.getItem('auth_token');
  if (!token) return url;
  return `${url}${url.includes('?') ? '&' : '?'}token=${encodeURIComponent(token)}`;
}

function normalizeMessageDocuments(message: any): DocumentInfo[] {
  const raw = Array.isArray(message?.documents)
    ? message.documents
    : (message?.document ? [message.document] : []);
  const docs: DocumentInfo[] = [];
  const seen = new Set<string>();

  for (const doc of raw) {
    if (!doc || typeof doc !== 'object') continue;
    const key = doc.id || doc.url || doc.filename || `${doc.title || 'doc'}-${docs.length}`;
    if (seen.has(key)) continue;
    seen.add(key);
    docs.push(doc as DocumentInfo);
  }

  // Extract documents from Write tool calls, then apply subsequent Edit operations
  const previewExts = ['md', 'txt', 'html', 'json', 'xml', 'yaml', 'yml', 'csv'];
  if (Array.isArray(message?.toolCalls)) {
    // First pass: collect initial Write content per file path
    const fileContents = new Map<string, string>();
    const fileOrder: string[] = [];
    for (const tc of message.toolCalls) {
      if (tc.name === 'Write' && tc.input?.file_path && tc.input?.content) {
        const fp = tc.input.file_path as string;
        fileContents.set(fp, tc.input.content);
        if (!fileOrder.includes(fp)) fileOrder.push(fp);
      }
    }
    // Second pass: apply Edit operations to the accumulated content
    for (const tc of message.toolCalls) {
      if ((tc.name === 'Edit' || tc.name === 'MultiEdit') && tc.input?.file_path && tc.input?.old_string != null && tc.input?.new_string != null) {
        const fp = tc.input.file_path as string;
        const current = fileContents.get(fp);
        if (current != null) {
          fileContents.set(fp, current.replaceAll(tc.input.old_string, tc.input.new_string));
        }
      }
    }
    // Create document entries from final content
    for (const fp of fileOrder) {
      const fileName = fp.split(/[/\\]/).pop() || fp;
      const ext = fileName.split('.').pop()?.toLowerCase() || '';
      if (!previewExts.includes(ext)) continue;
      const key = `write-${fp}`;
      if (seen.has(key)) continue;
      seen.add(key);
      docs.push({
        id: key,
        title: fileName,
        filename: fileName,
        url: '',
        content: fileContents.get(fp) || '',
        format: ext === 'md' ? 'markdown' : 'text',
      });
    }
  }

  return docs;
}

function parseInlineArtifactDisplay(content: any): { cleanedContent: string; draft: DocumentDraftInfo | null } | null {
  if (typeof content !== 'string' || !content.includes('<cp_artifact')) return null;

  const openMatch = content.match(/<cp_artifact\s+([^>]*)>/i);
  if (!openMatch || openMatch.index === undefined) return null;

  const attrsRaw = openMatch[1] || '';
  const title = (attrsRaw.match(/title="([^"]*)"/i)?.[1] || '').trim() || 'Untitled document';
  const format = (attrsRaw.match(/format="([^"]*)"/i)?.[1] || 'markdown').trim() || 'markdown';
  const openTag = openMatch[0];
  const bodyStart = openMatch.index + openTag.length;
  const closeTag = '</cp_artifact>';
  const closeIdx = content.indexOf(closeTag, bodyStart);

  if (closeIdx === -1) {
    const preview = content.slice(bodyStart).replace(/^\n/, '');
    const cleanedContent = content.slice(0, openMatch.index).trim().replace(/\n{3,}/g, '\n\n');
    return {
      cleanedContent,
      draft: {
        draftId: `inline-${title}-${format}`,
        title,
        format,
        preview,
        previewAvailable: preview.length > 0,
        done: false,
      },
    };
  }

  const preview = content.slice(bodyStart, closeIdx).replace(/^\n/, '');
  const before = content.slice(0, openMatch.index);
  const after = content.slice(closeIdx + closeTag.length);
  const cleanedContent = `${before}${after}`.trim().replace(/\n{3,}/g, '\n\n');

  return {
    cleanedContent,
    draft: {
      draftId: `inline-${title}-${format}`,
      title,
      format,
      preview,
      previewAvailable: preview.length > 0,
      done: true,
    },
  };
}

// Apply a research_* SSE event to the last assistant message in a messages array.
// Returns a new messages array (mutates a clone of the last message).
function applyResearchEvent(prev: any[], event: string, data: any): any[] {
  const newMsgs = [...prev];
  const lastIdx = newMsgs.length - 1;
  const lastMsg = newMsgs[lastIdx];
  if (!lastMsg || lastMsg.role !== 'assistant') return prev;
  const research = { ...(lastMsg.research || { sub_agents: [], sources: [], phase: null, plan: null, report: null, completed: false }) };
  research.sub_agents = [...(research.sub_agents || [])];
  research.sources = [...(research.sources || [])];
  switch (event) {
    case 'research_phase':
      research.phase = data.phase;
      research.phase_label = data.label;
      break;
    case 'research_plan':
      research.plan = { title: data.title, sub_questions: data.sub_questions };
      break;
    case 'research_subagent_started': {
      const exists = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (!exists) {
        research.sub_agents.push({
          id: data.sub_agent_id,
          index: data.index,
          sub_question: data.sub_question,
          status: 'running',
          sources: [],
          findings: '',
        });
      }
      break;
    }
    case 'research_source': {
      const sub = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (sub) {
        sub.sources = [...sub.sources, data.source];
      }
      // Global dedupe
      const exists = research.sources.find((s: any) => s.url === data.source.url);
      if (!exists) research.sources.push(data.source);
      break;
    }
    case 'research_finding': {
      const sub = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (sub) {
        sub.findings = data.markdown || '';
      }
      break;
    }
    case 'research_subagent_done': {
      const sub = research.sub_agents.find((a: any) => a.id === data.sub_agent_id);
      if (sub) {
        sub.status = data.error ? 'error' : 'done';
        if (data.error) sub.error = data.error;
      }
      break;
    }
    case 'research_report':
      research.report = data.markdown;
      break;
    case 'research_done':
      research.completed = true;
      research.duration_ms = data.duration_ms;
      break;
    case 'research_error':
      research.error = data.error;
      research.completed = true;
      break;
  }
  newMsgs[lastIdx] = { ...lastMsg, research };
  return newMsgs;
}

function sanitizeInlineArtifactMessage(message: any) {
  if (!message || message.role !== 'assistant') return message;
  const parsed = parseInlineArtifactDisplay(message.content);
  if (!parsed) return message;

  let next = { ...message, content: parsed.cleanedContent };
  if (parsed.draft && normalizeMessageDocuments(next).length === 0) {
    next = mergeDocumentDraftIntoMessage(next, parsed.draft);
  }
  return next;
}

function mergeDocumentsIntoMessage(message: any, incomingDoc?: DocumentInfo | null, incomingDocs?: DocumentInfo[] | null) {
  const merged = [...normalizeMessageDocuments(message)];
  const queue = [
    ...(Array.isArray(incomingDocs) ? incomingDocs : []),
    ...(incomingDoc ? [incomingDoc] : []),
  ];

  for (const doc of queue) {
    if (!doc || typeof doc !== 'object') continue;
    const key = doc.id || doc.url || doc.filename || doc.title;
    if (!key) continue;
    const index = merged.findIndex(item => (item.id || item.url || item.filename || item.title) === key);
    if (index >= 0) merged[index] = doc;
    else merged.push(doc);
  }

  if (merged.length === 0) return message;
  return { ...message, document: merged[merged.length - 1], documents: merged };
}

function applyGenerationState(message: any, state: any) {
  const base = {
    ...message,
    content: state.text || message.content,
    thinking: state.thinking || message.thinking,
    thinkingSummary: state.thinkingSummary || message.thinkingSummary,
    citations: state.citations?.length ? state.citations : message.citations,
    searchLogs: state.searchLogs?.length ? state.searchLogs : message.searchLogs,
    isThinking: !state.text && !!state.thinking,
  };
  const withDocuments = mergeDocumentsIntoMessage(base, state.document, state.documents);
  const drafts = Array.isArray(state?.documentDrafts) ? state.documentDrafts : [];
  const withDrafts = drafts.length === 0
    ? withDocuments
    : drafts.reduce((acc: any, draft: any) => mergeDocumentDraftIntoMessage(acc, draft), withDocuments);
  return sanitizeInlineArtifactMessage(withDrafts);
}

function normalizeDocumentDrafts(message: any): DocumentDraftInfo[] {
  const raw = Array.isArray(message?.documentDrafts) ? message.documentDrafts : [];
  const last = raw[raw.length - 1];
  if (!last || typeof last !== 'object') return [];
  const key = last.draftId || last.draft_id || last.title || 'draft';
  return [{
    draftId: key,
    title: last.title,
    format: last.format,
    preview: last.preview,
    previewAvailable: last.previewAvailable ?? last.preview_available,
    done: !!last.done,
  }];
}

function mergeDocumentDraftIntoMessage(message: any, incomingDraft: any) {
  if (!incomingDraft || typeof incomingDraft !== 'object') return message;
  const draftId = incomingDraft.draftId || incomingDraft.draft_id || incomingDraft.title;
  if (!draftId) return message;

  const current = normalizeDocumentDrafts(message)[0] || null;
  const nextDraft: DocumentDraftInfo = {
    draftId,
    title: incomingDraft.title,
    format: incomingDraft.format,
    preview: incomingDraft.preview ?? incomingDraft.document?.content,
    previewAvailable: incomingDraft.previewAvailable ?? incomingDraft.preview_available ?? !!incomingDraft.document?.content,
    done: !!incomingDraft.done,
  };
  const merged: DocumentDraftInfo = current
    ? {
      ...current,
      ...nextDraft,
      draftId: current.draftId || nextDraft.draftId,
      title: nextDraft.title || current.title,
      format: nextDraft.format || current.format,
      preview: nextDraft.preview ?? current.preview,
      previewAvailable: nextDraft.previewAvailable ?? current.previewAvailable,
      done: typeof incomingDraft.done === 'boolean' ? incomingDraft.done : current.done,
    }
    : nextDraft;

  return { ...message, documentDrafts: [merged] };
}

interface MainContentProps {
  onNewChat: () => void; // Callback to tell sidebar to refresh
  resetKey?: number;
  tunerConfig?: any;
  onOpenDocument?: (doc: DocumentInfo) => void;
  onArtifactsUpdate?: (docs: DocumentInfo[]) => void;
  onOpenArtifacts?: () => void;
  onTitleChange?: (title: string) => void;
  onChatModeChange?: (isChat: boolean) => void;
}

// 草稿存储：在切换对话、打开设置页面时保留输入内容和附件
const draftsStore = new Map<string, { text: string; files: PendingFile[]; height: number }>();

interface ModelCatalog {
  common: SelectableModel[];
  all: SelectableModel[];
  fallback_model: string | null;
}

/** Memoized message list — skips re-render when only inputText changes */
interface MessageListProps {
  messages: any[];
  loading: boolean;
  expandedMessages: Set<number>;
  editingMessageIdx: number | null;
  editingContent: string;
  copiedMessageIdx: number | null;
  compactStatus: { state: string; message?: string };
  onSetEditingContent: (v: string) => void;
  onEditCancel: () => void;
  onEditSave: () => void;
  onToggleExpand: (idx: number) => void;
  onResend: (content: string, idx: number) => void;
  onEdit: (content: string, idx: number) => void;
  onCopy: (content: string, idx: number) => void;
  onOpenDocument?: (doc: DocumentInfo) => void;
  onSetMessages: React.Dispatch<React.SetStateAction<any[]>>;
  messageContentRefs: React.MutableRefObject<Map<number, HTMLDivElement>>;
  onOpenResearch?: (msgId: string) => void;
}

const MessageList = React.memo<MessageListProps>(({
  messages, loading, expandedMessages, editingMessageIdx, editingContent,
  copiedMessageIdx, compactStatus, onSetEditingContent, onEditCancel, onEditSave,
  onToggleExpand, onResend, onEdit, onCopy, onOpenDocument, onSetMessages,
  messageContentRefs, onOpenResearch,
}) => {
  return (
    <>
      <style>{`
        @keyframes shimmer {
          0% { background-position: 200% 0; }
          100% { background-position: -200% 0; }
        }
        .animate-shimmer-text {
          background: linear-gradient(90deg, var(--text-claude-secondary) 45%, var(--text-claude-main) 50%, var(--text-claude-secondary) 55%);
          background-size: 200% 100%;
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
          animation: shimmer 4s linear infinite;
        }
      `}</style>
      {messages.map((msg: any, idx: number) => (
        <div key={idx} className="mb-6 group">
          {(msg.is_summary === 1 || msg.is_compact_boundary) && (
            <div className="flex items-center gap-3 mb-5 mt-2">
              <div className="flex-1 h-px bg-claude-border" />
              <span className="text-[12px] text-claude-textSecondary whitespace-nowrap">Context compacted above this point</span>
              <div className="flex-1 h-px bg-claude-border" />
            </div>
          )}
          {(msg.is_summary === 1 || msg.is_compact_boundary) ? null : msg.role === 'user' ? (
            editingMessageIdx === idx ? (
              <div className="w-full bg-[#F0EEE7] dark:bg-claude-btnHover rounded-xl p-3 border border-black/5 dark:border-white/10">
                <div className="bg-white dark:bg-black/20 rounded-lg border border-black/10 dark:border-white/10 focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 transition-all p-3">
                  <textarea
                    className="w-full bg-transparent text-claude-text outline-none resize-none text-[16px] leading-relaxed font-sans font-[350] block"
                    value={editingContent}
                    onChange={(e) => {
                      onSetEditingContent(e.target.value);
                      e.target.style.height = 'auto';
                      e.target.style.height = e.target.scrollHeight + 'px';
                    }}
                    onKeyDown={(e) => { if (e.key === 'Escape') onEditCancel(); }}
                    ref={(el) => {
                      if (el) {
                        el.style.height = 'auto';
                        el.style.height = el.scrollHeight + 'px';
                        el.focus();
                      }
                    }}
                    style={{ minHeight: '60px' }}
                  />
                </div>
                <div className="flex items-start justify-between mt-3 px-1 gap-4">
                  <div className="flex items-start gap-2 text-claude-textSecondary text-[13px] leading-tight pt-1">
                    <Info size={14} className="mt-0.5 shrink-0" />
                    <span>
                      Editing this message will create a new conversation branch. You can switch between branches using the arrow navigation buttons.
                    </span>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    <button
                      onClick={onEditCancel}
                      className="px-3 py-1.5 text-[13px] font-medium text-claude-text bg-white dark:bg-claude-bg border border-black/10 dark:border-white/10 hover:bg-gray-50 dark:hover:bg-claude-hover rounded-lg transition-colors"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={onEditSave}
                      disabled={!editingContent.trim() || editingContent === msg.content}
                      className="px-3 py-1.5 text-[13px] font-medium text-white bg-claude-text hover:bg-claude-textSecondary rounded-lg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
                    >
                      Save
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="flex flex-col items-end">
                {msg.attachments && msg.attachments.length > 0 && (
                  <div className="max-w-[85%] w-fit mb-1">
                    <MessageAttachments attachments={msg.attachments} onOpenDocument={onOpenDocument} />
                  </div>
                )}
                {(!msg.attachments || msg.attachments.length === 0) && msg.has_attachments === 1 && (
                  <div className="max-w-[85%] w-fit mb-1">
                    <div className="bg-[#F0EEE7] dark:bg-claude-btnHover text-claude-textSecondary px-3.5 py-2 text-[14px] rounded-2xl font-sans italic">
                      📎 Files attached
                    </div>
                  </div>
                )}
                {(() => { const displayText = extractTextContent(msg.content); return displayText && displayText.trim() !== ''; })() && (
                  <div className="max-w-[85%] w-fit relative">
                    <div
                      className="bg-[#F0EEE7] dark:bg-claude-btnHover text-claude-text px-3.5 py-2.5 text-[16px] leading-relaxed font-sans font-[350] whitespace-pre-wrap break-words relative overflow-hidden"
                      style={{
                        maxHeight: expandedMessages.has(idx) ? 'none' : '300px',
                        borderRadius: ((() => {
                          const el = messageContentRefs.current.get(idx);
                          const isOverflow = el && el.scrollHeight > 300;
                          return isOverflow;
                        })()) ? '16px 16px 0 0' : '16px',
                      }}
                      ref={(el) => { if (el) messageContentRefs.current.set(idx, el); }}
                    >
                      {(() => {
                        try {
                          const text = extractTextContent(msg.content);
                          if (!text) return '';
                          const skillMatch = text.match(/^\/([a-zA-Z0-9_-]+)(\s|$)/);
                          if (skillMatch) {
                            const slug = skillMatch[1];
                            const rest = text.slice(skillMatch[0].length);
                            return <>
                              <span className="text-[#4B9EFA] font-medium">/{slug}</span>
                              {rest ? ' ' + rest : ''}
                            </>;
                          }
                          return text;
                        } catch { return extractTextContent(msg.content) || ''; }
                      })()}
                      {!expandedMessages.has(idx) && (() => {
                        const el = messageContentRefs.current.get(idx);
                        return el && el.scrollHeight > 300;
                      })() && (
                          <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-[#F0EEE7] dark:from-claude-btnHover to-transparent pointer-events-none" />
                        )}
                    </div>
                    {(() => {
                      const el = messageContentRefs.current.get(idx);
                      const isOverflow = el && el.scrollHeight > 300;
                      if (!isOverflow) return null;
                      return (
                        <div className="bg-[#F0EEE7] dark:bg-claude-btnHover rounded-b-2xl px-3.5 pb-3 pt-1 -mt-[1px] relative" style={{ borderTopLeftRadius: 0, borderTopRightRadius: 0 }}>
                          <button onClick={() => onToggleExpand(idx)} className="text-[13px] text-claude-textSecondary hover:text-claude-text transition-colors">
                            {expandedMessages.has(idx) ? 'Show less' : 'Show more'}
                          </button>
                        </div>
                      );
                    })()}
                  </div>
                )}
                <div className="flex items-center gap-1.5 mt-1.5 pr-1">
                  {msg.created_at && (
                    <span className="text-[12px] text-claude-textSecondary mr-1">{formatMessageTime(msg.created_at)}</span>
                  )}
                  <div className="flex items-center gap-0.5 overflow-hidden transition-all duration-200 ease-in-out max-w-0 opacity-0 group-hover:max-w-[200px] group-hover:opacity-100">
                    <button onClick={() => onResend(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="重新发送"><RotateCcw size={14} /></button>
                    <button onClick={() => onEdit(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="编辑"><Pencil size={14} /></button>
                    <button onClick={() => onCopy(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="复制">
                      {copiedMessageIdx === idx ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
                    </button>
                  </div>
                </div>
              </div>
            )
          ) : (
            <div className="px-1 text-claude-text text-[16.5px] leading-normal mt-2">
              {msg.thinking && (
                <div className="mb-4">
                  <div
                    className="flex items-center gap-2 cursor-pointer select-none group/think text-claude-textSecondary hover:text-claude-text transition-colors"
                    onClick={() => {
                      onSetMessages(prev =>
                        prev.map((m, i) =>
                          i === idx ? { ...m, isThinkingExpanded: !m.isThinkingExpanded } : m
                        )
                      );
                    }}
                  >
                    {msg.isThinking && (
                      <ClaudeLogo autoAnimate style={{ width: '30px', height: '30px' }} />
                    )}
                    <span className={`text-[14px] ${msg.isThinking ? 'animate-shimmer-text' : 'text-claude-textSecondary'}`}>
                      {(() => {
                        if (msg.thinking_summary) return msg.thinking_summary;
                        const text = (msg.thinking || '').trim();
                        const lines = text.split('\n').filter((l: string) => l.trim());
                        const last = lines[lines.length - 1] || '';
                        const summary = last.length > 40 ? last.slice(0, 40) + '...' : last;
                        return summary || 'Thinking...';
                      })()}
                    </span>
                    <ChevronDown size={14} className={`transform transition-transform duration-200 ${msg.isThinkingExpanded ? 'rotate-180' : ''}`} />
                  </div>

                  {msg.isThinkingExpanded && (
                    <div className="mt-2 ml-1 pl-4 border-l-2 border-claude-border">
                      <div className="flex flex-col">
                        <div className="relative">
                          <div
                            className="text-claude-textSecondary text-[14px] leading-normal whitespace-pre-wrap overflow-hidden"
                            style={{ maxHeight: expandedMessages.has(idx) ? 'none' : '300px' }}
                            ref={(el) => { if (el) messageContentRefs.current.set(idx, el); }}
                          >
                            {msg.thinking}
                          </div>
                          {!expandedMessages.has(idx) && (() => {
                            const el = messageContentRefs.current.get(idx);
                            return el && el.scrollHeight > 300;
                          })() && (
                              <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-claude-bg to-transparent pointer-events-none" />
                            )}
                        </div>
                        {(() => {
                          const el = messageContentRefs.current.get(idx);
                          const isOverflow = el && el.scrollHeight > 300;
                          if (!isOverflow) return null;
                          return (
                            <div className="pt-1">
                              <button onClick={() => onToggleExpand(idx)} className="text-[13px] text-claude-text hover:text-claude-textSecondary transition-colors font-medium">
                                {expandedMessages.has(idx) ? 'Show less' : 'Show more'}
                              </button>
                            </div>
                          );
                        })()}
                      </div>
                      {!msg.isThinking && (
                        <div className="flex items-center gap-2 mt-2 text-claude-textSecondary">
                          <Check size={16} />
                          <span className="text-[14px]">Done</span>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )}
              {/* Research badge */}
              {msg.research && (
                <button
                  onClick={() => onOpenResearch && onOpenResearch(msg.id)}
                  className="mb-3 inline-flex items-center gap-2 px-3 py-2 rounded-lg bg-[#DBEAFE] dark:bg-[#1E3A5F] hover:bg-[#BFDBFE] dark:hover:bg-[#2A4A75] transition-colors"
                >
                  {msg.research.completed ? (
                    <IconResearch size={16} className="text-[#2E7CF6]" />
                  ) : (
                    <Loader2 size={16} className="text-[#2E7CF6] animate-spin" />
                  )}
                  <div className="text-left">
                    <div className="text-[12.5px] font-medium text-[#2E7CF6] leading-tight">
                      {msg.research.completed
                        ? `Research complete · ${(msg.research.sources || []).length} sources`
                        : msg.research.phase_label || 'Researching...'}
                    </div>
                    {msg.research.plan?.title && (
                      <div className="text-[11px] text-[#2E7CF6]/70 leading-tight mt-0.5 truncate max-w-[400px]">
                        {msg.research.plan.title}
                      </div>
                    )}
                  </div>
                </button>
              )}
              {/* Tool calls display */}
              {msg.toolCalls && msg.toolCalls.length > 0 && (() => {
                const FRONTEND_HIDDEN = new Set(['WebSearch', 'WebFetch']);
                const visibleToolCalls = msg.toolCalls.filter((tc: any) => !FRONTEND_HIDDEN.has(tc.name));
                if (visibleToolCalls.length === 0) return null;
                const isCurrentMsg = idx === messages.length - 1;
                const isStale = (!loading && isCurrentMsg) || (idx < messages.length - 1);

                // Split text: work text (during tools) vs final text (after last tool done)
                const fullText = extractTextContent(msg.content);
                const offset = msg.toolTextEndOffset;
                const hasOffset = offset && offset > 0 && offset < fullText.length;
                const workText = hasOffset ? fullText.slice(0, offset).trim() : '';
                const finalText = hasOffset ? fullText.slice(offset).trim() : '';
                const isCurrentlyStreaming = loading && idx === messages.length - 1;
                // Tag message for MarkdownRenderer below:
                // - Streaming with tools: show nothing in main area (all text in tool section)
                // - Complete with offset: show only final text
                // - Complete without offset: show full text (fallback)
                // During streaming: compute pending text (text after last tool's textBefore)
                let consumedLen = 0;
                for (const tc of visibleToolCalls) {
                  if (tc.textBefore) consumedLen += tc.textBefore.length;
                }
                // Text currently being typed that hasn't been associated with a tool yet
                const pendingWorkText_ui = isCurrentlyStreaming ? fullText.slice(consumedLen).trim() : '';

                (msg as any)._finalText = isCurrentlyStreaming
                  ? ''  // During streaming, all text goes in tool section
                  : (hasOffset ? finalText : null);

                const toolNames = visibleToolCalls.map((tc: any) => {
                  const nameMap: Record<string, string> = {
                    'Read': 'Read file', 'Write': 'Write file', 'Edit': 'Edit file',
                    'Bash': 'Run command', 'ListDir': 'List directory',
                    'MultiEdit': 'Edit files', 'Search': 'Search',
                  };
                  return nameMap[tc.name] || tc.name;
                });
                const uniqueNames = [...new Set(toolNames)];
                const allDone = visibleToolCalls.every((tc: any) => {
                  const rs = (tc.status === 'running' && isStale) ? 'canceled' : tc.status;
                  return rs !== 'running';
                });
                const hasError = visibleToolCalls.some((tc: any) => tc.status === 'error');
                const summary = uniqueNames.join(', ');

                return (
                  <div className="mb-4">
                    <div className={`rounded-lg overflow-hidden ${!allDone ? 'bg-black/[0.04] dark:bg-white/[0.04]' : ''}`}>
                    <div
                      className="flex items-center gap-2 cursor-pointer select-none group/tool text-claude-textSecondary hover:text-claude-text transition-colors px-2 py-1.5"
                      onClick={() => {
                        onSetMessages(prev =>
                          prev.map((m, i) =>
                            i === idx ? { ...m, isToolCallsExpanded: !m.isToolCallsExpanded } : m
                          )
                        );
                      }}
                    >
                      {!allDone && (
                        <FileText size={16} className="text-claude-textSecondary animate-pulse" />
                      )}
                      {allDone && !hasError && (
                        <Check size={16} className="text-claude-textSecondary" />
                      )}
                      {allDone && hasError && (
                        <span className="text-red-400 text-[14px]">✗</span>
                      )}
                      <span className={`text-[14px] ${!allDone ? 'animate-shimmer-text' : 'text-claude-textSecondary'}`}>
                        {summary}
                      </span>
                      <ChevronDown size={14} className={`transform transition-transform duration-200 ${(msg.isToolCallsExpanded ?? (isCurrentlyStreaming || !allDone)) ? 'rotate-180' : ''}`} />
                    </div>
                    </div>

                    {(msg.isToolCallsExpanded ?? (isCurrentlyStreaming || !allDone)) && (
                      <div className="mt-2 ml-1 pl-4 border-l-2 border-claude-border space-y-2">
                        {visibleToolCalls.map((tc: any, tcIdx: number) => {
                          const inputStr = tc.input ? (typeof tc.input === 'string' ? tc.input : JSON.stringify(tc.input, null, 2)) : '';
                          const rawPath = tc.input?.file_path || tc.input?.path || '';
                          const shortPath = rawPath ? rawPath.split(/[/\\]/).pop() || rawPath : '';
                          const actionLabel: Record<string, string> = {
                            'Read': 'Read', 'Write': 'Write', 'Edit': 'Edit',
                            'MultiEdit': 'Edit', 'Bash': '', 'Grep': 'Search',
                            'Glob': 'Find', 'ListDir': 'List', 'Skill': 'Skill',
                          };
                          const prefix = actionLabel[tc.name] ?? tc.name;
                          const fileOrCmd = shortPath || tc.input?.command || (inputStr.length > 80 ? inputStr.slice(0, 80) + '...' : inputStr);
                          const inputPreview = (prefix && fileOrCmd) ? `${prefix} ${fileOrCmd}` : (fileOrCmd || prefix || tc.name);
                          const realStatus = (tc.status === 'running' && isStale) ? 'canceled' : tc.status;
                          const expandable = hasExpandableContent(tc.name, tc.input, tc.result);
                          const stats = getToolStats(tc.name, tc.input);

                          return (
                            <div key={tc.id || tcIdx}>
                              {/* Interleaved text: what the model said BEFORE this tool call */}
                              {tc.textBefore && (
                                <div className="text-[13px] text-claude-textSecondary px-1 py-1.5 leading-relaxed">
                                  {tc.textBefore}
                                </div>
                              )}
                              {/* Tool card */}
                              <div className="text-[13px] bg-black/5 dark:bg-black/20 rounded-lg overflow-hidden border border-black/5 dark:border-white/5 mx-1 w-full">
                                <div
                                  className={`flex items-center justify-between px-3 py-2 transition-colors ${expandable ? 'cursor-pointer hover:bg-black/5 dark:hover:bg-white/5' : ''}`}
                                  onClick={() => {
                                    if (!expandable) return;
                                    onSetMessages(prev =>
                                      prev.map((m, i) => {
                                        if (i !== idx) return m;
                                        const newTc = [...m.toolCalls];
                                        newTc[tcIdx] = { ...newTc[tcIdx], isExpanded: newTc[tcIdx].isExpanded === undefined ? true : !newTc[tcIdx].isExpanded };
                                        return { ...m, toolCalls: newTc };
                                      })
                                    );
                                  }}
                                >
                                  <div className="flex items-center gap-2 overflow-hidden">
                                    {tc.name === 'Bash' ? (
                                      <span className="text-claude-textSecondary font-mono font-bold">&gt;_</span>
                                    ) : (
                                      <FileText size={14} className="text-claude-textSecondary flex-shrink-0" />
                                    )}
                                    <span className="text-claude-text font-mono text-[12px] truncate">
                                      {inputPreview || tc.name}
                                    </span>
                                  </div>
                                  <div className="flex items-center gap-2 flex-shrink-0 ml-4">
                                    {stats && realStatus !== 'running' && (
                                      <span className="text-[11px] font-mono flex items-center gap-1.5">
                                        {stats.added > 0 && <span className="text-green-500 dark:text-green-400">+{stats.added}</span>}
                                        {stats.removed > 0 && <span className="text-red-500 dark:text-red-400">-{stats.removed}</span>}
                                      </span>
                                    )}
                                    {realStatus === 'running' && <span className="text-claude-textSecondary text-[12px] animate-shimmer-text">Running...</span>}
                                    {realStatus === 'error' && <span className="text-red-400/80 text-[12px]">Failed</span>}
                                    {expandable && (
                                      <ChevronDown size={14} className={`text-claude-textSecondary transform transition-transform duration-200 ${(tc.isExpanded ?? false) ? 'rotate-180' : ''}`} />
                                    )}
                                  </div>
                                </div>
                                {expandable && (tc.isExpanded ?? false) && (
                                  <div className="px-2 py-2 border-t border-black/5 dark:border-white/5">
                                    {shouldUseDiffView(tc.name, tc.input) ? (
                                      <ToolDiffView toolName={tc.name} input={tc.input} result={tc.result} />
                                    ) : tc.result != null ? (
                                      <div className="px-1 text-claude-textSecondary text-[12px] font-mono max-h-[400px] overflow-y-auto whitespace-pre-wrap bg-black/5 dark:bg-black/40 rounded-md p-2">
                                        {typeof tc.result === 'string' ? (tc.result.length > 2000 ? tc.result.slice(0, 2000) + '...' : tc.result || '(Empty output)') : JSON.stringify(tc.result).slice(0, 2000)}
                                      </div>
                                    ) : null}
                                  </div>
                                )}
                              </div>
                            </div>
                          );
                        })}
                        {/* Streaming: show latest text being generated */}
                        {isCurrentlyStreaming && pendingWorkText_ui && (
                          <div className="text-[13px] text-claude-textSecondary px-1 py-1.5 leading-relaxed animate-shimmer-text">
                            {pendingWorkText_ui}
                          </div>
                        )}
                        {allDone && !isCurrentlyStreaming && (
                          <div className="flex items-center gap-2 text-claude-textSecondary pt-1 pb-1">
                            <Check size={14} />
                            <span className="text-[13px]">Done</span>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })()}
              {msg.searchStatus && (!msg.searchLogs || msg.searchLogs.length === 0) && (!msg.content || msg.content.length === (msg._contentLenBeforeSearch || 0)) && loading && idx === messages.length - 1 && (
                <div className="flex items-center justify-center gap-2 text-[15px] font-medium mb-4 w-full">
                  <Globe size={18} className="text-claude-textSecondary" />
                  <span className="animate-shimmer-text">
                    Searching the web
                  </span>
                </div>
              )}

              {msg.searchLogs && msg.searchLogs.length > 0 && (
                <SearchProcess logs={msg.searchLogs} isThinking={msg.isThinking} isDone={(msg.content || '').length > (msg._contentLenBeforeSearch || 0)} />
              )}

              {normalizeDocumentDrafts(msg).length > 0 && (
                <DocumentCreationProcess drafts={normalizeDocumentDrafts(msg)} />
              )}

              <MarkdownRenderer content={(msg as any)._finalText ?? extractTextContent(msg.content)} citations={msg.citations} />
              {normalizeMessageDocuments(msg).length > 0 && (
                <div className="mt-2 mb-1 space-y-2">
                  {normalizeMessageDocuments(msg).map((doc, docIdx) => (
                    <DocumentCard
                      key={doc.id || `${idx}-${docIdx}`}
                      document={doc}
                      onOpen={(openedDoc) => onOpenDocument?.(openedDoc)}
                    />
                  ))}
                </div>
              )}
              {msg.codeExecution && (
                <CodeExecution
                  code={msg.codeExecution.code}
                  status={msg.codeExecution.status}
                  stdout={msg.codeExecution.stdout}
                  stderr={msg.codeExecution.stderr}
                  images={msg.codeExecution.images}
                  error={msg.codeExecution.error}
                />
              )}
              {!msg.codeExecution && (msg as any).codeImages && (msg as any).codeImages.length > 0 && (
                <div className="my-3 space-y-2">
                  {(msg as any).codeImages.map((url: string, i: number) => (
                    <div key={i} className="rounded-lg overflow-hidden">
                      <img src={withAuthToken(url)} alt={`图表 ${i + 1}`} className="max-w-full" />
                    </div>
                  ))}
                </div>
              )}
              {loading && idx === messages.length - 1 && !msg.content && !msg.thinking && !msg.searchStatus && normalizeDocumentDrafts(msg).length === 0 && !(msg.toolCalls && msg.toolCalls.length > 0) && (
                <span className="inline-block ml-1 align-middle" style={{ verticalAlign: 'middle' }}>
                  <ClaudeLogo breathe style={{ width: '40px', height: '40px', display: 'inline-block' }} />
                </span>
              )}
              {loading && idx === messages.length - 1 && !msg.isThinking && (msg.content || (msg.searchStatus && msg.content)) && (
                <span className="inline-block ml-1 align-middle" style={{ verticalAlign: 'middle' }}>
                  <ClaudeLogo autoAnimate style={{ width: '40px', height: '40px', display: 'inline-block' }} />
                </span>
              )}
              {!loading && idx === messages.length - 1 && msg.content && (
                <div className="flex items-start gap-4 mt-6 ml-1 mb-2">
                  <ClaudeLogo breathe={compactStatus.state === 'compacting'} style={{ width: '36px', height: '36px', flexShrink: 0, marginTop: '2px' }} />
                  {compactStatus.state === 'compacting' && <CompactingStatus />}
                </div>
              )}
            </div>
          )}
        </div>
      ))}
    </>
  );
});

const MainContent = ({ onNewChat, resetKey, tunerConfig, onOpenDocument, onArtifactsUpdate, onOpenArtifacts, onTitleChange, onChatModeChange }: MainContentProps) => {
  const { id } = useParams(); // Get conversation ID from URL
  const location = useLocation();
  const [localId, setLocalId] = useState<string | null>(null);
  const [showEntranceAnimation, setShowEntranceAnimation] = useState(false);

  // Use localId if we just created a chat, effectively overriding the lack of URL param until next true navigation
  const activeId = id || localId || null;

  const navigate = useNavigate();
  const [inputText, setInputText] = useState("");
  const [messages, setMessages] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  // Notify parent about artifacts
  useEffect(() => {
    if (onArtifactsUpdate) {
      const docsMap = new Map<string, DocumentInfo>();
      for (const message of messages) {
        for (const doc of normalizeMessageDocuments(message)) {
          const key = doc.id || doc.url || doc.filename || doc.title;
          if (!key) continue;
          docsMap.set(key, doc);
        }
      }
      const docs = Array.from(docsMap.values());
      onArtifactsUpdate(docs);
    }
  }, [messages, onArtifactsUpdate]);

  // Notify parent about Chat Mode and Title
  useEffect(() => {
    const isChat = !!(activeId || messages.length > 0);
    onChatModeChange?.(isChat);
  }, [activeId, messages.length, onChatModeChange]);


  // Model state
  const [modelCatalog, setModelCatalog] = useState<ModelCatalog | null>(null);
  const isSelfHostedMode = localStorage.getItem('user_mode') === 'selfhosted';

  // Self-hosted: read chat_models from localStorage synchronously to avoid flash of wrong models
  const selfHostedModels = useMemo<SelectableModel[]>(() => {
    if (!isSelfHostedMode) return [];
    try {
      const chatModels = JSON.parse(localStorage.getItem('chat_models') || '[]');
      if (chatModels.length === 0) return [];
      const tierDescMap: Record<string, string> = {
        'opus': 'Most capable for ambitious work',
        'sonnet': 'Most efficient for everyday tasks',
        'haiku': 'Fastest for quick answers',
      };
      return chatModels.map((m: any) => ({
        id: m.id,
        name: m.name || m.id,
        enabled: 1,
        tier: m.tier || 'extra',
        description: m.tier && tierDescMap[m.tier] ? tierDescMap[m.tier] : undefined,
      }));
    } catch { return []; }
  }, [isSelfHostedMode]);

  const fallbackCommonModels = useMemo<SelectableModel[]>(() => {
    // Self-hosted: use user-configured models as fallback, not hardcoded Claude models
    if (isSelfHostedMode && selfHostedModels.length > 0) {
      const tierOrder = ['opus', 'sonnet', 'haiku'];
      const common = tierOrder.map(t => selfHostedModels.find(m => m.tier === t)).filter(Boolean) as SelectableModel[];
      return common.length > 0 ? common : selfHostedModels;
    }
    return [
      { id: 'claude-opus-4-6', name: 'Opus 4.6', enabled: 1, description: 'Most capable for ambitious work' },
      { id: 'claude-sonnet-4-6', name: 'Sonnet 4.6', enabled: 1, description: 'Most efficient for everyday tasks' },
      { id: 'claude-haiku-4-5-20251001', name: 'Haiku 4.5', enabled: 1, description: 'Fastest for quick answers' },
    ];
  }, [isSelfHostedMode, selfHostedModels]);

  const displayCommonModels = modelCatalog?.common?.length ? modelCatalog.common : fallbackCommonModels;
  const selectorModels = useMemo<SelectableModel[]>(() => {
    const visible = [...displayCommonModels];
    // Only add extra models (e.g. GPT) for self-hosted mode
    if (isSelfHostedMode) {
      const seen = new Set(visible.map(m => m.id));
      const extraModels = (modelCatalog?.all || []).filter(m => !seen.has(m.id));
      for (const model of extraModels) {
        // Tag non-tier models as 'extra' so ModelSelector can split them into "More models"
        visible.push({ ...model, tier: model.tier || 'extra' });
        seen.add(model.id);
      }
    }
    return visible;
  }, [displayCommonModels, modelCatalog, isSelfHostedMode]);

  // Initial model: for self-hosted, prefer first configured model over hardcoded claude-sonnet-4-6
  const [currentModelString, setCurrentModelString] = useState(() => {
    const saved = localStorage.getItem('default_model');
    if (saved) return saved;
    if (isSelfHostedMode && selfHostedModels.length > 0) return selfHostedModels[0].id;
    return 'claude-sonnet-4-6';
  });
  const [conversationTitle, setConversationTitle] = useState("");
  // Cross-mode warning: when an existing conversation's model isn't available in the
  // current user_mode (e.g. user opened a clawparrot-opus chat after switching to
  // selfhosted), we delay falling back. The first send attempt triggers a modal so
  // the user explicitly picks "keep cross-mode" or "switch to current mode model".
  const [crossModeWarning, setCrossModeWarning] = useState<{
    convId: string;
    originalModel: string;
    otherMode: 'clawparrot' | 'selfhosted';
    fallbackModel: string;
  } | null>(null);
  // After user picks "switch model", we proceed to send. Stash the pending send args
  // here so the modal callback can fire them after dismissal.
  const pendingCrossModeSendRef = useRef<(() => void) | null>(null);
  // Clawparrot login-required gate: shown when an un-logged-in clawparrot user
  // tries to send their first message.
  const [showLoginRequired, setShowLoginRequired] = useState(false);
  const pendingLoginSendRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    onTitleChange?.(conversationTitle);
  }, [conversationTitle, onTitleChange]);

  const [user, setUser] = useState<any>(null);

  // Welcome greeting — randomized per new chat, time-aware
  const welcomeGreeting = useMemo(() => {
    const hour = new Date().getHours();
    const name = user?.display_name || user?.nickname || 'there';
    const timeGreetings = hour < 6
      ? [`Night owl mode, ${name}`, `Burning the midnight oil, ${name}?`, `Still up, ${name}?`]
      : hour < 12
        ? [`Good morning, ${name}`, `Morning, ${name}`, `Rise and shine, ${name}`]
        : hour < 18
          ? [`Good afternoon, ${name}`, `Hey there, ${name}`, `What's on your mind, ${name}?`]
          : [`Good evening, ${name}`, `Evening, ${name}`, `Winding down, ${name}?`];
    const general = [`What can I help with?`, `How can I help you today?`, `Let's get to work, ${name}`, `Ready when you are, ${name}`];
    const pool = [...timeGreetings, ...general];
    return pool[Math.floor(Math.random() * pool.length)];
  }, [resetKey, user?.nickname]);

  // 输入栏参数
  const inputBarWidth = 768;
  const inputBarMinHeight = 32;
  const inputBarRadius = 22;
  const inputBarBottom = 0;
  const inputBarBaseHeight = inputBarMinHeight + 16; // border-box: content + padding (pt-4=16px + pb-0=0px)
  const textareaHeightVal = useRef(inputBarBaseHeight);

  const isCreatingRef = useRef(false);
  const pendingInitialMessageRef = useRef<string | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);
  const activeRequestCountRef = useRef(0);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastResetKeyRef = useRef(0);
  const streamConversationIdRef = useRef<string | null>(null);
  const streamRequestIdRef = useRef(0);

  // Per-conversation message buffer for multi-conversation streaming isolation
  const viewingIdRef = useRef<string | null>(null);
  const messagesBufferRef = useRef(new Map<string, any[]>());

  // Update messages for a specific conversation — only touches React state if it's the active conversation.
  //
  // Backfill safety net: setMessagesFor is called exclusively from streaming SSE event
  // handlers (text deltas, thinking deltas, tool events, done/error callbacks). They all
  // mutate the trailing assistant placeholder. If a race causes the updater to run BEFORE
  // the placeholder push has committed (rare but real — depends on React batching, async
  // boundaries, and SSE chunk timing), the original updaters silently dropped the event
  // via their `lastMsg.role === 'assistant'` guard.
  //
  // The fix: ensure the tail of `prev` is an assistant message before invoking the
  // updater. Existing callers don't change — their guard now always passes, and the
  // event lands on the backfilled placeholder. The bridge will overwrite this placeholder
  // with the canonical message + toolCalls when finishTurn flushes to db, so even if a
  // re-load races with backfill the persistent state stays correct.
  const setMessagesFor = useCallback((convId: string, updater: (prev: any[]) => any[]) => {
    const ensureUpdater = (prev: any[]) => {
      if (prev.length === 0) return updater(prev); // empty conv: don't synthesize a phantom placeholder
      const last = prev[prev.length - 1];
      if (last && last.role === 'assistant') return updater(prev);
      // Tail is a user message (or other non-assistant). Backfill an assistant
      // placeholder so the trailing SSE event has somewhere to land instead of
      // being silently dropped by the updater's `lastMsg.role === 'assistant'` guard.
      return updater([...prev, { role: 'assistant', content: '' }]);
    };

    if (viewingIdRef.current === convId) {
      setMessages(prev => {
        const result = ensureUpdater(prev);
        messagesBufferRef.current.set(convId, result);
        return result;
      });
    } else {
      const prev = messagesBufferRef.current.get(convId) || [];
      messagesBufferRef.current.set(convId, ensureUpdater(prev));
    }
  }, []);

  const isModelSelectable = useCallback((modelString: string) => {
    const base = stripThinking(modelString);
    const pool = modelCatalog?.all || displayCommonModels;
    const found = pool.find(m => m.id === base);
    return !!found && Number(found.enabled) === 1;
  }, [modelCatalog, displayCommonModels]);

  const resolveModelForNewChat = useCallback((preferredModel?: string | null) => {
    const saved = preferredModel || localStorage.getItem('default_model') || 'claude-sonnet-4-6';
    const thinking = isThinkingModel(saved);
    const base = stripThinking(saved);
    const all = modelCatalog?.all || displayCommonModels;
    const preferred = all.find(m => m.id === base);
    if (preferred && Number(preferred.enabled) === 1) {
      return withThinking(base, thinking);
    }

    const fallbackBase = modelCatalog?.fallback_model
      || all.find(m => /sonnet/i.test(m.id) && Number(m.enabled) === 1)?.id
      || all.find(m => Number(m.enabled) === 1)?.id
      || base
      || 'claude-sonnet-4-6';
    return withThinking(fallbackBase, thinking);
  }, [displayCommonModels, modelCatalog]);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);
  const userScrolledUpRef = useRef(false);
  const [scrollbarWidth, setScrollbarWidth] = useState(0);
  const [expandedMessages, setExpandedMessages] = useState<Set<number>>(new Set());
  const [copiedMessageIdx, setCopiedMessageIdx] = useState<number | null>(null);
  const [editingMessageIdx, setEditingMessageIdx] = useState<number | null>(null);
  const [editingContent, setEditingContent] = useState('');
  const messageContentRefs = useRef<Map<number, HTMLDivElement>>(new Map());
  const [inputHeight, setInputHeight] = useState(160);
  const inputWrapperRef = useRef<HTMLDivElement>(null);
  const [pendingFiles, setPendingFiles] = useState<PendingFile[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [showPlusMenu, setShowPlusMenu] = useState(false);
  const [researchMode, setResearchMode] = useState(false);
  const [openedResearchMsgId, setOpenedResearchMsgId] = useState<string | null>(null);
  const toggleResearchMode = useCallback(async () => {
    const next = !researchMode;
    setResearchMode(next);
    if (activeId) {
      try { await updateConversation(activeId, { research_mode: next }); } catch (_) {}
    }
  }, [researchMode, activeId]);

  // Provider web-search capability (derived from the current model's provider).
  // The bridge strips web_search_20250305 automatically when the provider doesn't support it,
  // so this state is purely a UI indicator — no need to persist or toggle.
  const [providersCache, setProvidersCache] = useState<Provider[]>([]);
  const [webSearchToast, setWebSearchToast] = useState<string | null>(null);
  useEffect(() => {
    getProviders().then(setProvidersCache).catch(() => {});
  }, []);
  const currentProviderSupportsWebSearch = useMemo(() => {
    if (!providersCache.length) return false;
    const bareModel = (currentModelString || '').replace(/-thinking$/, '');
    for (const p of providersCache) {
      if ((p.models || []).some(m => m.id === bareModel)) {
        return p.supportsWebSearch === true;
      }
    }
    return false;
  }, [providersCache, currentModelString]);
  useEffect(() => {
    if (!webSearchToast) return;
    const t = setTimeout(() => setWebSearchToast(null), 2800);
    return () => clearTimeout(t);
  }, [webSearchToast]);
  const [showSkillsSubmenu, setShowSkillsSubmenu] = useState(false);
  const [enabledSkills, setEnabledSkills] = useState<Array<{ id: string; name: string; description?: string }>>([]);
  const [selectedSkill, setSelectedSkill] = useState<{ name: string; slug: string; description?: string } | null>(null);
  const plusMenuRef = useRef<HTMLDivElement>(null);
  const plusBtnRef = useRef<HTMLButtonElement>(null);
  // Add-to-project state
  const [showProjectsSubmenu, setShowProjectsSubmenu] = useState(false);
  const [projectList, setProjectList] = useState<Project[]>([]);
  const [currentProjectId, setCurrentProjectId] = useState<string | null>(null);
  const [pendingProjectId, setPendingProjectId] = useState<string | null>(null);
  const [showNewProjectDialog, setShowNewProjectDialog] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [newProjectDescription, setNewProjectDescription] = useState('');
  const [projectAddToast, setProjectAddToast] = useState<string | null>(null);
  const [compactStatus, setCompactStatus] = useState<{ state: 'idle' | 'compacting' | 'done' | 'error'; message?: string }>({ state: 'idle' });
  const [showCompactDialog, setShowCompactDialog] = useState(false);
  const [compactInstruction, setCompactInstruction] = useState('');
  const [showGithubModal, setShowGithubModal] = useState(false);
  const [hasSubscription, setHasSubscription] = useState<boolean | null>(null); // null = loading
  const [contextInfo, setContextInfo] = useState<{ tokens: number; limit: number } | null>(null);

  // AskUserQuestion state
  const [askUserDialog, setAskUserDialog] = useState<{
    request_id: string;
    tool_use_id: string;
    questions: Array<{ question: string; header?: string; options?: Array<{ label: string; description?: string }>; multiSelect?: boolean }>;
    answers: Record<string, string>;
  } | null>(null);

  // Task/Agent progress state
  const [activeTasks, setActiveTasks] = useState<Map<string, { description: string; status?: string; summary?: string; last_tool_name?: string }>>(new Map());

  // Plan mode state
  const [planMode, setPlanMode] = useState(false);

  // 草稿持久化 refs（跟踪最新值，供 effect cleanup 读取）
  const inputTextRef = useRef(inputText);
  inputTextRef.current = inputText;
  const pendingFilesRef = useRef(pendingFiles);
  pendingFilesRef.current = pendingFiles;
  const textareaHeightRef = useRef(textareaHeightVal.current);
  textareaHeightRef.current = textareaHeightVal.current;

  // textarea 高度计算改为在 onChange 中直接操作 DOM（见 adjustTextareaHeight）
  const adjustTextareaHeight = useCallback(() => {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = `${inputBarBaseHeight}px`;
    const sh = el.scrollHeight;
    const newH = sh > inputBarBaseHeight ? Math.min(sh, 316) : inputBarBaseHeight;
    el.style.height = `${newH}px`;
    el.style.overflowY = newH >= 316 ? 'auto' : 'hidden';
    textareaHeightVal.current = newH;
  }, [inputBarBaseHeight]);

  useEffect(() => {
    // If we have a URL param ID, clear any local ID to ensure we sync with source of truth
    if (id) {
      setLocalId(null);
    }
  }, [id]);

  // 检测滚动条宽度
  useEffect(() => {
    const el = scrollContainerRef.current;
    if (!el) return;
    const update = () => setScrollbarWidth(el.offsetWidth - el.clientWidth);
    update();
    const observer = new ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, [messages]);

  // 动态调整 paddingBottom，使聊天列表能滚到输入框上方
  useEffect(() => {
    const el = inputWrapperRef.current;
    if (!el) return;

    const updateHeight = () => {
      // 底部留白 = 输入框高度 + 底部边距(48px)
      setInputHeight(el.offsetHeight + 48);
    };

    // 初始测量
    updateHeight();

    const observer = new ResizeObserver(updateHeight);
    observer.observe(el);

    return () => observer.disconnect();
  }, [activeId, messages.length]);

  // 用户滚轮向上时，立刻中止自动滚动
  useEffect(() => {
    const el = scrollContainerRef.current;
    if (!el) return;
    const handleWheel = (e: WheelEvent) => {
      if (e.deltaY < 0) {
        userScrolledUpRef.current = true;
        isAtBottomRef.current = false;
        // 取消正在进行的 smooth scroll 动画
        el.scrollTo({ top: el.scrollTop });
      }
    };
    el.addEventListener('wheel', handleWheel, { passive: true });
    return () => el.removeEventListener('wheel', handleWheel);
  }, []);

  // Load enabled skills for the plus menu
  useEffect(() => {
    if (!showPlusMenu) { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); return; }
    getSkills().then((data: any) => {
      const all = [...(data.examples || []), ...(data.my_skills || [])];
      setEnabledSkills(all.filter((s: any) => s.enabled).map((s: any) => ({ id: s.id, name: s.name, description: s.description })));
    }).catch(() => {});
    getProjects().then((data: Project[]) => {
      setProjectList((data || []).filter(p => !p.is_archived));
    }).catch(() => {});
  }, [showPlusMenu]);

  // 点击外部关闭加号菜单
  useEffect(() => {
    if (!showPlusMenu) return;
    const handleClick = (e: MouseEvent) => {
      const target = e.target as Node;
      const insideMenu = plusMenuRef.current && plusMenuRef.current.contains(target);
      const insideButton = plusBtnRef.current && plusBtnRef.current.contains(target);
      if (!insideMenu && !insideButton) {
        setShowPlusMenu(false);
        setShowSkillsSubmenu(false);
        setShowProjectsSubmenu(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [showPlusMenu]);

  // Reset when resetKey changes (New Chat clicked)
  useEffect(() => {
    if (resetKey && resetKey !== lastResetKeyRef.current) {
      lastResetKeyRef.current = resetKey;
      setLocalId(null);
      setMessages([]);
      setCurrentModelString(resolveModelForNewChat());
      setConversationTitle("");
      setContextInfo(null);
      setCurrentProjectId(null);
      setPendingProjectId(null);
      // 触发入场动画
      setShowEntranceAnimation(true);
      setTimeout(() => setShowEntranceAnimation(false), 800);
      isAtBottomRef.current = true;

      // Check for prefill input (from Create with Claude)
      const prefillInput = sessionStorage.getItem('prefill_input');
      if (prefillInput) {
        sessionStorage.removeItem('prefill_input');
        setTimeout(() => {
          setInputText(prefillInput);
          // Auto-resize textarea
          const ta = document.querySelector('textarea');
          if (ta) {
            ta.style.height = 'auto';
            ta.style.height = Math.min(ta.scrollHeight, 316) + 'px';
          }
        }, 200);
      }

      // Check for artifact prompt (from Artifacts page)
      const artifactPrompt = sessionStorage.getItem('artifact_prompt');
      if (artifactPrompt) {
        sessionStorage.removeItem('artifact_prompt');
        if (artifactPrompt === '__remix__') {
          // Remix mode: pre-load artifact into conversation
          const remixData = sessionStorage.getItem('artifact_remix');
          sessionStorage.removeItem('artifact_remix');
          if (remixData) {
            try {
              const remix = JSON.parse(remixData);
              // Inject pre-baked assistant message with artifact info
              const assistantMsg = {
                id: 'remix-' + Date.now(),
                role: 'assistant' as const,
                content: JSON.stringify([{ type: 'text', text: `I'll customize this artifact:\n\n**${remix.name}**\n\nTransform any artifact into something uniquely yours by customizing its core elements.\n\n1. Change the topic - Adapt the content for a different subject\n2. Update the style - Refresh the visuals or overall design\n3. Make it personal - Tailor specifically for your needs\n4. Share your vision - I'll bring it to life\n\nWhere would you like to begin?` }]),
                created_at: new Date().toISOString(),
              };
              setTimeout(() => {
                setMessages([assistantMsg]);
                // Open the artifact in DocumentPanel
                if (remix.code?.content && onOpenDocument) {
                  const isReactArtifact = remix.code?.type === 'application/vnd.ant.react';
                  onOpenDocument({
                    id: 'remix-artifact',
                    title: remix.code?.title || remix.name,
                    filename: (remix.code?.title || remix.name) + (isReactArtifact ? '.jsx' : '.html'),
                    url: '',
                    content: remix.code.content,
                    format: isReactArtifact ? 'jsx' : 'html',
                  });
                }
              }, 200);
            } catch {}
          }
        } else {
          // Normal artifact prompt: auto-send
          setTimeout(() => handleSend(artifactPrompt), 300);
        }
      }
    }
  }, [resetKey, resolveModelForNewChat]);

  useEffect(() => {
    let cancelled = false;
    const isSelfHosted = localStorage.getItem('user_mode') === 'selfhosted';
    const loadModels = async () => {
      try {
        let data: any;
        if (isSelfHosted) {
          // Self-hosted: use chat_models from localStorage (configured in Models settings)
          let chatModels: any[] = [];
          try { chatModels = JSON.parse(localStorage.getItem('chat_models') || '[]'); } catch {}
          if (chatModels.length > 0) {
            const tierDescMap: Record<string, string> = {
              'opus': 'Most capable for ambitious work',
              'sonnet': 'Most efficient for everyday tasks',
              'haiku': 'Fastest for quick answers',
            };
            const all = chatModels.map((m: any) => ({
              id: m.id,
              name: m.name || m.id,
              enabled: 1,
              tier: m.tier || 'extra',
              description: m.tier && tierDescMap[m.tier] ? tierDescMap[m.tier] : undefined,
            }));
            // Common = tier models (opus/sonnet/haiku), ordered by tier
            const tierOrder = ['opus', 'sonnet', 'haiku'];
            const common = tierOrder.map(t => all.find((m: any) => m.tier === t)).filter(Boolean);
            data = { all, common: common.length > 0 ? common : all, fallback_model: localStorage.getItem('default_model') || all[0]?.id || 'claude-sonnet-4-6' };
          } else {
            // Fallback: load all from providers
            const pModels = await getProviderModels();
            const all = pModels.map(m => ({ id: m.id, name: m.name || m.id, enabled: 1 }));
            data = { all, common: all, fallback_model: all[0]?.id || 'claude-sonnet-4-6' };
          }
        } else {
          data = await getUserModels();
          // Enrich known Anthropic models with descriptions
          const descMap: Record<string, string> = {
            'claude-opus-4-6': 'Most capable for ambitious work',
            'claude-sonnet-4-6': 'Most efficient for everyday tasks',
            'claude-haiku-4-5-20251001': 'Fastest for quick answers',
          };
          for (const list of [data?.common, data?.all]) {
            if (!Array.isArray(list)) continue;
            for (const m of list) {
              if (descMap[m.id] && !m.description) m.description = descMap[m.id];
            }
          }
        }
        if (cancelled) return;
        setModelCatalog(data);
        if (!viewingIdRef.current) {
          setCurrentModelString(prev => {
            const current = prev || localStorage.getItem('default_model') || 'claude-sonnet-4-6';
            const thinking = isThinkingModel(current);
            const base = stripThinking(current);
            const all: SelectableModel[] = data?.all?.length ? data.all : fallbackCommonModels;
            const preferred = all.find((m: SelectableModel) => m.id === base && Number(m.enabled) === 1);
            if (preferred) return withThinking(base, thinking);
            const fallbackBase = data?.fallback_model
              || all.find((m: SelectableModel) => /sonnet/i.test(m.id) && Number(m.enabled) === 1)?.id
              || all.find((m: SelectableModel) => Number(m.enabled) === 1)?.id
              || base
              || 'claude-sonnet-4-6';
            return withThinking(fallbackBase, thinking);
          });
        }
      } catch {
        // ignore
      }
    };
    loadModels();
    const timer = setInterval(loadModels, 60000);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fallbackCommonModels]);

  // 草稿持久化：切换对话 / 打开设置页面时保存，切回时恢复
  const draftKey = activeId || '__new__';
  useEffect(() => {
    const saved = draftsStore.get(draftKey);
    if (saved) {
      setInputText(saved.text);
      setPendingFiles(saved.files);
      textareaHeightVal.current = saved.height;
      // Apply saved height to DOM
      if (inputRef.current) {
        inputRef.current.style.height = `${saved.height}px`;
        inputRef.current.style.overflowY = saved.height >= 316 ? 'auto' : 'hidden';
      }
      draftsStore.delete(draftKey);
    } else {
      setInputText('');
      setPendingFiles([]);
      textareaHeightVal.current = inputBarBaseHeight;
    }
    return () => {
      const text = inputTextRef.current;
      const files = pendingFilesRef.current;
      const height = textareaHeightRef.current;
      if (text.trim() || files.length > 0) {
        draftsStore.set(draftKey, { text, files, height });
      } else {
        draftsStore.delete(draftKey);
      }
    };
  }, [draftKey]);

  // 路由变化时也触发入场动画
  useEffect(() => {
    if (location.pathname === '/' || location.pathname === '') {
      setShowEntranceAnimation(true);
      setTimeout(() => setShowEntranceAnimation(false), 800);
    }
  }, [location.pathname]);

  useEffect(() => {
    setUser(getUser());
    // Check subscription status
    getUserUsage().then(usage => {
      const hasSub = !!(usage.plan && usage.plan.status === 'active');
      const hasQuota = usage.token_quota > 0 && usage.token_remaining > 0;
      setHasSubscription(hasSub || hasQuota);
    }).catch(() => setHasSubscription(false));
  }, [activeId]);

  useEffect(() => {
    // Reset state when switching conversations — each conversation has independent streaming
    setPlanMode(false);
    setActiveTasks(new Map());
    setAskUserDialog(null);
    isCreatingRef.current = false;
    viewingIdRef.current = activeId || null;

    // Pre-warm engine when user opens a conversation (init in background before they send)
    if (activeId) warmEngine(activeId);

    if (activeId) {
      // Check if there's a live buffer for this conversation (e.g. streaming in background)
      const buffered = messagesBufferRef.current.get(activeId);
      if (buffered && buffered.length > 0) {
        setMessages(buffered);
        setLoading(isStreaming(activeId));
        // Restore model from server even when using buffer for messages
        const buffConvId = activeId;
        getConversation(buffConvId).then(data => {
          if (data?.model && viewingIdRef.current === buffConvId) {
            setCurrentModelString(isModelSelectable(data.model) ? data.model : resolveModelForNewChat(data.model));
          }
        }).catch(() => {});
      } else {
        setLoading(false);
        loadConversation(activeId);
        // Check if server has an active stream we can reconnect to
        const convId = activeId;
        getStreamStatus(convId).then(status => {
          if (status.active && viewingIdRef.current === convId) {
            setLoading(true);
            addStreaming(convId);
            // Seed buffer from current messages + placeholder
            setMessages(prev => {
              const msgs = prev.length > 0 ? prev : [];
              // Add assistant placeholder if last message isn't one
              if (msgs.length === 0 || msgs[msgs.length - 1].role !== 'assistant') {
                const withPlaceholder = [...msgs, { role: 'assistant', content: '' }];
                messagesBufferRef.current.set(convId, withPlaceholder);
                return withPlaceholder;
              }
              messagesBufferRef.current.set(convId, msgs);
              return msgs;
            });
            const reconnectController = new AbortController();
            abortControllerRef.current = reconnectController;
            reconnectStream(
              convId,
              (delta, full) => {
                setMessagesFor(convId, prev => {
                  const newMsgs = [...prev];
                  const lastMsg = newMsgs[newMsgs.length - 1];
                  if (lastMsg && lastMsg.role === 'assistant') { lastMsg.content = full; lastMsg.isThinking = false; }
                  return newMsgs;
                });
              },
              (full) => {
                removeStreaming(convId);
                messagesBufferRef.current.delete(convId);
                if (viewingIdRef.current === convId) setLoading(false);
                abortControllerRef.current = null;
                setMessagesFor(convId, prev => {
                  const newMsgs = [...prev];
                  const lastMsg = newMsgs[newMsgs.length - 1];
                  if (lastMsg && lastMsg.role === 'assistant') { lastMsg.content = full; lastMsg.isThinking = false; }
                  return newMsgs;
                });
              },
              (err) => {
                removeStreaming(convId);
                messagesBufferRef.current.delete(convId);
                if (viewingIdRef.current === convId) setLoading(false);
                abortControllerRef.current = null;
              },
              (thinkingDelta, thinkingFull) => {
                setMessagesFor(convId, prev => {
                  const newMsgs = [...prev];
                  const lastMsg = newMsgs[newMsgs.length - 1];
                  if (lastMsg && lastMsg.role === 'assistant') { lastMsg.thinking = thinkingFull; lastMsg.isThinking = true; }
                  return newMsgs;
                });
              },
              (event, message, data) => {
                if (event === 'ask_user' && data) {
                  setAskUserDialog({ request_id: data.request_id, tool_use_id: data.tool_use_id, questions: data.questions || [], answers: {} });
                }
                if (event === 'task_event' && data) {
                  setActiveTasks(prev => {
                    const next = new Map(prev);
                    if (data.subtype === 'task_started') next.set(data.task_id, { description: data.description || 'Running task...' });
                    else if (data.subtype === 'task_progress') { const e = next.get(data.task_id); if (e) next.set(data.task_id, { ...e, last_tool_name: data.last_tool_name }); }
                    else if (data.subtype === 'task_notification') next.delete(data.task_id);
                    return next;
                  });
                }
              },
              (toolEvent) => {
                if (toolEvent.type === 'done' && toolEvent.tool_name === 'EnterPlanMode') setPlanMode(true);
                if (toolEvent.type === 'done' && toolEvent.tool_name === 'ExitPlanMode') setPlanMode(false);
                const INTERNAL_TOOLS = new Set(['EnterPlanMode', 'ExitPlanMode', 'TaskCreate', 'TaskUpdate', 'TaskGet', 'TaskList', 'TaskOutput', 'TaskStop']);
                if (INTERNAL_TOOLS.has(toolEvent.tool_name || '')) return;
                setMessagesFor(convId, prev => {
                  const newMsgs = [...prev];
                  const lastMsg = newMsgs[newMsgs.length - 1];
                  if (!lastMsg || lastMsg.role !== 'assistant') return prev;
                  const toolCalls = lastMsg.toolCalls || [];
                  if (toolEvent.type === 'start') {
                    let existing = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
                    if (existing) {
                      existing.name = toolEvent.tool_name || existing.name;
                      if (toolEvent.tool_input && Object.keys(toolEvent.tool_input).length > 0) existing.input = toolEvent.tool_input;
                      if (toolEvent.textBefore) existing.textBefore = toolEvent.textBefore;
                    } else {
                      toolCalls.push({ id: toolEvent.tool_use_id, name: toolEvent.tool_name || 'unknown', input: toolEvent.tool_input || {}, status: 'running' as const, textBefore: toolEvent.textBefore || '' });
                    }
                  }
                  else if (toolEvent.type === 'input') {
                    const tc = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
                    if (tc) tc.input = toolEvent.tool_input || {};
                  }
                  else if (toolEvent.type === 'done') {
                    let tc = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
                    if (!tc) { tc = { id: toolEvent.tool_use_id, name: toolEvent.tool_name || 'unknown', input: {}, status: 'done' as const, result: toolEvent.content }; toolCalls.push(tc); }
                    else { tc.status = toolEvent.is_error ? 'error' as const : 'done' as const; tc.result = toolEvent.content; }
                  }
                  lastMsg.toolCalls = toolCalls;
                  return newMsgs;
                });
              },
              reconnectController.signal
            );
          }
        }).catch(() => {});
      }
      getContextSize(activeId).then(setContextInfo).catch(() => { });
      isAtBottomRef.current = true;

      // Handle initialMessage from Project page navigation
      const navState = location.state as any;
      if (navState?.initialMessage) {
        pendingInitialMessageRef.current = navState.initialMessage;
        if (navState.model) setCurrentModelString(navState.model);
        // Clear location state to prevent re-sends on refresh
        navigate(location.pathname, { replace: true, state: {} });
      }
      return;
    }

    setLoading(false);
    setMessages([]);
    setContextInfo(null);
    setCurrentModelString(resolveModelForNewChat());
  }, [activeId]);

  const stopPolling = useCallback(() => {
    if (pollingRef.current) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
  }, []);

  const beginStreamSession = useCallback((conversationId: string) => {
    const nextId = streamRequestIdRef.current + 1;
    streamRequestIdRef.current = nextId;
    streamConversationIdRef.current = conversationId;
    return nextId;
  }, []);

  const isStreamSessionActive = useCallback((conversationId: string, requestId: number) => {
    return streamConversationIdRef.current === conversationId && streamRequestIdRef.current === requestId;
  }, []);

  const clearStreamSession = useCallback((conversationId: string, requestId: number) => {
    if (!isStreamSessionActive(conversationId, requestId)) return false;
    streamConversationIdRef.current = null;
    return true;
  }, [isStreamSessionActive]);

  const abortStreamSession = useCallback((targetConversationId?: string) => {
    const trackedConversationId = streamConversationIdRef.current;
    if (!trackedConversationId) return false;
    if (targetConversationId && trackedConversationId !== targetConversationId) return false;

    streamRequestIdRef.current += 1;
    streamConversationIdRef.current = null;

    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
      activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
    } else if (pollingRef.current) {
      stopPolling();
      stopGeneration(trackedConversationId).catch(e => console.error('[Stop] error:', e));
    }

    removeStreaming(trackedConversationId);
    setLoading(false);
    isCreatingRef.current = false;
    return true;
  }, [stopPolling]);

  // 组件卸载或对话切换时停止轮询
  useEffect(() => {
    return () => { stopPolling(); };
  }, [activeId, stopPolling]);

  // 对话删除前先中止流式请求，避免旧会话的输出串到当前界面
  useEffect(() => {
    const handleConversationDeleting = (evt: Event) => {
      const customEvt = evt as CustomEvent<{ id?: string }>;
      const conversationId = customEvt.detail?.id;
      if (!conversationId) return;
      abortStreamSession(conversationId);
    };

    window.addEventListener('conversationDeleting', handleConversationDeleting as EventListener);
    return () => {
      window.removeEventListener('conversationDeleting', handleConversationDeleting as EventListener);
    };
  }, [abortStreamSession]);

  useEffect(() => {
    // 只在加载中（模型正在生成）或用户刚发送消息时才自动滚动
    // 对话结束后不要自动滚动，避免用户正在查看历史消息时被打断
    if (isAtBottomRef.current && loading && !userScrolledUpRef.current) {
      scrollToBottom('auto');
    }
  }, [messages, loading]);

  // 当输入框高度变化时，如果已经在底部，则保持在底部
  useEffect(() => {
    if (isAtBottomRef.current && scrollContainerRef.current) {
      scrollContainerRef.current.scrollTop = scrollContainerRef.current.scrollHeight;
    }
  }, [inputHeight]);

  const handleScroll = () => {
    if (scrollContainerRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = scrollContainerRef.current;
      const isBottom = Math.abs(scrollHeight - clientHeight - scrollTop) < 50;
      if (isBottom && userScrolledUpRef.current) {
        // 用户自己滚回了底部，重新启用自动滚动
        userScrolledUpRef.current = false;
      }
      if (!userScrolledUpRef.current) {
        isAtBottomRef.current = isBottom;
      }
    }
  };

  const scrollToBottom = (behavior: ScrollBehavior = 'auto') => {
    const el = scrollContainerRef.current;
    if (el) {
      el.scrollTo({ top: el.scrollHeight, behavior });
    }
  };

  const scheduleScrollToBottomAfterRender = useCallback((attempts = 6) => {
    const run = (remaining: number) => {
      // Respect user scroll: if user scrolled up, abort all scheduled scrolls
      if (userScrolledUpRef.current || !isAtBottomRef.current) return;
      const el = scrollContainerRef.current;
      if (el) {
        el.scrollTop = el.scrollHeight;
      }
      if (remaining > 0) {
        requestAnimationFrame(() => run(remaining - 1));
      }
    };

    requestAnimationFrame(() => run(attempts));

    // 某些内容（Markdown、文档卡片、字体回流）会在首帧后继续撑高高度，
    // 仅靠 rAF 可能还会停在上方，因此再补几次延迟滚动。
    // 但必须在每次执行前检查用户是否已经主动滚动了！
    [80, 200, 400, 800, 1200].forEach((delay) => {
      window.setTimeout(() => {
        // Skip if user has scrolled away
        if (userScrolledUpRef.current || !isAtBottomRef.current) return;
        const el = scrollContainerRef.current;
        if (el) {
          el.scrollTop = el.scrollHeight;
        }
      }, delay);
    });
  }, []);

  const loadConversation = async (conversationId: string) => {
    stopPolling();
    try {
      const data = await getConversation(conversationId);
      // Restore conversation model. If the stored model isn't available in the
      // current user_mode (typical case: user switched modes after the conv was
      // created), DON'T silently fall back — keep showing the original model and
      // arm a cross-mode warning that fires on the next send attempt. The user
      // gets to explicitly choose between (a) keep using the cross-mode model or
      // (b) switch to a model from the current mode.
      if (data.model) {
        const currentMode = (localStorage.getItem('user_mode') === 'selfhosted' ? 'selfhosted' : 'clawparrot') as 'clawparrot' | 'selfhosted';
        const otherMode = currentMode === 'selfhosted' ? 'clawparrot' : 'selfhosted';
        const existingOverride = getCrossModeOverride(conversationId);
        if (isModelSelectable(data.model)) {
          // Available in current mode → just use it.
          setCurrentModelString(data.model);
          setCrossModeWarning(null);
        } else if (existingOverride === otherMode) {
          // User already opted into cross-mode for this conv earlier; keep silent.
          setCurrentModelString(data.model);
          setCrossModeWarning(null);
        } else {
          // Cross-mode mismatch with no prior choice — arm the warning. We keep
          // currentModelString = original model (NOT fallback) so the model
          // selector reflects what the conversation actually uses.
          setCurrentModelString(data.model);
          setCrossModeWarning({
            convId: conversationId,
            originalModel: data.model,
            otherMode,
            fallbackModel: resolveModelForNewChat(data.model),
          });
        }
      }
      // Restore research mode toggle
      setResearchMode(!!data.research_mode);
      const normalizedMessages = (data.messages || []).map((msg: any) => {
        // Normalize attachment field names (bridge-server uses camelCase, component expects snake_case)
        if (msg.attachments && Array.isArray(msg.attachments)) {
          msg.attachments = msg.attachments.map((att: any) => ({
            id: att.id || att.fileId || att.file_id || '',
            file_name: att.file_name || att.fileName || 'file',
            file_type: att.file_type || att.fileType || 'document',
            mime_type: att.mime_type || att.mimeType || '',
            file_size: att.file_size || att.size || 0,
            ...att,
          }));
        }
        return sanitizeInlineArtifactMessage(msg);
      });
      setMessages(normalizedMessages);
      isAtBottomRef.current = true;
      scheduleScrollToBottomAfterRender();
      setConversationTitle(data.title || 'New Chat');
      setCurrentProjectId(data.project_id || null);

      // 检查是否有活跃的后台生成
      try {
        const genStatus = await getGenerationStatus(conversationId);
        if (genStatus.active && genStatus.status === 'generating') {
          // 追加占位 assistant 消息（如果最后一条不是 assistant）
          setMessages(prev => {
            const last = prev[prev.length - 1];
            if (
              last &&
              last.role === 'assistant' &&
              !last.content &&
              !genStatus.text &&
              !genStatus.thinking &&
              !(genStatus.documents && genStatus.documents.length > 0) &&
              !genStatus.document
            ) {
              // 已有空占位，更新它
              return prev;
            }
            if (last && last.role === 'assistant') {
              // 更新现有 assistant 消息
              const newMsgs = [...prev];
              newMsgs[newMsgs.length - 1] = applyGenerationState(last, genStatus);
              return newMsgs;
            }
            // 追加新的 assistant 占位
            return [...prev, mergeDocumentsIntoMessage({
              role: 'assistant',
              content: genStatus.text || '',
              thinking: genStatus.thinking || '',
              thinkingSummary: genStatus.thinkingSummary,
              citations: genStatus.citations,
              searchLogs: genStatus.searchLogs,
              isThinking: !genStatus.text && !!genStatus.thinking,
            }, genStatus.document, genStatus.documents)];
          });
          setLoading(true);
          isAtBottomRef.current = true;

          // 启动轮询
          pollingRef.current = setInterval(async () => {
            try {
              const s = await getGenerationStatus(conversationId);
              if (!s.active || s.status !== 'generating') {
                // 生成结束，停止轮询，重新加载最终数据
                stopPolling();
                setLoading(false);
                const final_ = await getConversation(conversationId);
                setMessages((final_.messages || []).map((msg: any) => sanitizeInlineArtifactMessage(msg)));
                isAtBottomRef.current = true;
                scheduleScrollToBottomAfterRender();
                if (final_.title) setConversationTitle(final_.title);
                getContextSize(conversationId).then(setContextInfo).catch(() => { });
                return;
              }
              // 跨进程轮询：内容在另一个进程，从数据库拉最新消息
              if (s.crossProcess) {
                const fresh = await getConversation(conversationId);
                const freshMsgs = (fresh.messages || []).map((msg: any) => sanitizeInlineArtifactMessage(msg));
                isAtBottomRef.current = true;
                scheduleScrollToBottomAfterRender();
                // 如果数据库里最后一条是 assistant，说明有新内容，更新
                // 否则保留当前显示的内容（助手消息可能还没存到数据库）
                setMessages(prev => {
                  const lastFresh = freshMsgs[freshMsgs.length - 1];
                  const lastPrev = prev[prev.length - 1];
                  if (lastFresh && lastFresh.role === 'assistant') {
                    return freshMsgs;
                  }
                  // 数据库里还没有助手消息，保留当前显示的占位消息
                  if (lastPrev && lastPrev.role === 'assistant') {
                    return prev;
                  }
                  return freshMsgs;
                });
                return;
              }
              // 更新进度
              setMessages(prev => {
                const newMsgs = [...prev];
                const last = newMsgs[newMsgs.length - 1];
                if (last && last.role === 'assistant') {
                  newMsgs[newMsgs.length - 1] = applyGenerationState(last, s);
                }
                return newMsgs;
              });
            } catch (e) {
              console.error('[Polling] error:', e);
              stopPolling();
              setLoading(false);
            }
          }, 1500);
        } else {
          setLoading(false);
        }
      } catch {
        // generation-status 接口失败不影响正常加载
        setLoading(false);
      }
    } catch (err) {
      console.error(err);
      setLoading(false);
    }
  };

  const handleModelChange = async (newModelString: string) => {
    if (!isModelSelectable(newModelString)) return;
    setCurrentModelString(newModelString);

    // If in an existing conversation, we should update the conversation's model immediately
    if (activeId && !isCreatingRef.current) {
      try {
        const updated = await updateConversation(activeId, { model: newModelString });
        if (updated?.model) {
          setCurrentModelString(updated.model);
        }
      } catch (err) {
        console.error("Failed to update conversation model", err);
      }
    }
  };

  const handleAttachToProject = async (project: Project) => {
    setShowPlusMenu(false);
    setShowProjectsSubmenu(false);
    if (activeId) {
      if (currentProjectId === project.id) return;
      try {
        await updateConversation(activeId, { project_id: project.id });
        setCurrentProjectId(project.id);
        onNewChat();
        setProjectAddToast(`Added to ${project.name}`);
        setTimeout(() => setProjectAddToast(null), 2500);
      } catch (err) {
        console.error('Failed to add conversation to project', err);
      }
    } else {
      setPendingProjectId(project.id);
      setProjectAddToast(`Will add to ${project.name} on send`);
      setTimeout(() => setProjectAddToast(null), 2500);
    }
  };

  const handleCreateProjectFromMenu = async () => {
    const name = newProjectName.trim();
    if (!name) return;
    try {
      const project = await createProject(name, newProjectDescription.trim());
      setShowNewProjectDialog(false);
      setNewProjectName('');
      setNewProjectDescription('');
      setProjectList(prev => [project, ...prev]);
      await handleAttachToProject(project);
    } catch (err) {
      console.error('Failed to create project', err);
    }
  };

  const handleSend = async (overrideText?: string) => {
    const effectiveText = (typeof overrideText === 'string') ? overrideText : inputText;
    // Skill slug is already in the text (inserted when selected from menu)
    setSelectedSkill(null);
    const hasFiles = pendingFiles.some(f => f.status === 'done');
    const hasErrorFiles = pendingFiles.some(f => f.status === 'error');
    if ((!effectiveText.trim() && !hasFiles) || loading) {
      if (!loading && !effectiveText.trim() && !hasFiles && hasErrorFiles) {
        alert('有文件上传失败，请先删除失败文件后再发送');
      }
      return;
    }
    if (activeRequestCountRef.current >= 2) {
      alert('最多同时进行 2 个对话，请等待其他对话完成');
      return;
    }
    const isUploading = pendingFiles.some(f => f.status === 'uploading');
    if (isUploading) {
      alert('文件仍在上传中，请稍等完成后再发送');
      return;
    }

    // Clawparrot login gate: Electron users in clawparrot mode without a
    // gateway API key get prompted to login on first send. Replaces the old
    // hard redirect to /login at app start — users can now explore the app
    // freely before deciding to login or switch modes.
    const isTauriApp = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;
    const userMode = localStorage.getItem('user_mode');
    const hasGatewayKey = localStorage.getItem('ANTHROPIC_API_KEY') && localStorage.getItem('gateway_user');
    if (isTauriApp && userMode !== 'selfhosted' && !hasGatewayKey) {
      pendingLoginSendRef.current = () => handleSend(effectiveText);
      setShowLoginRequired(true);
      return;
    }

    // Cross-mode warning: if the conversation's model belongs to a different
    // user_mode and the user hasn't yet chosen what to do, defer the send and
    // show the modal. The modal's callbacks will re-invoke handleSend after
    // the user picks "keep cross-mode" or "switch model".
    if (crossModeWarning && crossModeWarning.convId === activeId) {
      pendingCrossModeSendRef.current = () => handleSend(effectiveText);
      return;
    }

    const userMessageText = effectiveText;
    setInputText(""); // Clear input

    // 收集已上传的附件
    const uploadedFiles = pendingFiles.filter(f => f.status === 'done' && f.fileId);
    const githubFiles = pendingFiles.filter(f => f.status === 'done' && f.source === 'github');
    const uploadedPayload = uploadedFiles.map(f => ({ fileId: f.fileId!, fileName: f.fileName, fileType: f.fileType, mimeType: f.mimeType, size: f.size }));
    const githubPayload = githubFiles.map(f => ({
      fileId: `github:${f.ghRepo || f.fileName}`,
      fileName: f.ghRepo || f.fileName,
      fileType: 'github' as any,
      mimeType: 'application/x-github',
      size: 0,
      source: 'github',
      ghRepo: f.ghRepo,
      ghRef: f.ghRef,
    }));
    const attachmentsPayload = (uploadedPayload.length + githubPayload.length) > 0
      ? [...uploadedPayload, ...githubPayload]
      : null;

    // 构建乐观 UI 的附件数据
    const optimisticAttachments: any[] = uploadedFiles.map(f => ({
      id: f.fileId!,
      file_type: f.fileType || 'text',
      file_name: f.fileName,
      mime_type: f.mimeType,
      file_size: f.size,
      line_count: f.lineCount,
    }));
    for (const g of githubFiles) {
      optimisticAttachments.push({
        id: `github:${g.ghRepo || g.fileName}`,
        file_type: 'github',
        file_name: g.ghRepo || g.fileName,
        mime_type: 'application/x-github',
        file_size: 0,
        source: 'github',
        gh_repo: g.ghRepo,
        gh_ref: g.ghRef,
      });
    }

    // 清空 pendingFiles 并释放预览 URL
    pendingFiles.forEach(f => { if (f.previewUrl) URL.revokeObjectURL(f.previewUrl); });
    setPendingFiles([]);
    draftsStore.delete(activeId || '__new__');

    // 重置 textarea 高度
    textareaHeightVal.current = inputBarBaseHeight;
    if (inputRef.current) {
      inputRef.current.style.height = `${inputBarBaseHeight}px`;
      inputRef.current.style.overflowY = 'hidden';
    }

    // Optimistic UI: Add user message immediately
    const tempUserMsg: any = { role: 'user', content: userMessageText, created_at: new Date().toISOString() };
    if (optimisticAttachments.length > 0) {
      tempUserMsg.has_attachments = 1;
      tempUserMsg.attachments = optimisticAttachments;
    }
    setMessages(prev => [...prev, tempUserMsg]);

    // Force scroll to bottom and track state
    isAtBottomRef.current = true;
    setTimeout(() => scrollToBottom('auto'), 50);

    // Prepare assistant message placeholder
    const assistantMsgIndex = messages.length + 1;
    setMessages(prev => [...prev, { role: 'assistant', content: '' }]);

    let conversationId = activeId;

    // If no ID, create conversation first
    if (!conversationId) {
      isCreatingRef.current = true; // Block useEffect fetch
      try {
        const modelForCreate = isModelSelectable(currentModelString)
          ? currentModelString
          : resolveModelForNewChat(currentModelString);
        if (modelForCreate !== currentModelString) {
          setCurrentModelString(modelForCreate);
        }
        // 不传临时标题，让后端生成
        console.log("Creating conversation with model:", modelForCreate);
        const newConv = await createConversation(undefined, modelForCreate, { research_mode: researchMode });
        console.log("Created conversation response:", newConv);

        if (!newConv || !newConv.id) {
          throw new Error("Invalid conversation response from server");
        }

        conversationId = newConv.id;
        console.log("New Conversation ID:", conversationId);
        // Attach to pending project if user chose one before sending
        if (pendingProjectId) {
          try {
            await updateConversation(conversationId!, { project_id: pendingProjectId });
            setCurrentProjectId(pendingProjectId);
          } catch (e) {
            console.error('Failed to attach new conversation to project', e);
          }
          setPendingProjectId(null);
        }
        if (conversationId) warmEngine(conversationId); // Pre-warm engine while user waits

        // Use React Router navigate so useParams stays in sync with the URL
        // isCreatingRef prevents the activeId effect from reloading during streaming
        navigate(`/chat/${conversationId}`, { replace: true });
        if (newConv.model) {
          setCurrentModelString(newConv.model);
        }
        setConversationTitle(newConv.title || 'New Chat');

        onNewChat(); // Refresh sidebar
      } catch (err: any) {
        console.error("Failed to create conversation", err);
        isCreatingRef.current = false;
        setMessages(prev => {
          const newMsgs = [...prev];
          // Find the last assistant message (placeholder) and update it
          if (newMsgs.length > 0 && newMsgs[newMsgs.length - 1].role === 'assistant') {
            newMsgs[newMsgs.length - 1].content = "Error: Failed to create conversation. " + (err.message || err);
          }
          return newMsgs;
        });
        return;
      }
    }

    // Call streaming API — seed buffer with current messages so background streaming works
    messagesBufferRef.current.set(conversationId!, [...messages, tempUserMsg, { role: 'assistant', content: '' }]);
    const controller = new AbortController();
    const streamRequestId = beginStreamSession(conversationId!);
    abortControllerRef.current = controller;
    setLoading(true);
    addStreaming(conversationId!);
    activeRequestCountRef.current += 1;
    await sendMessage(
      conversationId!,
      userMessageText,
      attachmentsPayload,
      (delta, full) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.content = full;
            lastMsg.isThinking = false; // Switch to text mode
          }
          return newMsgs;
        });
      },
      (full) => {
        // Always clean up streaming state and request count, even if session changed
        removeStreaming(conversationId!);
        messagesBufferRef.current.delete(conversationId!);
        activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        if (viewingIdRef.current === conversationId) setLoading(false);
        abortControllerRef.current = null;
        isCreatingRef.current = false; // Reset flag
        clearStreamSession(conversationId!, streamRequestId);
        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.content = full;
            lastMsg.isThinking = false;
          }
          return newMsgs;
        });

        // Refresh conversation to get generated title (if any)
        // 标题生成是异步的，可能需要几秒钟，所以需要延迟轮询
        if (conversationId) {
          const refreshTitle = async () => {
            try {
              const data = await getConversation(conversationId);
              console.log('[MainContent] Polling title for', conversationId, ':', data?.title);
              if (data && data.title) {
                setConversationTitle(data.title);
                // 使用 CustomEvent 通知侧边栏刷新，避免触发 resetKey 变化
                window.dispatchEvent(new CustomEvent('conversationTitleUpdated'));
              }
            } catch (err) {
              console.error('[MainContent] Error polling title:', err);
            }
          };

          // 立即刷新一次
          refreshTitle();
          // 3秒后再刷新一次（此时标题生成应该已完成）
          setTimeout(refreshTitle, 3000);
          // 6秒后再刷新一次（备用）
          setTimeout(refreshTitle, 6000);
        }
      },
      (err) => {
        // Always clean up streaming state and request count, even if session changed
        removeStreaming(conversationId!);
        messagesBufferRef.current.delete(conversationId!);
        activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        if (viewingIdRef.current === conversationId) setLoading(false);
        abortControllerRef.current = null;
        isCreatingRef.current = false;
        clearStreamSession(conversationId!, streamRequestId);
        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          if (newMsgs[newMsgs.length - 1] && newMsgs[newMsgs.length - 1].role === 'assistant') {
            newMsgs[newMsgs.length - 1].content = formatChatError(err);
            newMsgs[newMsgs.length - 1].isThinking = false;
          }
          return newMsgs;
        });
      },
      (thinkingDelta, thinkingFull) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.thinking = thinkingFull;
            lastMsg.isThinking = true;
            delete lastMsg.searchStatus;
          }
          return newMsgs;
        });
      },
      (event, message, data) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        // Handle metadata (update user message ID)
        if (event === 'metadata' && data && data.user_message_id) {
          setMessages(prev => {
            const newMsgs = [...prev];
            const userIdx = newMsgs.length - 2;
            if (userIdx >= 0 && newMsgs[userIdx].role === 'user') {
              newMsgs[userIdx] = { ...newMsgs[userIdx], id: data.user_message_id };
            }
            return newMsgs;
          });
        }
        // Handle system/status events (e.g. web search status)
        if (event === 'status' && message) {
          if (!isSearchStatusMessage(message)) return;
          setMessagesFor(conversationId!, prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.searchStatus = message;
              lastMsg._contentLenBeforeSearch = (lastMsg.content || '').length;
            }
            return newMsgs;
          });
        }
        // Handle thinking summary
        if (event === 'thinking_summary' && message) {
          setMessages(prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.thinking_summary = message;
            }
            return newMsgs;
          });
        }
        // Handle auto compaction progress
        if (event === 'compaction_start') {
          setCompactStatus({ state: 'compacting' });
        }
        if (event === 'compaction_done') {
          if (data && data.messagesCompacted > 0) {
            setCompactStatus({ state: 'done', message: `Compacted ${data.messagesCompacted} messages, saved ~${data.tokensSaved} tokens` });
            setTimeout(() => setCompactStatus({ state: 'idle' }), 4000);
          } else {
            setCompactStatus({ state: 'idle' });
          }
        }
        // Handle compact_boundary from engine auto-compact during normal chat
        if (event === 'compact_boundary') {
          const meta = data?.compact_metadata || {};
          const preTokens = meta.pre_tokens || 0;
          const saved = preTokens ? Math.round(preTokens * 0.7) : 0;
          setCompactStatus({ state: 'done', message: saved > 0 ? `Auto-compacted, saved ~${saved} tokens` : 'Context auto-compacted' });
          setTimeout(() => setCompactStatus({ state: 'idle' }), 4000);
          // Reload messages to reflect compacted state
          if (activeId) {
            loadConversation(activeId);
            getContextSize(activeId).then(setContextInfo).catch(() => {});
          }
        }
        if (event === 'context_size' && data) {
          setContextInfo({ tokens: data.tokens, limit: data.limit });
        }
        if (event === 'tool_text_offset' && data && data.offset != null) {
          setMessages(prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.toolTextEndOffset = data.offset;
            }
            return newMsgs;
          });
        }
        if (event && event.startsWith('research_')) {
          setMessages(prev => applyResearchEvent(prev, event, data));
        }
        // AskUserQuestion — engine needs user input
        if (event === 'ask_user' && data) {
          setAskUserDialog({
            request_id: data.request_id,
            tool_use_id: data.tool_use_id,
            questions: data.questions || [],
            answers: {},
          });
        }
        // Task/Agent progress
        if (event === 'task_event' && data) {
          setActiveTasks(prev => {
            const next = new Map(prev);
            if (data.subtype === 'task_started') {
              next.set(data.task_id, { description: data.description || 'Running task...' });
            } else if (data.subtype === 'task_progress') {
              const existing = next.get(data.task_id);
              if (existing) {
                next.set(data.task_id, { ...existing, last_tool_name: data.last_tool_name, summary: data.summary });
              }
            } else if (data.subtype === 'task_notification') {
              next.delete(data.task_id);
            }
            return next;
          });
        }
      },
      (sources, query, tokens) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        // Handle search_sources — collect citation sources
        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            const existing = lastMsg.citations || [];

            // 去重合并
            const existingUrls = new Set(existing.map((s: any) => s.url));
            const newSources = sources.filter((s: any) => !existingUrls.has(s.url));
            lastMsg.citations = [...existing, ...newSources];

            if (query) {
              const logs = lastMsg.searchLogs || [];
              // 检查是否已存在相同的 query
              const existingLogIndex = logs.findIndex((log: any) => log.query === query);
              if (existingLogIndex !== -1) {
                // 更新现有 log 的 results 和 tokens
                const existingLog = logs[existingLogIndex];
                const currentResults = existingLog.results || [];
                const currentUrls = new Set(currentResults.map((r: any) => r.url));
                const uniqueNewResults = sources.filter((s: any) => !currentUrls.has(s.url));
                existingLog.results = [...currentResults, ...uniqueNewResults];
                if (tokens !== undefined) {
                  existingLog.tokens = tokens;
                }
              } else {
                // 添加新 log
                logs.push({ query, results: sources, tokens });
              }
              lastMsg.searchLogs = logs;
            }
          }
          return newMsgs;
        });
      },
      (doc) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          const lastIdx = newMsgs.length - 1;
          if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
            newMsgs[lastIdx] = mergeDocumentsIntoMessage(newMsgs[lastIdx], doc);
          }
          return newMsgs;
        });
      },
      (draft) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          const lastIdx = newMsgs.length - 1;
          if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
            newMsgs[lastIdx] = mergeDocumentDraftIntoMessage(newMsgs[lastIdx], draft);
          }
          return newMsgs;
        });
      },
      async (data) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;
        // Handle code_execution / code_result events
        if (data.type === 'code_execution') {
          // 收到代码执行请求 — 更新消息状态 + 在 Pyodide 中执行
          setMessages(prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.codeExecution = {
                code: data.code || '',
                status: 'running' as const,
                stdout: '',
                stderr: '',
                images: [],
                error: null,
              };
            }
            return newMsgs;
          });

          // 构建文件列表（附件 URL）
          const authToken = localStorage.getItem('auth_token') || '';
          const files = (data.files || []).map((f: any) => ({
            name: f.name,
            url: (() => {
              const baseUrl = getAttachmentUrl(f.id);
              if (!authToken) return baseUrl;
              return `${baseUrl}${baseUrl.includes('?') ? '&' : '?'}token=${encodeURIComponent(authToken)}`;
            })(),
          }));

          try {
            const result = await executeCode(data.code || '', files, data.executionId);
            // 发送结果回后端
            await sendCodeResult(data.executionId, result);
          } catch (e: any) {
            // 发送错误结果回后端
            await sendCodeResult(data.executionId, {
              stdout: '',
              stderr: '',
              images: [],
              error: e.message || 'Pyodide 执行失败',
            });
          }
        }

        if (data.type === 'code_result') {
          // 收到执行结果 — 更新消息状态
          setMessages(prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant' && lastMsg.codeExecution) {
              lastMsg.codeExecution = {
                ...lastMsg.codeExecution,
                status: data.error ? 'error' as const : 'done' as const,
                stdout: data.stdout || '',
                stderr: data.stderr || '',
                images: data.images || [],
                error: data.error || null,
              };
            }
            return newMsgs;
          });
        }
      },
      // Handle tool use events from SDK
      (toolEvent) => {
        if (!isStreamSessionActive(conversationId!, streamRequestId)) return;

        // Track plan mode from tool events
        if (toolEvent.type === 'done' && toolEvent.tool_name === 'EnterPlanMode') setPlanMode(true);
        if (toolEvent.type === 'done' && toolEvent.tool_name === 'ExitPlanMode') setPlanMode(false);

        // Don't add internal tools to UI tool list
        const INTERNAL_TOOLS = new Set(['EnterPlanMode', 'ExitPlanMode', 'TaskCreate', 'TaskUpdate', 'TaskGet', 'TaskList', 'TaskOutput', 'TaskStop']);
        if (INTERNAL_TOOLS.has(toolEvent.tool_name || '')) return;

        setMessagesFor(conversationId!, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (!lastMsg || lastMsg.role !== 'assistant') return prev;

          const toolCalls = lastMsg.toolCalls || [];

          if (toolEvent.type === 'start') {
            // Dedupe by id (we may receive a placeholder tool_use_start before
            // the input has finished streaming, then a tool_use_input later)
            let existing = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
            if (existing) {
              existing.name = toolEvent.tool_name || existing.name;
              if (toolEvent.tool_input && Object.keys(toolEvent.tool_input).length > 0) existing.input = toolEvent.tool_input;
              if (toolEvent.textBefore) existing.textBefore = toolEvent.textBefore;
            } else {
              toolCalls.push({
                id: toolEvent.tool_use_id,
                name: toolEvent.tool_name || 'unknown',
                input: toolEvent.tool_input || {},
                status: 'running' as const,
                textBefore: toolEvent.textBefore || '',
              });
            }
          } else if (toolEvent.type === 'input') {
            // Update an existing tool's input after the JSON has fully streamed
            const tc = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
            if (tc) tc.input = toolEvent.tool_input || {};
          } else if (toolEvent.type === 'done') {
            let tc = toolCalls.find((t: any) => t.id === toolEvent.tool_use_id);
            if (!tc) {
              // tool_use_start was missed — back-fill the entry so the card still renders
              tc = { id: toolEvent.tool_use_id, name: toolEvent.tool_name || 'unknown', input: {}, status: 'done' as const, result: toolEvent.content };
              toolCalls.push(tc);
            } else {
              tc.status = toolEvent.is_error ? 'error' as const : 'done' as const;
              tc.result = toolEvent.content;
            }
          }

          lastMsg.toolCalls = toolCalls;
          return newMsgs;
        });
      },
      controller.signal
    );
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key !== 'Enter' || e.nativeEvent.isComposing) return;

    const sendKey = localStorage.getItem('sendKey') || 'enter';
    // Normalize format (settings uses underscore, old might use plus)
    const sk = sendKey.replace('+', '_').toLowerCase();

    let shouldSend = false;
    if (sk === 'enter') {
      if (!e.shiftKey && !e.ctrlKey && !e.altKey && !e.metaKey) shouldSend = true;
    } else if (sk === 'ctrl_enter') {
      if (e.ctrlKey) shouldSend = true;
    } else if (sk === 'cmd_enter') {
      if (e.metaKey) shouldSend = true;
    } else if (sk === 'alt_enter') {
      if (e.altKey) shouldSend = true;
    }

    if (shouldSend) {
      e.preventDefault();
      handleSend();
    }
  };

  // Auto-send initialMessage from Project page navigation
  useEffect(() => {
    if (pendingInitialMessageRef.current && activeId && !loading) {
      const msg = pendingInitialMessageRef.current;
      pendingInitialMessageRef.current = null;
      // Small delay to let conversation finish loading
      setTimeout(() => handleSend(msg), 150);
    }
  }, [activeId, loading]);

  // 停止生成（双模式：SSE 直连 or 轮询模式）
  const handleStop = () => {
    if (abortStreamSession(activeId || undefined)) {
      if (activeId) removeStreaming(activeId);
      return;
    }
    if (pollingRef.current && activeId) {
      // 轮询模式：调用后端停止接口
      stopGeneration(activeId).catch(e => console.error('[Stop] error:', e));
      stopPolling();
    }
    if (activeId) removeStreaming(activeId);
    setLoading(false);
    isCreatingRef.current = false;
  };

  // 复制消息内容
  // 复制消息内容
  const handleCopyMessage = (content: string, idx: number) => {
    copyToClipboard(content).then((success) => {
      if (success) {
        setCopiedMessageIdx(idx);
        setTimeout(() => setCopiedMessageIdx(null), 2000);
      }
    });
  };

  const extractMessageAttachments = (msg: any) => {
    const raw = Array.isArray(msg?.attachments)
      ? msg.attachments.filter((att: any) => att && ((typeof att.id === 'string' && att.id.trim()) || (typeof att.fileId === 'string' && att.fileId.trim())))
      : [];
    // Normalize to snake_case for component compatibility
    const attachments = raw.map((att: any) => ({
      file_name: att.file_name || att.fileName || 'file',
      file_type: att.file_type || att.fileType || 'document',
      mime_type: att.mime_type || att.mimeType || '',
      file_size: att.file_size || att.size || 0,
      ...att,
      id: att.id || att.fileId || '', // ensure id wins over ...att spread
    }));
    const attachmentIds = attachments.map((att: any) => att.id);
    return {
      attachmentIds,
      attachmentsPayload: attachments.length > 0
        ? attachments.map((att: any) => ({ fileId: att.id, fileName: att.file_name, fileType: att.file_type, mimeType: att.mime_type, size: att.file_size }))
        : null,
      optimisticAttachments: attachments,
    };
  };

  // 重新发送消息
  const handleResendMessage = async (content: string, idx: number) => {
    if (loading) return;
    if (activeRequestCountRef.current >= 2) {
      alert('最多同时进行 2 个对话，请等待其他对话完成');
      return;
    }
    const msg = messages[idx];
    const { attachmentIds, attachmentsPayload, optimisticAttachments } = extractMessageAttachments(msg);
    const tempUserMsg: any = { role: 'user', content, created_at: new Date().toISOString() };
    if (optimisticAttachments.length > 0) {
      tempUserMsg.has_attachments = 1;
      tempUserMsg.attachments = optimisticAttachments;
    }
    // 删除当前消息及其后续消息（前端），然后重新添加用户消息 + assistant 占位
    setMessages(prev => [
      ...prev.slice(0, idx),
      tempUserMsg,
      { role: 'assistant', content: '' },
    ]);
    // 删除后端消息（regenerate）
    if (activeId) {
      try {
        if (msg.id) {
          await deleteMessagesFrom(activeId, msg.id, attachmentIds);
        } else {
          const tailCount = messages.length - idx;
          if (tailCount > 0) await deleteMessagesTail(activeId, tailCount, attachmentIds);
        }
      } catch (err) {
        console.error('Failed to delete messages from backend:', err);
      }
    }
    // 直接重新发送
    isAtBottomRef.current = true;
    setTimeout(() => scrollToBottom('auto'), 50);
    const controller = new AbortController();
    const conversationId = activeId!;
    const streamRequestId = beginStreamSession(conversationId);
    abortControllerRef.current = controller;
    setLoading(true);
    addStreaming(conversationId);
    activeRequestCountRef.current += 1;
    await sendMessage(
      conversationId,
      content,
      attachmentsPayload,
      (delta, full) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.content = full;
            lastMsg.isThinking = false;
          }
          return newMsgs;
        });
      },
      (full) => {
        // Always clean up streaming state and request count
        removeStreaming(conversationId);
        messagesBufferRef.current.delete(conversationId);
        activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        if (viewingIdRef.current === conversationId) setLoading(false);
        abortControllerRef.current = null;
        clearStreamSession(conversationId, streamRequestId);
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.content = full;
            lastMsg.isThinking = false;
          }
          return newMsgs;
        });
      },
      (err) => {
        // Always clean up streaming state and request count
        removeStreaming(conversationId);
        activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setLoading(false);
        abortControllerRef.current = null;
        clearStreamSession(conversationId, streamRequestId);
        setMessages(prev => {
          const newMsgs = [...prev];
          if (newMsgs[newMsgs.length - 1] && newMsgs[newMsgs.length - 1].role === 'assistant') {
            newMsgs[newMsgs.length - 1].content = formatChatError(err);
            newMsgs[newMsgs.length - 1].isThinking = false;
          }
          return newMsgs;
        });
      },
      (thinkingDelta, thinkingFull) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.thinking = thinkingFull;
            lastMsg.isThinking = true;
            delete lastMsg.searchStatus;
          }
          return newMsgs;
        });
      },
      (event, message, data) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        if (event === 'metadata' && data && data.user_message_id) {
          setMessagesFor(conversationId, prev => {
            const newMsgs = [...prev];
            const userIdx = newMsgs.length - 2;
            if (userIdx >= 0 && newMsgs[userIdx].role === 'user') {
              newMsgs[userIdx] = { ...newMsgs[userIdx], id: data.user_message_id };
            }
            return newMsgs;
          });
        }
        if (event === 'thinking_summary' && message) {
          setMessagesFor(conversationId, prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.thinking_summary = message;
            }
            return newMsgs;
          });
        }
        if (event === 'context_size' && data) {
          setContextInfo({ tokens: data.tokens, limit: data.limit });
        }
        if (event === 'tool_text_offset' && data && data.offset != null) {
          setMessages(prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.toolTextEndOffset = data.offset;
            }
            return newMsgs;
          });
        }
        if (event && event.startsWith('research_')) {
          setMessages(prev => applyResearchEvent(prev, event, data));
        }
      },
      undefined,
      (doc) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastIdx = newMsgs.length - 1;
          if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
            newMsgs[lastIdx] = mergeDocumentsIntoMessage(newMsgs[lastIdx], doc);
          }
          return newMsgs;
        });
      },
      (draft) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastIdx = newMsgs.length - 1;
          if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
            newMsgs[lastIdx] = mergeDocumentDraftIntoMessage(newMsgs[lastIdx], draft);
          }
          return newMsgs;
        });
      },
      undefined,
      undefined,
      controller.signal
    );
  };

  // 编辑消息 — 进入原地编辑模式（不立即删除后续消息）
  const handleEditMessage = (content: string, idx: number) => {
    if (loading) return;
    setEditingMessageIdx(idx);
    setEditingContent(content);
  };

  // 取消编辑
  const handleEditCancel = () => {
    setEditingMessageIdx(null);
    setEditingContent('');
  };

  // 保存编辑 — 删除当前及后续消息，用新内容重新发送
  const handleEditSave = async () => {
    if (editingMessageIdx === null || !editingContent.trim() || loading) return;
    if (activeRequestCountRef.current >= 2) {
      alert('最多同时进行 2 个对话，请等待其他对话完成');
      return;
    }
    const idx = editingMessageIdx;
    const msg = messages[idx];
    const newContent = editingContent.trim();
    const { attachmentIds, attachmentsPayload, optimisticAttachments } = extractMessageAttachments(msg);

    // 退出编辑模式
    setEditingMessageIdx(null);
    setEditingContent('');

    const tempUserMsg: any = { role: 'user', content: newContent, created_at: new Date().toISOString() };
    if (optimisticAttachments.length > 0) {
      tempUserMsg.has_attachments = 1;
      tempUserMsg.attachments = optimisticAttachments;
    }

    // 删除当前消息及其后续消息（前端），同时加入新的用户消息和 assistant 占位
    setMessages(prev => [
      ...prev.slice(0, idx),
      tempUserMsg,
      { role: 'assistant', content: '' },
    ]);

    // 删除后端消息（regenerate）
    if (activeId) {
      try {
        if (msg.id) {
          await deleteMessagesFrom(activeId, msg.id, attachmentIds);
        } else {
          const tailCount = messages.length - idx;
          if (tailCount > 0) await deleteMessagesTail(activeId, tailCount, attachmentIds);
        }
      } catch (err) {
        console.error('Failed to delete messages from backend:', err);
      }
    }

    // 直接发送新内容
    isAtBottomRef.current = true;
    setTimeout(() => scrollToBottom('auto'), 50);

    const conversationId = activeId;
    if (!conversationId) return;

    const controller = new AbortController();
    const streamRequestId = beginStreamSession(conversationId);
    abortControllerRef.current = controller;
    setLoading(true);
    addStreaming(conversationId);
    activeRequestCountRef.current += 1;
    await sendMessage(
      conversationId,
      newContent,
      attachmentsPayload,
      (delta, full) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.content = full;
            lastMsg.isThinking = false;
          }
          return newMsgs;
        });
      },
      (full) => {
        // Always clean up streaming state and request count
        removeStreaming(conversationId);
        messagesBufferRef.current.delete(conversationId);
        activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        if (viewingIdRef.current === conversationId) setLoading(false);
        abortControllerRef.current = null;
        clearStreamSession(conversationId, streamRequestId);
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.content = full;
            lastMsg.isThinking = false;
          }
          return newMsgs;
        });
      },
      (err) => {
        // Always clean up streaming state and request count
        removeStreaming(conversationId);
        activeRequestCountRef.current = Math.max(0, activeRequestCountRef.current - 1);
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setLoading(false);
        abortControllerRef.current = null;
        clearStreamSession(conversationId, streamRequestId);
        setMessages(prev => {
          const newMsgs = [...prev];
          if (newMsgs[newMsgs.length - 1] && newMsgs[newMsgs.length - 1].role === 'assistant') {
            newMsgs[newMsgs.length - 1].content = formatChatError(err);
            newMsgs[newMsgs.length - 1].isThinking = false;
          }
          return newMsgs;
        });
      },
      (thinkingDelta, thinkingFull) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastMsg = newMsgs[newMsgs.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            lastMsg.thinking = thinkingFull;
            lastMsg.isThinking = true;
            delete lastMsg.searchStatus;
          }
          return newMsgs;
        });
      },
      (event, message, data) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        if (event === 'metadata' && data && data.user_message_id) {
          setMessagesFor(conversationId, prev => {
            const newMsgs = [...prev];
            const userIdx = newMsgs.length - 2;
            if (userIdx >= 0 && newMsgs[userIdx].role === 'user') {
              newMsgs[userIdx] = { ...newMsgs[userIdx], id: data.user_message_id };
            }
            return newMsgs;
          });
        }
        if (event === 'thinking_summary' && message) {
          setMessagesFor(conversationId, prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.thinking_summary = message;
            }
            return newMsgs;
          });
        }
        if (event === 'context_size' && data) {
          setContextInfo({ tokens: data.tokens, limit: data.limit });
        }
        if (event === 'tool_text_offset' && data && data.offset != null) {
          setMessages(prev => {
            const newMsgs = [...prev];
            const lastMsg = newMsgs[newMsgs.length - 1];
            if (lastMsg && lastMsg.role === 'assistant') {
              lastMsg.toolTextEndOffset = data.offset;
            }
            return newMsgs;
          });
        }
        if (event && event.startsWith('research_')) {
          setMessages(prev => applyResearchEvent(prev, event, data));
        }
      },
      undefined,
      (doc) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastIdx = newMsgs.length - 1;
          if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
            newMsgs[lastIdx] = mergeDocumentsIntoMessage(newMsgs[lastIdx], doc);
          }
          return newMsgs;
        });
      },
      (draft) => {
        if (!isStreamSessionActive(conversationId, streamRequestId)) return;
        setMessagesFor(conversationId, prev => {
          const newMsgs = [...prev];
          const lastIdx = newMsgs.length - 1;
          if (newMsgs[lastIdx] && newMsgs[lastIdx].role === 'assistant') {
            newMsgs[lastIdx] = mergeDocumentDraftIntoMessage(newMsgs[lastIdx], draft);
          }
          return newMsgs;
        });
      },
      undefined,
      undefined,
      controller.signal
    );
  };

  // 切换消息展开/折叠
  const toggleMessageExpand = (idx: number) => {
    setExpandedMessages(prev => {
      const next = new Set(prev);
      if (next.has(idx)) {
        next.delete(idx);
      } else {
        next.add(idx);
      }
      return next;
    });
  };

  // === 文件上传相关 ===
  const ACCEPTED_TYPES = 'image/png,image/jpeg,image/jpg,image/gif,image/webp,application/pdf,.docx,.xlsx,.pptx,.odt,.rtf,.epub,.txt,.md,.csv,.json,.xml,.yaml,.yml,.js,.jsx,.ts,.tsx,.py,.java,.cpp,.c,.h,.cs,.go,.rs,.rb,.php,.swift,.kt,.scala,.html,.css,.scss,.less,.sql,.sh,.bash,.vue,.svelte,.lua,.r,.m,.pl,.ex,.exs';

  const handleFilesSelected = (files: FileList | File[], defaults?: Partial<PendingFile>) => {
    const fileArray = Array.from(files);
    const maxFiles = 20;
    const currentCount = pendingFiles.length;
    const allowed = fileArray.slice(0, maxFiles - currentCount);

    for (const file of allowed) {
      const id = Math.random().toString(36).slice(2);
      const isImage = file.type.startsWith('image/');
      const previewUrl = isImage ? URL.createObjectURL(file) : undefined;

      const pending: PendingFile = {
        id,
        file,
        fileName: file.name,
        mimeType: file.type,
        size: file.size,
        progress: 0,
        status: 'uploading',
        previewUrl,
        ...(defaults || {}),
      };

      setPendingFiles(prev => [...prev, pending]);

      // Calculate lines for text files
      const textExtensions = /\.(txt|md|csv|json|xml|yaml|yml|js|jsx|ts|tsx|py|java|cpp|c|h|cs|go|rs|rb|php|swift|kt|scala|html|css|scss|less|sql|sh|bash|vue|svelte|lua|r|m|pl|ex|exs)$/i;
      if (file.size < 5 * 1024 * 1024 && (file.type.startsWith('text/') || textExtensions.test(file.name))) {
        const reader = new FileReader();
        reader.onload = (e) => {
          const text = e.target?.result as string;
          if (text) {
            const lines = text.split(/\r\n|\r|\n/).length;
            setPendingFiles(prev => prev.map(f => f.id === id ? { ...f, lineCount: lines } : f));
          }
        };
        reader.readAsText(file);
      }

      if (!activeId) return;
      uploadFile(file, (percent) => {
        setPendingFiles(prev => prev.map(f => f.id === id ? { ...f, progress: percent } : f));
      }, activeId).then((result) => {
        setPendingFiles(prev => prev.map(f => f.id === id ? {
          ...f,
          fileId: result.fileId,
          fileType: result.fileType,
          status: 'done' as const,
          progress: 100,
        } : f));
      }).catch((err) => {
        setPendingFiles(prev => prev.map(f => f.id === id ? {
          ...f,
          status: 'error' as const,
          error: err.message,
        } : f));
      });
    }
  };

  const handleRemoveFile = (id: string) => {
    setPendingFiles(prev => {
      const file = prev.find(f => f.id === id);
      if (file?.previewUrl) URL.revokeObjectURL(file.previewUrl);
      // 已上传的文件调后端删除，释放存储空间
      if (file?.fileId) {
        deleteAttachment(file.fileId).catch(() => { });
      }
      return prev.filter(f => f.id !== id);
    });
  };

  // Handle "Add from GitHub" confirmation: resolve/create conversation,
  // materialize files into its workspace, then add a visual-only card.
  const handleGithubAdd = async (payload: GithubAddPayload): Promise<void> => {
    let convId: string | null = activeId;
    let createdNewConv = false;
    if (!convId) {
      const modelForCreate = isModelSelectable(currentModelString)
        ? currentModelString
        : resolveModelForNewChat(currentModelString);
      const newConv = await createConversation(undefined, modelForCreate, { research_mode: researchMode });
      if (!newConv || !newConv.id) throw new Error('Failed to create conversation');
      convId = newConv.id;
      createdNewConv = true;
      if (convId) warmEngine(convId);
      onNewChat();
    }

    const result = await materializeGithub(
      convId!,
      payload.repoFullName,
      payload.ref,
      payload.selections,
    );

    const githubCard: PendingFile = {
      id: Math.random().toString(36).slice(2),
      file: new File([], 'github-placeholder'),
      fileName: payload.repoFullName,
      mimeType: 'application/x-github',
      size: 0,
      progress: 100,
      status: 'done',
      source: 'github',
      ghRepo: payload.repoFullName,
      ghRef: payload.ref,
      lineCount: result.fileCount,
    };

    if (createdNewConv) {
      // Stash the card into draftsStore under the NEW conv key so the mount
      // effect restores it. Then navigate — the useEffect will pick it up.
      const text = inputTextRef.current || '';
      const height = textareaHeightRef.current || inputBarBaseHeight;
      draftsStore.set(convId!, { text, files: [githubCard], height });
      navigate(`/chat/${convId}`, { replace: true });
    } else {
      setPendingFiles(prev => [...prev, githubCard]);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    if (e.dataTransfer.files.length > 0) {
      handleFilesSelected(e.dataTransfer.files);
    }
  };

  const handlePaste = (e: React.ClipboardEvent) => {
    // 1. 优先检查图片
    const items = e.clipboardData?.items;
    if (items) {
      const imageFiles: File[] = [];
      for (let i = 0; i < items.length; i++) {
        if (items[i].type.startsWith('image/')) {
          const file = items[i].getAsFile();
          if (file) imageFiles.push(file);
        }
      }
      if (imageFiles.length > 0) {
        e.preventDefault();
        handleFilesSelected(imageFiles);
        return;
      }
    }

    // 2. 检查长文本 (超过 10000 字符或 100 行自动转为附件)
    const text = e.clipboardData.getData('text');
    if (text) {
      const lineCount = text.split('\n').length;
      if (text.length > 10000 || lineCount > 100) {
        e.preventDefault();
        const blob = new Blob([text], { type: 'text/plain' });
        const file = new File([blob], 'Pasted-Text.txt', { type: 'text/plain' });
        handleFilesSelected([file]);
      }
    }
  };


  // --- Render Logic ---

  // Shared overlays rendered in both MODE 1 and MODE 2
  const sharedProjectOverlays = (
    <>
      {showNewProjectDialog && (
        <div
          className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40"
          onClick={() => { setShowNewProjectDialog(false); setNewProjectName(''); setNewProjectDescription(''); }}
        >
          <div className="bg-claude-bg border border-claude-border rounded-2xl shadow-xl w-[560px] max-w-[92vw] overflow-hidden" onClick={e => e.stopPropagation()}>
            <div className="flex items-start justify-between px-7 pt-6 pb-4">
              <h2 className="font-[Spectral] text-[22px] text-claude-text" style={{ fontWeight: 600 }}>Create a project</h2>
              <button
                onClick={() => { setShowNewProjectDialog(false); setNewProjectName(''); setNewProjectDescription(''); }}
                className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
              >
                <X size={18} />
              </button>
            </div>
            <div className="px-7 pb-4 space-y-5">
              <div>
                <label className="block text-[15px] font-medium text-claude-textSecondary mb-2">What are you working on?</label>
                <input
                  type="text"
                  placeholder="Name your project"
                  value={newProjectName}
                  onChange={e => setNewProjectName(e.target.value)}
                  onKeyDown={e => { if (e.key === 'Enter' && newProjectName.trim()) handleCreateProjectFromMenu(); }}
                  className="w-full px-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:border-[#387ee0] focus:ring-0 transition-all text-[15px]"
                  autoFocus
                />
              </div>
              <div>
                <label className="block text-[15px] font-medium text-claude-textSecondary mb-2">What are you trying to achieve?</label>
                <textarea
                  placeholder="Describe your project, goals, subject, etc..."
                  rows={3}
                  value={newProjectDescription}
                  onChange={e => setNewProjectDescription(e.target.value)}
                  className="w-full px-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:border-[#387ee0] focus:ring-0 transition-all text-[15px] resize-none"
                />
              </div>
            </div>
            <div className="flex items-center justify-end gap-3 px-7 pb-6 pt-2">
              <button
                onClick={() => { setShowNewProjectDialog(false); setNewProjectName(''); setNewProjectDescription(''); }}
                className="px-5 py-2.5 text-[15px] font-medium text-claude-text bg-white dark:bg-claude-bg border border-gray-300 dark:border-claude-border hover:bg-gray-50 dark:hover:bg-claude-hover rounded-xl transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateProjectFromMenu}
                disabled={!newProjectName.trim()}
                className="px-5 py-2.5 text-[15px] font-medium text-claude-bg bg-black dark:bg-white dark:text-black hover:opacity-90 rounded-xl transition-opacity disabled:opacity-40"
              >
                Create project
              </button>
            </div>
          </div>
        </div>
      )}
      {projectAddToast && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[300] px-4 py-2 bg-claude-input border border-claude-border rounded-lg shadow-lg text-[13px] text-claude-text flex items-center gap-2">
          <Check size={14} className="text-[#C6613F]" />
          {projectAddToast}
        </div>
      )}
      {webSearchToast && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[300] px-4 py-2 bg-claude-input border border-claude-border rounded-lg shadow-lg text-[13px] text-claude-text flex items-center gap-2">
          <IconWebSearch size={14} className="text-claude-textSecondary" />
          {webSearchToast}
        </div>
      )}
    </>
  );

  // MODE 1: Landing Page (No ID)
  if (!activeId && messages.length === 0) {
    return (
      <div className={`flex-1 bg-claude-bg h-screen flex flex-col relative overflow-hidden text-claude-text chat-font-scope ${showEntranceAnimation ? 'animate-slide-in' : ''}`}>

        {/* Centered Content */}
        <div
          className="flex-1 flex flex-col items-center w-full mx-auto px-4"
          style={{
            maxWidth: `${tunerConfig?.mainContentWidth || 768}px`,
            marginTop: `${tunerConfig?.mainContentMt || 0}px`,
            paddingTop: '40vh'
          }}
        >

          <div
            className="flex items-center gap-4"
            style={{ marginBottom: `${tunerConfig?.welcomeMb || 40}px` }}
          >
            <div className="w-[80px] h-[80px] shrink-0 flex items-center justify-center -mx-[16px]" style={{ marginTop: '-16px', marginBottom: '-16px' }}>
              <ClaudeLogo color="#D97757" maxScale={0.17} />
            </div>
            <h1
              className="text-claude-text dark:!text-[#d6cec3] tracking-tight leading-none pt-1 transition-all duration-100 ease-out whitespace-nowrap"
              style={{
                fontFamily: 'Optima, Candara, "Segoe UI", Segoe, "Humanist 521", sans-serif',
                fontSize: '46px',
                fontWeight: 500,
                letterSpacing: '-0.05em',
              }}
            >
              {welcomeGreeting}
            </h1>
          </div>

          {/* 输入框区域 */}
          <div className="w-full relative group">
            <input
              type="file"
              ref={fileInputRef}
              className="hidden"
              multiple
              accept={ACCEPTED_TYPES}
              onChange={(e) => {
                if (e.target.files) handleFilesSelected(e.target.files);
                e.target.value = '';
              }}
            />
            <div
              className={`bg-claude-input border shadow-[0_2px_8px_rgba(0,0,0,0.02)] hover:shadow-[0_2px_8px_rgba(0,0,0,0.08)] hover:border-[#CCC] dark:hover:border-[#5a5a58] focus-within:shadow-[0_2px_8px_rgba(0,0,0,0.08)] focus-within:border-[#CCC] dark:focus-within:border-[#5a5a58] transition-all duration-200 flex flex-col max-h-[60vh] font-sans ${isDragging ? 'border-[#D97757] bg-orange-50/30' : 'border-claude-border dark:border-[#3a3a38]'}`}
              style={{ borderRadius: `${tunerConfig?.inputRadius || 16}px` }}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            >
              <div className="flex-1 overflow-y-auto min-h-0">
                <FileUploadPreview files={pendingFiles} onRemove={handleRemoveFile} />
                <div className="relative">
                  <SkillInputOverlay
                    text={inputText}
                    className="pl-5 pr-4 pt-5 pb-1 text-[16px] font-sans font-[350] overflow-hidden"
                    style={{ minHeight: '48px' }}
                  />
                  <textarea
                    ref={inputRef}
                    className={`w-full pl-5 pr-4 pt-5 pb-1 placeholder:text-claude-textSecondary text-[16px] outline-none resize-none overflow-hidden bg-transparent font-sans font-[350] ${inputText.match(/^\/[a-zA-Z0-9_-]+/) ? 'text-transparent caret-claude-text' : 'text-claude-text'}`}
                    style={{ minHeight: '48px', borderRadius: `${tunerConfig?.inputRadius || 16}px ${tunerConfig?.inputRadius || 16}px 0 0` }}
                    placeholder={selectedSkill ? `Describe what you want ${selectedSkill.name} to do...` : "How can I help you today?"}
                    value={inputText}
                    onChange={(e) => {
                      setInputText(e.target.value);
                      e.target.style.height = 'auto';
                      e.target.style.height = Math.min(e.target.scrollHeight, 300) + 'px';
                      e.target.style.overflowY = e.target.scrollHeight > 300 ? 'auto' : 'hidden';
                    }}
                    onKeyDown={(e) => {
                      // Backspace deletes entire /skill-name as a unit
                      if (e.key === 'Backspace' && selectedSkill) {
                        const pos = (e.target as HTMLTextAreaElement).selectionStart;
                        const skillPrefix = `/${selectedSkill.slug} `;
                        if (pos > 0 && pos <= skillPrefix.length && inputText.startsWith(skillPrefix.slice(0, pos))) {
                          e.preventDefault();
                          setInputText(inputText.slice(skillPrefix.length));
                          setSelectedSkill(null);
                          return;
                        }
                      }
                      handleKeyDown(e);
                    }}
                    onPaste={handlePaste}
                  />
                </div>
              </div>
              <div className="px-4 pb-3 pt-1 flex items-center justify-between flex-shrink-0">
                <div className="relative flex items-center">
                  <button
                    ref={plusBtnRef}
                    onClick={() => setShowPlusMenu(prev => !prev)}
                    className="p-2 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
                  >
                    <IconPlus size={20} />
                  </button>
                  {showPlusMenu && (
                    <div
                      ref={plusMenuRef}
                      className="absolute bottom-full left-0 mb-2 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50"
                    >
                      <button
                        onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                        onClick={() => { setShowPlusMenu(false); fileInputRef.current?.click(); }}
                        className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                      >
                        <Paperclip size={16} className="text-claude-textSecondary" />
                        Add files or photos
                      </button>
                      <button
                        onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                        onClick={() => { setShowPlusMenu(false); setShowGithubModal(true); }}
                        className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                      >
                        <Github size={16} className="text-claude-textSecondary" />
                        Add from GitHub
                      </button>
                      {/* Add to project submenu */}
                      <div className="relative" onMouseLeave={() => setShowProjectsSubmenu(false)}>
                        <button
                          onMouseEnter={() => { setShowProjectsSubmenu(true); setShowSkillsSubmenu(false); }}
                          onClick={() => setShowProjectsSubmenu(prev => !prev)}
                          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                        >
                          <div className="flex items-center gap-3">
                            <IconProjects size={16} className="text-claude-textSecondary scale-[1.6] dark:[filter:brightness(0)_invert(1)_brightness(0.68)_sepia(0.18)]" />
                            Add to project
                          </div>
                          <ChevronDown size={14} className="text-claude-textSecondary -rotate-90" />
                        </button>
                        {showProjectsSubmenu && (
                          <div className="absolute left-full bottom-0 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50 max-h-[30vh] overflow-y-auto">
                            {projectList.length > 0 ? projectList.map(p => {
                              const isSelected = (activeId && currentProjectId === p.id) || (!activeId && pendingProjectId === p.id);
                              return (
                                <button
                                  key={p.id}
                                  onClick={() => handleAttachToProject(p)}
                                  className="w-full flex items-center justify-between gap-2 px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors text-left"
                                >
                                  <div className="flex items-center gap-2 min-w-0">
                                    <IconProjects size={26} className="text-claude-textSecondary flex-shrink-0 dark:[filter:brightness(0)_invert(1)_brightness(0.68)_sepia(0.18)]" />
                                    <span className="truncate">{p.name}</span>
                                  </div>
                                  {isSelected && <Check size={14} className="text-claude-textSecondary flex-shrink-0" />}
                                </button>
                              );
                            }) : (
                              <div className="px-4 py-2 text-[12px] text-claude-textSecondary italic">No projects yet</div>
                            )}
                            <div className="border-t border-claude-border mt-1 pt-1">
                              <button
                                onClick={() => {
                                  setShowProjectsSubmenu(false);
                                  setShowPlusMenu(false);
                                  setNewProjectName('');
                                  setNewProjectDescription('');
                                  setShowNewProjectDialog(true);
                                }}
                                className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"
                              >
                                <Plus size={14} />
                                Start a new project
                              </button>
                            </div>
                          </div>
                        )}
                      </div>
                      <div className="relative" onMouseLeave={() => setShowSkillsSubmenu(false)}>
                        <button
                          onMouseEnter={() => { setShowSkillsSubmenu(true); setShowProjectsSubmenu(false); }}
                          onClick={() => setShowSkillsSubmenu(prev => !prev)}
                          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                        >
                          <div className="flex items-center gap-3">
                            <FileText size={16} className="text-claude-textSecondary" />
                            Skills
                          </div>
                          <ChevronDown size={14} className="text-claude-textSecondary -rotate-90" />
                        </button>
                        {showSkillsSubmenu && (
                          <div className="absolute left-full bottom-0 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50 max-h-[30vh] overflow-y-auto">
                            {enabledSkills.length > 0 ? enabledSkills.map(skill => (
                              <button
                                key={skill.id}
                                onClick={() => {
                                  setShowPlusMenu(false); setShowSkillsSubmenu(false);
                                  const slug = skill.name.toLowerCase().replace(/\s+/g, '-');
                                  setSelectedSkill({ name: skill.name, slug, description: skill.description });
                                  setInputText(prev => prev ? `/${slug} ${prev}` : `/${slug} `);
                                  inputRef.current?.focus();
                                }}
                                className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors truncate"
                              >
                                {skill.name}
                              </button>
                            )) : (
                              <div className="px-4 py-2 text-[12px] text-claude-textSecondary italic">No skills enabled</div>
                            )}
                            <div className="border-t border-claude-border mt-1 pt-1">
                              <button
                                onClick={() => { setShowPlusMenu(false); window.location.hash = '#/customize'; }}
                                className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"
                              >
                                <FileText size={14} />
                                Manage skills
                              </button>
                            </div>
                          </div>
                        )}
                      </div>
                      {/* Research toggle */}
                      <div className="border-t border-claude-border mt-1 pt-1">
                        <button
                          onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                          onClick={() => { toggleResearchMode(); setShowPlusMenu(false); }}
                          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] hover:bg-claude-hover transition-colors"
                        >
                          <div className="flex items-center gap-3">
                            <IconResearch size={16} className={researchMode ? 'text-[#2E7CF6]' : 'text-claude-textSecondary'} />
                            <span className={researchMode ? 'text-[#2E7CF6] font-medium' : 'text-claude-text'}>Research</span>
                          </div>
                          {researchMode && <Check size={14} className="text-[#2E7CF6]" />}
                        </button>
                      </div>
                      {/* Web search indicator — always on when provider supports it, not togglable */}
                      <div>
                        <button
                          onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                          onClick={() => {
                            if (currentProviderSupportsWebSearch) {
                              setShowPlusMenu(false);
                            } else {
                              setWebSearchToast('当前模型的供应商不支持网页搜索');
                              setShowPlusMenu(false);
                            }
                          }}
                          className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] hover:bg-claude-hover transition-colors"
                        >
                          <div className="flex items-center gap-3">
                            <IconWebSearch size={16} className={currentProviderSupportsWebSearch ? 'text-[#2E7CF6]' : 'text-claude-textSecondary'} />
                            <span className={currentProviderSupportsWebSearch ? 'text-[#2E7CF6] font-medium' : 'text-claude-text'}>Web search</span>
                          </div>
                          {currentProviderSupportsWebSearch && <Check size={14} className="text-[#2E7CF6]" />}
                        </button>
                      </div>
                    </div>
                  )}
                  {/* Blue research badge next to + button when enabled */}
                  {researchMode && (
                    <div className="group/research relative ml-1 flex items-center bg-[#DBEAFE] dark:bg-[#1E3A5F] rounded-lg p-1.5">
                      <IconResearch size={16} className="text-[#2E7CF6] flex-shrink-0" />
                      <span className="inline-flex items-center overflow-hidden w-0 group-hover/research:w-[18px] transition-[width] duration-150 ease-out">
                        <button
                          onClick={toggleResearchMode}
                          className="ml-1 flex-shrink-0 flex items-center justify-center hover:opacity-70 transition-opacity"
                          aria-label="Disable research mode"
                        >
                          <X size={14} className="text-[#2E7CF6]" />
                        </button>
                      </span>
                      <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-1 bg-[#2a2a2a] text-white dark:bg-[#e8e8e8] dark:text-[#1a1a1a] rounded-md text-[11px] whitespace-nowrap opacity-0 group-hover/research:opacity-100 pointer-events-none transition-opacity">
                        Research mode
                      </div>
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-3">
                  <ModelSelector
                    currentModelString={currentModelString}
                    models={selectorModels}
                    onModelChange={handleModelChange}
                    isNewChat={true}
                  />
                  <button
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={() => handleSend()}
                    disabled={(!inputText.trim() && !pendingFiles.some(f => f.status === 'done')) || loading || pendingFiles.some(f => f.status === 'uploading')}
                    className="p-2 bg-[#C6613F] text-white rounded-lg hover:bg-[#D97757] transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                  >
                    <ArrowUp size={22} strokeWidth={2.5} />
                  </button>
                </div>
              </div>
            </div>
            {false && (
              <div className="mx-4 flex items-center justify-between px-4 py-1.5 bg-claude-bgSecondary border-x border-b border-claude-border rounded-b-xl text-claude-textSecondary text-xs">
                <span>您当前没有可用套餐，无法发送消息</span>
                <button
                  onClick={() => window.dispatchEvent(new CustomEvent('open-upgrade'))}
                  className="px-2 py-0.5 bg-claude-btnHover hover:bg-claude-hover text-claude-text text-xs font-medium rounded transition-colors border border-claude-border hover:border-blue-500 hover:text-blue-600"
                >
                  购买套餐
                </button>
              </div>
            )}
          </div>
        </div>
        <AddFromGithubModal
          isOpen={showGithubModal}
          onClose={() => setShowGithubModal(false)}
          currentContextTokens={contextInfo?.tokens || 0}
          contextLimit={contextInfo?.limit || 200000}
          onConfirm={handleGithubAdd}
        />
        {sharedProjectOverlays}
      </div>
    );
  }

  // MODE 2: Chat Interface (Has ID or Messages)
  return (
    <div className="flex-1 bg-claude-bg h-full flex flex-col overflow-clip text-claude-text chat-root chat-font-scope">
      {/* Content area - positioning container for scroll + bottom bars */}
      <div className="flex-1 min-h-0 relative">
        <div
          className="absolute inset-0 overflow-y-auto chat-scroll"
          style={{ paddingBottom: `${inputHeight}px` }}
          ref={scrollContainerRef}
          onScroll={handleScroll}
        >
          <div
            className="w-full mx-auto px-4 py-8 pb-2"
            style={{ maxWidth: `${tunerConfig?.mainContentWidth || 768}px` }}
          >
            <MessageList
              messages={messages}
              loading={loading}
              expandedMessages={expandedMessages}
              editingMessageIdx={editingMessageIdx}
              editingContent={editingContent}
              copiedMessageIdx={copiedMessageIdx}
              compactStatus={compactStatus}
              onSetEditingContent={setEditingContent}
              onEditCancel={handleEditCancel}
              onEditSave={handleEditSave}
              onToggleExpand={toggleMessageExpand}
              onResend={handleResendMessage}
              onEdit={handleEditMessage}
              onCopy={handleCopyMessage}
              onOpenDocument={onOpenDocument}
              onSetMessages={setMessages}
              messageContentRefs={messageContentRefs}
              onOpenResearch={(msgId) => setOpenedResearchMsgId(msgId)}
            />
            <div ref={messagesEndRef} />
          </div>
        </div>

        {/* 免责声明 - 固定在最底部 */}
        <div className="absolute bottom-0 left-0 z-10 bg-claude-bg flex items-center justify-center text-[12px] text-claude-textSecondary h-7 pointer-events-none font-sans" style={{ right: `${scrollbarWidth}px` }}>
          Claude is AI and can make mistakes. Please double-check responses.
        </div>

        {/* 输入框 - 浮动在内容上方，底部距离可调 */}
        <div className="absolute left-0 right-0 z-20 pointer-events-none" style={{ bottom: `${inputBarBottom + 28}px`, paddingLeft: '16px', paddingRight: `${16 + scrollbarWidth}px` }}>
          <div
            className="mx-auto pointer-events-auto"
            style={{ maxWidth: `${inputBarWidth}px` }}
          >
            <div className="w-full relative group" ref={inputWrapperRef}>
              <input
                type="file"
                ref={fileInputRef}
                className="hidden"
                multiple
                accept={ACCEPTED_TYPES}
                onChange={(e) => {
                  if (e.target.files) handleFilesSelected(e.target.files);
                  e.target.value = '';
                }}
              />
              <div
                className={`bg-claude-input border shadow-[0_2px_8px_rgba(0,0,0,0.02)] hover:shadow-[0_2px_8px_rgba(0,0,0,0.08)] hover:border-[#CCC] dark:hover:border-[#5a5a58] focus-within:shadow-[0_2px_8px_rgba(0,0,0,0.08)] focus-within:border-[#CCC] dark:focus-within:border-[#5a5a58] transition-all duration-200 flex flex-col font-sans ${isDragging ? 'border-[#D97757] bg-orange-50/30' : 'border-claude-border dark:border-[#3a3a38]'}`}
                style={{ borderRadius: `${inputBarRadius}px` }}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
              >
                <FileUploadPreview files={pendingFiles} onRemove={handleRemoveFile} />
                <div className="relative">
                  <SkillInputOverlay
                    text={inputText}
                    className="px-4 pt-4 pb-0 text-[16px] font-sans font-[350]"
                    style={{ height: `${inputBarBaseHeight}px`, minHeight: '16px', boxSizing: 'border-box', overflow: 'hidden' }}
                  />
                  <textarea
                    ref={inputRef}
                    className={`w-full px-4 pt-4 pb-0 placeholder:text-claude-textSecondary text-[16px] outline-none resize-none bg-transparent font-sans font-[350] ${inputText.match(/^\/[a-zA-Z0-9_-]+/) ? 'text-transparent caret-claude-text' : 'text-claude-text'}`}
                    style={{ height: `${inputBarBaseHeight}px`, minHeight: '16px', boxSizing: 'border-box', overflowY: 'hidden' }}
                    placeholder={selectedSkill ? `Describe what you want ${selectedSkill.name} to do...` : "How can I help you today?"}
                    value={inputText}
                    onChange={(e) => {
                      setInputText(e.target.value);
                      adjustTextareaHeight();
                    }}
                    onKeyDown={(e) => {
                      if (e.key === 'Backspace' && selectedSkill) {
                        const pos = (e.target as HTMLTextAreaElement).selectionStart;
                        const skillPrefix = `/${selectedSkill.slug} `;
                        if (pos > 0 && pos <= skillPrefix.length && inputText.startsWith(skillPrefix.slice(0, pos))) {
                          e.preventDefault();
                          setInputText(inputText.slice(skillPrefix.length));
                          setSelectedSkill(null);
                          return;
                        }
                      }
                      handleKeyDown(e);
                    }}
                    onPaste={handlePaste}
                  />
                </div>
                <div className="px-4 pb-3 pt-1 flex items-center justify-between">
                  <div className="relative flex items-center">
                    <button
                      ref={plusBtnRef}
                      onClick={() => setShowPlusMenu(prev => !prev)}
                      className="p-2 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
                    >
                      <IconPlus size={20} />
                    </button>
                    {showPlusMenu && (
                      <div
                        ref={plusMenuRef}
                        className="absolute bottom-full left-0 mb-2 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50"
                      >
                        <button
                          onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                          onClick={() => {
                            setShowPlusMenu(false);
                            fileInputRef.current?.click();
                          }}
                          className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                        >
                          <Paperclip size={16} className="text-claude-textSecondary" />
                          Add files or photos
                        </button>
                        <button
                          onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                          onClick={() => {
                            setShowPlusMenu(false);
                            setShowGithubModal(true);
                          }}
                          className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                        >
                          <Github size={16} className="text-claude-textSecondary" />
                          Add from GitHub
                        </button>
                        {/* Add to project submenu */}
                        <div className="relative" onMouseLeave={() => setShowProjectsSubmenu(false)}>
                          <button
                            onMouseEnter={() => { setShowProjectsSubmenu(true); setShowSkillsSubmenu(false); }}
                            onClick={() => setShowProjectsSubmenu(prev => !prev)}
                            className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                          >
                            <div className="flex items-center gap-3">
                              <IconProjects size={16} className="text-claude-textSecondary scale-[1.6] dark:[filter:brightness(0)_invert(1)_brightness(0.68)_sepia(0.18)]" />
                              Add to project
                            </div>
                            <ChevronDown size={14} className="text-claude-textSecondary -rotate-90" />
                          </button>
                          {showProjectsSubmenu && (
                            <div className="absolute left-full bottom-0 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50 max-h-[30vh] overflow-y-auto">
                              {projectList.length > 0 ? projectList.map(p => {
                                const isSelected = (activeId && currentProjectId === p.id) || (!activeId && pendingProjectId === p.id);
                                return (
                                  <button
                                    key={p.id}
                                    onClick={() => handleAttachToProject(p)}
                                    className="w-full flex items-center justify-between gap-2 px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors text-left"
                                  >
                                    <div className="flex items-center gap-2 min-w-0">
                                      <IconProjects size={26} className="text-claude-textSecondary flex-shrink-0 dark:[filter:brightness(0)_invert(1)_brightness(0.68)_sepia(0.18)]" />
                                      <span className="truncate">{p.name}</span>
                                    </div>
                                    {isSelected && <Check size={14} className="text-claude-textSecondary flex-shrink-0" />}
                                  </button>
                                );
                              }) : (
                                <div className="px-4 py-2 text-[12px] text-claude-textSecondary italic">No projects yet</div>
                              )}
                              <div className="border-t border-claude-border mt-1 pt-1">
                                <button
                                  onClick={() => {
                                    setShowProjectsSubmenu(false);
                                    setShowPlusMenu(false);
                                    setNewProjectName('');
                                    setNewProjectDescription('');
                                    setShowNewProjectDialog(true);
                                  }}
                                  className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"
                                >
                                  <Plus size={14} />
                                  Start a new project
                                </button>
                              </div>
                            </div>
                          )}
                        </div>
                        {/* Skills submenu */}
                        <div className="relative" onMouseLeave={() => setShowSkillsSubmenu(false)}>
                          <button
                            onMouseEnter={() => { setShowSkillsSubmenu(true); setShowProjectsSubmenu(false); }}
                            onClick={(e) => { e.stopPropagation(); setShowSkillsSubmenu(prev => !prev); }}
                            className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                          >
                            <div className="flex items-center gap-3">
                              <FileText size={16} className="text-claude-textSecondary" />
                              Skills
                            </div>
                            <ChevronDown size={14} className="text-claude-textSecondary -rotate-90" />
                          </button>
                          {showSkillsSubmenu && enabledSkills.length > 0 && (
                            <div className="absolute left-full bottom-0 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50 max-h-[30vh] overflow-y-auto">
                              {enabledSkills.map(skill => (
                                <button
                                  key={skill.id}
                                  onClick={() => {
                                    setShowPlusMenu(false);
                                    setShowSkillsSubmenu(false);
                                    const slug = skill.name.toLowerCase().replace(/\s+/g, '-');
                                    setSelectedSkill({ name: skill.name, slug, description: skill.description });
                                    setInputText(prev => prev ? `/${slug} ${prev}` : `/${slug} `);
                                    inputRef.current?.focus();
                                  }}
                                  className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors truncate"
                                >
                                  {skill.name}
                                </button>
                              ))}
                              <div className="border-t border-claude-border mt-1 pt-1">
                                <button
                                  onClick={() => {
                                    setShowPlusMenu(false);
                                    setShowSkillsSubmenu(false);
                                    window.location.hash = '#/customize';
                                  }}
                                  className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"
                                >
                                  <FileText size={14} />
                                  Manage skills
                                </button>
                              </div>
                            </div>
                          )}
                          {showSkillsSubmenu && enabledSkills.length === 0 && (
                            <div className="absolute left-full bottom-0 w-[220px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.12)] py-1.5 z-50">
                              <div className="px-4 py-2 text-[12px] text-claude-textSecondary italic">No skills enabled</div>
                              <div className="border-t border-claude-border mt-1 pt-1">
                                <button
                                  onClick={() => {
                                    setShowPlusMenu(false);
                                    window.location.hash = '#/customize';
                                  }}
                                  className="w-full flex items-center gap-3 px-4 py-2 text-[13px] text-claude-textSecondary hover:bg-claude-hover transition-colors"
                                >
                                  <FileText size={14} />
                                  Manage skills
                                </button>
                              </div>
                            </div>
                          )}
                        </div>
                        <button
                          onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                          onClick={() => {
                            setShowPlusMenu(false);
                            if (!activeId || compactStatus.state === 'compacting') return;
                            setCompactInstruction('');
                            setShowCompactDialog(true);
                          }}
                          className="w-full flex items-center gap-3 px-4 py-2.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                        >
                          <ListCollapse size={16} className="text-claude-textSecondary" />
                          Compact conversation
                        </button>
                        {/* Research toggle */}
                        <div className="border-t border-claude-border mt-1 pt-1">
                          <button
                            onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                            onClick={() => { toggleResearchMode(); setShowPlusMenu(false); }}
                            className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] hover:bg-claude-hover transition-colors"
                          >
                            <div className="flex items-center gap-3">
                              <IconResearch size={16} className={researchMode ? 'text-[#2E7CF6]' : 'text-claude-textSecondary'} />
                              <span className={researchMode ? 'text-[#2E7CF6] font-medium' : 'text-claude-text'}>Research</span>
                            </div>
                            {researchMode && <Check size={14} className="text-[#2E7CF6]" />}
                          </button>
                        </div>
                        {/* Web search indicator — always on when provider supports it, not togglable */}
                        <div>
                          <button
                            onMouseEnter={() => { setShowSkillsSubmenu(false); setShowProjectsSubmenu(false); }}
                            onClick={() => {
                              if (currentProviderSupportsWebSearch) {
                                setShowPlusMenu(false);
                              } else {
                                setWebSearchToast('当前模型的供应商不支持网页搜索');
                                setShowPlusMenu(false);
                              }
                            }}
                            className="w-full flex items-center justify-between px-4 py-2.5 text-[13px] hover:bg-claude-hover transition-colors"
                          >
                            <div className="flex items-center gap-3">
                              <IconWebSearch size={16} className={currentProviderSupportsWebSearch ? 'text-[#2E7CF6]' : 'text-claude-textSecondary'} />
                              <span className={currentProviderSupportsWebSearch ? 'text-[#2E7CF6] font-medium' : 'text-claude-text'}>Web search</span>
                            </div>
                            {currentProviderSupportsWebSearch && <Check size={14} className="text-[#2E7CF6]" />}
                          </button>
                        </div>
                      </div>
                    )}
                    {/* Blue research badge next to + button when enabled */}
                    {researchMode && (
                      <div className="group/research relative ml-1 flex items-center bg-[#DBEAFE] dark:bg-[#1E3A5F] rounded-lg p-1.5">
                        <IconResearch size={16} className="text-[#2E7CF6] flex-shrink-0" />
                        <span className="inline-flex items-center overflow-hidden w-0 group-hover/research:w-[18px] transition-[width] duration-150 ease-out">
                          <button
                            onClick={toggleResearchMode}
                            className="ml-1 flex-shrink-0 flex items-center justify-center hover:opacity-70 transition-opacity"
                            aria-label="Disable research mode"
                          >
                            <X size={14} className="text-[#2E7CF6]" />
                          </button>
                        </span>
                        <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-1 bg-[#2a2a2a] text-white dark:bg-[#e8e8e8] dark:text-[#1a1a1a] rounded-md text-[11px] whitespace-nowrap opacity-0 group-hover/research:opacity-100 pointer-events-none transition-opacity">
                          Research mode
                        </div>
                      </div>
                    )}
                    {contextInfo && contextInfo.tokens > 0 && (() => {
                      const pct = Math.min(contextInfo.tokens / contextInfo.limit, 1);
                      const color = pct > 0.8 ? '#dc2626' : pct > 0.5 ? '#d97706' : '#6b7280';
                      const r = 7, c = 2 * Math.PI * r, dash = pct * c;
                      const label = contextInfo.tokens.toLocaleString() + ' tokens';
                      const pctLabel = (pct * 100).toFixed(1) + '% 上下文已使用';
                      return (
                        <div className="flex items-center gap-1 ml-1 select-none" title={pctLabel}>
                          <svg width="18" height="18" viewBox="0 0 18 18">
                            <circle cx="9" cy="9" r={r} fill="none" stroke="#d4d4d4" strokeWidth="2" />
                            <circle cx="9" cy="9" r={r} fill="none" stroke={color} strokeWidth="2"
                              strokeDasharray={`${dash} ${c}`} strokeLinecap="round"
                              transform="rotate(-90 9 9)" />
                          </svg>
                          <span className="text-[11px] whitespace-nowrap" style={{ color: '#6b7280' }}>{label}</span>
                        </div>
                      );
                    })()}
                  </div>
                  <div className="flex items-center gap-3">
                    <ModelSelector
                      currentModelString={currentModelString}
                      models={selectorModels}
                      onModelChange={handleModelChange}
                      isNewChat={false}
                      dropdownPosition="top"
                    />
                    {loading ? (
                      <button
                        onClick={handleStop}
                        className="p-2 text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="24"
                          height="24"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        >
                          <circle cx="12" cy="12" r="10" />
                          <rect x="9" y="9" width="6" height="6" fill="currentColor" stroke="none" />
                        </svg>
                      </button>
                    ) : (
                      <button
                        onMouseDown={(e) => e.preventDefault()}
                        onClick={() => handleSend()}
                        disabled={(!inputText.trim() && !pendingFiles.some(f => f.status === 'done')) || pendingFiles.some(f => f.status === 'uploading')}
                        className="p-2 bg-[#C6613F] text-white rounded-lg hover:bg-[#D97757] transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                      >
                        <ArrowUp size={22} strokeWidth={2.5} />
                      </button>
                    )}
                  </div>
                </div>
              </div>
              {false && (
                <div className="mx-4 flex items-center justify-between px-4 py-1.5 bg-claude-bgSecondary border-x border-b border-claude-border rounded-b-xl text-claude-textSecondary text-xs pointer-events-auto">
                  <span>您当前没有可用套餐，无法发送消息</span>
                  <button
                    onClick={() => window.dispatchEvent(new CustomEvent('open-upgrade'))}
                    className="px-2 py-0.5 bg-claude-btnHover hover:bg-claude-hover text-claude-text text-xs font-medium rounded transition-colors border border-claude-border hover:border-blue-500 hover:text-blue-600"
                  >
                    购买套餐
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Plan mode banner */}
      {planMode && (
        <div className="fixed top-0 left-0 right-0 z-[100] flex items-center justify-center pointer-events-none" style={{ paddingLeft: 'var(--sidebar-width, 260px)' }}>
          <div className="mt-2 px-4 py-1.5 bg-amber-500/90 text-white text-[13px] font-medium rounded-full shadow-lg pointer-events-auto flex items-center gap-2">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></svg>
            Plan Mode — Claude is planning, not executing
          </div>
        </div>
      )}

      {/* Active tasks progress */}
      {activeTasks.size > 0 && (
        <div className="fixed bottom-[140px] right-6 z-[90] flex flex-col gap-1.5 max-w-[320px]">
          {Array.from(activeTasks.entries()).map(([taskId, task]) => (
            <div key={taskId} className="bg-claude-bg border border-claude-border rounded-lg px-3 py-2 shadow-lg flex items-center gap-2 text-[12px] text-claude-textSecondary animate-pulse">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="animate-spin flex-shrink-0"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
              <span className="truncate">{task.last_tool_name ? `${task.description} (${task.last_tool_name})` : task.description}</span>
            </div>
          ))}
        </div>
      )}

      {/* AskUserQuestion dialog */}
      {askUserDialog && (
        <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40">
          <div className="bg-claude-bg border border-claude-border rounded-2xl shadow-xl w-[480px] max-h-[80vh] overflow-y-auto" onClick={e => e.stopPropagation()}>
            <div className="px-5 pt-5 pb-3">
              <h3 className="text-[15px] font-semibold text-claude-text mb-1 flex items-center gap-2">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                Claude needs your input
              </h3>
            </div>
            <div className="px-5 pb-4 flex flex-col gap-4">
              {askUserDialog.questions.map((q, qi) => (
                <div key={qi} className="flex flex-col gap-1.5">
                  <label className="text-[13px] font-medium text-claude-text">{q.question}</label>
                  {q.options && q.options.length > 0 ? (
                    <div className="flex flex-col gap-1">
                      {q.options.map((opt, oi) => {
                        const selected = askUserDialog.answers[q.question] === opt.label;
                        return (
                          <button
                            key={oi}
                            onClick={() => setAskUserDialog(prev => prev ? { ...prev, answers: { ...prev.answers, [q.question]: opt.label } } : null)}
                            className={`text-left px-3 py-2 rounded-lg border text-[13px] transition-colors ${selected ? 'border-[#C6613F] bg-[#C6613F]/10 text-claude-text' : 'border-claude-border hover:bg-claude-hover text-claude-textSecondary'}`}
                          >
                            <div className="font-medium text-claude-text">{opt.label}</div>
                            {opt.description && <div className="text-[12px] text-claude-textSecondary mt-0.5">{opt.description}</div>}
                          </button>
                        );
                      })}
                    </div>
                  ) : (
                    <input
                      type="text"
                      className="w-full bg-claude-input border border-claude-border rounded-lg px-3 py-2 text-[13px] text-claude-text outline-none focus:border-claude-textSecondary/40 transition-colors"
                      placeholder="Type your answer..."
                      value={askUserDialog.answers[q.question] || ''}
                      onChange={e => setAskUserDialog(prev => prev ? { ...prev, answers: { ...prev.answers, [q.question]: e.target.value } } : null)}
                      onKeyDown={e => {
                        if (e.key === 'Enter') {
                          e.preventDefault();
                          document.getElementById('ask-user-submit-btn')?.click();
                        }
                      }}
                      autoFocus={qi === 0}
                    />
                  )}
                </div>
              ))}
            </div>
            <div className="flex items-center justify-end gap-2 px-5 pb-4">
              <button
                id="ask-user-submit-btn"
                onClick={async () => {
                  if (!askUserDialog || !activeId) return;
                  const { request_id, tool_use_id, answers } = askUserDialog;
                  setAskUserDialog(null);
                  try {
                    await answerUserQuestion(activeId, request_id, tool_use_id, answers);
                  } catch (err) {
                    console.error('Failed to send answer:', err);
                  }
                }}
                className="px-4 py-1.5 text-[13px] text-white bg-[#C6613F] hover:bg-[#D97757] rounded-lg transition-colors font-medium"
              >
                Submit
              </button>
            </div>
          </div>
        </div>
      )}

      <AddFromGithubModal
        isOpen={showGithubModal}
        onClose={() => setShowGithubModal(false)}
        currentContextTokens={contextInfo?.tokens || 0}
        contextLimit={contextInfo?.limit || 200000}
        onConfirm={(bundle: { repoFullName: string; ref: string; file?: File }) => {
          if (!bundle || !bundle.file) return;
          handleFilesSelected([bundle.file], {
            source: 'github',
            ghRepo: bundle.repoFullName,
            ghRef: bundle.ref,
          });
        }}
      />

      {/* Clawparrot login-required modal: shown when a non-logged-in clawparrot
          user tries to send their first message. Lets them go to login page or
          switch to self-hosted mode in settings. */}
      {showLoginRequired && (
        <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40 backdrop-blur-sm p-4">
          <div className="bg-claude-input border border-claude-border rounded-2xl shadow-xl w-[460px] overflow-hidden">
            <div className="px-6 pt-6 pb-4">
              <h3 className="text-[16px] font-semibold text-claude-text mb-3">需要登录 Claude Rust 账号</h3>
              <p className="text-[14px] text-claude-textSecondary leading-relaxed">
                你当前在 <span className="text-claude-text font-medium">Claude Rust</span> 模式下，需要登录 <span className="font-mono text-claude-text">Claude Rust</span> 账号才能使用。
                <br /><br />
                如果你还没有账号，请先去 <span className="font-mono text-claude-text">Claude Rust</span> 注册一个。
                <br /><br />
                或者你也可以在设置的 General 页面切换到 <span className="text-claude-text font-medium">自部署</span> 模式，用你自己的 API Key。
              </p>
            </div>
            <div className="px-5 pb-5 pt-2 flex flex-col gap-2">
              <button
                onClick={() => {
                  pendingLoginSendRef.current = null;
                  setShowLoginRequired(false);
                  navigate('/login');
                }}
                className="w-full px-5 py-2.5 text-[14px] font-medium bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity"
              >
                去登录
              </button>
              <button
                onClick={() => {
                  pendingLoginSendRef.current = null;
                  setShowLoginRequired(false);
                }}
                className="w-full px-5 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text transition-colors"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Cross-mode warning: conversation's model isn't available in current user_mode */}
      {crossModeWarning && crossModeWarning.convId === activeId && (
        <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40 backdrop-blur-sm p-4">
          <div className="bg-claude-input border border-claude-border rounded-2xl shadow-xl w-[460px] overflow-hidden">
            <div className="px-6 pt-6 pb-4">
              <h3 className="text-[16px] font-semibold text-claude-text mb-3">对话模型与当前模式不匹配</h3>
              <p className="text-[14px] text-claude-textSecondary leading-relaxed">
                此对话使用的是 <span className="font-mono text-claude-text">{crossModeWarning.originalModel}</span>，
                它属于 <span className="text-claude-text font-medium">{crossModeWarning.otherMode === 'selfhosted' ? '自部署' : 'Clawparrot'}</span> 模式下的模型。
                <br /><br />
                你当前在 <span className="text-claude-text font-medium">{crossModeWarning.otherMode === 'selfhosted' ? 'Clawparrot' : '自部署'}</span> 模式。
                是要继续在原模式下使用这个模型，还是切换到当前模式下的模型？
              </p>
            </div>
            <div className="px-5 pb-5 pt-2 flex flex-col gap-2">
              <button
                onClick={() => {
                  // Keep cross-mode: persist override, dismiss, then proceed with the pending send
                  setCrossModeOverride(crossModeWarning.convId, crossModeWarning.otherMode);
                  const fire = pendingCrossModeSendRef.current;
                  pendingCrossModeSendRef.current = null;
                  setCrossModeWarning(null);
                  if (fire) setTimeout(fire, 0);
                }}
                className="w-full px-5 py-2.5 text-[14px] font-medium text-claude-text border border-claude-border hover:bg-claude-hover rounded-lg transition-colors text-left"
              >
                继续使用 <span className="font-mono">{crossModeWarning.originalModel}</span>
                <div className="text-[11px] text-claude-textSecondary mt-0.5 font-normal">
                  这次和以后都通过 {crossModeWarning.otherMode === 'selfhosted' ? '自部署' : 'Clawparrot'} 模式发送
                </div>
              </button>
              <button
                onClick={async () => {
                  // Switch model: change conv.model to the current-mode fallback, clear any
                  // stale override, dismiss the warning, then proceed with the pending send.
                  const target = crossModeWarning.fallbackModel;
                  const convId = crossModeWarning.convId;
                  clearCrossModeOverride(convId);
                  setCurrentModelString(target);
                  try { await updateConversation(convId, { model: target }); } catch {}
                  const fire = pendingCrossModeSendRef.current;
                  pendingCrossModeSendRef.current = null;
                  setCrossModeWarning(null);
                  if (fire) setTimeout(fire, 0);
                }}
                className="w-full px-5 py-2.5 text-[14px] font-medium bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity text-left"
              >
                切换到 <span className="font-mono">{crossModeWarning.fallbackModel}</span>
                <div className="text-[11px] opacity-70 mt-0.5 font-normal">
                  切完后这个对话会用当前 {crossModeWarning.otherMode === 'selfhosted' ? 'Clawparrot' : '自部署'} 模式
                </div>
              </button>
              <button
                onClick={() => {
                  pendingCrossModeSendRef.current = null;
                  setCrossModeWarning(null);
                }}
                className="w-full px-5 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text transition-colors mt-1"
              >
                取消，先不发送
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Compact conversation dialog */}
      {showCompactDialog && (
        <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40" onClick={() => setShowCompactDialog(false)}>
          <div className="bg-claude-bg border border-claude-border rounded-2xl shadow-xl w-[440px] overflow-hidden" onClick={e => e.stopPropagation()}>
            <div className="px-5 pt-5 pb-3">
              <h3 className="text-[15px] font-semibold text-claude-text mb-1">Compact conversation</h3>
              <p className="text-[13px] text-claude-textSecondary leading-snug">
                Summarize the conversation history to free up context space. The engine will preserve key decisions and context.
              </p>
            </div>
            <div className="px-5 pb-3">
              <textarea
                className="w-full bg-claude-input border border-claude-border rounded-lg px-3 py-2 text-[13px] text-claude-text placeholder:text-claude-textSecondary/50 outline-none focus:border-claude-textSecondary/40 transition-colors resize-none"
                rows={3}
                placeholder="Optional: add instructions for the summary (e.g. 'preserve all API endpoint details')"
                value={compactInstruction}
                onChange={e => setCompactInstruction(e.target.value)}
                onKeyDown={e => {
                  if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                    e.preventDefault();
                    document.getElementById('compact-confirm-btn')?.click();
                  }
                }}
                autoFocus
              />
            </div>
            <div className="flex items-center justify-end gap-2 px-5 pb-4">
              <button
                onClick={() => setShowCompactDialog(false)}
                className="px-3.5 py-1.5 text-[13px] text-claude-textSecondary hover:text-claude-text rounded-lg hover:bg-claude-hover transition-colors"
              >
                Cancel
              </button>
              <button
                id="compact-confirm-btn"
                onClick={async () => {
                  setShowCompactDialog(false);
                  if (!activeId || compactStatus.state === 'compacting') return;
                  setCompactStatus({ state: 'compacting' });
                  try {
                    const instruction = compactInstruction.trim() || undefined;
                    const result = await compactConversation(activeId, instruction);
                    await loadConversation(activeId);
                    const newContextInfo = await getContextSize(activeId);
                    setContextInfo(newContextInfo);
                    setCompactStatus({ state: 'done', message: `Compacted ${result.messagesCompacted} messages, saved ~${result.tokensSaved} tokens` });
                    setTimeout(() => setCompactStatus({ state: 'idle' }), 4000);
                  } catch (err) {
                    console.error('Compact failed:', err);
                    setCompactStatus({ state: 'error', message: 'Compaction failed' });
                    setTimeout(() => setCompactStatus({ state: 'idle' }), 3000);
                  }
                }}
                className="px-3.5 py-1.5 text-[13px] text-white bg-[#C6613F] hover:bg-[#D97757] rounded-lg transition-colors font-medium"
              >
                Compact
              </button>
            </div>
          </div>
        </div>
      )}

      {sharedProjectOverlays}

      {/* Research panel — fixed right-side drawer */}
      {openedResearchMsgId && (() => {
        const liveMsg = messages.find(m => m.id === openedResearchMsgId);
        if (!liveMsg || !liveMsg.research) return null;
        return (
          <>
            <div
              className="fixed inset-0 z-[60] bg-black/20"
              onClick={() => setOpenedResearchMsgId(null)}
            />
            <div className="fixed top-0 right-0 bottom-0 w-[440px] z-[61] bg-claude-bg border-l border-claude-border shadow-2xl flex flex-col">
              <ResearchPanel research={liveMsg.research} onClose={() => setOpenedResearchMsgId(null)} />
            </div>
          </>
        );
      })()}
    </div>
  );
};

export default MainContent;
