import React, { useState, useRef, useCallback, useEffect } from 'react';
import {
  FolderOpen, File, FileText, Plus, X, Terminal, Play, Hammer,
  CheckCircle2, Loader2, ChevronRight, ChevronDown, Save, GitBranch,
  Search, Settings, PanelLeftClose, PanelLeftOpen, Folder,
} from 'lucide-react';
import { getFileSystemTree, readFileContent, writeFileContent, type FsFileNode } from '../api';
import { getErrorMessage } from '../utils/errorHelpers';

const CODE_EXT = new Set(['ts','tsx','js','jsx','rs','py','go','java','c','cpp','h','css','scss','html','json','yaml','yml','toml','md','sh','bash','sql','xml','svg']);
const IGNORE_DIRS = new Set(['.git','node_modules','dist','build','.next','.nuxt','__pycache__','.venv','target','.cargo']);

/* ── File Tree ── */
function FileTree({ tree, onSelect, selectedPath, expandedPaths, onToggle }:
  { tree: FsFileNode[]; onSelect: (n: FsFileNode) => void; selectedPath: string | null;
    expandedPaths: Set<string>; onToggle: (p: string) => void }) {
  return (
    <div className="flex-1 overflow-y-auto overflow-x-hidden py-1 text-[13px]">
      {tree.filter(n => !n.is_dir || !IGNORE_DIRS.has(n.name)).map(node => (
        <TreeNode key={node.path} node={node} depth={0}
          selectedPath={selectedPath} onSelect={onSelect}
          expandedPaths={expandedPaths} onToggle={onToggle} />
      ))}
    </div>
  );
}

function TreeNode({ node, depth, selectedPath, onSelect, expandedPaths, onToggle }:
  { node: FsFileNode; depth: number; selectedPath: string | null;
    onSelect: (n: FsFileNode) => void; expandedPaths: Set<string>; onToggle: (p: string) => void }) {
  const isExpanded = expandedPaths.has(node.path);
  const isSelected = selectedPath === node.path;
  const ext = node.name.split('.').pop()?.toLowerCase() || '';
  const iconColor = !node.is_dir ? (
    ext === 'tsx'||ext==='ts' ? 'text-[#3178C6]' : ext==='jsx'||ext==='js' ? 'text-[#F7DF1E]' :
    ext==='rs' ? 'text-[#DEA584]' : ext==='py' ? 'text-[#3572A5]' : ext==='go' ? 'text-[#00ADD8]' :
    ext==='json' ? 'text-[#F0DB4F]' : ext==='css' ? 'text-[#2965F1]' : ext==='md' ? 'text-[#42A5F5]' :
    'text-neutral-400') : 'text-[#DCB67A]';

  return (
    <div>
      <div className={`flex items-center gap-1 py-0.5 pr-2 rounded cursor-pointer select-none transition-colors
        ${isSelected ? 'bg-blue-500/15 text-blue-400' : 'hover:bg-white/5 text-neutral-300'}`}
        style={{ paddingLeft: `${depth * 16 + 4}px` }}
        onClick={(e) => { e.stopPropagation(); node.is_dir ? onToggle(node.path) : onSelect(node); }}>
        <span className="w-4 h-4 flex items-center justify-center flex-shrink-0">
          {node.is_dir && (isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />)}
        </span>
        <span className={`text-[11px] ${iconColor} flex-shrink-0 font-mono font-bold w-5 text-center`}>
          {node.is_dir ? '📁' : ext?.toUpperCase().slice(0,2) || '📄'}
        </span>
        <span className="text-[12px] truncate leading-none">{node.name}</span>
      </div>
      {node.is_dir && isExpanded && node.children && (
        <div>
          {node.children.filter(c => !c.is_dir || !IGNORE_DIRS.has(c.name)).map(c => (
            <TreeNode key={c.path} node={c} depth={depth+1}
              selectedPath={selectedPath} onSelect={onSelect}
              expandedPaths={expandedPaths} onToggle={onToggle} />
          ))}
        </div>
      )}
    </div>
  );
}

