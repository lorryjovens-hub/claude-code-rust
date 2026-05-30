import React, { useState, useEffect, useRef } from 'react';
import { ChevronDown, FileText, Check, Globe, Loader2 } from 'lucide-react';
import ClaudeLogo from '../ClaudeLogo';
import MarkdownRenderer from '../MarkdownRenderer';
import ToolDiffView, { shouldUseDiffView, hasExpandableContent, getToolStats } from '../ToolDiffView';
import DiffViewer from '../DiffViewer';
import SearchProcess from '../SearchProcess';
import DocumentCreationProcess, { DocumentDraftInfo } from '../DocumentCreationProcess';
import CodeExecution from '../CodeExecution';
import DocumentCard, { DocumentInfo } from '../DocumentCard';
import MessageAttachments from '../MessageAttachments';
import MessageExportActions from './MessageExportActions';
import { extractTextContent, normalizeMessageDocuments, normalizeDocumentDrafts, withAuthToken } from '../../utils/messageHelpers';
import { invoke } from '@tauri-apps/api/core';

export interface AssistantMessageProps {
  msg: any;
  idx: number;
  messagesLength: number;
  loading: boolean;
  isExpanded: boolean;
  compactStatus: { state: string; message?: string };
  onSetMessages: (messages: any[] | ((prev: any[]) => any[])) => void;
  onToggleExpand: (idx: number) => void;
  onOpenDocument?: (doc: DocumentInfo) => void;
  onOpenResearch?: (msgId: string) => void;
}

