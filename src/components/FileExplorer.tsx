import React, { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import {
  Folder, FolderOpen, FileText, File, ChevronRight, ChevronDown,
  Plus, Search, RefreshCw, MoreVertical, Download, Copy, Trash2,
  ExternalLink, Edit3, FolderPlus, FilePlus,
} from 'lucide-react';
import { getFileSystemTree, readFileContent, writeFileContent, createFileOrDir, deleteFileOrDir, type FsFileNode } from '../api';
import { getErrorMessage } from '../utils/errorHelpers';

const IGNORED_DIRS = new Set([
  '.git', 'node_modules', 'dist', 'build', '.next',
  'target', '__pycache__', '.venv', '.idea', '.vscode',
]);

const CODE_EXTENSIONS: Record<string, { label: string; color: string }> = {
  ts:   { label: 'TS',  color: 'bg-blue-500/20 text-blue-400' },
  tsx:  { label: 'TS',  color: 'bg-blue-500/20 text-blue-400' },
  js:   { label: 'JS',  color: 'bg-yellow-500/20 text-yellow-400' },
  jsx:  { label: 'JS',  color: 'bg-yellow-500/20 text-yellow-400' },
  rs:   { label: 'RS',  color: 'bg-orange-500/20 text-orange-400' },
  py:   { label: 'PY',  color: 'bg-blue-400/20 text-blue-300' },
  go:   { label: 'GO',  color: 'bg-cyan-500/20 text-cyan-400' },
  json: { label: '{}',  color: 'bg-yellow-500/20 text-yellow-400' },
  css:  { label: '#',   color: 'bg-blue-500/20 text-blue-400' },
  html: { label: '<>',  color: 'bg-orange-500/20 text-orange-400' },
  md:   { label: 'MD',  color: 'bg-blue-500/20 text-blue-400' },
  vue:  { label: 'V',   color: 'bg-green-500/20 text-green-400' },
  svelte: { label: 'S', color: 'bg-orange-500/20 text-orange-400' },
  sql:  { label: 'SQ',  color: 'bg-purple-500/20 text-purple-400' },
};

type GitStatus = 'modified' | 'new' | 'untracked' | 'deleted' | null;

interface FsFileNodeWithGit extends FsFileNode {
  git_status?: GitStatus;
}

interface FileExplorerProps {
  onFileSelect: (filePath: string) => void;
  workspacePath?: string;
}

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  node: FsFileNodeWithGit | null;
}

function getExtension(name: string): string {
  const parts = name.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

function getGitDot(status: GitStatus): { color: string } | null {
  switch (status) {
    case 'modified': return { color: 'bg-yellow-400' };
    case 'new':
    case 'untracked': return { color: 'bg-green-400' };
    case 'deleted': return { color: 'bg-red-400' };
    default: return null;
  }
}

const ContextMenu: React.FC<{
  state: ContextMenuState;
  onClose: () => void;
  onNewFile: (parentPath: string) => void;
  onNewFolder: (parentPath: string) => void;
  onRename: (node: FsFileNodeWithGit) => void;
  onDelete: (node: FsFileNodeWithGit) => void;
  onCopyPath: (node: FsFileNodeWithGit) => void;
  onDownload: (node: FsFileNodeWithGit) => void;
  onOpenInSystem: (node: FsFileNodeWithGit) => void;
}> = ({ state, onClose, onNewFile, onNewFolder, onRename, onDelete, onCopyPath, onDownload, onOpenInSystem }) => {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const keyHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (state.visible) {
      document.addEventListener('mousedown', handler);
      document.addEventListener('keydown', keyHandler);
    }
    return () => {
      document.removeEventListener('mousedown', handler);
      document.removeEventListener('keydown', keyHandler);
    };
  }, [state.visible, onClose]);

  if (!state.visible || !state.node) return null;

  const node = state.node;
  const parentPath = node.is_dir ? node.path : node.path.substring(0, node.path.lastIndexOf('/') || node.path.lastIndexOf('\\'));

  const items = [
    ...(node.is_dir ? [
      { icon: FilePlus, label: 'New File', action: () => { onNewFile(node.path); onClose(); } },
      { icon: FolderPlus, label: 'New Folder', action: () => { onNewFolder(node.path); onClose(); } },
      { separator: true },
    ] : []),
    { icon: Edit3, label: 'Rename', action: () => { onRename(node); onClose(); } },
    { icon: Trash2, label: 'Delete', action: () => { onDelete(node); onClose(); } },
    { separator: true },
    { icon: Copy, label: 'Copy Path', action: () => { onCopyPath(node); onClose(); } },
    { icon: Download, label: 'Download File', action: () => { onDownload(node); onClose(); }, hide: node.is_dir },
    { icon: ExternalLink, label: 'Open in System App', action: () => { onOpenInSystem(node); onClose(); } },
  ];

  const style: React.CSSProperties = {
    position: 'fixed',
    left: state.x,
    top: state.y,
    zIndex: 100,
  };

  return (
    <div ref={ref} style={style} className="bg-[#12121a] border border-[#2d2d44] rounded-lg shadow-2xl py-1 min-w-[180px]">
      {items.map((item, i) => {
        if ('separator' in item) {
          return <div key={`sep-${i}`} className="my-1 border-t border-[#2d2d44]" />;
        }
        if (item.hide) return null;
        return (
          <button
            key={item.label}
            onClick={item.action}
            className="w-full flex items-center gap-2 px-3 py-1.5 text-[12px] text-[#e0e0e8] hover:bg-[#252545] transition-colors text-left"
          >
            <item.icon size={13} className="text-neutral-400" />
            {item.label}
          </button>
        );
      })}
    </div>
  );
};

