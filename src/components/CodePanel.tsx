import React, { useState, useRef, useEffect, useCallback } from 'react';
import { X, Code, Eye, Download, Copy, Check, ChevronDown, ChevronRight, PanelLeftClose, PanelLeftOpen } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneLight, vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import MarkdownRenderer from './MarkdownRenderer';
import SlidePreview from './SlidePreview';
import DocxPreview from './DocxPreview';
import PdfPreview from './PdfPreview';
import { DocumentInfo } from './DocumentCard';
import { copyToClipboard } from '../utils/clipboard';
import { buildArtifactHtml } from '../utils/artifactRenderer';
import { getFileSystemTree, readFileContent, writeFileContent, type FsFileNode } from '../api';
import { getErrorMessage } from '../utils/errorHelpers';

/* ── Artifact Preview (sandboxed iframe) ── */
const ArtifactPreview: React.FC<{ content: string; type: string }> = ({ content, type }) => {
  const [blobUrl, setBlobUrl] = React.useState<string | null>(null);
  React.useEffect(() => {
    const html = buildArtifactHtml(content, type);
    const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    setBlobUrl(url);
    return () => URL.revokeObjectURL(url);
  }, [content, type]);
  if (!blobUrl) return null;
  return <iframe src={blobUrl} className="w-full h-full border-0 bg-white" title="Artifact Preview" />;
};

/* ── Format helpers ── */
const CODE_EXTENSIONS = new Set(['ts', 'tsx', 'js', 'jsx', 'rs', 'py', 'go', 'java', 'c', 'cpp', 'h', 'css', 'scss', 'html', 'json', 'yaml', 'yml', 'toml', 'md', 'sh', 'bash', 'sql', 'xml', 'svg', 'graphql']);
const IGNORED_DIRS = new Set(['.git', 'node_modules', 'dist', 'build', '.next', '.nuxt', '__pycache__', '.venv', 'target', '.cargo', '.idea', '.vscode']);
const BINARY_FORMATS = ['pptx', 'docx', 'xlsx', 'pdf'];
const LANG_TO_EXT: Record<string, string> = {
  markdown: 'md', python: 'py', javascript: 'js', typescript: 'ts',
  java: 'java', c: 'c', cpp: 'cpp', csharp: 'cs',
  go: 'go', rust: 'rs', ruby: 'rb', php: 'php',
  swift: 'swift', kotlin: 'kt', scala: 'scala',
  html: 'html', css: 'css', scss: 'scss',
  sql: 'sql', shell: 'sh', bash: 'sh', powershell: 'ps1',
  yaml: 'yml', json: 'json', xml: 'xml', toml: 'toml',
  ini: 'ini', dockerfile: 'Dockerfile',
  r: 'r', matlab: 'm', lua: 'lua', perl: 'pl',
  dart: 'dart', vue: 'vue', svelte: 'svelte',
};

function detectFormat(doc: DocumentInfo) {
  const fmt = (doc.format || 'markdown').toLowerCase();
  const extMatch = (doc.title || '').match(/\.(\w+)$/);
  const ext = extMatch ? extMatch[1].toLowerCase() : '';
  const isBinary = BINARY_FORMATS.includes(fmt);
  const isMarkdown = ['markdown', 'md'].includes(fmt) || ext === 'md';
  const contentStart = (doc.content || '').trimStart().slice(0, 50).toLowerCase();
  const isHtml = ['html', 'htm'].includes(fmt) || ['html', 'htm'].includes(ext) || contentStart.startsWith('<!doctype html') || contentStart.startsWith('<html');
  const isReact = ['jsx', 'tsx'].includes(fmt) || ['jsx', 'tsx'].includes(ext);
  const isCode = !isBinary && !isMarkdown && !isHtml && !isReact;
  return { fmt, ext, isBinary, isMarkdown, isHtml, isReact, isCode };
}

function fsNodeToDoc(node: FsFileNode): DocumentInfo {
  const name = node.name;
  const ext = name.split('.').pop()?.toLowerCase() || '';
  const formatMap: Record<string, string> = { md: 'markdown', html: 'html', htm: 'html', jsx: 'jsx', tsx: 'tsx', py: 'python', rs: 'rust', go: 'go', java: 'java' };
  return {
    id: `fs:${node.path}`,
    title: name,
    filename: name,
    content: '', // loaded on demand
    format: formatMap[ext] || ext || 'text',
    url: '',
  };
}

