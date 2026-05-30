import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, Star, GitFork, ExternalLink, TrendingUp, Bookmark, Plus, X, RefreshCw, Loader2, Globe, Lock, User, Bell, Zap, Database, Puzzle, Cpu, Share2, Layers, ArrowRight, Download, Code } from 'lucide-react';

interface GitHubRepo {
  id: number; name: string; full_name: string; description: string;
  html_url: string; stars: number; forks: number; language: string;
  topics: string[]; license: string | null; updated_at: string;
}

interface FusionDirection { pattern: string; description: string; icon: string; difficulty: number; }

const FUSION_DIRECTIONS: FusionDirection[] = [
  { pattern: 'plugin-system', description: '将一个项目的插件系统集成到另一个项目', icon: '🧩', difficulty: 6 },
  { pattern: 'pipeline-integration', description: '两个工具链通过 CI/CD 管道串联', icon: '🔗', difficulty: 4 },
  { pattern: 'micro-frontend', description: '多前端项目通过微前端架构融合', icon: '🧱', difficulty: 7 },
  { pattern: 'api-gateway', description: '多服务通过统一 API 网关整合', icon: '🚪', difficulty: 5 },
  { pattern: 'data-pipeline', description: '数据采集+处理+可视化工具融合', icon: '📊', difficulty: 5 },
  { pattern: 'mcp-server', description: '项目封装为 MCP 服务器让 AI 调用', icon: '🤖', difficulty: 3 },
  { pattern: 'cross-platform', description: '优秀工具跨平台适配', icon: '🔄', difficulty: 6 },
  { pattern: 'template-generation', description: '提取脚手架模板作为新项目起点', icon: '🏗️', difficulty: 2 },
];

type TabType = 'trending' | 'search' | 'myrepos' | 'watch' | 'fusion';

const trendingLangs = ['', 'TypeScript', 'Python', 'Rust', 'Go', 'JavaScript', 'Swift', 'Kotlin'];

const GitHubHubPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [tab, setTab] = useState<TabType>('trending');
  const [repos, setRepos] = useState<GitHubRepo[]>([]);
  const [loading, setLoading] = useState(false);
  const [since, setSince] = useState('daily');
  const [lang, setLang] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [token, setToken] = useState<string | null>(null);
  const [watched, setWatched] = useState<string[]>([]);
  const [selectedRepos, setSelectedRepos] = useState<Set<string>>(new Set());
  const [fusionResults, setFusionResults] = useState<any[] | null>(null);
  const [oauthUrl, setOauthUrl] = useState('');

  useEffect(() => { loadTrending(); }, [since, lang]);

  const loadTrending = useCallback(async () => {
    setLoading(true);
    try {
      const r = await invoke<GitHubRepo[]>('gh_trending', { since, language: lang || null });
      setRepos(r);
    } catch (e) { console.error(e); }
    setLoading(false);
  }, [since, lang]);

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;
    setLoading(true);
    try {
      const r = await invoke<GitHubRepo[]>('gh_search', { query: searchQuery });
      setRepos(r);
      setTab('search');
    } catch (e) { console.error(e); }
    setLoading(false);
  };

  const loadMyRepos = async () => {
    setLoading(true);
    try {
      const r = await invoke<GitHubRepo[]>('gh_user_repos');
      setRepos(r);
    } catch (e: any) {
      if (e.includes('Not authenticated')) {
        const url = await invoke<string>('gh_oauth_url', { stateParam: `state_${Date.now()}` });
        setOauthUrl(url);
      }
    }
    setLoading(false);
  };

  const handleWatch = async (fullName: string) => {
    try { await invoke('gh_watch', { fullName }); setWatched(prev => [...prev, fullName]); } catch {}
  };

  const toggleSelectRepo = (name: string) => {
    setSelectedRepos(prev => {
      const next = new Set(prev);
      next.has(name) ? next.delete(name) : next.add(name);
      return next;
    });
  };

  const analyzeFusion = () => {
    const selected = repos.filter(r => selectedRepos.has(r.full_name));
    if (selected.length < 2) return;
    const results: any[] = [];
    for (let i = 0; i < selected.length; i++) {
      for (let j = i + 1; j < selected.length; j++) {
        const a = selected[i], b = selected[j];
        const commonTechs = [a.language, b.language].filter(Boolean);
        if (a.language && b.language && a.language === b.language) {
          for (const dir of FUSION_DIRECTIONS) {
            results.push({
              projects: [a.full_name, b.full_name],
              direction: dir,
              rationale: `将 ${a.full_name} 的 ${dir.icon} ${dir.description} 到 ${b.full_name}`,
              score: Math.max(1, 10 - dir.difficulty),
            });
          }
        }
      }
    }
    setFusionResults(results.sort((x, y) => y.score - x.score));
    setTab('fusion');
  };

  const renderRepoCard = (repo: GitHubRepo) => (
    <div key={repo.id} className="flex items-start gap-3 px-3 py-2.5 border-b border-claude-border/30 hover:bg-claude-hover/40 transition-colors group">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-[12px] font-semibold text-claude-text truncate">{repo.full_name}</span>
          {repo.language && <span className="text-[9px] px-1.5 py-0.5 rounded-full bg-blue-500/10 text-blue-400 shrink-0">{repo.language}</span>}
        </div>
        <div className="text-[11px] text-claude-textSecondary/70 mt-0.5 line-clamp-2">{repo.description || '—'}</div>
        <div className="flex items-center gap-3 mt-1.5 text-[10px] text-claude-textSecondary">
          <span className="flex items-center gap-0.5"><Star size={10} />{repo.stars}</span>
          <span className="flex items-center gap-0.5"><GitFork size={10} />{repo.forks}</span>
          {repo.topics?.slice(0, 3).map(t => <span key={t} className="px-1 py-0.5 rounded bg-claude-hover text-[9px]">#{t}</span>)}
        </div>
      </div>
      <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        <button onClick={() => window.open(repo.html_url, '_blank')} className="p-1.5 rounded hover:bg-claude-hover text-claude-textSecondary"><ExternalLink size={12} /></button>
        {tab !== 'fusion' && <label className="p-1.5 rounded hover:bg-claude-hover cursor-pointer"><input type="checkbox" checked={selectedRepos.has(repo.full_name)} onChange={() => toggleSelectRepo(repo.full_name)} className="w-3 h-3" /></label>}
        <button onClick={() => handleWatch(repo.full_name)} className="p-1.5 rounded hover:bg-claude-hover text-claude-textSecondary"><Bell size={12} /></button>
      </div>
    </div>
  );

  return (
    <div className="h-full flex flex-col bg-claude-bg">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-claude-border shrink-0">
        <div className="flex items-center gap-2">
          <Globe size={16} className="text-blue-400" />
          <span className="text-[13px] font-semibold text-claude-text">GitHub 智能中心</span>
        </div>
        <div className="flex items-center gap-1">
          {oauthUrl && <a href={oauthUrl} target="_blank" className="text-[11px] text-blue-400 hover:underline flex items-center gap-1"><Lock size={10} /> 授权</a>}
          <button onClick={onClose} className="p-1 rounded hover:bg-claude-hover text-claude-textSecondary"><X size={14} /></button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-claude-border overflow-x-auto" style={{ scrollbarWidth: 'thin' }}>
        {[
          { id: 'trending' as TabType, label: '热门', icon: TrendingUp },
          { id: 'search' as TabType, label: '搜索', icon: Search },
          { id: 'myrepos' as TabType, label: '我的', icon: User },
          { id: 'watch' as TabType, label: '监控', icon: Bell },
          { id: 'fusion' as TabType, label: '融合', icon: Puzzle },
        ].map(t => (
          <button key={t.id} onClick={() => setTab(t.id)}
            className={`flex items-center gap-1 px-3 py-2 text-[11px] font-medium transition-colors shrink-0 ${tab === t.id ? 'text-blue-400 border-b-2 border-blue-400' : 'text-claude-textSecondary hover:text-claude-text'}`}>
            <t.icon size={13} />{t.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {tab === 'trending' && (
          <>
            <div className="flex items-center gap-2 px-3 py-2 border-b border-claude-border/30">
              <select value={since} onChange={e => setSince(e.target.value)} className="text-[11px] bg-claude-hover text-claude-text px-2 py-1 rounded outline-none">
                <option value="daily">今日</option><option value="weekly">本周</option><option value="monthly">本月</option>
              </select>
              <select value={lang} onChange={e => setLang(e.target.value)} className="text-[11px] bg-claude-hover text-claude-text px-2 py-1 rounded outline-none">
                <option value="">所有语言</option>{trendingLangs.filter(Boolean).map(l => <option key={l} value={l}>{l}</option>)}
              </select>
              <button onClick={loadTrending} className="p-1 ml-auto text-claude-textSecondary hover:text-claude-text"><RefreshCw size={12} /></button>
            </div>
            {loading ? <div className="flex items-center justify-center py-8"><Loader2 size={16} className="animate-spin text-blue-400" /></div>
            : repos.map(renderRepoCard)}
          </>
        )}

        {tab === 'search' && (
          <>
            <div className="flex items-center gap-2 px-3 py-2 border-b border-claude-border/30">
              <div className="flex-1 flex items-center gap-2 bg-claude-hover rounded-lg px-2.5 py-1.5">
                <Search size={12} className="text-claude-textSecondary" />
                <input value={searchQuery} onChange={e => setSearchQuery(e.target.value)} onKeyDown={e => e.key === 'Enter' && handleSearch()}
                  placeholder="搜索 GitHub 仓库..." className="flex-1 bg-transparent text-[12px] text-claude-text outline-none" />
              </div>
              <button onClick={handleSearch} className="text-[11px] px-2.5 py-1.5 bg-blue-500/10 text-blue-400 rounded-lg hover:bg-blue-500/20">搜索</button>
            </div>
            {repos.length > 0 && <div className="text-[10px] text-claude-textSecondary px-3 py-1">{repos.length} 个结果 · 勾选后到融合 Tab 分析</div>}
            {repos.map(renderRepoCard)}
          </>
        )}

        {tab === 'myrepos' && (
          <div className="p-4">
            {oauthUrl ? (
              <div className="flex flex-col items-center gap-3 py-8 text-center">
                <Lock size={32} className="text-claude-textSecondary/30" />
                <p className="text-[13px] text-claude-textSecondary">需要 GitHub 授权</p>
                <a href={oauthUrl} target="_blank"
                  className="px-4 py-2 bg-[#24292e] hover:bg-[#1b1f23] text-white text-[12px] rounded-lg transition-colors flex items-center gap-2">
                  <Globe size={14} /> 在浏览器中授权
                </a>
                <p className="text-[10px] text-claude-textSecondary/50">授权后返回此页面点击刷新</p>
              </div>
            ) : (
              <button onClick={loadMyRepos} className="w-full py-2 text-[12px] bg-claude-hover rounded-lg hover:bg-claude-btnHover flex items-center justify-center gap-2">
                <RefreshCw size={12} /> 加载我的仓库
              </button>
            )}
          </div>
        )}

        {tab === 'watch' && (
          <div className="p-4 text-center text-[12px] text-claude-textSecondary">
            {watched.length === 0 ? '尚未监控任何仓库 — 在热门/搜索中点击 🔔 图标添加' : watched.map(w => <div key={w} className="text-left text-claude-text p-2">{w}</div>)}
          </div>
        )}

        {tab === 'fusion' && (
          <div className="p-3">
            {selectedRepos.size < 2 ? (
              <div className="text-center py-8 text-claude-textSecondary text-[12px]">
                <Puzzle size={28} className="mx-auto mb-2 opacity-30" />
                <p>在热门/搜索 Tab 勾选 2 个以上仓库</p>
                <p className="text-[11px] mt-1">然后回到此 Tab 查看融合方案</p>
                <button onClick={analyzeFusion} disabled={selectedRepos.size < 2}
                  className="mt-3 px-4 py-2 bg-purple-500/20 text-purple-400 text-[12px] rounded-lg hover:bg-purple-500/30 disabled:opacity-30">
                  分析 {selectedRepos.size} 个仓库
                </button>
              </div>
            ) : fusionResults ? (
              <div className="space-y-3">
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-[11px] font-semibold text-claude-text">融合方案</span>
                  <span className="text-[10px] text-claude-textSecondary">{fusionResults.length} 个建议</span>
                </div>
                {fusionResults.slice(0, 10).map((r, i) => (
                  <div key={i} className="p-3 rounded-xl border border-purple-500/20 bg-purple-500/5 space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="text-[22px]">{r.direction.icon}</span>
                      <span className={`text-[10px] px-1.5 py-0.5 rounded-full ${r.score >= 7 ? 'bg-green-500/15 text-green-400' : r.score >= 4 ? 'bg-amber-500/15 text-amber-400' : 'bg-claude-hover text-claude-textSecondary'}`}>
                        创新评分 {r.score}/10
                      </span>
                    </div>
                    <div className="text-[12px] font-medium text-claude-text">{r.direction.pattern}</div>
                    <div className="text-[11px] text-claude-textSecondary leading-relaxed">{r.rationale}</div>
                    <div className="flex items-center gap-1 text-[10px] text-claude-textSecondary">
                      {r.projects.map((p: string) => <span key={p} className="px-1.5 py-0.5 rounded bg-claude-hover">{p.split('/')[1]}</span>)}
                      <ArrowRight size={10} className="mx-1" />
                      <span className="text-purple-400">融合创新</span>
                    </div>
                    <div className="flex gap-1">
                      <button className="px-2 py-1 text-[10px] bg-purple-500/15 text-purple-400 rounded-lg hover:bg-purple-500/25">查看方案</button>
                      <button className="px-2 py-1 text-[10px] bg-claude-hover text-claude-textSecondary rounded-lg">生成 Skill</button>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8">
                <button onClick={analyzeFusion} className="px-4 py-2 bg-purple-500/20 text-purple-400 text-[12px] rounded-lg">
                  分析 {selectedRepos.size} 个仓库的融合机会
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default GitHubHubPanel;