/* ── Tab Interfaces ── */
interface CodeTab {
  id: string;
  title: string;
  path: string;
  language: string;
  content: string;
  originalContent: string;
  isDirty: boolean;
}

/* ── Main Component ── */
const CodeWorkspace: React.FC = () => {
  const [currentPath, setCurrentPath] = useState('');
  const [tree, setTree] = useState<FsFileNode[]>([]);
  const [treeLoading, setTreeLoading] = useState(false);
  const [tabs, setTabs] = useState<CodeTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [showSidebar, setShowSidebar] = useState(true);
  const [saving, setSaving] = useState<string | null>(null);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const activeTabIdRef = useRef<string | null>(null);

  const activeTab = tabs.find(t => t.id === activeTabId) || null;

  // Keep ref in sync
  useEffect(() => { activeTabIdRef.current = activeTabId; }, [activeTabId]);

  const loadTree = useCallback(async (dir?: string) => {
    setTreeLoading(true);
    try {
      const res = await getFileSystemTree(dir);
      setTree(res.tree);
      setCurrentPath(res.path);
    } catch (e: unknown) { setStatusMsg(`Load failed: ${getErrorMessage(e)}`);
    } finally { setTreeLoading(false); }
  }, []);

  useEffect(() => { loadTree(); }, [loadTree]);

  const handleFileSelect = useCallback(async (node: FsFileNode) => {
    if (node.is_dir) return;
    setSelectedPath(node.path);
    const tabId = `tab-${node.path}`;
    const ext = node.name.split('.').pop()?.toLowerCase() || '';
    const langMap: Record<string, string> = { ts:'typescript',tsx:'tsx',js:'javascript',jsx:'jsx',rs:'rust',py:'python',go:'go',java:'java',css:'css',html:'html',json:'json',md:'markdown',yaml:'yaml',yml:'yaml',sh:'bash',bash:'bash',sql:'sql',xml:'xml',toml:'toml' };
    const language = langMap[ext] || 'text';

    if (tabs.some(t => t.id === tabId)) {
      setActiveTabId(tabId);
      return;
    }

    setTabs(prev => [...prev, { id: tabId, title: node.name, path: node.path, language, content: '加载中...', originalContent: '', isDirty: false }]);
    setActiveTabId(tabId);

    try {
      const res = await readFileContent(node.path);
      setTabs(prev => prev.map(t => t.id === tabId ? { ...t, content: res.content, originalContent: res.content } : t));
    } catch { setTabs(prev => prev.map(t => t.id === tabId ? { ...t, content: 'Error loading file' } : t)); }
  }, [tabs]);

  const handleEditorChange = (tabId: string, newContent: string) => {
    setTabs(prev => prev.map(t => t.id === tabId ? { ...t, content: newContent, isDirty: newContent !== t.originalContent } : t));
  };

  const handleSave = async (tabId: string) => {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab || !tab.isDirty) return;
    setSaving(tabId);
    try {
      await writeFileContent(tab.path, tab.content);
      setTabs(prev => prev.map(t => t.id === tabId ? { ...t, originalContent: t.content, isDirty: false } : t));
      setStatusMsg('Saved');
      setTimeout(() => setStatusMsg(null), 2000);
    } catch (e: unknown) { setStatusMsg(`Save failed: ${getErrorMessage(e)}`);
    } finally { setSaving(null); }
  };

  const closeTab = (tabId: string) => {
    const currentActive = activeTabIdRef.current;
    setTabs(prev => {
      const idx = prev.findIndex(t => t.id === tabId);
      const next = prev.filter(t => t.id !== tabId);
      if (currentActive === tabId && next.length > 0) {
        const newIdx = Math.min(idx, next.length - 1);
        setActiveTabId(next[newIdx].id);
      } else if (next.length === 0) {
        setActiveTabId(null);
      }
      return next;
    });
  };

  const handleQuickAction = (action: 'run' | 'build' | 'test') => {
    const labels = { run: 'Run', build: 'Build', test: 'Test' };
    setStatusMsg(`${labels[action]} triggered (terminal output will appear here)`);
    setTimeout(() => setStatusMsg(null), 3000);
  };

  const toggleDir = (path: string) => {
    setExpandedPaths(prev => { const n = new Set(prev); n.has(path) ? n.delete(path) : n.add(path); return n; });
  };

  const activeContent = activeTab?.content || '';

  return (
    <div className="h-full flex flex-col bg-[#1e1e1e] text-[#cccccc]">
      {/* ── Top Bar ── */}
      <div className="flex items-center justify-between px-3 py-1 bg-[#2d2d2d] border-b border-[#3c3c3c] shrink-0">
        <div className="flex items-center gap-2">
          <button onClick={() => setShowSidebar(!showSidebar)} className="p-1 hover:bg-[#3c3c3c] rounded text-[#888] hover:text-white transition-colors">
            {showSidebar ? <PanelLeftClose size={14} /> : <PanelLeftOpen size={14} />}
          </button>
          <span className="text-[12px] font-medium text-[#aaa]">{currentPath ? currentPath.split('/').pop() || currentPath.split('\\').pop() : 'No folder opened'}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <button onClick={() => handleQuickAction('run')} className="flex items-center gap-1 px-2.5 py-1 text-[11px] bg-[#2ea043] hover:bg-[#2ea043]/80 text-white rounded transition-colors"><Play size={11} /> Run</button>
          <button onClick={() => handleQuickAction('build')} className="flex items-center gap-1 px-2.5 py-1 text-[11px] bg-[#1f6feb] hover:bg-[#1f6feb]/80 text-white rounded transition-colors"><Hammer size={11} /> Build</button>
          <button onClick={() => handleQuickAction('test')} className="flex items-center gap-1 px-2.5 py-1 text-[11px] bg-[#8957e5] hover:bg-[#8957e5]/80 text-white rounded transition-colors"><CheckCircle2 size={11} /> Test</button>
          <div className="w-px h-4 bg-[#3c3c3c] mx-1" />
          <button onClick={() => loadTree()} className="p-1 hover:bg-[#3c3c3c] rounded text-[#888] hover:text-white" title="Refresh"><Folder size={13} /></button>
          <button className="p-1 hover:bg-[#3c3c3c] rounded text-[#888] hover:text-white" title="Search"><Search size={13} /></button>
          <button className="p-1 hover:bg-[#3c3c3c] rounded text-[#888] hover:text-white" title="Settings"><Settings size={13} /></button>
        </div>
      </div>

      {/* ── Main Area ── */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar: file tree */}
        {showSidebar && (
          <div className="w-[220px] shrink-0 border-r border-[#3c3c3c] bg-[#252526] flex flex-col">
            <div className="flex items-center justify-between px-3 py-2 border-b border-[#3c3c3c]">
              <span className="text-[11px] font-semibold text-[#888] uppercase tracking-wide">Explorer</span>
              <button onClick={() => loadTree()} className="text-[#888] hover:text-white text-[11px]">Refresh</button>
            </div>
            {treeLoading ? (
              <div className="flex items-center justify-center py-8 text-[#888] text-[12px]"><Loader2 size={14} className="animate-spin mr-2" />Loading...</div>
            ) : tree.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-8 px-4 text-center">
                <FolderOpen size={28} className="text-[#555] mb-3" />
                <p className="text-[12px] text-[#888] mb-2">No project opened</p>
                <p className="text-[11px] text-[#666] mb-4">Use the file API to load a project</p>
                <button onClick={() => loadTree()} className="text-[11px] text-blue-400 hover:underline">Load current directory</button>
              </div>
            ) : (
              <FileTree tree={tree} onSelect={handleFileSelect} selectedPath={selectedPath}
                expandedPaths={expandedPaths} onToggle={toggleDir} />
            )}
          </div>
        )}

        {/* Editor area */}
        <div className="flex-1 flex flex-col min-w-0">
          {/* Tab bar */}
          <div className="flex items-center bg-[#252526] border-b border-[#3c3c3c] overflow-x-auto shrink-0" style={{ scrollbarWidth: 'thin' }}>
            {tabs.length === 0 ? (
              <div className="px-3 py-2 text-[12px] text-[#666]">No files opened</div>
            ) : tabs.map(tab => (
              <div key={tab.id}
                className={`group flex items-center gap-1.5 px-3 py-1.5 text-[12px] cursor-pointer border-r border-[#3c3c3c] shrink-0
                  ${activeTabId === tab.id ? 'bg-[#1e1e1e] text-white border-b-2 border-b-[#1f6feb]' : 'bg-[#2d2d2d] text-[#888] hover:text-white'}`}
                onClick={() => setActiveTabId(tab.id)}>
                <FileText size={12} className="text-blue-400" />
                <span className="truncate max-w-[120px]">{tab.title}</span>
                {tab.isDirty && <span className="text-[#e2b714] text-[10px]">●</span>}
                <button onClick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
                  className="p-0.5 rounded hover:bg-[#3c3c3c] opacity-0 group-hover:opacity-100 transition-opacity"><X size={10} /></button>
              </div>
            ))}
          </div>

          {/* Editor / Welcome */}
          {activeTab ? (
            <div className="flex-1 flex flex-col">
              <textarea
                value={activeTab.content}
                onChange={(e) => handleEditorChange(activeTab.id, e.target.value)}
                className="flex-1 bg-[#1e1e1e] text-[13px] text-[#d4d4d4] p-4 font-mono resize-none outline-none leading-relaxed border-0"
                spellCheck={false}
              />
              <div className="flex items-center justify-between px-4 py-1.5 bg-[#007acc] text-[11px] text-white shrink-0">
                <span>{activeTab.language}</span>
                <div className="flex items-center gap-3">
                  {activeTab.isDirty && <span>未保存的更改</span>}
                  <button onClick={() => handleSave(activeTab.id)} disabled={!activeTab.isDirty || saving === activeTab.id}
                    className="flex items-center gap-1 px-2 py-0.5 bg-white/20 hover:bg-white/30 rounded disabled:opacity-50 transition-colors">
                    {saving === activeTab.id ? <Loader2 size={11} className="animate-spin" /> : <Save size={11} />} Save
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center bg-[#1e1e1e]">
              <div className="text-center max-w-[400px]">
                <FolderOpen size={48} className="mx-auto mb-4 text-[#333]" />
                <p className="text-[15px] text-[#888] mb-1">Code Workspace</p>
                <p className="text-[12px] text-[#666] mb-6">Select a file from the explorer to start editing.<br />Open a folder to browse project files.</p>
                <button onClick={() => loadTree()}
                  className="px-4 py-2 bg-[#2ea043] hover:bg-[#2ea043]/80 text-white text-[13px] rounded-lg transition-colors inline-flex items-center gap-2">
                  <FolderOpen size={14} /> Load Project Files
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* ── Status Bar ── */}
      <div className="flex items-center justify-between px-4 py-1 bg-[#007acc] text-[11px] text-white shrink-0">
        <div className="flex items-center gap-3">
          {statusMsg && (
            <span className="flex items-center gap-1">{statusMsg}</span>
          )}
        </div>
        <div className="flex items-center gap-3">
          {activeTab && (
            <>
              <span>Ln {activeContent.split('\n').length}</span>
              <span>{activeTab.language}</span>
            </>
          )}
          <span className="flex items-center gap-1"><GitBranch size={10} /> main</span>
        </div>
      </div>
    </div>
  );
};

export default CodeWorkspace;
