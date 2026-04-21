import React, { useState, useEffect, useRef, useCallback } from 'react';
import { X, Github, ChevronDown, Link2, Folder, FileText, ChevronLeft, Check, Loader2 } from 'lucide-react';
import { getGithubStatus, getGithubAuthUrl, getGithubRepos, getGithubContents, getGithubTree } from '../api';

interface Repo {
  id: number;
  name: string;
  full_name: string;
  description?: string;
  private?: boolean;
  html_url?: string;
  language?: string;
  updated_at?: string;
}

interface GhEntry {
  name: string;
  path: string;
  sha: string;
  size: number;
  type: 'file' | 'dir' | 'symlink' | 'submodule';
  download_url?: string | null;
  content?: string;
  encoding?: string;
}

interface TreeEntry {
  path: string;
  type: string; // 'blob' | 'tree'
  size: number;
}

interface SelectedItem {
  key: string;
  repoFullName: string;
  path: string;
  name: string;
  size: number;
  isFolder: boolean;
}

export interface GithubAddPayload {
  repoFullName: string;
  ref: string;
  selections: Array<{ path: string; isFolder: boolean }>;
}

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (payload: GithubAddPayload) => Promise<void> | void;
  currentContextTokens?: number;
  contextLimit?: number;
}

const BLUE = '#4B9EFA';
const REPOS_CACHE_KEY = 'gh_repos_cache_v1';
const TREE_CACHE_PREFIX = 'gh_tree_cache_v1:';
const REPOS_TTL_MS = 60 * 60 * 1000; // 1 hour
const TREE_TTL_MS = 30 * 60 * 1000;  // 30 min

function readCache<T>(key: string, ttl: number): T | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed.timestamp !== 'number') return null;
    if (Date.now() - parsed.timestamp > ttl) return null;
    return parsed.data as T;
  } catch {
    return null;
  }
}

function writeCache(key: string, data: any) {
  try {
    localStorage.setItem(key, JSON.stringify({ data, timestamp: Date.now() }));
  } catch {
    // ignore quota errors
  }
}

