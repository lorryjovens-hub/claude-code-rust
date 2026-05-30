import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  X, Loader2, CheckCircle2, XCircle, Clock, GitBranch, FileText,
  Eye, Terminal, Code, ChevronDown, ChevronRight, AlertTriangle,
  Search, Zap, Sparkles, BookOpen, Cpu,
} from 'lucide-react';
import { getSkills } from '../api';

/* ── Types ── */
interface AgentActivity {
  id: string;
  type: 'thinking' | 'tool_call' | 'file_change' | 'code_gen' | 'search' | 'complete' | 'error';
  title: string;
  detail?: string;
  timestamp: number;
  status: 'running' | 'done' | 'error';
  duration_ms?: number;
}

interface SkillInfo {
  id: string;
  name: string;
  description?: string;
  enabled: boolean;
  category?: string;
}

/* ── Props ── */
interface AgentExecutionViewProps {
  onClose: () => void;
}

/* ── Component ── */
const AgentExecutionView: React.FC<AgentExecutionViewProps> = ({ onClose }) => {
  const [activities, setActivities] = useState<AgentActivity[]>([]);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [skillsLoading, setSkillsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'live' | 'skills'>('live');
  const listRef = useRef<HTMLDivElement>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  /* ── Load skills on mount ── */
  useEffect(() => {
    setSkillsLoading(true);
    getSkills().then((data: any) => {
      const all = Array.isArray(data) ? data : (data?.skills || []);
      setSkills(all);
    }).catch(() => {}).finally(() => setSkillsLoading(false));
  }, []);

  /* ── Subscribe to agent events from SSE ── */
  useEffect(() => {
    // Listen for streaming events from the main chat
    const handler = (e: Event) => {
      const customEvent = e as CustomEvent;
      const data = customEvent.detail;
      if (!data) return;

      const activity: AgentActivity = {
        id: data.tool_use_id || `act-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        type: data.type === 'code_gen' ? 'code_gen' :
              data.type === 'tool_call' ? 'tool_call' :
              data.type === 'error' ? 'error' :
              data.type === 'thinking' ? 'thinking' :
              data.type === 'search' ? 'search' :
              data.type === 'complete' ? 'complete' :
              data.type === 'text' ? 'code_gen' :
              data.type === 'tool_use' ? 'tool_call' :
              data.type === 'file_change' ? 'file_change' :
              data.tool_name === 'FileWrite' || data.tool_name === 'FileEdit' || data.tool_name === 'Write' || data.tool_name === 'Edit' ? 'file_change' :
              'tool_call',
        title: data.tool_name || data.type || 'Agent Activity',
        detail: data.content || data.input || '',
        timestamp: Date.now(),
        status: data.is_error ? 'error' : 'running',
      };

      setActivities(prev => [activity, ...prev].slice(0, 50));

      // Mark running activities as done after timeout (simplified)
      if (activity.type === 'tool_call') {
        setTimeout(() => {
          setActivities(prev => prev.map(a =>
            a.id === activity.id && a.status === 'running'
              ? { ...a, status: 'done' as const, duration_ms: Date.now() - a.timestamp }
              : a
          ));
        }, 2000);
      }
    };

    window.addEventListener('agent-activity', handler as EventListener);
    return () => window.removeEventListener('agent-activity', handler as EventListener);
  }, []);

  /* ── Clear history ── */
  const clearHistory = () => setActivities([]);

  /* ── Activity icon ── */
  const getActivityIcon = (act: AgentActivity) => {
    if (act.status === 'error') return <XCircle size={14} className="text-red-400" />;
    if (act.status === 'running' && act.type === 'thinking') return <Loader2 size={14} className="text-blue-400 animate-spin" />;
    if (act.status === 'running') return <Loader2 size={14} className="text-amber-400 animate-spin" />;
    if (act.status === 'done') return <CheckCircle2 size={14} className="text-green-400" />;

    switch (act.type) {
      case 'thinking': return <Cpu size={14} className="text-blue-400" />;
      case 'tool_call': return <Terminal size={14} className="text-purple-400" />;
      case 'file_change': return <FileText size={14} className="text-cyan-400" />;
      case 'code_gen': return <Code size={14} className="text-emerald-400" />;
      case 'search': return <Search size={14} className="text-yellow-400" />;
      default: return <Clock size={14} className="text-gray-400" />;
    }
  };

  /* ── Render live tab ── */
  const renderLiveTab = () => (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border">
        <div className="flex items-center gap-2">
          <Cpu size={16} className="text-blue-400" />
          <span className="text-[13px] font-semibold text-claude-text">Agent 实时执行</span>
        </div>
        <div className="flex items-center gap-2">
          {activities.length > 0 && (
            <button onClick={clearHistory} className="text-[11px] text-claude-textSecondary hover:text-claude-text transition-colors px-2 py-0.5 rounded hover:bg-claude-hover">
              清空
            </button>
          )}
          <button onClick={onClose} className="p-1 rounded hover:bg-claude-hover text-claude-textSecondary">
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Activity stream */}
      <div ref={listRef} className="flex-1 overflow-y-auto py-1 space-y-0.5">
        {activities.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-claude-textSecondary gap-3 px-6">
            <div className="w-12 h-12 rounded-full bg-claude-hover flex items-center justify-center">
              <Cpu size={24} className="text-claude-textSecondary/50" />
            </div>
            <span className="text-[12px] text-center leading-relaxed">
              等待 Agent 活动...<br />
              <span className="text-[11px] opacity-60">发送消息后，Agent 的执行步骤将实时显示在这里</span>
            </span>
          </div>
        ) : (
          activities.map((act) => (
            <div key={act.id}>
              <div
                className="flex items-start gap-2.5 px-4 py-2 hover:bg-claude-hover/40 cursor-pointer transition-colors"
                onClick={() => setExpandedId(expandedId === act.id ? null : act.id)}
              >
                <div className="mt-0.5 flex-shrink-0">{getActivityIcon(act)}</div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className={`text-[12px] font-medium truncate ${
                      act.status === 'error' ? 'text-red-400' :
                      act.status === 'running' ? 'text-claude-text' :
                      'text-claude-text'
                    }`}>{act.title}</span>
                    {act.duration_ms && (
                      <span className="text-[10px] text-claude-textSecondary/50 flex-shrink-0">
                        {(act.duration_ms / 1000).toFixed(1)}s
                      </span>
                    )}
                  </div>
                  {act.detail && (
                    <div className="text-[11px] text-claude-textSecondary truncate mt-0.5 leading-snug">
                      {act.type === 'tool_call' ? `执行工具: ${act.title}` :
                       act.type === 'file_change' ? `修改文件` :
                       act.type === 'search' ? act.detail.slice(0, 100) :
                       act.type === 'code_gen' ? '生成内容...' :
                       act.type === 'thinking' ? '思考中...' :
                       act.detail.slice(0, 100)}
                    </div>
                  )}
                </div>
                {expandedId === act.id ? <ChevronDown size={12} className="flex-shrink-0 text-claude-textSecondary" /> : <ChevronRight size={12} className="flex-shrink-0 text-claude-textSecondary" />}
              </div>
              {expandedId === act.id && act.detail && (
                <div className="mx-4 mb-2 px-3 py-2 rounded-lg bg-claude-hover/60 text-[11px] font-mono text-claude-textSecondary whitespace-pre-wrap leading-relaxed max-h-[200px] overflow-y-auto">
                  {act.detail}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );

  /* ── Render skills tab ── */
  const renderSkillsTab = () => (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border">
        <div className="flex items-center gap-2">
          <Sparkles size={16} className="text-amber-400" />
          <span className="text-[13px] font-semibold text-claude-text">Skills 市场</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-[11px] text-claude-textSecondary">{skills.length} 个已安装</span>
          <button onClick={onClose} className="p-1 rounded hover:bg-claude-hover text-claude-textSecondary">
            <X size={14} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Featured repos */}
        <div className="px-4 pt-3 pb-2">
          <span className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider">推荐集成</span>
        </div>
        <div className="px-4 pb-3 space-y-2">
          {[
            { name: 'obra/superpowers', desc: '174k ⭐ TDD、代码审查、分支工作流', url: 'https://github.com/obra/superpowers' },
            { name: 'Owl-Listener/designer-skills', desc: '91 个设计技能全覆盖', url: 'https://github.com/Owl-Listener/designer-skills' },
            { name: 'ConardLi/garden-skills', desc: 'web-design-engineer 设计工作流', url: 'https://github.com/ConardLi/garden-skills' },
            { name: 'mattpocock/skills', desc: '严谨工程实践工作流', url: 'https://github.com/mattpocock/skills' },
            { name: 'EricGrill/agents-skills-plugins', desc: '社区聚合插件市场', url: 'https://github.com/EricGrill/agents-skills-plugins' },
          ].map(repo => (
            <div key={repo.name} className="flex items-start gap-3 p-2.5 rounded-xl border border-claude-border hover:bg-claude-hover/40 transition-colors group">
              <div className="w-8 h-8 rounded-lg bg-claude-hover flex items-center justify-center flex-shrink-0">
                <BookOpen size={16} className="text-claude-textSecondary" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-[12px] font-medium text-claude-text">{repo.name}</div>
                <div className="text-[11px] text-claude-textSecondary mt-0.5">{repo.desc}</div>
              </div>
              <button
                onClick={() => window.open(repo.url, '_blank')}
                className="shrink-0 px-2.5 py-1 text-[11px] text-blue-400 hover:bg-blue-500/10 rounded-lg transition-colors opacity-0 group-hover:opacity-100"
              >查看</button>
            </div>
          ))}
        </div>

        {/* Installed skills list */}
        <div className="px-4 pt-2 pb-2 border-t border-claude-border">
          <span className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider">已安装 Skills</span>
        </div>
        <div className="px-4 pb-4 space-y-0.5">
          {skillsLoading ? (
            <div className="flex items-center justify-center py-6"><Loader2 size={16} className="animate-spin text-blue-400" /></div>
          ) : skills.length === 0 ? (
            <div className="text-[12px] text-claude-textSecondary text-center py-6">暂无已安装的 skills</div>
          ) : (
            skills.slice(0, 50).map(skill => (
              <div key={skill.id} className="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-claude-hover/40 transition-colors">
                <div className={`w-1.5 h-1.5 rounded-full ${skill.enabled ? 'bg-green-500' : 'bg-claude-textSecondary/30'}`} />
                <span className="text-[12px] text-claude-text flex-1 truncate">{skill.name}</span>
                <span className={`text-[10px] px-1.5 py-0.5 rounded-full ${skill.enabled ? 'bg-green-500/10 text-green-400' : 'bg-claude-hover text-claude-textSecondary'}`}>
                  {skill.enabled ? '启用' : '禁用'}
                </span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );

  return (
    <div className="h-full flex flex-col bg-claude-bg">
      {/* Tab bar */}
      <div className="flex border-b border-claude-border">
        <button
          onClick={() => setActiveTab('live')}
          className={`flex-1 py-2 text-[12px] font-medium transition-colors ${
            activeTab === 'live' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-claude-textSecondary hover:text-claude-text'
          }`}
        ><Cpu size={14} className="inline mr-1 -mt-0.5" />实时执行</button>
        <button
          onClick={() => setActiveTab('skills')}
          className={`flex-1 py-2 text-[12px] font-medium transition-colors ${
            activeTab === 'skills' ? 'text-amber-400 border-b-2 border-amber-400' : 'text-claude-textSecondary hover:text-claude-text'
          }`}
        ><Sparkles size={14} className="inline mr-1 -mt-0.5" />Skills 市场</button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {activeTab === 'live' ? renderLiveTab() : renderSkillsTab()}
      </div>
    </div>
  );
};

export default AgentExecutionView;
