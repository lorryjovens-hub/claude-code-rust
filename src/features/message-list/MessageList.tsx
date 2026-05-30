import React from 'react';
import {
  ChevronDown,
  FileText,
  RotateCcw,
  Pencil,
  Copy,
  Check,
  Globe,
  Info,
  Loader2,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import ClaudeLogo from '../../components/ClaudeLogo';
import CodeExecution from '../../components/CodeExecution';
import MarkdownRenderer from '../../components/MarkdownRenderer';
import SearchProcess from '../../components/SearchProcess';
import DocumentCreationProcess from '../../components/DocumentCreationProcess';
import ToolDiffView, { shouldUseDiffView, hasExpandableContent, getToolStats } from '../../components/ToolDiffView';
import DiffViewer from '../../components/DiffViewer';
import DocumentCard, { DocumentInfo } from '../../components/DocumentCard';
import MessageAttachments from '../../components/MessageAttachments';
import { IconResearch } from '../../components/Icons';
import CompactingStatus from '../compact/CompactingStatus';
import MessageExportActions from '../../components/chat/MessageExportActions';
import { PendingFile } from '../../components/FileUploadPreview';
import {
  extractTextContent,
  formatMessageTime,
  withAuthToken,
  normalizeMessageDocuments,
  normalizeDocumentDrafts,
} from '../../utils/messageHelpers';

// 草稿存储：在切换对话、打开设置页面时保留输入内容和附件
export const draftsStore = new Map<string, { text: string; files: PendingFile[]; height: number }>();

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
  onBranch: (idx: number) => void;
  onOpenDocument?: (doc: DocumentInfo) => void;
  onSetMessages: (messages: any[] | ((prev: any[]) => any[])) => void;
  messageContentRefs: React.MutableRefObject<Map<number, HTMLDivElement>>;
  onOpenResearch?: (msgId: string) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
}

