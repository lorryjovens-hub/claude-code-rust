import React, { useState, useEffect, useMemo } from 'react';
import { Copy, Check, ChevronDown, ChevronRight, FileCode, X, CheckCircle } from 'lucide-react';
import { copyToClipboard } from '../utils/clipboard';

interface DiffLine {
  type: 'added' | 'removed' | 'context';
  content: string;
}

interface NumberedDiffLine extends DiffLine {
  oldNum: number | null;
  newNum: number | null;
}

function computeDiff(oldLines: string[], newLines: string[]): DiffLine[] {
  const m = oldLines.length;
  const n = newLines.length;

  if (m * n > 500000) {
    const result: DiffLine[] = [];
    for (const line of oldLines) result.push({ type: 'removed', content: line });
    for (const line of newLines) result.push({ type: 'added', content: line });
    return result;
  }

  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (oldLines[i - 1] === newLines[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  const result: DiffLine[] = [];
  let i = m, j = n;
  const stack: DiffLine[] = [];

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && oldLines[i - 1] === newLines[j - 1]) {
      stack.push({ type: 'context', content: oldLines[i - 1] });
      i--; j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      stack.push({ type: 'added', content: newLines[j - 1] });
      j--;
    } else {
      stack.push({ type: 'removed', content: oldLines[i - 1] });
      i--;
    }
  }

  stack.reverse();
  return stack;
}

export interface DiffViewerProps {
  filePath: string;
  originalContent?: string;
  modifiedContent?: string;
  diffText?: string;
  status: 'pending' | 'applied' | 'rejected';
  onApply?: () => void;
  onReject?: () => void;
}

