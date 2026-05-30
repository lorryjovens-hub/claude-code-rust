import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  FolderOpen, Folder, Clock, X, Plus, Star, History, ExternalLink,
} from 'lucide-react';

interface WorkspaceEntry {
  name: string;
  path: string;
  lastOpened: number;
}

interface WorkspacePanelProps {
  currentWorkspace?: string;
  onWorkspaceChange: (path: string) => void;
}

const STORAGE_KEY = 'recent_workspaces';
const MAX_RECENT = 20;

function getFolderName(fullPath: string): string {
  const normalized = fullPath.replace(/[/\\]+$/, '');
  const segments = normalized.split(/[/\\]/);
  return segments[segments.length - 1] || fullPath;
}

function loadRecentWorkspaces(): WorkspaceEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (e: unknown) =>
        e &&
        typeof e === 'object' &&
        typeof (e as WorkspaceEntry).name === 'string' &&
        typeof (e as WorkspaceEntry).path === 'string' &&
        typeof (e as WorkspaceEntry).lastOpened === 'number'
    );
  } catch {
    return [];
  }
}

function saveRecentWorkspaces(entries: WorkspaceEntry[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries.slice(0, MAX_RECENT)));
  } catch {
    // ignore quota errors
  }
}

function updateRecentList(
  prev: WorkspaceEntry[],
  folderPath: string
): WorkspaceEntry[] {
  const name = getFolderName(folderPath);
  const now = Date.now();
  const filtered = prev.filter((e) => e.path !== folderPath);
  return [{ name, path: folderPath, lastOpened: now }, ...filtered].slice(0, MAX_RECENT);
}

function formatTimestamp(ts: number): string {
  const diff = Date.now() - ts;
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return 'Just now';
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  return new Date(ts).toLocaleDateString();
}

