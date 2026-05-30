import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Search, Plus, Trash2, FileText, GitBranch, Upload, X, Loader2, Eye, Edit3, Network, List } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

/* ── Types ── */
interface KnowledgeNode {
  id: string; title: string; content: string; tags: string[];
  links: string[]; backlinks: string[]; created_at: string; updated_at: string;
  source: string; file_path: string | null;
}
interface GraphEdge { source: string; target: string; weight: number; }
interface KnowledgeGraph { nodes: KnowledgeNode[]; edges: GraphEdge[]; }

/* ── API ── */
const kb = {
  list: () => invoke<KnowledgeNode[]>('kb_list'),
  get: (id: string) => invoke<KnowledgeNode | null>('kb_get', { id }),
  search: (query: string) => invoke<KnowledgeNode[]>('kb_search', { query }),
  add: (node: KnowledgeNode) => invoke<void>('kb_add', { node }),
  delete: (id: string) => invoke<void>('kb_delete', { id }),
  graph: () => invoke<KnowledgeGraph>('kb_graph'),
  import: (filePath: string, content: string) => invoke<string>('kb_import', { filePath, content }),
};

/* ── Force-Directed Graph (Canvas) ── */
interface GraphNode extends KnowledgeNode { x: number; y: number; vx: number; vy: number; }

function runForceSimulation(graphNodes: KnowledgeNode[], edges: GraphEdge[]): GraphNode[] {
  const nodes: GraphNode[] = graphNodes.map(n => ({ ...n, x: Math.random() * 500, y: Math.random() * 400, vx: 0, vy: 0 }));
  const idMap = new Map(nodes.map(n => [n.id, n]));
  const center = { x: 250, y: 200 };

  for (let iter = 0; iter < 100; iter++) {
    // Center gravity
    for (const n of nodes) { n.vx += (center.x - n.x) * 0.01; n.vy += (center.y - n.y) * 0.01; }
    // Repulsion
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const dx = nodes[j].x - nodes[i].x, dy = nodes[j].y - nodes[i].y;
        const dist = Math.max(Math.sqrt(dx * dx + dy * dy), 10);
        const force = 500 / (dist * dist);
        nodes[i].vx -= force * dx / dist; nodes[i].vy -= force * dy / dist;
        nodes[j].vx += force * dx / dist; nodes[j].vy += force * dy / dist;
      }
    }
    // Attraction along edges
    for (const e of edges) {
      const s = idMap.get(e.source), t = idMap.get(e.target);
      if (!s || !t) continue;
      const dx = t.x - s.x, dy = t.y - s.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const force = (dist - 80) * 0.005;
      s.vx += force * dx / dist; s.vy += force * dy / dist;
      t.vx -= force * dx / dist; t.vy -= force * dy / dist;
    }
    // Apply velocity + damping
    for (const n of nodes) { n.x += n.vx; n.y += n.vy; n.vx *= 0.85; n.vy *= 0.85; }
  }
  return nodes;
}