const CompactingStatus: React.FC = () => {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setProgress(prev => {
        if (prev >= 95) return prev;
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

const AssistantMessage: React.FC<AssistantMessageProps> = ({
  msg,
  idx,
  messagesLength,
  loading,
  isExpanded,
  compactStatus,
  onSetMessages,
  onToggleExpand,
  onOpenDocument,
  onOpenResearch,
}) => {
  const messageContentRef = useRef<HTMLDivElement>(null);

  const isCurrentMsg = idx === messagesLength - 1;
  const isStale = (!loading && isCurrentMsg) || (idx < messagesLength - 1);
  const isCurrentlyStreaming = loading && idx === messagesLength - 1;

  // Thinking expansion handler
  const handleThinkingToggle = () => {
    onSetMessages(prev =>
      prev.map((m, i) =>
        i === idx ? { ...m, isThinkingExpanded: !m.isThinkingExpanded } : m
      )
    );
  };

  // Tool calls expansion handler
  const handleToolCallsToggle = () => {
    onSetMessages(prev =>
      prev.map((m, i) =>
        i === idx ? { ...m, isToolCallsExpanded: !m.isToolCallsExpanded } : m
      )
    );
  };

  // Individual tool expansion handler
  const handleToolExpand = (tcIdx: number) => {
    onSetMessages(prev =>
      prev.map((m, i) => {
        if (i !== idx) return m;
        const newTc = [...m.toolCalls];
        newTc[tcIdx] = { ...newTc[tcIdx], isExpanded: newTc[tcIdx].isExpanded === undefined ? true : !newTc[tcIdx].isExpanded };
        return { ...m, toolCalls: newTc };
      })
    );
  };

  // Render thinking section with enhanced animations
  const renderThinking = () => {
    if (!msg.thinking) return null;

    return (
      <div className="mb-4 scan-light-wrapper rounded-lg p-2 -mx-2">
        <div
          className="flex items-center gap-2 cursor-pointer select-none group/think text-claude-textSecondary hover:text-claude-text transition-colors"
          onClick={handleThinkingToggle}
        >
          {msg.isThinking && (
            <span className="loading-ring flex-shrink-0" />
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
          <ChevronDown size={14} className={`transform transition-transform duration-300 ${msg.isThinkingExpanded ? 'rotate-180' : ''}`} />
        </div>

        {msg.isThinkingExpanded && (
          <div className="mt-2 ml-1 pl-4 border-l-2 border-claude-border animate-expand-in">
            <div className="flex flex-col">
              <div className="relative">
                <div
                  className="text-claude-textSecondary text-[14px] leading-normal whitespace-pre-wrap overflow-hidden"
                  style={{ maxHeight: isExpanded ? 'none' : '300px' }}
                  ref={(el) => { if (el) messageContentRef.current = el; }}
                >
                  {msg.thinking}
                </div>
                {!isExpanded && (() => {
                  return messageContentRef.current && messageContentRef.current.scrollHeight > 300;
                })() && (
                  <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-claude-bg to-transparent pointer-events-none" />
                )}
              </div>
              {(() => {
                if (!messageContentRef.current) return null;
                const isOverflow = messageContentRef.current.scrollHeight > 300;
                if (!isOverflow) return null;
                return (
                  <div className="pt-1">
                    <button onClick={() => onToggleExpand(idx)} className="text-[13px] text-claude-text hover:text-claude-textSecondary transition-colors font-medium">
                      {isExpanded ? 'Show less' : 'Show more'}
                    </button>
                  </div>
                );
              })()}
            </div>
            {!msg.isThinking && (
              <div className="flex items-center gap-2 mt-2 text-claude-textSecondary animate-fade-in">
                <Check size={16} />
                <span className="text-[14px]">Done</span>
              </div>
            )}
          </div>
        )}
      </div>
    );
  };

  // Render tool calls section
  const renderToolCalls = () => {
    if (!msg.toolCalls || msg.toolCalls.length === 0) return null;

    const FRONTEND_HIDDEN = new Set(['WebSearch', 'WebFetch']);
    const visibleToolCalls = msg.toolCalls.filter((tc: any) => !FRONTEND_HIDDEN.has(tc.name));
    if (visibleToolCalls.length === 0) return null;

    const allDone = visibleToolCalls.every((tc: any) => {
      const rs = (tc.status === 'running' && isStale) ? 'canceled' : tc.status;
      return rs !== 'running';
    });
    const hasError = visibleToolCalls.some((tc: any) => tc.status === 'error');

    const toolNames = visibleToolCalls.map((tc: any) => {
      const nameMap: Record<string, string> = {
        'Read': 'Read file', 'Write': 'Write file', 'Edit': 'Edit file',
        'Bash': 'Run command', 'ListDir': 'List directory',
        'MultiEdit': 'Edit files', 'Search': 'Search',
      };
      return nameMap[tc.name] || tc.name;
    });
    const uniqueNames = [...new Set(toolNames)];
    const summary = uniqueNames.join(', ');
    const expanded = msg.isToolCallsExpanded ?? (isCurrentlyStreaming || !allDone);

    // Compute pending work text during streaming
    const fullText = extractTextContent(msg.content);
    let consumedLen = 0;
    for (const tc of visibleToolCalls) {
      if (tc.textBefore) consumedLen += tc.textBefore.length;
    }
    const pendingWorkText = isCurrentlyStreaming ? fullText.slice(consumedLen).trim() : '';

    return (
      <div className="mb-4">
        <div className={`rounded-lg overflow-hidden ${!allDone ? 'bg-black/[0.04] dark:bg-white/[0.04]' : ''}`}>
          <div
            className="flex items-center gap-2 cursor-pointer select-none group/tool text-claude-textSecondary hover:text-claude-text transition-colors px-2 py-1.5"
            onClick={handleToolCallsToggle}
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
            <ChevronDown size={14} className={`transform transition-transform duration-200 ${expanded ? 'rotate-180' : ''}`} />
          </div>
        </div>

        {expanded && (
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
                  {tc.textBefore && (
                    <div className="text-[13px] text-claude-textSecondary px-1 py-1.5 leading-relaxed">
                      {tc.textBefore}
                    </div>
                  )}
                  <div className="text-[13px] bg-black/5 dark:bg-black/20 rounded-lg overflow-hidden border border-black/5 dark:border-white/5 mx-1 w-full">
                    <div
                      className={`flex items-center justify-between px-3 py-2 transition-colors ${expandable ? 'cursor-pointer hover:bg-black/5 dark:hover:bg-white/5' : ''}`}
                      onClick={() => {
                        if (!expandable) return;
                        handleToolExpand(tcIdx);
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
            {isCurrentlyStreaming && pendingWorkText && (
              <div className="text-[13px] text-claude-textSecondary px-1 py-1.5 leading-relaxed animate-shimmer-text">
                {pendingWorkText}
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
  };

  // Render code diffs
  const renderCodeDiffs = () => {
    if (!msg.codeDiffs || msg.codeDiffs.length === 0) return null;

    return (
      <div className="mt-2 mb-1 space-y-2">
        {msg.codeDiffs.map((diff: any) => (
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
                const updated = msg.codeDiffs.map((d: any) =>
                  d.id === diff.id ? { ...d, status: 'applied', applied_at: new Date().toISOString() } : d
                );
                onSetMessages((prev: any) =>
                  prev.map((m: any, i: number) => {
                    if (i !== idx) return m;
                    return { ...m, codeDiffs: updated };
                  })
                );
              } catch (e) {
                console.error('Failed to apply diff:', e);
              }
            }}
            onReject={async () => {
              try {
                await invoke('reject_diff', { diffId: diff.id });
                const updated = msg.codeDiffs.map((d: any) =>
                  d.id === diff.id ? { ...d, status: 'rejected' } : d
                );
                onSetMessages((prev: any) =>
                  prev.map((m: any, i: number) => {
                    if (i !== idx) return m;
                    return { ...m, codeDiffs: updated };
                  })
                );
              } catch (e) {
                console.error('Failed to reject diff:', e);
              }
            }}
          />
        ))}
      </div>
    );
  };

  // Render documents
  const renderDocuments = () => {
    const docs = normalizeMessageDocuments(msg);
    if (docs.length === 0) return null;

    return (
      <div className="mt-2 mb-1 space-y-2">
        {docs.map((doc, docIdx) => (
          <DocumentCard
            key={doc.id || `${idx}-${docIdx}`}
            document={doc}
            onOpen={(openedDoc) => onOpenDocument?.(openedDoc)}
          />
        ))}
      </div>
    );
  };

  // Render document drafts
  const renderDocumentDrafts = () => {
    const drafts = normalizeDocumentDrafts(msg);
    if (drafts.length === 0) return null;

    return <DocumentCreationProcess drafts={drafts} />;
  };

  // Render search process
  const renderSearchProcess = () => {
    if (!msg.searchLogs || msg.searchLogs.length === 0) {
      // Check if we should show searching indicator
      if (msg.searchStatus && (!msg.content || msg.content.length === (msg._contentLenBeforeSearch || 0)) && loading && idx === messagesLength - 1) {
        return (
          <div className="flex items-center justify-center gap-2 text-[15px] font-medium mb-4 w-full">
            <Globe size={18} className="text-claude-textSecondary" />
            <span className="animate-shimmer-text">Searching the web</span>
          </div>
        );
      }
      return null;
    }

    return <SearchProcess logs={msg.searchLogs} isThinking={msg.isThinking} isDone={(msg.content || '').length > (msg._contentLenBeforeSearch || 0)} />;
  };

  // Render code execution
  const renderCodeExecution = () => {
    if (!msg.codeExecution) return null;

    return (
      <CodeExecution
        code={msg.codeExecution.code}
        status={msg.codeExecution.status}
        stdout={msg.codeExecution.stdout}
        stderr={msg.codeExecution.stderr}
        images={msg.codeExecution.images}
        error={msg.codeExecution.error}
      />
    );
  };

  // Render code images
  const renderCodeImages = () => {
    if (msg.codeExecution) return null;
    if (!msg.codeImages || msg.codeImages.length === 0) return null;

    return (
      <div className="my-3 space-y-2">
        {msg.codeImages.map((url: string, i: number) => (
          <div key={i} className="rounded-lg overflow-hidden">
            <img src={withAuthToken(url)} alt={`图表 ${i + 1}`} className="max-w-full" loading="lazy" />
          </div>
        ))}
      </div>
    );
  };

  // Render loading indicator
  const renderLoadingIndicator = () => {
    if (!loading || idx !== messagesLength - 1) return null;
    if (msg.content || msg.thinking || msg.searchStatus || normalizeDocumentDrafts(msg).length > 0 || (msg.toolCalls && msg.toolCalls.length > 0)) {
      return (
        <span className="inline-block ml-1 align-middle" style={{ verticalAlign: 'middle' }}>
          <ClaudeLogo autoAnimate style={{ width: '40px', height: '40px', display: 'inline-block' }} />
        </span>
      );
    }

    return (
      <span className="inline-block ml-1 align-middle" style={{ verticalAlign: 'middle' }}>
        <ClaudeLogo breathe style={{ width: '40px', height: '40px', display: 'inline-block' }} />
      </span>
    );
  };

  // Render compacting status
  const renderCompactingStatus = () => {
    if (!loading || idx !== messagesLength - 1 || !msg.content) return null;

    return (
      <div className="flex items-start gap-4 mt-6 ml-1 mb-2">
        <ClaudeLogo breathe={compactStatus.state === 'compacting'} style={{ width: '36px', height: '36px', flexShrink: 0, marginTop: '2px' }} />
        {compactStatus.state === 'compacting' && <CompactingStatus />}
      </div>
    );
  };

  // Render research badge
  const renderResearchBadge = () => {
    if (!msg.research) return null;

    return (
      <button
        onClick={() => onOpenResearch && onOpenResearch(msg.id)}
        className="mb-3 inline-flex items-center gap-2 px-3 py-2 rounded-lg bg-[#DBEAFE] dark:bg-[#1E3A5F] hover:bg-[#BFDBFE] dark:hover:bg-[#2A4A75] transition-colors"
      >
        {msg.research.completed ? (
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#2E7CF6" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.3-4.3" />
          </svg>
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
    );
  };

  return (
    <div className={`px-1 text-claude-text text-[16.5px] leading-normal mt-2 group ${isCurrentlyStreaming ? 'streaming-active' : ''}`}>
      {renderThinking()}
      {renderResearchBadge()}
      {renderToolCalls()}
      {renderSearchProcess()}
      {renderDocumentDrafts()}
      <div className={isCurrentlyStreaming ? 'scan-light-wrapper' : ''}>
        <MarkdownRenderer content={(msg as any)._finalText ?? extractTextContent(msg.content)} citations={msg.citations} isStreaming={isCurrentlyStreaming} />
        {isCurrentlyStreaming && <span className="typing-cursor" />}
      </div>
      {renderCodeDiffs()}
      {renderDocuments()}
      {renderCodeExecution()}
      {renderCodeImages()}
      {renderLoadingIndicator()}
      {renderCompactingStatus()}

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
  );
};

export default React.memo(AssistantMessage);