export default function WorkspacePanel({ currentWorkspace, onWorkspaceChange }: WorkspacePanelProps) {
  const [isOpening, setIsOpening] = useState(false);
  const [recentWorkspaces, setRecentWorkspaces] = useState<WorkspaceEntry[]>([]);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    setRecentWorkspaces(loadRecentWorkspaces());
  }, []);

  useEffect(() => {
    if (currentWorkspace) {
      setRecentWorkspaces((prev) => {
        const updated = updateRecentList(prev, currentWorkspace);
        saveRecentWorkspaces(updated);
        return updated;
      });
    }
  }, [currentWorkspace]);

  const handleSelectFolder = useCallback(async () => {
    setIsOpening(true);
    try {
      const isTauri = '__TAURI_INTERNALS__' in window;

      if (isTauri) {
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          const selected = await invoke<string | null>('select_directory');
          if (selected) {
            onWorkspaceChange(selected);
          }
        } catch {
          fileInputRef.current?.click();
        }
      } else {
        fileInputRef.current?.click();
      }
    } finally {
      setIsOpening(false);
    }
  }, [onWorkspaceChange]);

  const handleFileInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files;
      if (!files || files.length === 0) return;

      const firstFile = files[0];
      let fullPath = '';

      if ('webkitRelativePath' in firstFile && firstFile.webkitRelativePath) {
        const relative = firstFile.webkitRelativePath;
        const rootName = relative.split('/')[0] || relative.split('\\')[0];
        fullPath = firstFile.name
          ? relative.replace(/[/\\][^/\\]*$/, '')
          : rootName;
      }

      if (!fullPath && files.length > 0) {
        const path = files[0].name || '';
        fullPath = path;
      }

      if (fullPath) {
        onWorkspaceChange(fullPath);
      }

      e.target.value = '';
    },
    [onWorkspaceChange]
  );

  const handleRecentClick = useCallback(
    (entry: WorkspaceEntry) => {
      onWorkspaceChange(entry.path);
    },
    [onWorkspaceChange]
  );

  const handleClearRecent = useCallback(() => {
    setRecentWorkspaces([]);
    saveRecentWorkspaces([]);
  }, []);

  const handleRemoveRecent = useCallback(
    (e: React.MouseEvent, entryPath: string) => {
      e.stopPropagation();
      setRecentWorkspaces((prev) => {
        const updated = prev.filter((r) => r.path !== entryPath);
        saveRecentWorkspaces(updated);
        return updated;
      });
    },
    []
  );

  return (
    <div className="flex flex-col h-full bg-[#1a1a2e] text-[#e0e0e8] text-[13px]">
      <div className="flex items-center justify-between px-3 py-2 border-b border-[#2d2d44]">
        <span className="text-[11px] font-semibold text-neutral-400 uppercase tracking-wide">
          Workspace
        </span>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Folder size={14} className="text-amber-400" />
            <span className="text-[11px] font-semibold text-neutral-400 uppercase tracking-wide">
              Current
            </span>
          </div>

          <button
            onClick={handleSelectFolder}
            disabled={isOpening}
            className="w-full flex items-center gap-2 px-3 py-2.5 rounded-lg border border-dashed border-[#3d3d5c] hover:border-[#d97706] bg-[#12121a] hover:bg-[#1e1e32] transition-colors disabled:opacity-50"
          >
            {isOpening ? (
              <>
                <div className="w-4 h-4 border-2 border-[#d97706] border-t-transparent rounded-full animate-spin" />
                <span className="text-[12px] text-neutral-400">Opening folder...</span>
              </>
            ) : currentWorkspace ? (
              <div className="flex flex-col items-start w-full min-w-0">
                <div className="flex items-center gap-2 w-full">
                  <FolderOpen size={14} className="text-amber-400 flex-shrink-0" />
                  <span className="text-[13px] font-medium text-[#e0e0e8] truncate">
                    {getFolderName(currentWorkspace)}
                  </span>
                </div>
                <span className="text-[10px] text-neutral-500 mt-0.5 truncate w-full text-left">
                  {currentWorkspace}
                </span>
              </div>
            ) : (
              <>
                <FolderOpen size={14} className="text-neutral-500" />
                <span className="text-[12px] text-neutral-500">No workspace — click to open folder</span>
              </>
            )}
          </button>

          <input
            ref={fileInputRef}
            type="file"
            className="hidden"
            {...({ webkitdirectory: '', directory: '' } as React.InputHTMLAttributes<HTMLInputElement>)}
            onChange={handleFileInputChange}
          />
        </div>

        <div>
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <History size={14} className="text-neutral-400" />
              <span className="text-[11px] font-semibold text-neutral-400 uppercase tracking-wide">
                Recent
              </span>
            </div>
            {recentWorkspaces.length > 0 && (
              <button
                onClick={handleClearRecent}
                className="text-[10px] text-neutral-500 hover:text-red-400 transition-colors"
              >
                Clear
              </button>
            )}
          </div>

          {recentWorkspaces.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-6 text-neutral-600 gap-2">
              <Clock size={24} className="opacity-40" />
              <span className="text-[12px]">No recent workspaces</span>
            </div>
          ) : (
            <div className="space-y-1">
              {recentWorkspaces.map((entry) => (
                <button
                  key={entry.path}
                  onClick={() => handleRecentClick(entry)}
                  className="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg hover:bg-[#252545] transition-colors group text-left"
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      {entry.path === currentWorkspace ? (
                        <Star size={13} className="text-amber-400 flex-shrink-0" />
                      ) : (
                        <Folder size={13} className="text-amber-400/60 flex-shrink-0" />
                      )}
                      <span
                        className={`text-[13px] truncate ${
                          entry.path === currentWorkspace
                            ? 'text-amber-400 font-medium'
                            : 'text-[#e0e0e8]'
                        }`}
                      >
                        {entry.name}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 mt-0.5 ml-[22px]">
                      <span className="text-[10px] text-neutral-500 truncate">{entry.path}</span>
                      <span className="text-[10px] text-neutral-600 flex-shrink-0">
                        {formatTimestamp(entry.lastOpened)}
                      </span>
                    </div>
                  </div>
                  <button
                    onClick={(e) => handleRemoveRecent(e, entry.path)}
                    className="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-[#3d3d5c] text-neutral-500 hover:text-red-400 transition-all flex-shrink-0"
                    title="Remove from recent"
                  >
                    <X size={12} />
                  </button>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}