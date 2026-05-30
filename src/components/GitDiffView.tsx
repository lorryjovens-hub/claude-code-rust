import React, { useState, useMemo } from 'react';
import { GitBranch, Plus, Minus, FileDiff, ChevronDown, ChevronRight, Copy, Check } from 'lucide-react';

interface GitDiffViewProps {
  diffContent: string;
  workspacePath?: string;
}

interface DiffHunk {
  header: string;
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  lines: DiffHunkLine[];
}

interface DiffHunkLine {
  type: 'add' | 'del' | 'context';
  oldNum: number | null;
  newNum: number | null;
  content: string;
}

interface DiffFile {
  oldPath: string;
  newPath: string;
  hunks: DiffHunk[];
  header: string;
  isNew: boolean;
  isDeleted: boolean;
}

function parseDiffContent(raw: string): DiffFile[] {
  const files: DiffFile[] = [];
  const fileChunks = raw.split(/^(?=diff --git )/m);

  for (const chunk of fileChunks) {
    if (!chunk.trim()) continue;

    const lines = chunk.split('\n');

    let oldPath = '';
    let newPath = '';
    let isNew = false;
    let isDeleted = false;

    for (const line of lines) {
      if (line.startsWith('--- ')) {
        const p = line.slice(4).trim();
        if (p === '/dev/null') {
          isNew = true;
        } else {
          oldPath = p.replace(/^a\//, '');
        }
      }
      if (line.startsWith('+++ ')) {
        const p = line.slice(4).trim();
        if (p === '/dev/null') {
          isDeleted = true;
        } else {
          newPath = p.replace(/^b\//, '');
        }
      }
    }

    const hunks: DiffHunk[] = [];
    let currentHunk: DiffHunk | null = null;
    let oldLineCounter = 0;
    let newLineCounter = 0;

    for (const line of lines) {
      const hunkMatch = line.match(/^@@ -(\d+),?(\d*) \+(\d+),?(\d*) @@(.*)$/);
      if (hunkMatch) {
        if (currentHunk) {
          hunks.push(currentHunk);
        }
        const oldStart = parseInt(hunkMatch[1], 10);
        const oldCount = hunkMatch[2] ? parseInt(hunkMatch[2], 10) : 1;
        const newStart = parseInt(hunkMatch[3], 10);
        const newCount = hunkMatch[4] ? parseInt(hunkMatch[4], 10) : 1;

        currentHunk = {
          header: line,
          oldStart,
          oldCount,
          newStart,
          newCount,
          lines: [],
        };
        oldLineCounter = oldStart;
        newLineCounter = newStart;
        continue;
      }

      if (currentHunk) {
        if (line.startsWith('+') && !line.startsWith('+++')) {
          currentHunk.lines.push({
            type: 'add',
            oldNum: null,
            newNum: newLineCounter++,
            content: line.slice(1),
          });
        } else if (line.startsWith('-') && !line.startsWith('---')) {
          currentHunk.lines.push({
            type: 'del',
            oldNum: oldLineCounter++,
            newNum: null,
            content: line.slice(1),
          });
        } else if (
          line.startsWith(' ') ||
          line === '' ||
          (!line.startsWith('+') && !line.startsWith('-') && !line.startsWith('diff ') && !line.startsWith('index ') && !line.startsWith('@@'))
        ) {
          const content = line.startsWith(' ') ? line.slice(1) : line;
          currentHunk.lines.push({
            type: 'context',
            oldNum: oldLineCounter++,
            newNum: newLineCounter++,
            content,
          });
        }
      }
    }

    if (currentHunk && currentHunk.lines.length > 0) {
      hunks.push(currentHunk);
    }

    if (oldPath || newPath) {
      files.push({
        oldPath: oldPath || newPath,
        newPath: newPath || oldPath,
        hunks,
        header: chunk.split('\n')[0],
        isNew,
        isDeleted,
      });
    }
  }

  return files;
}

function getFileName(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

function getStats(file: DiffFile): { added: number; deleted: number } {
  let added = 0;
  let deleted = 0;
  for (const hunk of file.hunks) {
    for (const line of hunk.lines) {
      if (line.type === 'add') added++;
      if (line.type === 'del') deleted++;
    }
  }
  return { added, deleted };
}

const GitDiffView: React.FC<GitDiffViewProps> = ({ diffContent, workspacePath }) => {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [collapsedFiles, setCollapsedFiles] = useState<Set<string>>(new Set());

  const files = useMemo(() => parseDiffContent(diffContent), [diffContent]);

  const totalStats = useMemo(() => {
    let added = 0;
    let deleted = 0;
    for (const f of files) {
      const s = getStats(f);
      added += s.added;
      deleted += s.deleted;
    }
    return { added, deleted };
  }, [files]);

  const activeFile = useMemo(() => {
    if (!selectedFile) return files[0] || null;
    return files.find(f => f.newPath === selectedFile || f.oldPath === selectedFile) || null;
  }, [files, selectedFile]);

  const toggleFileCollapse = (path: string) => {
    setCollapsedFiles(prev => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  };

  const handleCopy = () => {
    const text = files.map(f => {
      return `diff --git a/${f.oldPath} b/${f.newPath}\n--- a/${f.oldPath}\n+++ b/${f.newPath}\n` +
        f.hunks.map(h => h.header + '\n' + h.lines.map(l => {
          if (l.type === 'add') return `+${l.content}`;
          if (l.type === 'del') return `-${l.content}`;
          return ` ${l.content}`;
        }).join('\n')).join('\n');
    }).join('\n');
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex h-full bg-[#1a1a2e] border border-[#2d2d44] rounded-lg overflow-hidden">
      {/* File list sidebar */}
      <div className="w-64 flex-shrink-0 border-r border-[#2d2d44] overflow-y-auto">
        <div className="flex items-center justify-between px-3 py-2 border-b border-[#2d2d44]">
          <div className="flex items-center gap-2 text-[#e0e0e8] text-xs font-semibold">
            <GitBranch size={14} className="text-[#7ee787]" />
            <span>Files ({files.length})</span>
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={handleCopy}
              className="p-1 rounded hover:bg-[#2d2d44] transition-colors"
              title="Copy diff"
            >
              {copied ? <Check size={12} className="text-[#7ee787]" /> : <Copy size={12} className="text-[#888]" />}
            </button>
          </div>
        </div>
        {files.map((file, i) => {
          const stats = getStats(file);
          const fileKey = file.newPath || file.oldPath;
          const fileName = getFileName(fileKey);
          const isSelected = selectedFile === fileKey || (!selectedFile && i === 0);

          return (
            <button
              key={fileKey}
              onClick={() => setSelectedFile(fileKey)}
              className={`w-full text-left px-3 py-2 text-xs border-b border-[#2d2d44]/50 transition-colors ${
                isSelected ? 'bg-[#2d2d44] text-[#e0e0e8]' : 'text-[#a0a0b8] hover:bg-[#22223a]'
              }`}
            >
              <div className="flex items-center gap-1.5 truncate">
                <FileDiff size={12} className="text-[#7ee787] flex-shrink-0" />
                <span className="truncate">{fileName}</span>
                {file.isNew && (
                  <span className="text-[9px] px-1 rounded bg-green-900/40 text-green-400 flex-shrink-0">new</span>
                )}
                {file.isDeleted && (
                  <span className="text-[9px] px-1 rounded bg-red-900/40 text-red-400 flex-shrink-0">del</span>
                )}
              </div>
              <div className="flex items-center gap-2 mt-0.5 ml-5">
                <span className="flex items-center gap-0.5 text-green-400">
                  <Plus size={10} />{stats.added}
                </span>
                <span className="flex items-center gap-0.5 text-red-400">
                  <Minus size={10} />{stats.deleted}
                </span>
              </div>
            </button>
          );
        })}
        {files.length === 0 && (
          <div className="px-3 py-4 text-xs text-[#666] text-center">No changes detected</div>
        )}
      </div>

      {/* Diff content */}
      <div className="flex-1 overflow-auto">
        {activeFile ? (
          <div className="flex flex-col h-full">
            {/* File header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-[#2d2d44] bg-[#1e1e35]">
              <div className="flex items-center gap-2 text-xs">
                <FileDiff size={14} className="text-[#7ee787]" />
                <span className="text-[#e0e0e8] font-medium">{getFileName(activeFile.newPath)}</span>
                <span className="text-[#666]">
                  {activeFile.oldPath !== activeFile.newPath
                    ? `${activeFile.oldPath} → ${activeFile.newPath}`
                    : activeFile.newPath}
                </span>
              </div>
              <div className="flex items-center gap-3 text-xs">
                <span className="flex items-center gap-1 text-green-400">
                  <Plus size={12} />{getStats(activeFile).added}
                </span>
                <span className="flex items-center gap-1 text-red-400">
                  <Minus size={12} />{getStats(activeFile).deleted}
                </span>
              </div>
            </div>

            {/* Hunks */}
            <div className="flex-1 overflow-y-auto">
              {activeFile.hunks.map((hunk, hunkIdx) => (
                <div key={hunkIdx} className="border-b border-[#2d2d44]/50">
                  {/* Hunk header */}
                  <button
                    onClick={() => {
                      const key = `${activeFile.newPath}-${hunkIdx}`;
                      toggleFileCollapse(key);
                    }}
                    className="w-full flex items-center gap-2 px-3 py-1.5 bg-[#252548] text-xs text-[#8888cc] hover:bg-[#2a2a55] transition-colors"
                  >
                    {collapsedFiles.has(`${activeFile.newPath}-${hunkIdx}`) ? (
                      <ChevronRight size={12} />
                    ) : (
                      <ChevronDown size={12} />
                    )}
                    <span className="font-mono">
                      @@ -{hunk.oldStart},{hunk.oldCount} +{hunk.newStart},{hunk.newCount} @@
                    </span>
                  </button>

                  {!collapsedFiles.has(`${activeFile.newPath}-${hunkIdx}`) && (
                    <div className="font-mono text-xs leading-5">
                      {hunk.lines.map((line, lineIdx) => (
                        <div
                          key={lineIdx}
                          className={`flex ${
                            line.type === 'add'
                              ? 'bg-green-900/20'
                              : line.type === 'del'
                              ? 'bg-red-900/20'
                              : ''
                          }`}
                        >
                          {/* Old line number */}
                          <span className="w-12 flex-shrink-0 text-right px-2 select-none text-[#555] border-r border-[#2d2d44]">
                            {line.oldNum ?? ''}
                          </span>
                          {/* New line number */}
                          <span className="w-12 flex-shrink-0 text-right px-2 select-none text-[#555] border-r border-[#2d2d44]">
                            {line.newNum ?? ''}
                          </span>
                          {/* Sign column */}
                          <span className={`w-6 flex-shrink-0 text-center select-none font-bold ${
                            line.type === 'add'
                              ? 'text-green-400'
                              : line.type === 'del'
                              ? 'text-red-400'
                              : 'text-[#555]'
                          }`}>
                            {line.type === 'add' ? '+' : line.type === 'del' ? '-' : ' '}
                          </span>
                          {/* Content */}
                          <span className={`px-2 whitespace-pre-wrap break-all flex-1 ${
                            line.type === 'add'
                              ? 'text-green-400'
                              : line.type === 'del'
                              ? 'text-red-400'
                              : 'text-[#c0c0d0]'
                          }`}>
                            {line.content || '\u00A0'}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="flex items-center justify-center h-full text-[#666] text-sm">
            <div className="flex flex-col items-center gap-2">
              <GitBranch size={32} className="text-[#444]" />
              <span>Select a file to view diff</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default GitDiffView;
export type { GitDiffViewProps };