const AddFromGithubModal: React.FC<Props> = ({ isOpen, onClose, onConfirm, currentContextTokens = 0, contextLimit = 200000 }) => {
  const [connected, setConnected] = useState<boolean | null>(null);
  const [connectingPoll, setConnectingPoll] = useState(false);
  const [repos, setRepos] = useState<Repo[]>([]);
  const [reposLoading, setReposLoading] = useState(false);
  const [showRepoDropdown, setShowRepoDropdown] = useState(false);
  const [selectedRepo, setSelectedRepo] = useState<Repo | null>(null);

  const [currentPath, setCurrentPath] = useState('');
  const [entries, setEntries] = useState<GhEntry[]>([]);
  const [entriesLoading, setEntriesLoading] = useState(false);
  const [browseError, setBrowseError] = useState<string | null>(null);

  const [tree, setTree] = useState<TreeEntry[] | null>(null);
  const [treeLoading, setTreeLoading] = useState(false);

  const [selected, setSelected] = useState<Map<string, SelectedItem>>(new Map());
  const [confirming, setConfirming] = useState(false);
  const [confirmError, setConfirmError] = useState<string | null>(null);

  const [urlInput, setUrlInput] = useState('');
  const [showUrlInput, setShowUrlInput] = useState(false);

  const repoBtnRef = useRef<HTMLButtonElement>(null);

  const reset = () => {
    setSelectedRepo(null);
    setCurrentPath('');
    setEntries([]);
    setTree(null);
    setSelected(new Map());
    setBrowseError(null);
    setConfirmError(null);
    setUrlInput('');
    setShowUrlInput(false);
    setShowRepoDropdown(false);
  };

  useEffect(() => {
    if (!isOpen) return;
    let cancelled = false;
    getGithubStatus().then(data => {
      if (cancelled) return;
      setConnected(!!data?.connected);
    }).catch(() => {
      if (!cancelled) setConnected(false);
    });
    return () => { cancelled = true; };
  }, [isOpen]);

  useEffect(() => {
    if (!connectingPoll) return;
    const timer = setInterval(() => {
      getGithubStatus().then(data => {
        if (data?.connected) {
          setConnected(true);
          setConnectingPoll(false);
        }
      }).catch(() => { });
    }, 2000);
    return () => clearInterval(timer);
  }, [connectingPoll]);

  useEffect(() => {
    if (!isOpen || !connected) return;
    // Show cached repos immediately (stale-while-revalidate)
    const cached = readCache<Repo[]>(REPOS_CACHE_KEY, REPOS_TTL_MS);
    if (cached && Array.isArray(cached) && cached.length > 0) {
      setRepos(cached);
      setReposLoading(false);
    } else {
      setReposLoading(true);
    }
    // Refresh in background
    getGithubRepos(1).then(data => {
      if (Array.isArray(data)) {
        setRepos(data);
        writeCache(REPOS_CACHE_KEY, data);
      }
    }).catch(() => { }).finally(() => setReposLoading(false));
  }, [isOpen, connected]);

  const loadPath = useCallback(async (repo: Repo, path: string) => {
    setEntriesLoading(true);
    setBrowseError(null);
    try {
      const [owner, repoName] = repo.full_name.split('/');
      const data = await getGithubContents(owner, repoName, path);
      if (Array.isArray(data)) {
        const sorted = [...data].sort((a, b) => {
          if (a.type !== b.type) return a.type === 'dir' ? -1 : 1;
          return a.name.localeCompare(b.name);
        });
        setEntries(sorted);
      } else {
        setEntries([]);
      }
    } catch (e: any) {
      setBrowseError(e?.message || 'Failed to load');
      setEntries([]);
    } finally {
      setEntriesLoading(false);
    }
  }, []);

  const loadTree = useCallback(async (repo: Repo) => {
    const cacheKey = TREE_CACHE_PREFIX + repo.full_name;
    const cached = readCache<TreeEntry[]>(cacheKey, TREE_TTL_MS);
    if (cached && Array.isArray(cached)) {
      setTree(cached);
      setTreeLoading(false);
    } else {
      setTreeLoading(true);
      setTree(null);
    }
    try {
      const [owner, repoName] = repo.full_name.split('/');
      const data = await getGithubTree(owner, repoName);
      const treeArr = data?.tree || [];
      setTree(treeArr);
      writeCache(cacheKey, treeArr);
    } catch {
      if (!cached) setTree([]);
    } finally {
      setTreeLoading(false);
    }
  }, []);

  const handleSelectRepo = (repo: Repo) => {
    setSelectedRepo(repo);
    setShowRepoDropdown(false);
    setCurrentPath('');
    setSelected(new Map());
    loadPath(repo, '');
    loadTree(repo);
  };

  const handleEnterDir = (entry: GhEntry) => {
    if (!selectedRepo) return;
    setCurrentPath(entry.path);
    loadPath(selectedRepo, entry.path);
  };

  const handleGoUp = () => {
    if (!selectedRepo) return;
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    const next = parts.join('/');
    setCurrentPath(next);
    loadPath(selectedRepo, next);
  };

  // Compute size of a folder by summing all blob sizes under its path in the tree
  const getFolderSize = useCallback((path: string): number => {
    if (!tree) return 0;
    const prefix = path ? path + '/' : '';
    let total = 0;
    for (const t of tree) {
      if (t.type === 'blob' && (prefix === '' || t.path.startsWith(prefix))) {
        total += t.size || 0;
      }
    }
    return total;
  }, [tree]);

  const toggleEntry = (entry: GhEntry) => {
    if (!selectedRepo) return;
    const key = `${selectedRepo.full_name}:${entry.path}`;
    const isFolder = entry.type === 'dir';
    setSelected(prev => {
      const next = new Map(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.set(key, {
          key,
          repoFullName: selectedRepo.full_name,
          path: entry.path,
          name: entry.name,
          size: isFolder ? getFolderSize(entry.path) : (entry.size || 0),
          isFolder,
        });
      }
      return next;
    });
  };

  const handleConnect = async () => {
    try {
      const { url } = await getGithubAuthUrl();
      import('../utils/tauriAPI').then(m => {
        m.tauriAPI.openExternal(url);
      }).catch(() => window.open(url, '_blank'));
      setConnectingPoll(true);
    } catch (e) {
      console.error('GitHub auth error:', e);
    }
  };

  const handleUrlSubmit = async () => {
    const url = urlInput.trim();
    if (!url) return;
    const m = url.match(/github\.com[/:]([^/]+)\/([^/]+?)(?:\.git)?(?:\/(?:tree|blob)\/[^/]+\/(.+))?$/i);
    if (!m) {
      setBrowseError('Invalid GitHub URL');
      return;
    }
    const [, owner, repoName, innerPath] = m;
    const fullName = `${owner}/${repoName}`;
    const pseudoRepo: Repo = { id: -1, name: repoName, full_name: fullName };
    setSelectedRepo(pseudoRepo);
    setShowUrlInput(false);
    setSelected(new Map());
    setCurrentPath(innerPath || '');
    loadPath(pseudoRepo, innerPath || '');
    loadTree(pseudoRepo);
  };

  const handleConfirm = async () => {
    if (selected.size === 0 || !selectedRepo) return;
    setConfirming(true);
    setConfirmError(null);
    try {
      const selections: Array<{ path: string; isFolder: boolean }> = [];
      for (const item of selected.values()) {
        selections.push({ path: item.path, isFolder: !!item.isFolder });
      }
      if (selections.length === 0) {
        setConfirming(false);
        return;
      }
      await onConfirm({
        repoFullName: selectedRepo.full_name,
        ref: 'main',
        selections,
      });
      reset();
      onClose();
    } catch (e: any) {
      console.error('[GitHub Add] failed:', e);
      setConfirmError(e?.message || 'Failed to add files');
    } finally {
      setConfirming(false);
    }
  };

  const handleClose = () => {
    reset();
    onClose();
  };

  // Per-row capacity % preview (relative to context limit)
  const getEntryPctLabel = useCallback((entry: GhEntry): string => {
    if (!contextLimit) return '';
    const bytes = entry.type === 'file' ? (entry.size || 0) : getFolderSize(entry.path);
    if (!bytes) return '<1%';
    const tokens = Math.ceil(bytes / 4);
    const pct = (tokens / contextLimit) * 100;
    if (pct < 1) return '<1%';
    if (pct < 10) return pct.toFixed(1) + '%';
    return Math.round(pct).toLocaleString() + '%';
  }, [contextLimit, getFolderSize]);

  if (!isOpen) return null;

  const pathParts = currentPath ? currentPath.split('/').filter(Boolean) : [];

  const selectedBytes = Array.from(selected.values()).reduce((sum, f) => sum + (f.size || 0), 0);
  const selectedTokens = Math.ceil(selectedBytes / 4);
  const totalTokens = currentContextTokens + selectedTokens;
  const capacityPctRaw = contextLimit > 0 ? (totalTokens / contextLimit) * 100 : 0;
  const isOverflow = capacityPctRaw > 100;
  const capacityBarPct = Math.min(100, capacityPctRaw);
  const capacityLabel = capacityPctRaw === 0
    ? '0'
    : capacityPctRaw < 1
      ? capacityPctRaw.toFixed(1)
      : Math.round(capacityPctRaw).toLocaleString();

  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40 backdrop-blur-[2px]"
      onClick={handleClose}
    >
      <div
        className="bg-claude-bg border border-claude-border rounded-2xl shadow-2xl w-[min(720px,92vw)] h-[min(680px,88vh)] flex flex-col font-sans"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-start justify-between px-5 pt-5 pb-3">
          <div>
            <h2 className="text-[15px] font-semibold text-claude-text">Add content from GitHub</h2>
            <p className="text-[12px] text-claude-textSecondary mt-0.5">
              Select the files you would like to add to this chat
            </p>
          </div>
          <button
            onClick={handleClose}
            className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
          >
            <X size={16} />
          </button>
        </div>

        {/* Repo picker row */}
        <div className="px-5 pb-3 flex items-center gap-2">
          <Github size={18} className="text-claude-text flex-shrink-0" />
          <div className="relative">
            <button
              ref={repoBtnRef}
              disabled={!connected}
              onClick={() => setShowRepoDropdown(prev => !prev)}
              className="flex items-center gap-1.5 px-3 py-1.5 border border-claude-border rounded-lg text-[13px] text-claude-text bg-claude-input hover:bg-claude-hover transition-colors disabled:opacity-50 min-w-[180px]"
            >
              <span className="truncate">
                {selectedRepo ? selectedRepo.full_name : 'Select a repository'}
              </span>
              <ChevronDown size={14} className="text-claude-textSecondary ml-auto flex-shrink-0" />
            </button>
            {showRepoDropdown && connected && (
              <div className="absolute left-0 top-full mt-1 w-[300px] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_16px_rgba(0,0,0,0.16)] py-1 z-10 max-h-[260px] overflow-y-auto">
                {reposLoading && (
                  <div className="px-4 py-3 text-[12px] text-claude-textSecondary flex items-center gap-2">
                    <Loader2 size={12} className="animate-spin" /> Loading repositories...
                  </div>
                )}
                {!reposLoading && repos.length === 0 && (
                  <div className="px-4 py-3 text-[12px] text-claude-textSecondary">No repositories found</div>
                )}
                {!reposLoading && repos.map(r => (
                  <button
                    key={r.id}
                    onClick={() => handleSelectRepo(r)}
                    className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors truncate flex items-center gap-2"
                  >
                    <span className="text-claude-textSecondary">{r.full_name.split('/')[0]} /</span>
                    <span>{r.name}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
          <button
            onClick={() => setShowUrlInput(prev => !prev)}
            title="Paste a GitHub URL"
            className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
          >
            <Link2 size={16} />
          </button>
        </div>

        {showUrlInput && (
          <div className="px-5 pb-3 flex items-center gap-2">
            <input
              type="text"
              placeholder="https://github.com/owner/repo"
              value={urlInput}
              onChange={e => setUrlInput(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter') handleUrlSubmit(); }}
              className="flex-1 px-3 py-1.5 border border-claude-border rounded-lg text-[13px] bg-claude-input text-claude-text outline-none focus:border-[#CCC] dark:focus:border-[#5a5a58]"
            />
            <button
              onClick={handleUrlSubmit}
              className="px-3 py-1.5 bg-black text-white dark:bg-white dark:text-black rounded-lg text-[12px] hover:opacity-90 transition-opacity"
            >
              Open
            </button>
          </div>
        )}

        {/* Browser area */}
        <div className="mx-5 mb-3 flex-1 min-h-0 border border-claude-border rounded-xl bg-claude-input overflow-hidden flex flex-col">
          {connected === false && (
            <div className="flex-1 flex flex-col items-center justify-center gap-3 p-6 text-center">
              <Github size={36} className="text-claude-textSecondary" />
              <p className="text-[13px] text-claude-textSecondary">Connect your GitHub account to browse repositories.</p>
              <button
                onClick={handleConnect}
                className="px-4 py-1.5 bg-black text-white dark:bg-white dark:text-black rounded-lg text-[13px] hover:opacity-90 transition-opacity"
              >
                {connectingPoll ? 'Waiting for GitHub...' : 'Connect GitHub'}
              </button>
            </div>
          )}

          {connected && !selectedRepo && (
            <div className="flex-1 flex items-center justify-center px-6 text-center">
              <p className="text-[13px] text-claude-textSecondary">
                Select a repository or paste a URL above to get started
              </p>
            </div>
          )}

          {connected && selectedRepo && (
            <>
              <div className="flex items-center gap-1 px-3 py-2 border-b border-claude-border text-[12px] text-claude-textSecondary flex-shrink-0">
                {currentPath && (
                  <button
                    onClick={handleGoUp}
                    className="p-0.5 hover:text-claude-text hover:bg-claude-hover rounded"
                    title="Go up"
                  >
                    <ChevronLeft size={14} />
                  </button>
                )}
                <span className="truncate">
                  <span className="text-claude-text font-medium">{selectedRepo.name}</span>
                  {pathParts.length > 0 && <span> / {pathParts.join(' / ')}</span>}
                </span>
                {treeLoading && <Loader2 size={11} className="animate-spin ml-auto" />}
              </div>
              <div className="flex-1 overflow-y-auto">
                {entriesLoading && (
                  <div className="p-4 text-[12px] text-claude-textSecondary flex items-center gap-2">
                    <Loader2 size={12} className="animate-spin" /> Loading...
                  </div>
                )}
                {browseError && !entriesLoading && (
                  <div className="p-4 text-[12px] text-red-500">{browseError}</div>
                )}
                {!entriesLoading && !browseError && entries.length === 0 && (
                  <div className="p-4 text-[12px] text-claude-textSecondary">Empty folder</div>
                )}
                {!entriesLoading && entries.map(entry => {
                  const isFile = entry.type === 'file';
                  const key = `${selectedRepo.full_name}:${entry.path}`;
                  const isSelected = selected.has(key);
                  return (
                    <div
                      key={entry.sha || entry.path}
                      className="flex items-center gap-2 px-3 py-1.5 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                    >
                      <div
                        onClick={(e) => { e.stopPropagation(); toggleEntry(entry); }}
                        className={`w-4 h-4 border rounded flex items-center justify-center flex-shrink-0 cursor-pointer ${isSelected ? 'border-transparent' : 'border-claude-border'}`}
                        style={isSelected ? { backgroundColor: BLUE, borderColor: BLUE } : undefined}
                      >
                        {isSelected && <Check size={10} className="text-white" />}
                      </div>
                      <div
                        onClick={() => { if (!isFile) handleEnterDir(entry); else toggleEntry(entry); }}
                        className="flex items-center gap-2 flex-1 min-w-0 cursor-pointer"
                      >
                        {isFile ? (
                          <FileText size={14} className="text-claude-textSecondary flex-shrink-0" />
                        ) : (
                          <Folder size={14} className="flex-shrink-0" style={{ color: isSelected ? BLUE : undefined }} fill={isSelected ? BLUE : 'none'} />
                        )}
                        <span className="truncate flex-1">{entry.name}</span>
                        <span className="text-[11px] text-claude-textSecondary flex-shrink-0">
                          {isFile ? formatSize(entry.size) : getEntryPctLabel(entry)}
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          )}

          {connected === null && (
            <div className="flex-1 flex items-center justify-center">
              <Loader2 size={18} className="animate-spin text-claude-textSecondary" />
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-5 py-3 border-t border-claude-border">
          <div className="flex items-center justify-between gap-3 mb-2">
            <div className="flex items-center gap-2 text-[12px] text-claude-textSecondary">
              <span>
                {selected.size === 0
                  ? 'Select files to add to chat context'
                  : `${selected.size} item${selected.size > 1 ? 's' : ''} selected`}
              </span>
            </div>
            <div className="flex items-center gap-3">
              <span
                className="text-[11px] whitespace-nowrap"
                style={{ color: isOverflow ? '#dc2626' : undefined }}
              >
                <span className={isOverflow ? '' : 'text-claude-textSecondary'}>
                  {capacityLabel}% of capacity used
                </span>
              </span>
              <button
                onClick={handleConfirm}
                disabled={selected.size === 0 || confirming}
                className="px-3 py-1.5 bg-black text-white dark:bg-white dark:text-black rounded-lg text-[12px] hover:opacity-90 transition-opacity disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5"
              >
                {confirming && <Loader2 size={12} className="animate-spin" />}
                Add to chat
              </button>
            </div>
          </div>
          {/* Progress bar */}
          <div className="h-[6px] w-full rounded-full bg-transparent border border-claude-border overflow-hidden">
            <div
              className={`h-full transition-all duration-200 ${isOverflow ? 'bg-red-600' : 'bg-black dark:bg-white'}`}
              style={{ width: `${capacityBarPct}%` }}
            />
          </div>
        </div>
        {confirmError && (
          <div className="px-5 pb-3 text-[12px] text-red-500">{confirmError}</div>
        )}
      </div>
    </div>
  );
};

function formatSize(n: number): string {
  if (!n && n !== 0) return '';
  if (n < 1024) return n + ' B';
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB';
  return (n / (1024 * 1024)).toFixed(1) + ' MB';
}

export default AddFromGithubModal;