/* ── File Tree ── */
const FileTree: React.FC<{
  tree: FsFileNode[];
  loading: boolean;
  error: string | null;
  expandedPaths: Set<string>;
  selectedPath: string | null;
  onToggle: (path: string) => void;
  onSelect: (node: FsFileNode) => void;
  onRefresh: () => void;
}> = ({ tree, loading, error, expandedPaths, selectedPath, onToggle, onSelect, onRefresh }) => {
  return (
    <div className="flex flex-col h-full text-[13px]">
      <div className="flex items-center justify-between px-3 py-2 border-b border-white/10">
        <span className="text-[11px] font-semibold text-neutral-400 uppercase tracking-wide">Explorer</span>
        <button onClick={onRefresh} className="text-[11px] text-neutral-400 hover:text-neutral-200 transition-colors px-2 py-0.5 rounded hover:bg-white/5">Refresh</button>
      </div>
      <div className="flex-1 overflow-y-auto overflow-x-hidden py-1">
        {loading ? (
          <div className="flex items-center justify-center py-8 text-neutral-500 text-[12px]">Loading...</div>
        ) : error ? (
          <div className="px-3 py-4 text-red-400 text-[12px]">{error}</div>
        ) : tree.length === 0 ? (
          <div className="px-3 py-4 text-neutral-500 text-[12px]">
            <p className="mb-2">No files loaded</p>
            <button onClick={onRefresh} className="text-blue-400 hover:underline">Load project files</button>
          </div>
        ) : (
          tree.filter(n => !n.is_dir || !IGNORED_DIRS.has(n.name)).map(node => (
            <TreeNode key={node.path} node={node} depth={0} selectedPath={selectedPath} onSelect={onSelect} expandedPaths={expandedPaths} onToggle={onToggle} />
          ))
        )}
      </div>
    </div>
  );
};

const TreeNode: React.FC<{
  node: FsFileNode; depth: number; selectedPath: string | null;
  onSelect: (n: FsFileNode) => void; expandedPaths: Set<string>; onToggle: (p: string) => void;
}> = ({ node, depth, selectedPath, onSelect, expandedPaths, onToggle }) => {
  const isExpanded = expandedPaths.has(node.path);
  const isSelected = selectedPath === node.path;
  const hasChildren = node.is_dir && node.children && node.children.length > 0;
  const ext = node.name.split('.').pop()?.toLowerCase() || '';
  const iconColor = !node.is_dir ? (ext === 'tsx' || ext === 'ts' ? 'text-[#3178C6]' : ext === 'jsx' || ext === 'js' ? 'text-[#F7DF1E]' : ext === 'rs' ? 'text-[#DEA584]' : ext === 'py' ? 'text-[#3572A5]' : ext === 'go' ? 'text-[#00ADD8]' : ext === 'json' ? 'text-[#F0DB4F]' : ext === 'css' ? 'text-[#2965F1]' : ext === 'md' ? 'text-[#42A5F5]' : 'text-neutral-400') : 'text-[#DCB67A]';

  return (
    <div>
      <div
        className={`flex items-center gap-1 py-0.5 pr-2 rounded cursor-pointer select-none transition-colors ${isSelected ? 'bg-blue-500/20 text-blue-400' : 'hover:bg-white/5 text-neutral-300'}`}
        style={{ paddingLeft: `${depth * 16 + 4}px` }}
        onClick={(e) => { e.stopPropagation(); node.is_dir ? onToggle(node.path) : onSelect(node); }}
      >
        <span className="w-4 h-4 flex-shrink-0 flex items-center justify-center">
          {node.is_dir ? (hasChildren ? (isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />) : null) : null}
        </span>
        <span className={`text-[11px] ${iconColor} flex-shrink-0 font-mono font-bold w-5 text-center`}>
          {node.is_dir ? '📁' : (ext === 'tsx' || ext === 'ts' ? 'TS' : ext === 'jsx' || ext === 'js' ? 'JS' : ext?.toUpperCase().slice(0, 2) || '📄')}
        </span>
        <span className="text-[12px] truncate leading-none">{node.name}</span>
      </div>
      {node.is_dir && isExpanded && node.children && (
        <div>
          {node.children.filter(c => !c.is_dir || !IGNORED_DIRS.has(c.name)).map(c => (
            <TreeNode key={c.path} node={c} depth={depth + 1} selectedPath={selectedPath} onSelect={onSelect} expandedPaths={expandedPaths} onToggle={onToggle} />
          ))}
        </div>
      )}
    </div>
  );
};