const TreeNode: React.FC<{
  node: FsFileNodeWithGit;
  depth: number;
  selectedPath: string | null;
  onSelect: (node: FsFileNodeWithGit) => void;
  expandedPaths: Set<string>;
  onToggle: (path: string) => void;
  onContextMenu: (e: React.MouseEvent, node: FsFileNodeWithGit) => void;
  editingPath: string | null;
  editingValue: string;
  onEditChange: (value: string) => void;
  onEditSubmit: () => void;
  onEditCancel: () => void;
  editInputRef: React.RefObject<HTMLInputElement | null>;
  filterText: string;
}> = ({
  node, depth, selectedPath, onSelect, expandedPaths, onToggle,
  onContextMenu, editingPath, editingValue, onEditChange, onEditSubmit, onEditCancel,
  editInputRef, filterText,
}) => {
  const isExpanded = expandedPaths.has(node.path);
  const isSelected = selectedPath === node.path;
  const hasChildren = node.is_dir && node.children && node.children.length > 0;
  const isEditing = editingPath === node.path;
  const ext = getExtension(node.name);
  const extInfo = CODE_EXTENSIONS[ext];
  const gitDot = node.git_status ? getGitDot(node.git_status) : null;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (node.is_dir) {
      onToggle(node.path);
    } else {
      onSelect(node);
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    onContextMenu(e, node);
  };

  const visibleChildren = node.children
    ? node.children
        .filter((child) => !child.is_dir || !IGNORED_DIRS.has(child.name))
        .filter((child) => {
          if (!filterText) return true;
          return child.name.toLowerCase().includes(filterText.toLowerCase());
        })
    : [];

  const hasVisibleChildren = hasChildren && visibleChildren.length > 0;

  return (
    <div>
      <div
        className={`flex items-center gap-1 py-0.5 pr-2 rounded cursor-pointer select-none transition-colors ${
          isSelected
            ? 'bg-[#d97706]/20 text-[#e0e0e8]'
            : 'hover:bg-[#252545] text-[#e0e0e8]'
        }`}
        style={{ paddingLeft: `${depth * 16 + 4}px` }}
        onClick={handleClick}
        onContextMenu={handleContextMenu}
      >
        <span className="w-4 h-4 flex-shrink-0 flex items-center justify-center">
          {node.is_dir ? (
            hasVisibleChildren ? (
              isExpanded ? <ChevronDown size={12} className="text-neutral-500" /> : <ChevronRight size={12} className="text-neutral-500" />
            ) : null
          ) : null}
        </span>

        {node.is_dir ? (
          isExpanded ? (
            <FolderOpen size={14} className="flex-shrink-0 text-amber-400" />
          ) : (
            <Folder size={14} className="flex-shrink-0 text-amber-400" />
          )
        ) : extInfo ? (
          <span className={`flex-shrink-0 text-[9px] font-bold px-1 rounded ${extInfo.color}`}>
            {extInfo.label}
          </span>
        ) : (
          <FileText size={14} className="flex-shrink-0 text-neutral-500" />
        )}

        {gitDot && (
          <span className={`flex-shrink-0 w-1.5 h-1.5 rounded-full ${gitDot.color}`} />
        )}

        {isEditing ? (
          <input
            ref={editInputRef}
            value={editingValue}
            onChange={(e) => onEditChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') onEditSubmit();
              if (e.key === 'Escape') onEditCancel();
            }}
            onBlur={onEditSubmit}
            onClick={(e) => e.stopPropagation()}
            className="flex-1 min-w-0 bg-[#2d2d44] border border-[#d97706] rounded px-1 py-0 text-[12px] text-[#e0e0e8] outline-none"
            autoFocus
          />
        ) : (
          <span className="text-[12px] truncate leading-none">{node.name}</span>
        )}
      </div>

      {node.is_dir && isExpanded && hasVisibleChildren && (
        <div>
          {visibleChildren.map((child) => (
            <TreeNode
              key={child.path}
              node={child as FsFileNodeWithGit}
              depth={depth + 1}
              selectedPath={selectedPath}
              onSelect={onSelect}
              expandedPaths={expandedPaths}
              onToggle={onToggle}
              onContextMenu={onContextMenu}
              editingPath={editingPath}
              editingValue={editingValue}
              onEditChange={onEditChange}
              onEditSubmit={onEditSubmit}
              onEditCancel={onEditCancel}
              editInputRef={editInputRef}
              filterText={filterText}
            />
          ))}
        </div>
      )}
    </div>
  );
};

