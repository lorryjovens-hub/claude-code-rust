import React, { useState, useEffect, useCallback } from 'react';
import { Brain, Search, Database, Trash2, Settings, RefreshCw, Clock, Star, Hash, X, Layers, Sliders, BarChart3, MessageSquare, BookOpen } from 'lucide-react';
import { detectBridgePort } from '../api';
import { invoke } from '@tauri-apps/api/core';

interface MemoryItem {
  id: string;
  content: string;
  importance: number;
  created_at: string;
  metadata?: Record<string, string>;
  similarity_score?: number;
}

interface MemoryStats {
  total_memories: number;
  total_tokens_approx: number;
  backend: string;
}

interface MemoryConfig {
  enabled: boolean;
  backend_url: string;
  auto_ingest: boolean;
  auto_search: boolean;
  search_top_k: number;
  compression_threshold_tokens: number;
  min_importance_threshold: number;
}

function getBridgeUrl(port: number): string {
  return `http://127.0.0.1:${port}`;
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
  if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
  return n.toString();
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return '刚刚';
    if (diffMins < 60) return `${diffMins} 分钟前`;
    if (diffHours < 24) return `${diffHours} 小时前`;
    if (diffDays < 7) return `${diffDays} 天前`;
    return date.toLocaleDateString('zh-CN');
  } catch {
    return dateStr;
  }
}

function ImportanceBadge({ importance }: { importance: number }) {
  let color = '#6b7280';
  let label = '低';

  if (importance >= 0.7) {
    color = '#ef4444';
    label = '高';
  } else if (importance >= 0.4) {
    color = '#f59e0b';
    label = '中';
  }

  return (
    <span
      className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium"
      style={{ backgroundColor: color + '20', color }}
    >
      <Star size={10} fill={color} color={color} />
      {label} ({importance.toFixed(2)})
    </span>
  );
}