/* ── Tab content renderer ── */
const TabContent: React.FC<{ doc: DocumentInfo; isDark: boolean }> = ({ doc, isDark }) => {
  const { fmt, ext, isBinary, isMarkdown, isHtml, isReact, isCode } = detectFormat(doc);
  const isRenderable = isHtml || isReact;
  const hasContent = !!doc.content;
  const [viewMode, setViewMode] = useState<'preview' | 'code'>(isCode ? 'code' : 'preview');

  if (isBinary) {
    if (fmt === 'pptx') return <SlidePreview slides={doc.slides || []} title={doc.title} colorScheme={doc.colorScheme} />;
    if (fmt === 'docx') return <DocxPreview content={doc.content || ''} title={doc.title} />;
    if (fmt === 'pdf') return <PdfPreview sections={doc.sections || []} title={doc.title} />;
  }

  if (!hasContent) {
    return (
      <div className="flex items-center justify-center h-full text-neutral-500 text-[13px]">
        <p>Loading file content...</p>
      </div>
    );
  }

  return (
    <div className={`flex-1 overflow-y-auto ${isCode || viewMode === 'code' ? '!p-0 overflow-hidden bg-[#FAFAFA] dark:bg-[#1E1E1E]' : viewMode === 'preview' && isRenderable ? '!p-0 !overflow-hidden' : 'px-8 py-6'}`}>
      {/* Toggle for markdown/renderable */}
      {(isMarkdown || isRenderable) && (
        <div className="flex items-center gap-2 px-4 py-2 border-b border-claude-border">
          <div className="flex bg-claude-btnHover rounded-lg p-0.5">
            <button onClick={() => setViewMode('preview')} className={`p-1.5 rounded-md transition-all ${viewMode === 'preview' ? 'bg-white dark:bg-[#555] shadow-sm text-claude-text' : 'text-claude-textSecondary hover:text-claude-text'}`}><Eye size={16} /></button>
            <button onClick={() => setViewMode('code')} className={`p-1.5 rounded-md transition-all ${viewMode === 'code' ? 'bg-white dark:bg-[#555] shadow-sm text-claude-text' : 'text-claude-textSecondary hover:text-claude-text'}`}><Code size={16} /></button>
          </div>
        </div>
      )}

      {viewMode === 'preview' && isRenderable ? (
        <ArtifactPreview content={doc.content || ''} type={isReact ? 'application/vnd.ant.react' : 'text/html'} />
      ) : viewMode === 'preview' && isMarkdown ? (
        <MarkdownRenderer content={doc.content || ''} />
      ) : (
        <div className="flex h-full font-mono text-[13px] leading-relaxed relative bg-[#FAFAFA] dark:bg-[#1E1E1E] overflow-hidden">
          <div className="flex-1 overflow-auto">
            <div className="flex min-h-full">
              <div className="flex-none w-[40px] bg-[#FAFAFA] dark:bg-[#1E1E1E] text-right pt-4 pr-2 select-none text-claude-textSecondary opacity-50 sticky left-0">
                {(doc.content || '').split('\n').map((_: string, i: number) => (
                  <div key={i} style={{ lineHeight: '1.625' }}>{i + 1}</div>
                ))}
              </div>
              <div className="flex-1 min-w-0">
                <SyntaxHighlighter
                  language={isCode ? fmt : 'markdown'}
                  style={isDark ? vscDarkPlus : oneLight}
                  customStyle={{ margin: 0, padding: '16px 16px 16px 8px', background: 'transparent', fontSize: '14px', fontFamily: 'Menlo, Monaco, SF Mono, Cascadia Code, Fira Code, Consolas, Courier New, monospace', lineHeight: '1.625', border: 'none', boxShadow: 'none', minHeight: '100%' }}
                  codeTagProps={{ style: { fontFamily: "inherit" } }}
                >
                  {doc.content || ''}
                </SyntaxHighlighter>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

/* ── Main CodePanel ── */
interface CodePanelProps {
  document: DocumentInfo;
  onClose: () => void;
}

const CodePanel: React.FC<CodePanelProps> = ({ document: initialDoc, onClose }) => {
  const [tabs, setTabs] = useState<DocumentInfo[]>([initialDoc]);
  const [activeTabId, setActiveTabId] = useState(initialDoc.id);
  const [showFileTree, setShowFileTree] = useState(true);
  const [tree, setTree] = useState<FsFileNode[]>([]);
  const [treeLoading, setTreeLoading] = useState(true);
  const [treeError, setTreeError] = useState<string | null>(null);
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [showCopyMenu, setShowCopyMenu] = useState(false);
  const copyMenuRef = useRef<HTMLDivElement>(null);
  const copyBtnRef = useRef<HTMLButtonElement>(null);
  const [isDark, setIsDark] = useState(() => {
    if (typeof window === 'undefined') return false;
    return window.document.documentElement.classList.contains('dark');
  });

  // Add initialDoc when it changes from outside
  useEffect(() => {
    setTabs(prev => {
      if (prev.some(t => t.id === initialDoc.id)) {
        return prev.map(t => t.id === initialDoc.id ? initialDoc : t);
      }
      return [...prev, initialDoc];
    });
    setActiveTabId(initialDoc.id);
  }, [initialDoc.id, initialDoc.content, initialDoc.title]);

  useEffect(() => {
    const checkDark = () => setIsDark(window.document.documentElement.classList.contains('dark'));
    checkDark();
    const observer = new MutationObserver(checkDark);
    observer.observe(window.document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (copyMenuRef.current && !copyMenuRef.current.contains(event.target as Node) && copyBtnRef.current && !copyBtnRef.current.contains(event.target as Node)) {
        setShowCopyMenu(false);
      }
    };
    if (showCopyMenu) document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showCopyMenu]);

  const activeTab = tabs.find(t => t.id === activeTabId) || tabs[0];

  const loadTree = useCallback(async (dirPath?: string) => {
    setTreeLoading(true);
    setTreeError(null);
    try {
      const res = await getFileSystemTree(dirPath);
      setTree(res.tree);
      if (res.tree.length > 0) {
        const firstDir = res.tree.find(n => n.is_dir);
        if (firstDir) setExpandedPaths(new Set([firstDir.path]));
      }
    } catch (e: unknown) {
      setTreeError(getErrorMessage(e) || 'Failed to load file tree');
    } finally {
      setTreeLoading(false);
    }
  }, []);

  useEffect(() => { loadTree(); }, [loadTree]);

  const handleFileSelect = async (node: FsFileNode) => {
    if (node.is_dir) return;
    setSelectedPath(node.path);
    const doc = fsNodeToDoc(node);

    // Check if already open
    const existing = tabs.find(t => t.id === doc.id);
    if (existing) {
      setActiveTabId(existing.id);
      return;
    }

    // Add tab with loading content
    setTabs(prev => [...prev, { ...doc, content: '' }]);
    setActiveTabId(doc.id);

    // Load content
    try {
      const res = await readFileContent(node.path);
      setTabs(prev => prev.map(t => t.id === doc.id ? { ...t, content: res.content } : t));
    } catch {
      setTabs(prev => prev.map(t => t.id === doc.id ? { ...t, content: 'Error loading file' } : t));
    }
  };

  const handleCloseTab = (tabId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setTabs(prev => {
      const next = prev.filter(t => t.id !== tabId);
      if (next.length === 0) {
        onClose();
        return prev;
      }
      if (activeTabId === tabId) {
        const idx = prev.findIndex(t => t.id === tabId);
        const newActive = next[Math.min(idx, next.length - 1)];
        setActiveTabId(newActive.id);
      }
      return next;
    });
  };

  const handleCopyContent = () => {
    if (activeTab?.content) {
      copyToClipboard(activeTab.content).then(success => {
        if (success) { setCopied(true); setTimeout(() => setCopied(false), 2000); }
      });
    }
  };

  const handleDownload = async () => {
    setShowCopyMenu(false);
    if (!activeTab) return;
    const { fmt } = detectFormat(activeTab);
    if (!BINARY_FORMATS.includes(fmt)) {
      const ext = LANG_TO_EXT[fmt] || fmt;
      const blob = new Blob([activeTab.content || ''], { type: 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const a = window.document.createElement('a');
      a.href = url;
      a.download = activeTab.title.includes('.') ? activeTab.title : `${activeTab.title}.${ext}`;
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  return (
    <div className="flex-1 h-full flex bg-claude-input border-l border-claude-border min-w-0">
      {/* File Tree Sidebar */}
      {showFileTree && (
        <div className="w-[220px] flex-shrink-0 border-r border-claude-border bg-[#1E1E2E] flex flex-col">
          <FileTree
            tree={tree}
            loading={treeLoading}
            error={treeError}
            expandedPaths={expandedPaths}
            selectedPath={selectedPath}
            onToggle={(path) => setExpandedPaths(prev => { const next = new Set(prev); if (next.has(path)) next.delete(path); else next.add(path); return next; })}
            onSelect={handleFileSelect}
            onRefresh={() => loadTree()}
          />
        </div>
      )}

      {/* Main area */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Top bar: sidebar toggle + tab bar + actions */}
        <div className="flex items-center border-b border-claude-border flex-shrink-0 bg-claude-bg">
          <button
            onClick={() => setShowFileTree(!showFileTree)}
            className="p-2 text-claude-textSecondary hover:text-claude-text hover:bg-claude-btnHover transition-colors flex-shrink-0"
            title={showFileTree ? 'Hide file tree' : 'Show file tree'}
          >
            {showFileTree ? <PanelLeftClose size={16} /> : <PanelLeftOpen size={16} />}
          </button>

          {/* Tab bar */}
          <div className="flex-1 flex items-center overflow-x-auto scrollbar-thin min-w-0" style={{ scrollbarWidth: 'thin' }}>
            {tabs.map(tab => (
              <div
                key={tab.id}
                onClick={() => setActiveTabId(tab.id)}
                className={`group flex items-center gap-1.5 px-3 py-2 text-[13px] cursor-pointer border-r border-claude-border transition-colors flex-shrink-0 max-w-[180px] ${activeTabId === tab.id ? 'bg-claude-input text-claude-text border-b-2 border-b-blue-500' : 'text-claude-textSecondary hover:bg-claude-hover'}`}
              >
                <span className="truncate text-[12px]">{tab.title}</span>
                <button
                  onClick={(e) => handleCloseTab(tab.id, e)}
                  className="p-0.5 rounded hover:bg-claude-btnHover opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
                >
                  <X size={12} />
                </button>
              </div>
            ))}
          </div>

          {/* Actions */}
          <div className="flex items-center gap-1 px-2 flex-shrink-0">
            <div className="relative flex items-center">
              <button onClick={handleCopyContent} className="h-7 flex items-center px-2 text-[12px] font-medium text-claude-text border border-claude-border border-r-0 rounded-l-lg hover:bg-claude-btnHover transition-colors">
                {copied ? <Check size={14} className="mr-1" /> : <Copy size={14} className="mr-1" />}
                {copied ? 'Copied' : 'Copy'}
              </button>
              <button ref={copyBtnRef} onClick={() => setShowCopyMenu(!showCopyMenu)} className="h-7 px-1.5 flex items-center justify-center border border-claude-border rounded-r-lg hover:bg-claude-btnHover transition-colors text-claude-text">
                <ChevronDown size={14} />
              </button>
              {showCopyMenu && (
                <div ref={copyMenuRef} className="absolute top-full right-0 mt-1 w-40 bg-white dark:bg-claude-input border border-claude-border rounded-lg shadow-lg py-1 z-50">
                  <button onClick={handleDownload} className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-btnHover transition-colors">Download</button>
                </div>
              )}
            </div>
            <button onClick={onClose} className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-btnHover rounded-lg transition-colors" title="Close panel">
              <X size={18} />
            </button>
          </div>
        </div>

        {/* Tab content */}
        {activeTab && <TabContent doc={activeTab} isDark={isDark} />}
      </div>
    </div>
  );
};

export default CodePanel;