/* ── Component ── */
const KnowledgePanel: React.FC<{ onClose?: () => void }> = ({ onClose }) => {
  const [nodes, setNodes] = useState<KnowledgeNode[]>([]);
  const [graph, setGraph] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [viewMode, setViewMode] = useState<'graph' | 'list'>('graph');
  const [editorOpen, setEditorOpen] = useState(false);
  const [editTitle, setEditTitle] = useState('');
  const [editContent, setEditContent] = useState('');
  const [editTags, setEditTags] = useState('');
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [importText, setImportText] = useState('');
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [allNodes, g] = await Promise.all([kb.list(), kb.graph()]);
      setNodes(allNodes);
      setEdges(g.edges);
      setGraph(runForceSimulation(g.nodes, g.edges));
    } catch (e) { console.error('KB load error:', e); }
    setLoading(false);
  }, []);

  useEffect(() => { loadData(); }, [loadData]);

  // Draw graph on canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || graph.length === 0) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = canvas.clientWidth * dpr;
    canvas.height = canvas.clientHeight * dpr;
    ctx.scale(dpr, dpr);
    const w = canvas.clientWidth, h = canvas.clientHeight;

    ctx.clearRect(0, 0, w, h);

    // Draw edges
    const nodeMap = new Map(graph.map(n => [n.id, n]));
    for (const e of edges) {
      const s = nodeMap.get(e.source), t = nodeMap.get(e.target);
      if (!s || !t) continue;
      ctx.strokeStyle = (hoveredId && (e.source === hoveredId || e.target === hoveredId))
        ? 'rgba(204,124,94,0.6)' : 'rgba(150,150,150,0.2)';
      ctx.lineWidth = (hoveredId && (e.source === hoveredId || e.target === hoveredId)) ? 2 : 1;
      ctx.beginPath();
      ctx.moveTo(s.x + w / 2, s.y + h / 2);
      ctx.lineTo(t.x + w / 2, t.y + h / 2);
      ctx.stroke();
    }

    // Draw nodes
    for (const n of graph) {
      const x = n.x + w / 2, y = n.y + h / 2;
      const isHover = hoveredId === n.id;
      const isSelected = selectedId === n.id;
      const radius = isHover ? 14 : (isSelected ? 12 : 8);
      const color = isHover || isSelected ? '#CC7C5E' : (n.source === 'chat_auto' ? '#89b4fa' : '#a6e3a1');

      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
      if (isSelected) { ctx.strokeStyle = '#CC7C5E'; ctx.lineWidth = 2; ctx.stroke(); }

      if (isHover || isSelected) {
        ctx.fillStyle = '#fff';
        ctx.font = '11px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText(n.title.length > 20 ? n.title.slice(0, 20) + '...' : n.title, x, y - 18);
      }
    }
  }, [graph, edges, hoveredId, selectedId]);

  const selectedNode = nodes.find(n => n.id === selectedId) || null;

  const handleCreate = async () => {
    const id = crypto.randomUUID();
    const now = new Date().toISOString();
    const node: KnowledgeNode = {
      id, title: 'New Note', content: '', tags: editTags.split(',').map(t => t.trim()).filter(Boolean),
      links: [], backlinks: [], created_at: now, updated_at: now, source: 'manual', file_path: null,
    };
    await kb.add(node);
    setSelectedId(id);
    setEditTitle('New Note');
    setEditContent('');
    setEditorOpen(true);
    await loadData();
  };

  const handleSave = async () => {
    if (!selectedNode) return;
    const now = new Date().toISOString();
    const updated = { ...selectedNode, title: editTitle || selectedNode.title, content: editContent, tags: editTags.split(',').map(t => t.trim()).filter(Boolean), updated_at: now };
    await kb.add(updated);
    setEditorOpen(false);
    await loadData();
  };

  const handleDelete = async (id: string) => {
    await kb.delete(id);
    if (selectedId === id) { setSelectedId(null); setEditorOpen(false); }
    await loadData();
  };

  const handleImport = async () => {
    if (!importText.trim()) return;
    const filePath = `import-${Date.now()}.md`;
    await kb.import(filePath, importText);
    setImportText('');
    await loadData();
  };

  const filteredNodes = searchQuery ? nodes.filter(n =>
    n.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    n.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
    n.tags.some(t => t.toLowerCase().includes(searchQuery.toLowerCase()))
  ) : nodes;

  return (
    <div className="h-full flex flex-col bg-claude-bg">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-claude-border shrink-0">
        <div className="flex items-center gap-2">
          <Network size={16} className="text-purple-400" />
          <span className="text-[13px] font-semibold text-claude-text">知识库</span>
          <span className="text-[10px] text-claude-textSecondary bg-claude-hover px-1.5 py-0.5 rounded-full">{nodes.length}</span>
        </div>
        <div className="flex items-center gap-1">
          <button onClick={() => setViewMode('graph')} className={`p-1.5 rounded ${viewMode === 'graph' ? 'bg-claude-hover text-purple-400' : 'text-claude-textSecondary hover:text-claude-text'}`}><Network size={14} /></button>
          <button onClick={() => setViewMode('list')} className={`p-1.5 rounded ${viewMode === 'list' ? 'bg-claude-hover text-purple-400' : 'text-claude-textSecondary hover:text-claude-text'}`}><List size={14} /></button>
          <button onClick={loadData} className="p-1.5 rounded text-claude-textSecondary hover:text-claude-text"><Loader2 size={14} /></button>
          {onClose && <button onClick={onClose} className="p-1.5 rounded hover:bg-claude-hover text-claude-textSecondary"><X size={14} /></button>}
        </div>
      </div>

      {/* Search */}
      <div className="px-3 py-2 border-b border-claude-border">
        <div className="flex items-center gap-2 bg-claude-hover rounded-lg px-2.5 py-1.5">
          <Search size={13} className="text-claude-textSecondary" />
          <input value={searchQuery} onChange={e => setSearchQuery(e.target.value)}
            placeholder="搜索知识库..." className="flex-1 bg-transparent text-[12px] text-claude-text outline-none placeholder:text-claude-textSecondary/50" />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left: Graph or List */}
        <div className={`${selectedNode ? 'w-1/2' : 'flex-1'} border-r border-claude-border flex flex-col`}>
          {loading ? (
            <div className="flex-1 flex items-center justify-center"><Loader2 size={20} className="animate-spin text-purple-400" /></div>
          ) : viewMode === 'graph' ? (
            <div className="flex-1 relative">
              <canvas ref={canvasRef} className="w-full h-full cursor-pointer"
                onMouseMove={e => {
                  const rect = canvasRef.current?.getBoundingClientRect();
                  if (!rect) return;
                  const mx = e.clientX - rect.left - rect.width / 2, my = e.clientY - rect.top - rect.height / 2;
                  const found = graph.find(n => Math.abs(n.x - mx) < 12 && Math.abs(n.y - my) < 12);
                  setHoveredId(found?.id || null);
                }}
                onClick={() => {
                  if (hoveredId) { setSelectedId(hoveredId);
                    const n = nodes.find(nn => nn.id === hoveredId);
                    if (n) { setEditTitle(n.title); setEditContent(n.content); setEditTags(n.tags.join(', ')); setEditorOpen(false); } }
                }}
              />
              <div className="absolute bottom-2 left-2 text-[10px] text-claude-textSecondary/50">图谱视图 · 悬停查看节点</div>
            </div>
          ) : (
            <div className="flex-1 overflow-y-auto">
              {filteredNodes.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-claude-textSecondary text-[12px] gap-3">
                  <FileText size={32} className="opacity-30" />
                  <p>{searchQuery ? '无匹配结果' : '知识库为空'}</p>
                </div>
              ) : filteredNodes.map(n => (
                <div key={n.id} onClick={() => { setSelectedId(n.id); setEditTitle(n.title); setEditContent(n.content); setEditTags(n.tags.join(', ')); setEditorOpen(false); }}
                  className={`flex items-start gap-2.5 px-3 py-2.5 cursor-pointer hover:bg-claude-hover/50 transition-colors border-b border-claude-border/30 ${selectedId === n.id ? 'bg-claude-hover/70' : ''}`}>
                  <span className="mt-0.5 text-[14px]">{n.source === 'chat_auto' ? '🧠' : '📄'}</span>
                  <div className="flex-1 min-w-0">
                    <div className="text-[12px] font-medium text-claude-text truncate">{n.title}</div>
                    <div className="text-[10px] text-claude-textSecondary truncate mt-0.5">{n.content.slice(0, 80)}</div>
                    {n.tags.length > 0 && <div className="flex gap-1 mt-1 flex-wrap">
                      {n.tags.slice(0, 3).map(t => <span key={t} className="text-[9px] px-1.5 py-0.5 rounded-full bg-purple-500/10 text-purple-400">#{t}</span>)}
                    </div>}
                  </div>
                  <span className="text-[10px] text-claude-textSecondary/50 shrink-0">{n.links.length} ↦</span>
                </div>
              ))}
            </div>
          )}
          {/* Action buttons */}
          <div className="flex items-center gap-1 px-3 py-2 border-t border-claude-border shrink-0">
            <button onClick={handleCreate} className="flex items-center gap-1 px-2.5 py-1.5 text-[11px] bg-purple-500/10 text-purple-400 hover:bg-purple-500/20 rounded-lg transition-colors"><Plus size={12} />新建</button>
            <button onClick={() => setImportText(importText ? '' : 'paste')} className="flex items-center gap-1 px-2.5 py-1.5 text-[11px] bg-claude-hover text-claude-textSecondary hover:text-claude-text rounded-lg transition-colors"><Upload size={12} />导入</button>
          </div>
          {importText === 'paste' && (
            <div className="px-3 py-2 border-t border-claude-border">
              <textarea value={importText === 'paste' ? '' : importText} onChange={e => setImportText(e.target.value)}
                placeholder="粘贴 Markdown 内容导入..." className="w-full h-20 bg-claude-hover text-[11px] text-claude-text p-2 rounded-lg outline-none resize-none" />
              <button onClick={handleImport} className="mt-1 text-[11px] text-blue-400 hover:underline">导入</button>
            </div>
          )}
        </div>

        {/* Right: Detail */}
        {selectedNode && (
          <div className="flex-1 flex flex-col overflow-hidden">
            {editorOpen ? (
              <div className="flex-1 flex flex-col p-3 gap-2 overflow-y-auto">
                <input value={editTitle} onChange={e => setEditTitle(e.target.value)}
                  className="w-full bg-claude-hover text-[14px] font-semibold text-claude-text px-2.5 py-1.5 rounded-lg outline-none" placeholder="标题" />
                <textarea value={editContent} onChange={e => setEditContent(e.target.value)}
                  className="flex-1 bg-claude-hover text-[12px] text-claude-text p-2.5 rounded-lg outline-none resize-none font-mono leading-relaxed" placeholder="内容 (Markdown)" />
                <input value={editTags} onChange={e => setEditTags(e.target.value)}
                  className="w-full bg-claude-hover text-[11px] text-claude-text px-2.5 py-1.5 rounded-lg outline-none" placeholder="标签 (逗号分隔)" />
                <div className="flex gap-2">
                  <button onClick={handleSave} className="px-3 py-1.5 bg-purple-500/20 text-purple-400 text-[11px] rounded-lg hover:bg-purple-500/30">保存</button>
                  <button onClick={() => setEditorOpen(false)} className="px-3 py-1.5 bg-claude-hover text-claude-textSecondary text-[11px] rounded-lg">取消</button>
                </div>
              </div>
            ) : (
              <div className="flex-1 overflow-y-auto p-4">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-[14px] font-semibold text-claude-text">{selectedNode.title}</h3>
                  <div className="flex gap-1">
                    <button onClick={() => setEditorOpen(true)} className="p-1.5 rounded hover:bg-claude-hover text-claude-textSecondary"><Edit3 size={13} /></button>
                    <button onClick={() => handleDelete(selectedNode.id)} className="p-1.5 rounded hover:bg-red-500/10 text-red-400"><Trash2 size={13} /></button>
                  </div>
                </div>
                <div className="flex gap-1 flex-wrap mb-3">
                  {selectedNode.tags.map(t => <span key={t} className="text-[10px] px-2 py-0.5 rounded-full bg-purple-500/10 text-purple-400">#{t}</span>)}
                  <span className="text-[9px] text-claude-textSecondary/50 px-1">{selectedNode.source}</span>
                </div>
                <div className="text-[12px] text-claude-text leading-relaxed whitespace-pre-wrap font-mono">
                  {selectedNode.content || '（空）'}
                </div>
                {selectedNode.links.length > 0 && (
                  <div className="mt-4 pt-3 border-t border-claude-border">
                    <span className="text-[10px] font-semibold text-claude-textSecondary uppercase">链接到</span>
                    <div className="mt-1 flex flex-wrap gap-1">
                      {selectedNode.links.map(l => {
                        const target = nodes.find(n => n.title === l);
                        return <span key={l} className="text-[11px] px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400">{target ? l : `${l} (未创建)`}</span>;
                      })}
                    </div>
                  </div>
                )}
                {selectedNode.backlinks.length > 0 && (
                  <div className="mt-3 pt-3 border-t border-claude-border">
                    <span className="text-[10px] font-semibold text-claude-textSecondary uppercase">被链接</span>
                    <div className="mt-1 flex flex-wrap gap-1">
                      {selectedNode.backlinks.map(b => {
                        const source = nodes.find(n => n.id === b);
                        return source ? <span key={b} className="text-[11px] px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400">{source.title}</span> : null;
                      })}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default KnowledgePanel;