export function MemoryPanel() {
  const [activeTab, setActiveTab] = useState<'search' | 'stats' | 'config' | 'context'>('search');
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<MemoryItem[]>([]);
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [config, setConfig] = useState<MemoryConfig | null>(null);
  const [cavemanStats, setCavemanStats] = useState<{
    total_segments: number; tokens_saved: number; avg_compression_ratio: number; total_token_importance: number;
  } | null>(null);
  const [contextInfo, setContextInfo] = useState<{ tokens: number; limit: number } | null>(null);
  const [convMemories, setConvMemories] = useState<{ id: string; title: string; importance: number }[]>([]);
  const [loading, setLoading] = useState(false);
  const [showConfig, setShowConfig] = useState(false);
  const [backendHealthy, setBackendHealthy] = useState(false);
  const [bridgePort, setBridgePort] = useState(30085);

  useEffect(() => {
    detectBridgePort().then(port => {
      console.log('[MemoryPanel] Bridge port detected:', port);
      setBridgePort(port);
    });
  }, []);

  const fetchStats = useCallback(async () => {
    try {
      const resp = await fetch(`${getBridgeUrl(bridgePort)}/api/memory/stats`);
      if (resp.ok) {
        const data = await resp.json();
        setStats(data);
        setBackendHealthy(true);
      }
      // Fetch Caveman RTK stats
      const cavResp = await fetch(`${getBridgeUrl(bridgePort)}/api/caveman/stats`);
      if (cavResp.ok) {
        const cavData = await cavResp.json();
        setCavemanStats(cavData);
      }
      // Fetch context info for active conversation
      try {
        const ctx = await invoke<{ tokens: number; limit: number }>('get_context_size', { conversationId: 'active' });
        setContextInfo(ctx);
      } catch {}
      // Fetch conversation memories from KB
      try {
        const nodes = await invoke<{ id: string; title: string; content: string }[]>('kb_list');
        setConvMemories(nodes.map(n => ({ id: n.id, title: n.title, importance: n.content.length > 500 ? 0.8 : 0.3 })));
      } catch {}
    } catch {
      setBackendHealthy(false);
    }
  }, [bridgePort]);

  const fetchConfig = useCallback(async () => {
    try {
      const resp = await fetch(`${getBridgeUrl(bridgePort)}/api/memory/config`);
      if (resp.ok) {
        const data = await resp.json();
        setConfig(data);
      }
    } catch (e) {
      console.error('Failed to fetch config:', e);
    }
  }, [bridgePort]);

  const handleSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    try {
      const resp = await fetch(`${getBridgeUrl(bridgePort)}/api/memory/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query, top_k: 10 }),
      });
      if (resp.ok) {
        const data = await resp.json();
        setResults(data);
      }
    } catch (e) {
      console.error('Search failed:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleClear = async () => {
    if (!confirm('确定要清空所有记忆吗？此操作不可撤销。')) return;
    try {
      await fetch(`${getBridgeUrl(bridgePort)}/api/memory/clear`, { method: 'POST' });
      setResults([]);
      fetchStats();
    } catch (e) {
      console.error('Clear failed:', e);
    }
  };

  const handleSaveConfig = async () => {
    if (!config) return;
    try {
      await fetch(`${getBridgeUrl(bridgePort)}/api/memory/config`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      setShowConfig(false);
    } catch (e) {
      console.error('Update config failed:', e);
    }
  };

  useEffect(() => {
    fetchStats();
    fetchConfig();
  }, [fetchStats, fetchConfig]);

  return (
    <div className="flex flex-col h-full bg-claude-bg">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border">
        <div className="flex items-center gap-2">
          <Brain size={20} className="text-claude-primary" />
          <h2 className="text-sm font-semibold text-claude-text">记忆管理</h2>
          <span className={`w-2 h-2 rounded-full ${backendHealthy ? 'bg-green-500' : 'bg-red-500'}`} />
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={fetchStats}
            className="p-1.5 rounded hover:bg-claude-hover transition-colors"
            title="刷新"
          >
            <RefreshCw size={14} className="text-claude-textSecondary" />
          </button>
          <button
            onClick={() => setShowConfig(!showConfig)}
            className="p-1.5 rounded hover:bg-claude-hover transition-colors"
            title="设置"
          >
            <Settings size={14} className="text-claude-textSecondary" />
          </button>
          <button
            onClick={handleClear}
            className="p-1.5 rounded hover:bg-red-500/10 transition-colors"
            title="清空记忆"
          >
            <Trash2 size={14} className="text-red-500" />
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-claude-border">
        {[
          { key: 'search' as const, label: '搜索', icon: Search },
          { key: 'stats' as const, label: '统计', icon: Database },
        ].map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex-1 flex items-center justify-center gap-1.5 py-2 text-xs font-medium transition-colors ${
              activeTab === tab.key
                ? 'text-claude-primary border-b-2 border-claude-primary'
                : 'text-claude-textSecondary hover:text-claude-text'
            }`}
          >
            <tab.icon size={12} />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {activeTab === 'search' && (
          <div className="p-4 space-y-4">
            {/* Search Input */}
            <div className="flex gap-2">
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                placeholder="搜索记忆..."
                className="flex-1 px-3 py-2 rounded-lg bg-claude-hover text-claude-text text-sm border border-claude-border focus:outline-none focus:border-claude-primary"
              />
              <button
                onClick={handleSearch}
                disabled={loading || !query.trim()}
                className="px-4 py-2 rounded-lg bg-claude-primary text-white text-sm font-medium hover:bg-claude-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {loading ? <RefreshCw size={14} className="animate-spin" /> : '搜索'}
              </button>
            </div>

            {/* Results */}
            {results.length > 0 && (
              <div className="space-y-2">
                <p className="text-xs text-claude-textSecondary">
                  找到 {results.length} 条相关记忆
                </p>
                {results.map((item, index) => (
                  <div
                    key={item.id || index}
                    className="p-3 rounded-lg bg-claude-surface border border-claude-border hover:border-claude-primary/30 transition-colors"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <ImportanceBadge importance={item.importance} />
                      <div className="flex items-center gap-2 text-xs text-claude-textSecondary">
                        <Clock size={10} />
                        {formatDate(item.created_at)}
                        {item.similarity_score && (
                          <span className="text-claude-primary">
                            相似度: {(item.similarity_score * 100).toFixed(0)}%
                          </span>
                        )}
                      </div>
                    </div>
                    <p className="text-sm text-claude-text whitespace-pre-wrap line-clamp-4">
                      {item.content}
                    </p>
                    {item.metadata && Object.keys(item.metadata).length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {Object.entries(item.metadata).map(([key, value]) => (
                          <span
                            key={key}
                            className="px-1.5 py-0.5 rounded text-xs bg-claude-hover text-claude-textSecondary"
                          >
                            {key}: {value}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {results.length === 0 && query && !loading && (
              <div className="text-center py-8 text-claude-textSecondary text-sm">
                未找到相关记忆
              </div>
            )}

            {!query && (
              <div className="text-center py-12 text-claude-textSecondary text-sm">
                <Brain size={32} className="mx-auto mb-3 opacity-30" />
                <p>输入关键词搜索历史记忆</p>
                <p className="text-xs mt-1">AI 会自动检索相关上下文以提供更准确的回答</p>
              </div>
            )}
          </div>
        )}

        {activeTab === 'stats' && stats && (
          <div className="p-4 space-y-4">
            <div className="grid grid-cols-2 gap-3">
              <div className="p-4 rounded-lg bg-claude-surface border border-claude-border">
                <div className="flex items-center gap-2 mb-2">
                  <Database size={16} className="text-claude-primary" />
                  <span className="text-xs text-claude-textSecondary">记忆总数</span>
                </div>
                <p className="text-2xl font-bold text-claude-text">
                  {formatNumber(stats.total_memories)}
                </p>
              </div>
              <div className="p-4 rounded-lg bg-claude-surface border border-claude-border">
                <div className="flex items-center gap-2 mb-2">
                  <Hash size={16} className="text-claude-primary" />
                  <span className="text-xs text-claude-textSecondary">Token 总量</span>
                </div>
                <p className="text-2xl font-bold text-claude-text">
                  {formatNumber(stats.total_tokens_approx)}
                </p>
              </div>
            </div>
            <div className="p-4 rounded-lg bg-claude-surface border border-claude-border">
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs text-claude-textSecondary">后端状态</span>
                <span className={`text-xs px-2 py-0.5 rounded ${backendHealthy ? 'bg-green-500/20 text-green-500' : 'bg-red-500/20 text-red-500'}`}>
                  {backendHealthy ? '运行中' : '未连接'}
                </span>
              </div>
              <p className="text-sm text-claude-text font-mono">{stats.backend}</p>
            </div>

            {/* Caveman RTK Stats */}
            {cavemanStats && (
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <Brain size={14} className="text-purple-400" />
                  <span className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider">RTK 压缩统计</span>
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <div className="p-3 rounded-lg bg-purple-500/5 border border-purple-500/15">
                    <span className="text-[10px] text-claude-textSecondary">压缩段数</span>
                    <p className="text-lg font-bold text-claude-text mt-0.5">{cavemanStats.total_segments}</p>
                  </div>
                  <div className="p-3 rounded-lg bg-emerald-500/5 border border-emerald-500/15">
                    <span className="text-[10px] text-claude-textSecondary">节省 Tokens</span>
                    <p className="text-lg font-bold text-emerald-400 mt-0.5">{formatNumber(cavemanStats.tokens_saved)}</p>
                  </div>
                  <div className="p-3 rounded-lg bg-amber-500/5 border border-amber-500/15">
                    <span className="text-[10px] text-claude-textSecondary">压缩率</span>
                    <p className="text-lg font-bold text-amber-400 mt-0.5">{(cavemanStats.avg_compression_ratio * 100).toFixed(0)}%</p>
                  </div>
                  <div className="p-3 rounded-lg bg-blue-500/5 border border-blue-500/15">
                    <span className="text-[10px] text-claude-textSecondary">Token 重要性</span>
                    <p className="text-lg font-bold text-blue-400 mt-0.5">{formatNumber(cavemanStats.total_token_importance)}</p>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Config Modal */}
      {showConfig && config && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-claude-bg rounded-lg shadow-xl w-full max-w-md mx-4 border border-claude-border">
            <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border">
              <h3 className="text-sm font-semibold text-claude-text">记忆配置</h3>
              <button onClick={() => setShowConfig(false)} className="p-1 rounded hover:bg-claude-hover">
                <X size={16} className="text-claude-textSecondary" />
              </button>
            </div>
            <div className="p-4 space-y-3">
              <label className="flex items-center justify-between">
                <span className="text-sm text-claude-text">启用记忆功能</span>
                <input
                  type="checkbox"
                  checked={config.enabled}
                  onChange={(e) => setConfig({ ...config, enabled: e.target.checked })}
                  className="w-4 h-4 rounded"
                />
              </label>
              <label className="flex items-center justify-between">
                <span className="text-sm text-claude-text">自动检索记忆</span>
                <input
                  type="checkbox"
                  checked={config.auto_search}
                  onChange={(e) => setConfig({ ...config, auto_search: e.target.checked })}
                  className="w-4 h-4 rounded"
                />
              </label>
              <label className="flex items-center justify-between">
                <span className="text-sm text-claude-text">自动存储记忆</span>
                <input
                  type="checkbox"
                  checked={config.auto_ingest}
                  onChange={(e) => setConfig({ ...config, auto_ingest: e.target.checked })}
                  className="w-4 h-4 rounded"
                />
              </label>
              <div>
                <label className="block text-sm text-claude-text mb-1">搜索返回数量</label>
                <input
                  type="number"
                  value={config.search_top_k}
                  onChange={(e) => setConfig({ ...config, search_top_k: parseInt(e.target.value) || 5 })}
                  className="w-full px-3 py-2 rounded bg-claude-hover text-claude-text text-sm border border-claude-border"
                />
              </div>
              <div>
                <label className="block text-sm text-claude-text mb-1">最低重要性阈值</label>
                <input
                  type="number"
                  step="0.1"
                  min="0"
                  max="1"
                  value={config.min_importance_threshold}
                  onChange={(e) => setConfig({ ...config, min_importance_threshold: parseFloat(e.target.value) || 0.3 })}
                  className="w-full px-3 py-2 rounded bg-claude-hover text-claude-text text-sm border border-claude-border"
                />
              </div>
              <div>
                <label className="block text-sm text-claude-text mb-1">后端地址</label>
                <input
                  type="text"
                  value={config.backend_url}
                  onChange={(e) => setConfig({ ...config, backend_url: e.target.value })}
                  className="w-full px-3 py-2 rounded bg-claude-hover text-claude-text text-sm border border-claude-border font-mono"
                />
              </div>
            </div>
            <div className="flex justify-end gap-2 px-4 py-3 border-t border-claude-border">
              <button
                onClick={() => setShowConfig(false)}
                className="px-4 py-2 rounded text-sm text-claude-textSecondary hover:bg-claude-hover transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleSaveConfig}
                className="px-4 py-2 rounded bg-claude-primary text-white text-sm font-medium hover:bg-claude-primary/90 transition-colors"
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default MemoryPanel;
