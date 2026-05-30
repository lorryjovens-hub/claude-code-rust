import React from 'react';
import { ChevronDown, Check, FileText } from 'lucide-react';
import ToolDiffView, { shouldUseDiffView, hasExpandableContent, getToolStats } from './ToolDiffView';

interface ToolCallDisplayProps {
  toolCalls: any[];
  msgIdx: number;
  messagesLength: number;
  loading: boolean;
  visibleToolCalls: any[];
  isToolCallsExpanded: boolean | undefined;
  onSetMessages: (messages: any[] | ((prev: any[]) => any[])) => void;
}

const ToolCallDisplay: React.FC<ToolCallDisplayProps> = ({
  toolCalls,
  msgIdx,
  messagesLength,
  loading,
  visibleToolCalls,
  isToolCallsExpanded,
  onSetMessages,
}) => {
  const idx = msgIdx;
  const isCurrentMsg = idx === messagesLength - 1;
  const isStale = (!loading && isCurrentMsg) || (idx < messagesLength - 1);
  const isCurrentlyStreaming = loading && idx === messagesLength - 1;

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
  const expanded = isToolCallsExpanded ?? (isCurrentlyStreaming || !allDone);
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
          <ChevronDown size={14} className={`transform transition-transform duration-200 ${(toolCalls[idx]?.isToolCallsExpanded ?? (isCurrentlyStreaming || !allDone)) ? 'rotate-180' : ''}`} />
        </div>
      </div>

      {(toolCalls[idx]?.isToolCallsExpanded ?? (isCurrentlyStreaming || !allDone)) && (
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

export default ToolCallDisplay;
