import React, { useState, useEffect, useCallback } from 'react';
import { GitCommit, CheckCircle, Plus, Minus, RefreshCw, ArrowUp, ChevronDown, GitBranch } from 'lucide-react';

interface GitFile {
  path: string;
  status: 'modified' | 'added' | 'deleted' | 'renamed';
  oldPath?: string;
}

interface GitCommitPanelProps {
  workspacePath: string;
  onCommit?: (message: string) => void;
  onRefresh?: () => void;
}

const MOCK_FILE_NAMES = [
  'src/components/Header.tsx',
  'src/utils/helpers.ts',
  'src/styles/theme.css',
  'package.json',
  'tsconfig.json',
  'README.md',
  'src/api/client.ts',
  'src/hooks/useAuth.ts',
  'src/components/Sidebar.tsx',
  'src/types/index.ts',
  'src/stores/appStore.ts',
  '.env.example',
];

const MOCK_STATUSES: GitFile['status'][] = ['modified', 'added', 'deleted', 'modified', 'modified', 'added', 'modified', 'renamed', 'modified', 'added', 'deleted', 'modified'];

function generateMockFiles(): GitFile[] {
  const count = 3 + Math.floor(Math.random() * 6);
  const shuffled = [...MOCK_FILE_NAMES].sort(() => Math.random() - 0.5).slice(0, count);
  const statuses = [...MOCK_STATUSES].sort(() => Math.random() - 0.5);

  return shuffled.map((path, i) => ({
    path,
    status: statuses[i % statuses.length],
  }));
}

function getStatusBadge(status: GitFile['status']): { label: string; className: string } {
  switch (status) {
    case 'modified':
      return { label: 'M', className: 'bg-yellow-900/40 text-yellow-400' };
    case 'added':
      return { label: 'A', className: 'bg-green-900/40 text-green-400' };
    case 'deleted':
      return { label: 'D', className: 'bg-red-900/40 text-red-400' };
    case 'renamed':
      return { label: 'R', className: 'bg-blue-900/40 text-blue-400' };
  }
}

