import React, { useState, useEffect } from 'react';
import { Server, Plus, Trash2, Power, PowerOff, Settings, Check, X, Loader2, Terminal } from 'lucide-react';
import { detectBridgePort } from '../api';

interface McpServer {
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
}

interface McpTool {
  name: string;
  description: string;
}

interface McpServerStatus {
  name: string;
  server_installed: boolean;
  tools: McpTool[];
  resources: any[];
  error: string | null;
}

export default function McpManagementPanel({ onClose }: { onClose: () => void }) {
  const [servers, setServers] = useState<McpServer[]>([]);
  const [statuses, setStatuses] = useState<Record<string, McpServerStatus>>({});
  const [loading, setLoading] = useState(false);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newServer, setNewServer] = useState<Partial<McpServer>>({
    name: '',
    command: '',
    args: [],
    env: {},
    enabled: true,
  });
  const [argInput, setArgInput] = useState('');
  const [envKeyInput, setEnvKeyInput] = useState('');
  const [envValueInput, setEnvValueInput] = useState('');

  useEffect(() => {
    loadServers();
  }, []);

  const loadServers = async () => {
    setLoading(true);
    try {
      const port = await detectBridgePort();
      const response = await fetch(`http://127.0.0.1:${port}/api/mcp/servers`, {
        headers: { 'Authorization': `Bearer ${localStorage.getItem('auth_token')}` },
      });
      if (response.ok) {
        const data = await response.json();
        setServers(data.servers || []);
      }
    } catch (e) {
      console.error('Failed to load MCP servers:', e);
    } finally {
      setLoading(false);
    }
  };

  const getConfigPath = async (): Promise<string> => {
    const appData = await (window as any).__TAURI_INTERNALS__?.invoke('plugin:dialog|open') || '';
    return appData;
  };

  const handleAddServer = async () => {
    if (!newServer.name || !newServer.command) return;

    const server: McpServer = {
      name: newServer.name,
      command: newServer.command,
      args: newServer.args || [],
      env: newServer.env || {},
      enabled: newServer.enabled ?? true,
    };

    try {
      const port = await detectBridgePort();
      await fetch(`http://127.0.0.1:${port}/api/mcp/servers`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`,
        },
        body: JSON.stringify(server),
      });
      setNewServer({ name: '', command: '', args: [], env: {}, enabled: true });
      setArgInput('');
      setEnvKeyInput('');
      setEnvValueInput('');
      setShowAddForm(false);
      loadServers();
    } catch (e) {
      console.error('Failed to add MCP server:', e);
    }
  };

  const handleRemoveServer = async (name: string) => {
    try {
      const port = await detectBridgePort();
      await fetch(`http://127.0.0.1:${port}/api/mcp/servers/${encodeURIComponent(name)}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${localStorage.getItem('auth_token')}` },
      });
      loadServers();
    } catch (e) {
      console.error('Failed to remove MCP server:', e);
    }
  };

  const handleToggleServer = async (name: string, enabled: boolean) => {
    try {
      const port = await detectBridgePort();
      await fetch(`http://127.0.0.1:${port}/api/mcp/servers/${encodeURIComponent(name)}/toggle`, {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`,
        },
        body: JSON.stringify({ enabled: !enabled }),
      });
      loadServers();
    } catch (e) {
      console.error('Failed to toggle MCP server:', e);
    }
  };

  const addArg = () => {
    if (argInput.trim()) {
      setNewServer(prev => ({
        ...prev,
        args: [...(prev.args || []), argInput.trim()],
      }));
      setArgInput('');
    }
  };

  const removeArg = (index: number) => {
    setNewServer(prev => ({
      ...prev,
      args: prev.args?.filter((_, i) => i !== index) || [],
    }));
  };

  const addEnv = () => {
    if (envKeyInput.trim()) {
      setNewServer(prev => ({
        ...prev,
        env: { ...(prev.env || {}), [envKeyInput.trim()]: envValueInput },
      }));
      setEnvKeyInput('');
      setEnvValueInput('');
    }
  };

  const removeEnv = (key: string) => {
    setNewServer(prev => {
      const newEnv = { ...prev.env };
      delete newEnv[key];
      return { ...prev, env: newEnv };
    });
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/40" onClick={onClose}>
      <div
        className="w-[600px] max-h-[80vh] bg-white dark:bg-[#2A2928] border border-claude-border rounded-xl shadow-2xl flex flex-col overflow-hidden"
        onClick={e => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-5 py-4 border-b border-claude-border">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-500/10 rounded-lg">
              <Server size={20} className="text-blue-500" />
            </div>
            <div>
              <h2 className="text-[16px] font-semibold text-claude-text">MCP Servers</h2>
              <p className="text-[12px] text-claude-textSecondary">Manage Model Context Protocol servers</p>
            </div>
          </div>
          <button onClick={onClose} className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-lg transition-colors">
            <X size={18} className="text-claude-textSecondary" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-5">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 size={24} className="animate-spin text-claude-textSecondary" />
            </div>
          ) : servers.length === 0 ? (
            <div className="text-center py-12">
              <Server size={40} className="mx-auto text-claude-textSecondary/30 mb-3" />
              <p className="text-claude-textSecondary text-[14px]">No MCP servers configured</p>
              <p className="text-claude-textSecondary/60 text-[12px] mt-1">Add a server to extend Claude's capabilities</p>
            </div>
          ) : (
            <div className="space-y-3">
              {servers.map((server) => (
                <div key={server.name} className="border border-claude-border rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <Terminal size={14} className="text-claude-textSecondary" />
                      <span className="text-[14px] font-medium text-claude-text">{server.name}</span>
                      <span className={`px-1.5 py-0.5 text-[10px] rounded-full ${
                        server.enabled
                          ? 'bg-green-500/10 text-green-600 dark:text-green-400'
                          : 'bg-gray-500/10 text-gray-500'
                      }`}>
                        {server.enabled ? 'Active' : 'Disabled'}
                      </span>
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => handleToggleServer(server.name, server.enabled)}
                        className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-md transition-colors"
                        title={server.enabled ? 'Disable' : 'Enable'}
                      >
                        {server.enabled ? <PowerOff size={14} className="text-claude-textSecondary" /> : <Power size={14} className="text-green-500" />}
                      </button>
                      <button
                        onClick={() => handleRemoveServer(server.name)}
                        className="p-1.5 hover:bg-red-500/10 rounded-md transition-colors"
                        title="Remove"
                      >
                        <Trash2 size={14} className="text-red-500" />
                      </button>
                    </div>
                  </div>
                  <div className="text-[12px] text-claude-textSecondary font-mono bg-claude-hover/50 rounded px-2 py-1">
                    {server.command} {server.args.join(' ')}
                  </div>
                  {Object.keys(server.env).length > 0 && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {Object.entries(server.env).map(([key, value]) => (
                        <span key={key} className="text-[10px] px-1.5 py-0.5 bg-blue-500/10 text-blue-600 dark:text-blue-400 rounded">
                          {key}={value}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {showAddForm && (
            <div className="mt-4 border border-claude-border rounded-lg p-4 bg-claude-hover/30">
              <h3 className="text-[14px] font-medium text-claude-text mb-3">Add MCP Server</h3>

              <div className="space-y-3">
                <div>
                  <label className="text-[12px] text-claude-textSecondary block mb-1">Name</label>
                  <input
                    type="text"
                    value={newServer.name}
                    onChange={e => setNewServer(prev => ({ ...prev, name: e.target.value }))}
                    className="w-full px-3 py-2 bg-transparent border border-claude-border rounded-lg text-claude-text text-[13px] focus:outline-none focus:border-blue-500"
                    placeholder="my-server"
                  />
                </div>

                <div>
                  <label className="text-[12px] text-claude-textSecondary block mb-1">Command</label>
                  <input
                    type="text"
                    value={newServer.command}
                    onChange={e => setNewServer(prev => ({ ...prev, command: e.target.value }))}
                    className="w-full px-3 py-2 bg-transparent border border-claude-border rounded-lg text-claude-text text-[13px] focus:outline-none focus:border-blue-500"
                    placeholder="npx -y @modelcontextprotocol/server-filesystem"
                  />
                </div>

                <div>
                  <label className="text-[12px] text-claude-textSecondary block mb-1">Arguments</label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={argInput}
                      onChange={e => setArgInput(e.target.value)}
                      onKeyDown={e => { if (e.key === 'Enter') { e.preventDefault(); addArg(); } }}
                      className="flex-1 px-3 py-2 bg-transparent border border-claude-border rounded-lg text-claude-text text-[13px] focus:outline-none focus:border-blue-500"
                      placeholder="--path /some/dir"
                    />
                    <button onClick={addArg} className="px-3 py-2 bg-blue-500 text-white rounded-lg text-[12px] hover:bg-blue-600">
                      <Plus size={14} />
                    </button>
                  </div>
                  {newServer.args && newServer.args.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-2">
                      {newServer.args.map((arg, i) => (
                        <span key={i} className="flex items-center gap-1 text-[11px] px-2 py-0.5 bg-claude-hover rounded">
                          {arg}
                          <button onClick={() => removeArg(i)} className="hover:text-red-500"><X size={10} /></button>
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                <div>
                  <label className="text-[12px] text-claude-textSecondary block mb-1">Environment Variables</label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={envKeyInput}
                      onChange={e => setEnvKeyInput(e.target.value)}
                      className="w-1/3 px-3 py-2 bg-transparent border border-claude-border rounded-lg text-claude-text text-[13px] focus:outline-none focus:border-blue-500"
                      placeholder="KEY"
                    />
                    <input
                      type="text"
                      value={envValueInput}
                      onChange={e => setEnvValueInput(e.target.value)}
                      onKeyDown={e => { if (e.key === 'Enter') { e.preventDefault(); addEnv(); } }}
                      className="flex-1 px-3 py-2 bg-transparent border border-claude-border rounded-lg text-claude-text text-[13px] focus:outline-none focus:border-blue-500"
                      placeholder="value"
                    />
                    <button onClick={addEnv} className="px-3 py-2 bg-blue-500 text-white rounded-lg text-[12px] hover:bg-blue-600">
                      <Plus size={14} />
                    </button>
                  </div>
                  {newServer.env && Object.keys(newServer.env).length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-2">
                      {Object.entries(newServer.env).map(([key, value]) => (
                        <span key={key} className="flex items-center gap-1 text-[11px] px-2 py-0.5 bg-blue-500/10 text-blue-600 dark:text-blue-400 rounded">
                          {key}={value}
                          <button onClick={() => removeEnv(key)} className="hover:text-red-500"><X size={10} /></button>
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                <div className="flex justify-end gap-2 pt-2">
                  <button
                    onClick={() => setShowAddForm(false)}
                    className="px-4 py-2 text-[13px] text-claude-textSecondary hover:text-claude-text transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleAddServer}
                    disabled={!newServer.name || !newServer.command}
                    className="px-4 py-2 text-[13px] text-white bg-blue-500 hover:bg-blue-600 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Add Server
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>

        <div className="px-5 py-3 border-t border-claude-border flex items-center justify-between">
          <span className="text-[12px] text-claude-textSecondary">{servers.length} server{servers.length !== 1 ? 's' : ''} configured</span>
          <button
            onClick={() => setShowAddForm(!showAddForm)}
            className="flex items-center gap-2 px-3 py-1.5 text-[13px] text-white bg-blue-500 hover:bg-blue-600 rounded-lg transition-colors"
          >
            <Plus size={14} />
            Add Server
          </button>
        </div>
      </div>
    </div>
  );
}
