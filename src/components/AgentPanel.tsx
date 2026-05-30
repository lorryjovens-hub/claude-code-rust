import React, { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../hooks/useI18n';
import { GitBranch, Plus, Trash2, Merge, RefreshCw, X, Loader2, CheckCircle, XCircle, AlertCircle, Monitor, Wifi, WifiOff, Plug } from 'lucide-react';
import {
  createWorktree, listWorktrees, removeWorktree, mergeWorktree, syncWorktrees,
  listAgents, cancelAgent,
  getIdeStatus, startIdeServer, stopIdeServer, getIdeConnections, disconnectIde,
} from '../api';

type WorktreeInfo = {
  id: string;
  path: string;
  branch: string;
  agent_id: string | null;
  status: 'Active' | 'Idle' | 'Merging' | 'Removed';
  created_at: string;
};

type AgentInfo = {
  id: string;
  name: string;
  worktree_id: string;
  task: string;
  status: 'Starting' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';
  model: string;
  created_at: string;
  result: string | null;
};

type IdeConnection = {
  id: string;
  ide_type: 'VSCode' | 'Cursor' | 'JetBrains' | 'Neovim' | 'Unknown';
  status: 'Connected' | 'Disconnected' | 'Reconnecting';
  workspace: string | null;
  connected_at: string;
  last_heartbeat: string;
};

type IdeBridgeStatus = {
  server_running: boolean;
  port: number;
  active_connections: number;
  total_connections: number;
};

const statusColors: Record<string, string> = {
  Active: 'bg-green-500',
  Idle: 'bg-yellow-500',
  Merging: 'bg-blue-500',
  Removed: 'bg-gray-400',
  Starting: 'bg-yellow-500',
  Running: 'bg-green-500',
  Completed: 'bg-blue-500',
  Failed: 'bg-red-500',
  Cancelled: 'bg-gray-400',
  Connected: 'bg-green-500',
  Disconnected: 'bg-red-500',
  Reconnecting: 'bg-yellow-500',
};

export default function AgentPanel({ onClose }: { onClose: () => void }) {
  const { t } = useI18n();
  const [tab, setTab] = useState<'worktrees' | 'agents' | 'ide'>('worktrees');
  const [worktrees, setWorktrees] = useState<WorktreeInfo[]>([]);
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [ideStatus, setIdeStatus] = useState<IdeBridgeStatus | null>(null);
  const [ideConnections, setIdeConnections] = useState<IdeConnection[]>([]);
  const [loading, setLoading] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [newAgentName, setNewAgentName] = useState('');
  const [newAgentTask, setNewAgentTask] = useState('');
  const [newAgentBranch, setNewAgentBranch] = useState('');

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [wtRes, agRes, ideRes, connRes] = await Promise.all([
        listWorktrees(),
        listAgents(),
        getIdeStatus(),
        getIdeConnections(),
      ]);
      if (wtRes.success) setWorktrees(wtRes.worktrees || []);
      if (agRes.success) setAgents(agRes.agents || []);
      if (ideRes.success) setIdeStatus(ideRes.status);
      if (connRes.success) setIdeConnections(connRes.connections || []);
    } catch (_) {}
    setLoading(false);
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const handleCreateWorktree = async () => {
    if (!newAgentName.trim()) return;
    try {
      await createWorktree({
        branch_prefix: newAgentBranch || undefined,
        agent_name: newAgentName,
        task: newAgentTask || undefined,
      });
      setShowCreate(false);
      setNewAgentName('');
      setNewAgentTask('');
      setNewAgentBranch('');
      refresh();
    } catch (_) {}
  };

  const handleRemove = async (id: string) => {
    try { await removeWorktree(id); refresh(); } catch (_) {}
  };

  const handleMerge = async (id: string) => {
    try { await mergeWorktree(id); refresh(); } catch (_) {}
  };

  const handleCancelAgent = async (id: string) => {
    try { await cancelAgent(id); refresh(); } catch (_) {}
  };

  const handleIdeToggle = async () => {
    try {
      if (ideStatus?.server_running) {
        await stopIdeServer();
      } else {
        await startIdeServer();
      }
      refresh();
    } catch (_) {}
  };

  const handleIdeDisconnect = async (id: string) => {
    try { await disconnectIde(id); refresh(); } catch (_) {}
  };

  return (
    <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40" onClick={onClose}>
      <div className="bg-claude-bg border border-claude-border rounded-2xl shadow-2xl w-full max-w-2xl max-h-[80vh] flex flex-col overflow-hidden" onClick={e => e.stopPropagation()}>
        <div className="flex items-center justify-between px-6 py-4 border-b border-claude-border">
          <h2 className="text-lg font-semibold text-claude-text">{t('agent.title')}</h2>
          <button onClick={onClose} className="p-1.5 hover:bg-claude-hover rounded-lg transition-colors text-claude-textSecondary hover:text-claude-text">
            <X size={18} />
          </button>
        </div>

        <div className="flex border-b border-claude-border">
          {([
            ['worktrees', t('agent.worktrees')],
            ['agents', t('agent.agents')],
            ['ide', t('agent.ide')],
          ] as const).map(([key, label]) => (
            <button
              key={key}
              onClick={() => setTab(key)}
              className={`flex-1 px-4 py-2.5 text-[14px] font-medium transition-colors ${tab === key ? 'text-claude-text border-b-2 border-[#C6613F]' : 'text-claude-textSecondary hover:text-claude-text'}`}
            >
              {label}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          {tab === 'worktrees' && (
            <div className="space-y-3">
              <div className="flex items-center justify-between mb-4">
                <span className="text-[13px] text-claude-textSecondary">{worktrees.length} worktree(s)</span>
                <div className="flex gap-2">
                  <button onClick={refresh} className="p-1.5 hover:bg-claude-hover rounded-lg text-claude-textSecondary hover:text-claude-text transition-colors" title={t('common.loading')}>
                    <RefreshCw size={16} className={loading ? 'animate-spin' : ''} />
                  </button>
                  <button onClick={() => syncWorktrees()} className="p-1.5 hover:bg-claude-hover rounded-lg text-claude-textSecondary hover:text-claude-text transition-colors" title={t('agent.refresh')}>
                    <GitBranch size={16} />
                  </button>
                  <button onClick={() => setShowCreate(true)} className="flex items-center gap-1.5 px-3 py-1.5 bg-[#C6613F] text-white text-[13px] font-medium rounded-lg hover:bg-[#D97757] transition-colors">
                    <Plus size={14} /> {t('common.newChat')}
                  </button>
                </div>
              </div>

              {showCreate && (
                <div className="p-4 bg-claude-input border border-claude-border rounded-xl space-y-3">
                  <input
                    value={newAgentName}
                    onChange={e => setNewAgentName(e.target.value)}
                    placeholder={t('agent.agentName')}
                    className="w-full px-3 py-2 bg-white dark:bg-[#2B2A29] border border-claude-border rounded-lg text-[14px] text-claude-text placeholder:text-claude-textSecondary focus:outline-none focus:border-[#C6613F]"
                  />
                  <input
                    value={newAgentTask}
                    onChange={e => setNewAgentTask(e.target.value)}
                    placeholder={t('agent.currentTask')}
                    className="w-full px-3 py-2 bg-white dark:bg-[#2B2A29] border border-claude-border rounded-lg text-[14px] text-claude-text placeholder:text-claude-textSecondary focus:outline-none focus:border-[#C6613F]"
                  />
                  <input
                    value={newAgentBranch}
                    onChange={e => setNewAgentBranch(e.target.value)}
                    placeholder={t('agent.worktreeBranch') + ' (optional)'}
                    className="w-full px-3 py-2 bg-white dark:bg-[#2B2A29] border border-claude-border rounded-lg text-[14px] text-claude-text placeholder:text-claude-textSecondary focus:outline-none focus:border-[#C6613F]"
                  />
                  <div className="flex justify-end gap-2">
                    <button onClick={() => setShowCreate(false)} className="px-3 py-1.5 text-[13px] text-claude-textSecondary hover:text-claude-text transition-colors">{t('common.cancel')}</button>
                    <button onClick={handleCreateWorktree} disabled={!newAgentName.trim()} className="px-3 py-1.5 text-[13px] font-medium bg-[#C6613F] text-white rounded-lg hover:bg-[#D97757] transition-colors disabled:opacity-40">{t('agent.createWorktree')}</button>
                  </div>
                </div>
              )}

              {worktrees.length === 0 ? (
                <div className="text-center py-8 text-claude-textSecondary text-[14px]">
                  {t('agent.noAgents')}
                </div>
              ) : worktrees.map(wt => (
                <div key={wt.id} className="p-4 bg-claude-input border border-claude-border rounded-xl">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <span className={`w-2 h-2 rounded-full ${statusColors[wt.status] || 'bg-gray-400'}`} />
                      <span className="text-[14px] font-medium text-claude-text">{wt.branch}</span>
                    </div>
                    <span className="text-[12px] text-claude-textSecondary">{wt.id}</span>
                  </div>
                  <div className="text-[12px] text-claude-textSecondary mb-3">{wt.path}</div>
                  <div className="flex gap-2">
                    {wt.status === 'Active' && (
                      <button onClick={() => handleMerge(wt.id)} className="flex items-center gap-1 px-2.5 py-1 text-[12px] font-medium text-blue-600 bg-blue-50 dark:bg-blue-900/20 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors">
                        <Merge size={12} /> {t('agent.mergeBranch')}
                      </button>
                    )}
                    <button onClick={() => handleRemove(wt.id)} className="flex items-center gap-1 px-2.5 py-1 text-[12px] font-medium text-red-600 bg-red-50 dark:bg-red-900/20 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors">
                      <Trash2 size={12} /> {t('common.delete')}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {tab === 'agents' && (
            <div className="space-y-3">
              <div className="flex items-center justify-between mb-4">
                <span className="text-[13px] text-claude-textSecondary">{agents.length} agent(s)</span>
                <button onClick={refresh} className="p-1.5 hover:bg-claude-hover rounded-lg text-claude-textSecondary hover:text-claude-text transition-colors">
                  <RefreshCw size={16} className={loading ? 'animate-spin' : ''} />
                </button>
              </div>

              {agents.length === 0 ? (
                <div className="text-center py-8 text-claude-textSecondary text-[14px]">
                  {t('agent.noTasks')}
                </div>
              ) : agents.map(ag => (
                <div key={ag.id} className="p-4 bg-claude-input border border-claude-border rounded-xl">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <span className={`w-2 h-2 rounded-full ${statusColors[ag.status] || 'bg-gray-400'}`} />
                      <span className="text-[14px] font-medium text-claude-text">{ag.name}</span>
                      <span className="text-[12px] text-claude-textSecondary bg-claude-hover px-1.5 py-0.5 rounded">{ag.model}</span>
                    </div>
                    <span className="text-[12px] text-claude-textSecondary">{ag.id}</span>
                  </div>
                  <div className="text-[13px] text-claude-textSecondary mb-2">{ag.task || t('agent.noTasks')}</div>
                  {ag.result && (
                    <div className="text-[12px] text-claude-text bg-white dark:bg-[#2B2A29] border border-claude-border rounded-lg p-2 mb-2 max-h-24 overflow-y-auto">
                      {ag.result}
                    </div>
                  )}
                  <div className="flex items-center gap-2">
                    {(ag.status === 'Running' || ag.status === 'Starting') && (
                      <button onClick={() => handleCancelAgent(ag.id)} className="flex items-center gap-1 px-2.5 py-1 text-[12px] font-medium text-red-600 bg-red-50 dark:bg-red-900/20 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors">
                        <XCircle size={12} /> {t('common.cancel')}
                      </button>
                    )}
                    {ag.status === 'Running' && <Loader2 size={14} className="animate-spin text-green-500" />}
                    {ag.status === 'Completed' && <CheckCircle size={14} className="text-blue-500" />}
                    {ag.status === 'Failed' && <AlertCircle size={14} className="text-red-500" />}
                  </div>
                </div>
              ))}
            </div>
          )}

          {tab === 'ide' && (
            <div className="space-y-4">
              <div className="p-4 bg-claude-input border border-claude-border rounded-xl">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <Monitor size={18} className="text-claude-textSecondary" />
                    <span className="text-[14px] font-medium text-claude-text">{t('agent.ide')} Bridge Server</span>
                  </div>
                  <button
                    onClick={handleIdeToggle}
                    className={`flex items-center gap-1.5 px-3 py-1.5 text-[13px] font-medium rounded-lg transition-colors ${
                      ideStatus?.server_running
                        ? 'bg-red-50 dark:bg-red-900/20 text-red-600 hover:bg-red-100 dark:hover:bg-red-900/30'
                        : 'bg-green-50 dark:bg-green-900/20 text-green-600 hover:bg-green-100 dark:hover:bg-green-900/30'
                    }`}
                  >
                    {ideStatus?.server_running ? <><WifiOff size={14} /> {t('agent.stop')}</> : <><Wifi size={14} /> {t('agent.run')}</>}
                  </button>
                </div>
                {ideStatus && (
                  <div className="grid grid-cols-3 gap-3 text-center">
                    <div className="p-2 bg-white dark:bg-[#2B2A29] rounded-lg">
                      <div className="text-[18px] font-semibold text-claude-text">{ideStatus.port || '-'}</div>
                      <div className="text-[11px] text-claude-textSecondary">Port</div>
                    </div>
                    <div className="p-2 bg-white dark:bg-[#2B2A29] rounded-lg">
                      <div className="text-[18px] font-semibold text-claude-text">{ideStatus.active_connections}</div>
                      <div className="text-[11px] text-claude-textSecondary">{t('agent.active')}</div>
                    </div>
                    <div className="p-2 bg-white dark:bg-[#2B2A29] rounded-lg">
                      <div className="text-[18px] font-semibold text-claude-text">{ideStatus.total_connections}</div>
                      <div className="text-[11px] text-claude-textSecondary">{t('agent.files')}</div>
                    </div>
                  </div>
                )}
              </div>

              <div>
                <div className="flex items-center justify-between mb-3">
                  <span className="text-[13px] text-claude-textSecondary">{t('agent.files')}</span>
                  <button onClick={refresh} className="p-1.5 hover:bg-claude-hover rounded-lg text-claude-textSecondary hover:text-claude-text transition-colors">
                    <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
                  </button>
                </div>

                {ideConnections.length === 0 ? (
                  <div className="text-center py-6 text-claude-textSecondary text-[14px]">
                    {ideStatus?.server_running
                      ? t('agent.noConnections')
                      : t('agent.startSwarm')}
                  </div>
                ) : ideConnections.map(conn => (
                  <div key={conn.id} className="p-3 bg-claude-input border border-claude-border rounded-xl mb-2">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className={`w-2 h-2 rounded-full ${statusColors[conn.status] || 'bg-gray-400'}`} />
                        <Plug size={14} className="text-claude-textSecondary" />
                        <span className="text-[14px] font-medium text-claude-text">{conn.ide_type}</span>
                      </div>
                      <button onClick={() => handleIdeDisconnect(conn.id)} className="p-1 hover:bg-claude-hover rounded text-claude-textSecondary hover:text-red-500 transition-colors">
                        <X size={14} />
                      </button>
                    </div>
                    {conn.workspace && (
                      <div className="text-[12px] text-claude-textSecondary mt-1 ml-6">{conn.workspace}</div>
                    )}
                  </div>
                ))}
              </div>

              {ideStatus?.server_running && (
                <div className="p-3 bg-blue-50 dark:bg-blue-900/10 border border-blue-200 dark:border-blue-900/30 rounded-xl">
                  <div className="text-[13px] font-medium text-blue-700 dark:text-blue-400 mb-1">VS Code Extension Setup</div>
                  <div className="text-[12px] text-blue-600 dark:text-blue-300">
                    Install the Claude Tauri Bridge extension and configure it to connect to <code className="bg-blue-100 dark:bg-blue-900/30 px-1 rounded">127.0.0.1:{ideStatus.port}</code>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