function getFileName(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

function getFileDir(path: string): string {
  const parts = path.split(/[/\\]/);
  parts.pop();
  return parts.join('/');
}

const GitCommitPanel: React.FC<GitCommitPanelProps> = ({ workspacePath, onCommit, onRefresh }) => {
  const [unstagedFiles, setUnstagedFiles] = useState<GitFile[]>([]);
  const [stagedFiles, setStagedFiles] = useState<GitFile[]>([]);
  const [commitMessage, setCommitMessage] = useState('');
  const [currentBranch, setCurrentBranch] = useState('main');
  const [isLoading, setIsLoading] = useState(false);
  const [commitStatus, setCommitStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [statusMessage, setStatusMessage] = useState('');
  const [selectedUnstaged, setSelectedUnstaged] = useState<Set<string>>(new Set());
  const [showStaged, setShowStaged] = useState(true);
  const [showUnstaged, setShowUnstaged] = useState(true);

  const refreshStatus = useCallback(() => {
    setIsLoading(true);
    setTimeout(() => {
      const mockFiles = generateMockFiles();
      const staged: GitFile[] = [];
      const unstaged: GitFile[] = [];

      const splitIdx = Math.floor(Math.random() * mockFiles.length);
      mockFiles.forEach((file, i) => {
        if (i < splitIdx) {
          staged.push(file);
        } else {
          unstaged.push(file);
        }
      });

      setStagedFiles(staged);
      setUnstagedFiles(unstaged);
      setCurrentBranch(['main', 'develop', 'feature/ui-update', 'bugfix/login-fix'][Math.floor(Math.random() * 4)]);
      setIsLoading(false);
      onRefresh?.();
    }, 500 + Math.random() * 500);
  }, [onRefresh]);

  useEffect(() => {
    refreshStatus();
  }, [refreshStatus]);

  const toggleUnstagedSelection = (filePath: string) => {
    setSelectedUnstaged(prev => {
      const next = new Set(prev);
      if (next.has(filePath)) {
        next.delete(filePath);
      } else {
        next.add(filePath);
      }
      return next;
    });
  };

  const stageSelected = () => {
    if (selectedUnstaged.size === 0) return;
    const toStage: GitFile[] = [];
    const remaining: GitFile[] = [];

    for (const file of unstagedFiles) {
      if (selectedUnstaged.has(file.path)) {
        toStage.push(file);
      } else {
        remaining.push(file);
      }
    }

    setStagedFiles(prev => [...prev, ...toStage]);
    setUnstagedFiles(remaining);
    setSelectedUnstaged(new Set());
  };

  const unstageFile = (filePath: string) => {
    const file = stagedFiles.find(f => f.path === filePath);
    if (!file) return;

    setStagedFiles(prev => prev.filter(f => f.path !== filePath));
    setUnstagedFiles(prev => [...prev, file]);
  };

  const handleCommit = () => {
    if (!commitMessage.trim()) {
      setCommitStatus('error');
      setStatusMessage('Please enter a commit message');
      setTimeout(() => setCommitStatus('idle'), 3000);
      return;
    }

    if (stagedFiles.length === 0) {
      setCommitStatus('error');
      setStatusMessage('No files staged for commit');
      setTimeout(() => setCommitStatus('idle'), 3000);
      return;
    }

    setIsLoading(true);
    setTimeout(() => {
      setCommitStatus('success');
      setStatusMessage(`Committed ${stagedFiles.length} file(s) to ${currentBranch}`);
      setCommitMessage('');
      setStagedFiles([]);
      setIsLoading(false);
      onCommit?.(commitMessage);

      setTimeout(() => {
        setCommitStatus('idle');
        setStatusMessage('');
        refreshStatus();
      }, 3000);
    }, 800);
  };

  const totalChanges = unstagedFiles.length + stagedFiles.length;

  return (
    <div className="flex flex-col h-full bg-[#1a1a2e] border border-[#2d2d44] rounded-lg overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[#2d2d44]">
        <div className="flex items-center gap-2">
          <GitCommit size={16} className="text-[#7ee787]" />
          <span className="text-[#e0e0e8] text-xs font-semibold">Git Commit</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-[#252548] text-xs">
            <GitBranch size={12} className="text-[#7ee787]" />
            <span className="text-[#c0c0d0]">{currentBranch}</span>
          </div>
          <button
            onClick={refreshStatus}
            disabled={isLoading}
            className="p-1 rounded hover:bg-[#2d2d44] transition-colors"
            title="Refresh git status"
          >
            <RefreshCw size={14} className={`text-[#888] ${isLoading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Status message */}
      {statusMessage && (
        <div className={`px-3 py-2 text-xs ${
          commitStatus === 'success'
            ? 'bg-green-900/20 text-green-400 border-b border-green-900/30'
            : 'bg-red-900/20 text-red-400 border-b border-red-900/30'
        }`}>
          {commitStatus === 'success' && <CheckCircle size={12} className="inline mr-1" />}
          {statusMessage}
        </div>
      )}

      {/* File lists */}
      <div className="flex-1 overflow-y-auto">
        {/* Staged files */}
        <div className="border-b border-[#2d2d44]/50">
          <button
            onClick={() => setShowStaged(!showStaged)}
            className="w-full flex items-center justify-between px-3 py-1.5 bg-[#1e1e35] hover:bg-[#22223a] transition-colors"
          >
            <div className="flex items-center gap-2 text-xs">
              <ChevronDown
                size={12}
                className={`text-[#888] transition-transform ${showStaged ? '' : '-rotate-90'}`}
              />
              <span className="text-[#e0e0e8] font-medium">Staged</span>
              <span className="text-[#7ee787] text-[11px]">({stagedFiles.length})</span>
            </div>
          </button>
          {showStaged && (
            <div className="border-l-2 border-green-500/60 ml-0">
              {stagedFiles.length === 0 ? (
                <div className="px-4 py-3 text-xs text-[#555]">No staged files</div>
              ) : (
                stagedFiles.map(file => {
                  const badge = getStatusBadge(file.status);
                  return (
                    <div
                      key={file.path}
                      className="flex items-center justify-between px-3 py-1.5 hover:bg-[#1e1e35] transition-colors group"
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <button
                          onClick={() => unstageFile(file.path)}
                          className="p-0.5 rounded hover:bg-[#2d2d44] transition-colors"
                          title="Unstage file"
                        >
                          <Minus size={12} className="text-[#888] group-hover:text-red-400" />
                        </button>
                        <span className={`text-[9px] px-1 rounded font-mono flex-shrink-0 ${badge.className}`}>
                          {badge.label}
                        </span>
                        <span className="text-xs text-[#c0c0d0] truncate">{getFileName(file.path)}</span>
                      </div>
                      <span className="text-[10px] text-[#555] flex-shrink-0 ml-2 truncate">
                        {getFileDir(file.path)}
                      </span>
                    </div>
                  );
                })
              )}
            </div>
          )}
        </div>

        {/* Unstaged files */}
        <div>
          <button
            onClick={() => setShowUnstaged(!showUnstaged)}
            className="w-full flex items-center justify-between px-3 py-1.5 bg-[#1e1e35] hover:bg-[#22223a] transition-colors"
          >
            <div className="flex items-center gap-2 text-xs">
              <ChevronDown
                size={12}
                className={`text-[#888] transition-transform ${showUnstaged ? '' : '-rotate-90'}`}
              />
              <span className="text-[#e0e0e8] font-medium">Unstaged</span>
              <span className="text-yellow-400 text-[11px]">({unstagedFiles.length})</span>
            </div>
          </button>
          {showUnstaged && (
            <div>
              {unstagedFiles.length === 0 ? (
                <div className="px-4 py-3 text-xs text-[#555]">No unstaged files</div>
              ) : (
                unstagedFiles.map(file => {
                  const badge = getStatusBadge(file.status);
                  const isSelected = selectedUnstaged.has(file.path);
                  return (
                    <div
                      key={file.path}
                      className="flex items-center justify-between px-3 py-1.5 hover:bg-[#1e1e35] transition-colors group"
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <button
                          onClick={() => toggleUnstagedSelection(file.path)}
                          className={`w-4 h-4 rounded border flex items-center justify-center flex-shrink-0 transition-colors ${
                            isSelected
                              ? 'bg-green-600 border-green-600'
                              : 'border-[#555] hover:border-[#888]'
                          }`}
                        >
                          {isSelected && <CheckCircle size={10} className="text-white" />}
                        </button>
                        <span className={`text-[9px] px-1 rounded font-mono flex-shrink-0 ${badge.className}`}>
                          {badge.label}
                        </span>
                        <span className="text-xs text-[#c0c0d0] truncate">{getFileName(file.path)}</span>
                      </div>
                      <span className="text-[10px] text-[#555] flex-shrink-0 ml-2 truncate">
                        {getFileDir(file.path)}
                      </span>
                    </div>
                  );
                })
              )}
            </div>
          )}
        </div>
      </div>

      {/* Stage selected button */}
      {selectedUnstaged.size > 0 && (
        <div className="px-3 py-1.5 border-t border-[#2d2d44] bg-[#1e1e35]">
          <button
            onClick={stageSelected}
            className="w-full flex items-center justify-center gap-1.5 py-1 px-3 rounded text-xs bg-green-900/30 text-green-400 hover:bg-green-900/50 transition-colors"
          >
            <Plus size={12} />
            Stage {selectedUnstaged.size} selected file(s)
          </button>
        </div>
      )}

      {/* Commit message area */}
      <div className="border-t border-[#2d2d44] p-3 bg-[#1a1a2e]">
        <textarea
          value={commitMessage}
          onChange={e => {
            setCommitMessage(e.target.value);
            if (commitStatus !== 'idle') setCommitStatus('idle');
          }}
          placeholder="Enter commit message..."
          rows={3}
          className="w-full bg-[#12121f] border border-[#2d2d44] rounded px-3 py-2 text-xs text-[#e0e0e8] placeholder-[#555] resize-none focus:outline-none focus:border-[#7ee787]/50 transition-colors"
          onKeyDown={e => {
            if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
              e.preventDefault();
              handleCommit();
            }
          }}
        />

        {/* Commit action bar */}
        <div className="flex items-center justify-between mt-2">
          <span className="text-[10px] text-[#555]">
            {totalChanges > 0
              ? `${totalChanges} change(s): ${stagedFiles.length} staged, ${unstagedFiles.length} unstaged`
              : 'No changes detected'}
          </span>
          <button
            onClick={handleCommit}
            disabled={isLoading || stagedFiles.length === 0 || !commitMessage.trim()}
            className="flex items-center gap-1.5 px-4 py-1.5 rounded text-xs font-medium transition-all
              bg-green-600 text-white
              hover:bg-green-500
              disabled:bg-[#2d2d44] disabled:text-[#555] disabled:cursor-not-allowed"
          >
            {isLoading ? (
              <>
                <RefreshCw size={12} className="animate-spin" />
                Committing...
              </>
            ) : (
              <>
                <ArrowUp size={12} />
                Commit
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default GitCommitPanel;
export type { GitCommitPanelProps, GitFile };