const FileExplorer: React.FC<FileExplorerProps> = ({ onFileSelect, workspacePath }) => {
  const [tree, setTree] = useState<FsFileNodeWithGit[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [filterText, setFilterText] = useState('');
  const [showSearch, setShowSearch] = useState(false);

  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    visible: false, x: 0, y: 0, node: null,
  });

  const [editingPath, setEditingPath] = useState<string | null>(null);
  const [editingValue, setEditingValue] = useState('');
  const [editMode, setEditMode] = useState<'rename' | 'createFile' | 'createDir' | null>(null);
  const editInputRef = useRef<HTMLInputElement | null>(null);

  const loadTree = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await getFileSystemTree(workspacePath);
      const nodesWithGit = (res.tree || []).map((n) => ({
        ...n,
        git_status: (n as FsFileNodeWithGit).git_status || null,
      }));
      setTree(nodesWithGit);
      if (nodesWithGit.length > 0) {
        const firstDir = nodesWithGit.find((n) => n.is_dir);
        if (firstDir) {
          setExpandedPaths(new Set([firstDir.path]));
        }
      }
    } catch (e: unknown) {
      setError(getErrorMessage(e) || 'Failed to load file tree');
    } finally {
      setLoading(false);
    }
  }, [workspacePath]);

  useEffect(() => {
    loadTree();
  }, [loadTree]);

  const handleSelect = useCallback(async (node: FsFileNodeWithGit) => {
    if (node.is_dir) return;
    setSelectedPath(node.path);
    onFileSelect(node.path);
  }, [onFileSelect]);

  const handleToggle = useCallback((path: string) => {
    setExpandedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  }, []);

  const handleContextMenu = useCallback((e: React.MouseEvent, node: FsFileNodeWithGit) => {
    e.preventDefault();
    e.stopPropagation();
    const menuWidth = 180;
    const menuHeight = 230;
    let x = e.clientX;
    let y = e.clientY;
    if (x + menuWidth > window.innerWidth) x = window.innerWidth - menuWidth - 8;
    if (y + menuHeight > window.innerHeight) y = window.innerHeight - menuHeight - 8;
    setContextMenu({ visible: true, x, y, node });
  }, []);

  const closeContextMenu = useCallback(() => {
    setContextMenu({ visible: false, x: 0, y: 0, node: null });
  }, []);

  const startInlineEdit = useCallback((nodePath: string, initialValue: string, mode: 'rename' | 'createFile' | 'createDir') => {
    setEditingPath(nodePath);
    setEditingValue(initialValue);
    setEditMode(mode);
    setTimeout(() => {
      editInputRef.current?.focus();
      editInputRef.current?.select();
    }, 0);
  }, []);

  const cancelEdit = useCallback(() => {
    setEditingPath(null);
    setEditingValue('');
    setEditMode(null);
  }, []);

  const submitEdit = useCallback(async () => {
    if (!editingPath || !editMode) return;

    const value = editingValue.trim();
    if (!value) {
      cancelEdit();
      return;
    }

    try {
      if (editMode === 'rename') {
        const separator = editingPath.includes('\\') ? '\\' : '/';
        const parentDir = editingPath.substring(0, editingPath.lastIndexOf(separator));
        const newPath = parentDir + separator + value;
        const content = await readFileContent(editingPath);
        await writeFileContent(newPath, content.content);
        try {
          await deleteFileOrDir(editingPath);
        } catch {
          // may fail on dirs
        }
      } else if (editMode === 'createFile') {
        await createFileOrDir(editingPath + (editingPath.includes('\\') ? '\\' : '/') + value, false);
      } else if (editMode === 'createDir') {
        await createFileOrDir(editingPath + (editingPath.includes('\\') ? '\\' : '/') + value, true);
      }
      await loadTree();
    } catch (e: unknown) {
      console.error('Edit failed:', getErrorMessage(e));
    }

    cancelEdit();
  }, [editingPath, editingValue, editMode, cancelEdit, loadTree]);

  const handleNewFile = useCallback((parentPath: string) => {
    setExpandedPaths((prev) => {
      const next = new Set(prev);
      next.add(parentPath);
      return next;
    });
    startInlineEdit(parentPath, '', 'createFile');
  }, [startInlineEdit]);

  const handleNewFolder = useCallback((parentPath: string) => {
    setExpandedPaths((prev) => {
      const next = new Set(prev);
      next.add(parentPath);
      return next;
    });
    startInlineEdit(parentPath, '', 'createDir');
  }, [startInlineEdit]);

  const handleRename = useCallback((node: FsFileNodeWithGit) => {
    startInlineEdit(node.path, node.name, 'rename');
  }, [startInlineEdit]);

  const handleDelete = useCallback(async (node: FsFileNodeWithGit) => {
    const confirmed = window.confirm(`Delete "${node.name}"?`);
    if (!confirmed) return;
    try {
      await deleteFileOrDir(node.path);
      if (selectedPath === node.path) {
        setSelectedPath(null);
      }
      await loadTree();
    } catch (e: unknown) {
      console.error('Delete failed:', getErrorMessage(e));
    }
  }, [selectedPath, loadTree]);

  const handleCopyPath = useCallback(async (node: FsFileNodeWithGit) => {
    try {
      await navigator.clipboard.writeText(node.path);
    } catch {
      // fallback silently
    }
  }, []);

  const handleDownload = useCallback(async (node: FsFileNodeWithGit) => {
    try {
      const res = await readFileContent(node.path);
      const blob = new Blob([res.content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = node.name;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e: unknown) {
      console.error('Download failed:', getErrorMessage(e));
    }
  }, []);

  const handleOpenInSystem = useCallback(async (node: FsFileNodeWithGit) => {
    try {
      const isTauri = '__TAURI_INTERNALS__' in window;
      if (isTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('open_in_system', { path: node.path });
      } else {
        window.open(`file://${node.path}`, '_blank');
      }
    } catch (e: unknown) {
      console.error('Open in system failed:', getErrorMessage(e));
    }
  }, []);

  const handleRefresh = useCallback(() => {
    loadTree();
  }, [loadTree]);

  const filteredTree = useMemo(() => {
    if (!filterText) return tree;
    return tree.filter((node) => {
      if (!IGNORED_DIRS.has(node.name) && node.name.toLowerCase().includes(filterText.toLowerCase())) {
        return true;
      }
      if (node.is_dir && node.children) {
        const hasMatchingChild = node.children.some((child) =>
          child.name.toLowerCase().includes(filterText.toLowerCase())
        );
        return hasMatchingChild;
      }
      return false;
    });
  }, [tree, filterText]);

  const handleNewFileAtRoot = useCallback(() => {
    const rootPath = workspacePath || tree[0]?.path?.split(/[/\\]/).slice(0, -1).join('/') || '.';
    startInlineEdit(rootPath, '', 'createFile');
  }, [workspacePath, tree, startInlineEdit]);

  const handleNewFolderAtRoot = useCallback(() => {
    const rootPath = workspacePath || tree[0]?.path?.split(/[/\\]/).slice(0, -1).join('/') || '.';
    startInlineEdit(rootPath, '', 'createDir');
  }, [workspacePath, tree, startInlineEdit]);

  return (
    <div className="flex flex-col h-full bg-[#1a1a2e] text-[#e0e0e8] text-[13px]">
      <div className="flex items-center justify-between px-3 py-2 border-b border-[#2d2d44]">
        <span className="text-[11px] font-semibold text-neutral-400 uppercase tracking-wide">
          Explorer
        </span>
        <div className="flex items-center gap-0.5">
          <button
            onClick={() => setShowSearch((v) => !v)}
            className="p-1 rounded hover:bg-[#252545] text-neutral-400 hover:text-[#e0e0e8] transition-colors"
            title="Search"
          >
            <Search size={13} />
          </button>
          <button
            onClick={handleRefresh}
            className="p-1 rounded hover:bg-[#252545] text-neutral-400 hover:text-[#e0e0e8] transition-colors"
            title="Refresh"
          >
            <RefreshCw size={13} />
          </button>
          <button
            onClick={handleNewFileAtRoot}
            className="p-1 rounded hover:bg-[#252545] text-neutral-400 hover:text-[#e0e0e8] transition-colors"
            title="New File"
          >
            <FilePlus size={13} />
          </button>
          <button
            onClick={handleNewFolderAtRoot}
            className="p-1 rounded hover:bg-[#252545] text-neutral-400 hover:text-[#e0e0e8] transition-colors"
            title="New Folder"
          >
            <FolderPlus size={13} />
          </button>
        </div>
      </div>

      {showSearch && (
        <div className="px-2 py-1.5 border-b border-[#2d2d44]">
          <div className="flex items-center gap-1.5 bg-[#12121a] rounded px-2 py-1 border border-[#2d2d44]">
            <Search size={12} className="text-neutral-500" />
            <input
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              placeholder="Filter files..."
              className="flex-1 bg-transparent text-[12px] text-[#e0e0e8] outline-none placeholder:text-neutral-600"
              autoFocus
            />
            {filterText && (
              <button
                onClick={() => setFilterText('')}
                className="text-neutral-500 hover:text-[#e0e0e8] text-[10px]"
              >
                ✕
              </button>
            )}
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto overflow-x-hidden py-1">
        {loading ? (
          <div className="flex items-center justify-center py-8 text-neutral-500 text-[12px]">
            Loading...
          </div>
        ) : error ? (
          <div className="px-3 py-4 text-red-400 text-[12px]">{error}</div>
        ) : filteredTree.length === 0 ? (
          <div className="px-3 py-4 text-neutral-500 text-[12px]">
            {filterText ? 'No matching files' : 'No files found'}
          </div>
        ) : (
          filteredTree
            .filter((node) => !node.is_dir || !IGNORED_DIRS.has(node.name))
            .map((node) => (
              <TreeNode
                key={node.path}
                node={node}
                depth={0}
                selectedPath={selectedPath}
                onSelect={handleSelect}
                expandedPaths={expandedPaths}
                onToggle={handleToggle}
                onContextMenu={handleContextMenu}
                editingPath={editingPath}
                editingValue={editingValue}
                onEditChange={setEditingValue}
                onEditSubmit={submitEdit}
                onEditCancel={cancelEdit}
                editInputRef={editInputRef}
                filterText={filterText}
              />
            ))
        )}
      </div>

      <ContextMenu
        state={contextMenu}
        onClose={closeContextMenu}
        onNewFile={handleNewFile}
        onNewFolder={handleNewFolder}
        onRename={handleRename}
        onDelete={handleDelete}
        onCopyPath={handleCopyPath}
        onDownload={handleDownload}
        onOpenInSystem={handleOpenInSystem}
      />
    </div>
  );
};

export default FileExplorer;