function getFileExtension(filePath: string): string {
  const parts = filePath.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

const DiffViewer: React.FC<DiffViewerProps> = ({
  filePath,
  originalContent,
  modifiedContent,
  diffText,
  status,
  onApply,
  onReject,
}) => {
  const [isDark, setIsDark] = useState(false);
  const [copied, setCopied] = useState(false);
  const [collapsed, setCollapsed] = useState(false);
  const [syncScroll, setSyncScroll] = useState(true);

  useEffect(() => {
    const check = () => setIsDark(document.documentElement.classList.contains('dark'));
    check();
    const observer = new MutationObserver(check);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => observer.disconnect();
  }, []);

  const fileName = filePath.split(/[/\\]/).pop() || filePath;
  const ext = getFileExtension(filePath);

  const numberedLines = useMemo<NumberedDiffLine[]>(() => {
    const oldLines = (originalContent || '').split('\n');
    const newLines = (modifiedContent || '').split('\n');
    const diffLines = computeDiff(oldLines, newLines);

    let oldLineNum = 1;
    let newLineNum = 1;

    return diffLines.map(line => {
      const numbered: NumberedDiffLine = {
        ...line,
        oldNum: null,
        newNum: null,
      };
      if (line.type === 'context') {
        numbered.oldNum = oldLineNum++;
        numbered.newNum = newLineNum++;
      } else if (line.type === 'removed') {
        numbered.oldNum = oldLineNum++;
      } else {
        numbered.newNum = newLineNum++;
      }
      return numbered;
    });
  }, [originalContent, modifiedContent]);

  const stats = useMemo(() => {
    let added = 0;
    let removed = 0;
    for (const line of numberedLines) {
      if (line.type === 'added') added++;
      else if (line.type === 'removed') removed++;
    }
    return { added, removed };
  }, [numberedLines]);

  const handleCopy = () => {
    const copyText = numberedLines.map(l => {
      const prefix = l.type === 'added' ? '+' : l.type === 'removed' ? '-' : ' ';
      return `${prefix} ${l.content}`;
    }).join('\n');
    copyToClipboard(copyText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleLeftScroll = (e: React.UIEvent<HTMLDivElement>) => {
    if (!syncScroll) return;
    const rightPane = document.getElementById(`diff-right-${filePath}`);
    if (rightPane) {
      rightPane.scrollTop = (e.target as HTMLDivElement).scrollTop;
    }
  };

  const handleRightScroll = (e: React.UIEvent<HTMLDivElement>) => {
    if (!syncScroll) return;
    const leftPane = document.getElementById(`diff-left-${filePath}`);
    if (leftPane) {
      leftPane.scrollTop = (e.target as HTMLDivElement).scrollTop;
    }
  };

  const statusBadge = () => {
    if (status === 'applied') {
      return (
        <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${isDark ? 'bg-green-900/40 text-green-400 border border-green-700/50' : 'bg-green-100 text-green-700 border border-green-300'}`}>
          Applied
        </span>
      );
    }
    if (status === 'rejected') {
      return (
        <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${isDark ? 'bg-red-900/40 text-red-400 border border-red-700/50' : 'bg-red-100 text-red-700 border border-red-300'}`}>
          Rejected
        </span>
      );
    }
    return (
      <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${isDark ? 'bg-yellow-900/40 text-yellow-400 border border-yellow-700/50' : 'bg-yellow-100 text-yellow-700 border border-yellow-300'}`}>
        Pending
      </span>
    );
  };

  return (
    <div className={`rounded-md overflow-hidden border text-[12px] font-mono ${isDark ? 'border-[#383836] bg-[#1e1e1e]' : 'border-[#E5E5E5] bg-[#FCFCFA]'}`}>
      {/* Header */}
      <div
        className={`flex items-center justify-between px-3 py-1.5 cursor-pointer ${isDark ? 'bg-[#2d2d2d] border-b border-[#383836]' : 'bg-[#f5f5f0] border-b border-[#E5E5E5]'}`}
        onClick={() => setCollapsed(!collapsed)}
      >
        <div className="flex items-center gap-2 min-w-0">
          <button className="p-0.5">
            {collapsed ? <ChevronRight size={14} className={isDark ? 'text-[#999]' : 'text-[#666]'} /> : <ChevronDown size={14} className={isDark ? 'text-[#999]' : 'text-[#666]'} />}
          </button>
          <FileCode size={14} className={isDark ? 'text-[#e0a370]' : 'text-[#b35c2a]'} />
          <span className={`truncate ${isDark ? 'text-[#e0a370]' : 'text-[#b35c2a]'}`}>{fileName}</span>
          {ext && <span className={isDark ? 'text-[#666]' : 'text-[#999]'}>{ext}</span>}
          <span className={`text-[11px] font-mono flex items-center gap-1 ml-1 ${isDark ? 'text-[#555]' : 'text-[#aaa]'}`}>
            <span className="text-green-500 dark:text-green-400">+{stats.added}</span>
            <span className="text-red-500 dark:text-red-400">-{stats.removed}</span>
          </span>
          {statusBadge()}
        </div>
        <div className="flex items-center gap-1 flex-shrink-0 ml-4">
          {status === 'pending' && (
            <>
              <button
                onClick={(e) => { e.stopPropagation(); onApply?.(); }}
                className={`flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-sans font-medium transition-colors ${isDark ? 'bg-green-800/40 hover:bg-green-700/60 text-green-400' : 'bg-green-100 hover:bg-green-200 text-green-700'}`}
              >
                <CheckCircle size={12} />
                Apply
              </button>
              <button
                onClick={(e) => { e.stopPropagation(); onReject?.(); }}
                className={`flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-sans font-medium transition-colors ${isDark ? 'bg-red-800/40 hover:bg-red-700/60 text-red-400' : 'bg-red-100 hover:bg-red-200 text-red-700'}`}
              >
                <X size={12} />
                Reject
              </button>
            </>
          )}
          {status === 'applied' && (
            <span className={`text-[11px] font-sans ${isDark ? 'text-green-500' : 'text-green-600'}`}>
              <CheckCircle size={12} className="inline mr-1" />
              Applied
            </span>
          )}
          {status === 'rejected' && (
            <span className={`text-[11px] font-sans ${isDark ? 'text-red-400' : 'text-red-500'}`}>
              <X size={12} className="inline mr-1" />
              Rejected
            </span>
          )}
          <button
            onClick={(e) => { e.stopPropagation(); handleCopy(); }}
            className={`p-1 rounded transition-colors ${isDark ? 'hover:bg-[#404040] text-[#999]' : 'hover:bg-[#e8e8e4] text-[#666]'}`}
            title="Copy diff"
          >
            {copied ? <Check size={12} /> : <Copy size={12} />}
          </button>
        </div>
      </div>

      {/* Diff body - dual pane */}
      {!collapsed && (
        <div className="flex flex-col">
          {/* Column headers */}
          <div className={`flex text-[10px] font-sans font-medium ${isDark ? 'bg-[#252525] text-[#888] border-b border-[#333]' : 'bg-[#f0f0eb] text-[#777] border-b border-[#e0e0dd]'}`}>
            <div className="flex-1 px-3 py-1 border-r border-inherit">Original</div>
            <div className="flex-1 px-3 py-1">Modified</div>
          </div>

          {/* Dual pane scrollable area */}
          <div className="flex max-h-[400px]">
            {/* Left pane - Original */}
            <div
              id={`diff-left-${filePath}`}
              className="flex-1 overflow-y-auto overflow-x-auto border-r"
              style={{ borderColor: isDark ? '#333' : '#e0e0dd' }}
              onScroll={handleLeftScroll}
            >
              <table className="w-full border-collapse">
                <tbody>
                  {numberedLines.map((line, i) => (
                    <tr key={i} className={
                      line.type === 'removed'
                        ? (isDark ? 'bg-[#3a1a1a]' : 'bg-[#ffebe9]')
                        : ''
                    }>
                      <td className={`select-none text-right px-2 w-[1%] whitespace-nowrap ${isDark ? 'text-[#555] border-r border-[#333]' : 'text-[#bbb] border-r border-[#eee]'}`}>
                        {line.oldNum ?? ''}
                      </td>
                      <td className={`select-none w-[1%] px-1 text-center ${line.type === 'removed' ? (isDark ? 'text-[#f47067]' : 'text-[#cf222e]') : (isDark ? 'text-[#555]' : 'text-[#bbb]')}`}>
                        {line.type === 'removed' ? '-' : line.type === 'context' ? ' ' : '\u00A0'}
                      </td>
                      <td className={`px-2 whitespace-pre-wrap break-all ${line.type === 'removed' ? (isDark ? 'text-[#d8afaf]' : 'text-[#3a1a1a]') : (isDark ? 'text-[#ccc]' : 'text-[#333]')}`}>
                        {line.type !== 'added' ? (line.content || '\u00A0') : '\u00A0'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Right pane - Modified */}
            <div
              id={`diff-right-${filePath}`}
              className="flex-1 overflow-y-auto overflow-x-auto"
              onScroll={handleRightScroll}
            >
              <table className="w-full border-collapse">
                <tbody>
                  {numberedLines.map((line, i) => (
                    <tr key={i} className={
                      line.type === 'added'
                        ? (isDark ? 'bg-[#1a3a2a]' : 'bg-[#e6ffec]')
                        : ''
                    }>
                      <td className={`select-none text-right px-2 w-[1%] whitespace-nowrap ${isDark ? 'text-[#555] border-r border-[#333]' : 'text-[#bbb] border-r border-[#eee]'}`}>
                        {line.newNum ?? ''}
                      </td>
                      <td className={`select-none w-[1%] px-1 text-center ${line.type === 'added' ? (isDark ? 'text-[#7ee787]' : 'text-[#1a7f37]') : (isDark ? 'text-[#555]' : 'text-[#bbb]')}`}>
                        {line.type === 'added' ? '+' : line.type === 'context' ? ' ' : '\u00A0'}
                      </td>
                      <td className={`px-2 whitespace-pre-wrap break-all ${line.type === 'added' ? (isDark ? 'text-[#afd8af]' : 'text-[#1a3a1a]') : (isDark ? 'text-[#ccc]' : 'text-[#333]')}`}>
                        {line.type !== 'removed' ? (line.content || '\u00A0') : '\u00A0'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default DiffViewer;