const MessageList = React.memo<MessageListProps>(({
  messages, loading, expandedMessages, editingMessageIdx, editingContent,
  copiedMessageIdx, compactStatus, onSetEditingContent, onEditCancel, onEditSave,
  onToggleExpand, onResend, onEdit, onCopy, onBranch, onOpenDocument, onSetMessages,
  messageContentRefs, onOpenResearch, t,
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
      {(() => {
        const pairBoundaries = new Set<number>();
        let lastRole = '';
        for (let i = 0; i < messages.length; i++) {
          const msgRole = messages[i].role;
          if (msgRole === 'user' && lastRole === 'assistant') {
            pairBoundaries.add(i);
          }
          lastRole = msgRole;
        }
        return pairBoundaries;
      })()}
      {messages.map((msg: any, idx: number) => {
        const isCurrentlyStreaming = loading && idx === messages.length - 1;
        const isPairStart = msg.role === 'user' && (idx === 0 || messages[idx - 1]?.role === 'assistant');
        return (
        <div key={idx} className="group" style={{ marginBottom: isPairStart && idx > 0 ? '28px' : 'var(--msg-gap)' }}>
          {isPairStart && idx > 0 && (
            <div className="flex items-center gap-3 mb-5 mt-2">
              <div className="flex-1 h-px bg-claude-border" />
            </div>
          )}
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
                      {t('chat.editingMessageWarning')}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    <button
                      onClick={onEditCancel}
                      className="px-3 py-1.5 text-[13px] font-medium text-claude-text bg-white dark:bg-claude-bg border border-black/10 dark:border-white/10 hover:bg-gray-50 dark:hover:bg-claude-hover rounded-lg transition-colors"
                    >
                      {t('chat.cancelEdit')}
                    </button>
                    <button
                      onClick={onEditSave}
                      disabled={!editingContent.trim() || editingContent === msg.content}
                      className="px-3 py-1.5 text-[13px] font-medium text-white bg-claude-text hover:bg-claude-textSecondary rounded-lg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
                    >
                      {t('chat.saveEdit')}
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="flex flex-col items-end gap-1.5">
                {msg.attachments && msg.attachments.length > 0 && (
                  <div className="max-w-[85%] w-fit">
                    <MessageAttachments attachments={msg.attachments} onOpenDocument={onOpenDocument} />
                  </div>
                )}
                {(!msg.attachments || msg.attachments.length === 0) && msg.has_attachments === 1 && (
                  <div className="max-w-[85%] w-fit">
                    <div className="bg-[#F0EEE7] dark:bg-claude-input/60 text-claude-textSecondary px-4 py-2 text-[14px] rounded-xl font-sans italic border border-black/5 dark:border-white/5">
                      📎 Files attached
                    </div>
                  </div>
                )}
                {(() => {
                  // Handle content blocks (text + images)
                  const content = msg.content;
                  const isContentBlocks = Array.isArray(content);
                  if (isContentBlocks) {
                    return (
                      <div className="space-y-3">
                        {content.map((block: any, bi: number) => {
                          if (block.type === 'image_url' || block.type === 'image') {
                            const url = block.image_url?.url || block.source?.data || '';
                            const isBase64 = url.startsWith('data:') || (!url.startsWith('http') && url.length > 100);
                            const imgSrc = isBase64 ? url : '';
                            return (
                              <div key={bi} className="rounded-xl overflow-hidden border border-claude-border max-w-[400px]">
                                <img src={imgSrc} alt="Uploaded image" className="w-full h-auto max-h-[400px] object-contain bg-white" />
                              </div>
                            );
                          }
                          if (block.type === 'text' && block.text?.trim()) {
                            return <div key={bi} className="text-[15px] leading-relaxed whitespace-pre-wrap break-words">{block.text}</div>;
                          }
                          return null;
                        })}
                      </div>
                    );
                  }
                  const displayText = extractTextContent(content); return displayText && displayText.trim() !== ''; })() && (
                  <div className="max-w-[85%] w-fit">
                    <div
                      className="bg-[var(--msg-user-bg)] text-claude-text px-4 py-[10px] text-[15px] leading-relaxed font-sans font-[400] whitespace-pre-wrap break-words relative overflow-hidden"
                      style={{ borderRadius: 'var(--msg-user-radius)', maxHeight: expandedMessages.has(idx) ? 'none' : '300px' }}
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
                  <div className="flex items-center gap-0.5 overflow-hidden transition-all duration-200 ease-in-out max-w-0 opacity-0 group-hover:max-w-[320px] group-hover:opacity-100">
                    <button onClick={() => onResend(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="重新发送"><RotateCcw size={14} /></button>
                    <button onClick={() => onEdit(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="编辑"><Pencil size={14} /></button>
                    <button onClick={() => onBranch(idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="分支"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg></button>
                    <button onClick={() => onCopy(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="复制">
                      {copiedMessageIdx === idx ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
                    </button>
                    <div className="ml-0.5 pl-0.5 border-l border-black/10 dark:border-white/10">
                      <MessageExportActions
                        content={msg.content}
                        role="user"
                        timestamp={msg.created_at ? formatMessageTime(msg.created_at) : undefined}
                      />
                    </div>
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
                        return summary || t('chat.thinking');
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

              <MarkdownRenderer content={(msg as any)._finalText ?? extractTextContent(msg.content)} citations={msg.citations} isStreaming={isCurrentlyStreaming} />
              {(msg as any).codeDiffs && (msg as any).codeDiffs.length > 0 && (
                <div className="mt-2 mb-1 space-y-2">
                  {(msg as any).codeDiffs.map((diff: any) => (
                    <DiffViewer
                      key={diff.id}
                      filePath={diff.file_path}
                      originalContent={diff.original_content}
                      modifiedContent={diff.modified_content}
                      diffText={diff.diff_text}
                      status={diff.status || 'pending'}
                      onApply={async () => {
                        try {
                          await invoke('apply_diff', { diffId: diff.id });
                          if ((msg as any).codeDiffs) {
                            const updated = (msg as any).codeDiffs.map((d: any) =>
                              d.id === diff.id ? { ...d, status: 'applied', applied_at: new Date().toISOString() } : d
                            );
                            onSetMessages((prev: any) =>
                              prev.map((m: any, i: number) => {
                                if (i !== idx) return m;
                                return { ...m, codeDiffs: updated };
                              })
                            );
                          }
                        } catch (e) {
                          console.error('Failed to apply diff:', e);
                        }
                      }}
                      onReject={async () => {
                        try {
                          await invoke('reject_diff', { diffId: diff.id });
                          if ((msg as any).codeDiffs) {
                            const updated = (msg as any).codeDiffs.map((d: any) =>
                              d.id === diff.id ? { ...d, status: 'rejected' } : d
                            );
                            onSetMessages((prev: any) =>
                              prev.map((m: any, i: number) => {
                                if (i !== idx) return m;
                                return { ...m, codeDiffs: updated };
                              })
                            );
                          }
                        } catch (e) {
                          console.error('Failed to reject diff:', e);
                        }
                      }}
                    />
                  ))}
                </div>
              )}
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
              {!isCurrentlyStreaming && msg.content && extractTextContent(msg.content).trim() && (
                <div className="flex justify-end mt-2 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                  <MessageExportActions
                    content={msg.content}
                    role="assistant"
                    timestamp={msg.created_at ? new Date(msg.created_at).toLocaleString() : undefined}
                  />
                </div>
              )}
            </div>
          )}
        </div>
        );
      })}
    </>
  );
});

export default MessageList;
