import React, { useState, useEffect } from 'react';
import { Copy, Check } from 'lucide-react';
import { copyToClipboard } from '../utils/clipboard';

// Simple line-level diff algorithm (Myers-like LCS)
function computeDiff(oldLines: string[], newLines: string[]): DiffLine[] {
  // LCS table
  const m = oldLines.length;
  const n = newLines.length;

  // For very large inputs, fall back to a simpler approach
  if (m * n > 500000) {
    return simpleDiff(oldLines, newLines);
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

  // Backtrack to produce diff
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

function simpleDiff(oldLines: string[], newLines: string[]): DiffLine[] {
  const result: DiffLine[] = [];
  for (const line of oldLines) result.push({ type: 'removed', content: line });
  for (const line of newLines) result.push({ type: 'added', content: line });
  return result;
}

interface DiffLine {
  type: 'added' | 'removed' | 'context';
  content: string;
}

interface ToolDiffViewProps {
  toolName: string;
  input: any;
  result?: string | any;
}

function getFileExtension(filePath: string): string {
  const parts = filePath.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

const ToolDiffView: React.FC<ToolDiffViewProps> = ({ toolName, input, result }) => {
  const [isDark, setIsDark] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const check = () => setIsDark(document.documentElement.classList.contains('dark'));
    check();
    const observer = new MutationObserver(check);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => observer.disconnect();
  }, []);

  const handleCopy = (text: string) => {
    copyToClipboard(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Determine what to render based on tool type
  if (toolName === 'Edit' || toolName === 'MultiEdit') {
    return renderEditDiff(input, isDark, copied, handleCopy);
  }

  if (toolName === 'Write') {
    return renderWriteView(input, isDark, copied, handleCopy);
  }

  if (toolName === 'Bash') {
    return renderBashView(input, result, isDark);
  }

  if (toolName === 'Read') {
    return renderReadView(input, result, isDark);
  }

  return null;
};

// ── Edit tool: old_string → new_string diff ──
function renderEditDiff(
  input: any,
  isDark: boolean,
  copied: boolean,
  handleCopy: (text: string) => void,
) {
  if (!input?.old_string && !input?.new_string) return null;

  const oldStr = input.old_string || '';
  const newStr = input.new_string || '';
  const filePath = input.file_path || '';
  const ext = getFileExtension(filePath);
  const fileName = filePath.split(/[/\\]/).pop() || filePath;

  const oldLines = oldStr.split('\n');
  const newLines = newStr.split('\n');
  const diffLines = computeDiff(oldLines, newLines);

  // Compute line numbers
  let oldLineNum = 1;
  let newLineNum = 1;
  const numberedLines = diffLines.map(line => {
    const numbered = {
      ...line,
      oldNum: null as number | null,
      newNum: null as number | null,
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

  const copyText = diffLines.map(l => {
    const prefix = l.type === 'added' ? '+' : l.type === 'removed' ? '-' : ' ';
    return `${prefix} ${l.content}`;
  }).join('\n');

  return (
    <div className={`rounded-md overflow-hidden border text-[12px] font-mono ${
      isDark ? 'border-[#383836] bg-[#1e1e1e]' : 'border-[#E5E5E5] bg-[#FCFCFA]'
    }`}>
      {/* Header */}
      <div className={`flex items-center justify-between px-3 py-1.5 ${
        isDark ? 'bg-[#2d2d2d] border-b border-[#383836]' : 'bg-[#f5f5f0] border-b border-[#E5E5E5]'
      }`}>
        <div className="flex items-center gap-2">
          <span className={isDark ? 'text-[#e0a370]' : 'text-[#b35c2a]'}>{fileName}</span>
          {ext && <span className={isDark ? 'text-[#666]' : 'text-[#999]'}>{ext}</span>}
        </div>
        <button
          onClick={() => handleCopy(copyText)}
          className={`p-1 rounded transition-colors ${
            isDark ? 'hover:bg-[#404040] text-[#999]' : 'hover:bg-[#e8e8e4] text-[#666]'
          }`}
        >
          {copied ? <Check size={12} /> : <Copy size={12} />}
        </button>
      </div>
      {/* Diff body */}
      <div className="overflow-x-auto max-h-[400px] overflow-y-auto">
        <table className="w-full border-collapse">
          <tbody>
            {numberedLines.map((line, i) => (
              <tr key={i} className={
                line.type === 'added'
                  ? (isDark ? 'bg-[#1a3a2a]' : 'bg-[#e6ffec]')
                  : line.type === 'removed'
                  ? (isDark ? 'bg-[#3a1a1a]' : 'bg-[#ffebe9]')
                  : ''
              }>
                {/* Old line number */}
                <td className={`select-none text-right px-2 w-[1%] whitespace-nowrap ${
                  isDark ? 'text-[#555] border-r border-[#333]' : 'text-[#bbb] border-r border-[#eee]'
                }`}>
                  {line.oldNum ?? ''}
                </td>
                {/* New line number */}
                <td className={`select-none text-right px-2 w-[1%] whitespace-nowrap ${
                  isDark ? 'text-[#555] border-r border-[#333]' : 'text-[#bbb] border-r border-[#eee]'
                }`}>
                  {line.newNum ?? ''}
                </td>
                {/* Marker */}
                <td className={`select-none w-[1%] px-1 text-center ${
                  line.type === 'added'
                    ? (isDark ? 'text-[#7ee787]' : 'text-[#1a7f37]')
                    : line.type === 'removed'
                    ? (isDark ? 'text-[#f47067]' : 'text-[#cf222e]')
                    : (isDark ? 'text-[#555]' : 'text-[#bbb]')
                }`}>
                  {line.type === 'added' ? '+' : line.type === 'removed' ? '-' : ' '}
                </td>
                {/* Content */}
                <td className={`px-2 whitespace-pre-wrap break-all ${
                  line.type === 'added'
                    ? (isDark ? 'text-[#afd8af]' : 'text-[#1a3a1a]')
                    : line.type === 'removed'
                    ? (isDark ? 'text-[#d8afaf]' : 'text-[#3a1a1a]')
                    : (isDark ? 'text-[#ccc]' : 'text-[#333]')
                }`}>
                  {line.content || '\u00A0'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// ── Write tool: show new file content ──
function renderWriteView(
  input: any,
  isDark: boolean,
  copied: boolean,
  handleCopy: (text: string) => void,
) {
  const content = input?.content;
  const filePath = input?.file_path || '';
  if (!content) return null;

  const fileName = filePath.split(/[/\\]/).pop() || filePath;
  const lines = content.split('\n');

  return (
    <div className={`rounded-md overflow-hidden border text-[12px] font-mono ${
      isDark ? 'border-[#383836] bg-[#1e1e1e]' : 'border-[#E5E5E5] bg-[#FCFCFA]'
    }`}>
      {/* Header */}
      <div className={`flex items-center justify-between px-3 py-1.5 ${
        isDark ? 'bg-[#2d2d2d] border-b border-[#383836]' : 'bg-[#f5f5f0] border-b border-[#E5E5E5]'
      }`}>
        <div className="flex items-center gap-2">
          <span className={isDark ? 'text-[#7ee787]' : 'text-[#1a7f37]'}>+ New file</span>
          <span className={isDark ? 'text-[#e0a370]' : 'text-[#b35c2a]'}>{fileName}</span>
        </div>
        <button
          onClick={() => handleCopy(content)}
          className={`p-1 rounded transition-colors ${
            isDark ? 'hover:bg-[#404040] text-[#999]' : 'hover:bg-[#e8e8e4] text-[#666]'
          }`}
        >
          {copied ? <Check size={12} /> : <Copy size={12} />}
        </button>
      </div>
      {/* File body - all lines shown as "added" */}
      <div className="overflow-x-auto max-h-[400px] overflow-y-auto">
        <table className="w-full border-collapse">
          <tbody>
            {lines.map((line: string, i: number) => (
              <tr key={i} className={isDark ? 'bg-[#1a3a2a]' : 'bg-[#e6ffec]'}>
                <td className={`select-none text-right px-2 w-[1%] whitespace-nowrap ${
                  isDark ? 'text-[#555] border-r border-[#333]' : 'text-[#bbb] border-r border-[#eee]'
                }`}>
                  {i + 1}
                </td>
                <td className={`select-none w-[1%] px-1 text-center ${
                  isDark ? 'text-[#7ee787]' : 'text-[#1a7f37]'
                }`}>+</td>
                <td className={`px-2 whitespace-pre-wrap break-all ${
                  isDark ? 'text-[#afd8af]' : 'text-[#1a3a1a]'
                }`}>
                  {line || '\u00A0'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// ── Bash tool: show command + output ──
function renderBashView(input: any, result: string | any, isDark: boolean) {
  const command = input?.command || '';
  const output = typeof result === 'string' ? result : (result != null ? JSON.stringify(result) : '');

  if (!command && !output) return null;

  return (
    <div className={`rounded-md overflow-hidden border text-[12px] font-mono ${
      isDark ? 'border-[#383836] bg-[#1e1e1e]' : 'border-[#E5E5E5] bg-[#FCFCFA]'
    }`}>
      {/* Command */}
      {command && (
        <div className={`px-3 py-2 ${
          isDark ? 'bg-[#2d2d2d] border-b border-[#383836]' : 'bg-[#f5f5f0] border-b border-[#E5E5E5]'
        }`}>
          <span className={isDark ? 'text-[#7ee787]' : 'text-[#1a7f37]'}>$</span>
          <span className={`ml-2 ${isDark ? 'text-[#ccc]' : 'text-[#333]'}`}>{command}</span>
        </div>
      )}
      {/* Output */}
      {output && (
        <div className={`px-3 py-2 max-h-[400px] overflow-y-auto overflow-x-auto whitespace-pre-wrap break-all ${
          isDark ? 'text-[#aaa]' : 'text-[#555]'
        }`}>
          {output.length > 3000 ? output.slice(0, 3000) + '\n...(truncated)' : output}
        </div>
      )}
    </div>
  );
}

// ── Read tool: show file content with line numbers ──
function renderReadView(input: any, result: string | any, isDark: boolean) {
  const filePath = input?.file_path || '';
  const output = typeof result === 'string' ? result : '';
  if (!output) return null;

  const fileName = filePath.split(/[/\\]/).pop() || filePath;
  const lines = output.split('\n');
  // Limit display
  const maxLines = 200;
  const truncated = lines.length > maxLines;
  const displayLines = truncated ? lines.slice(0, maxLines) : lines;

  return (
    <div className={`rounded-md overflow-hidden border text-[12px] font-mono ${
      isDark ? 'border-[#383836] bg-[#1e1e1e]' : 'border-[#E5E5E5] bg-[#FCFCFA]'
    }`}>
      <div className={`px-3 py-1.5 ${
        isDark ? 'bg-[#2d2d2d] border-b border-[#383836]' : 'bg-[#f5f5f0] border-b border-[#E5E5E5]'
      }`}>
        <span className={isDark ? 'text-[#e0a370]' : 'text-[#b35c2a]'}>{fileName}</span>
      </div>
      <div className="overflow-x-auto max-h-[400px] overflow-y-auto">
        <table className="w-full border-collapse">
          <tbody>
            {displayLines.map((line, i) => (
              <tr key={i}>
                <td className={`select-none text-right px-2 w-[1%] whitespace-nowrap ${
                  isDark ? 'text-[#555] border-r border-[#333]' : 'text-[#bbb] border-r border-[#eee]'
                }`}>
                  {i + 1}
                </td>
                <td className={`px-3 whitespace-pre-wrap break-all ${
                  isDark ? 'text-[#ccc]' : 'text-[#333]'
                }`}>
                  {line || '\u00A0'}
                </td>
              </tr>
            ))}
            {truncated && (
              <tr>
                <td colSpan={2} className={`px-3 py-2 text-center ${
                  isDark ? 'text-[#666]' : 'text-[#999]'
                }`}>
                  ...{lines.length - maxLines} more lines truncated
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// Check if a tool call should use the rich diff view
export function shouldUseDiffView(toolName: string, input: any): boolean {
  if (toolName === 'Edit' || toolName === 'MultiEdit') {
    return !!(input?.old_string || input?.new_string);
  }
  if (toolName === 'Write') {
    return !!input?.content;
  }
  if (toolName === 'Bash') {
    return !!input?.command;
  }
  if (toolName === 'Read') {
    return !!input?.file_path;
  }
  return false;
}

// Check if a tool call has expandable content (input-based or result-based)
export function hasExpandableContent(toolName: string, input: any, result: any): boolean {
  if (result != null && result !== '') return true;
  return shouldUseDiffView(toolName, input);
}

// Get +N/-N line change stats for display in the tool call header
export function getToolStats(toolName: string, input: any): { added: number; removed: number } | null {
  if (toolName === 'Edit' || toolName === 'MultiEdit') {
    const oldStr = input?.old_string || '';
    const newStr = input?.new_string || '';
    if (!oldStr && !newStr) return null;
    const oldLines = oldStr ? oldStr.split('\n').length : 0;
    const newLines = newStr ? newStr.split('\n').length : 0;
    return { added: newLines, removed: oldLines };
  }
  if (toolName === 'Write') {
    const content = input?.content || '';
    if (!content) return null;
    const lines = content.split('\n').length;
    return { added: lines, removed: 0 };
  }
  return null;
}

export default ToolDiffView;
