import React, { useState, useEffect } from 'react';
import { Plus, Play, Square, RotateCw, Trash2, Edit2, ChevronDown, ChevronRight, Wrench, Globe, Server } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface McpServer {
  id: string;
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  enabled: boolean;
  running: boolean;
  pid?: number;
  tools_count: number;
  resources_count: number;
  error?: string;
  transport_type: string;
}

interface McpTool {
  name: string;
  description: string;
  input_schema: any;
  server_name: string;
}

const McpSettingsPage = () => {
  const [servers, setServers] = useState<McpServer[]>([]);
  const [tools, setTools] = useState<McpTool[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingServer, setEditingServer] = useState<string | null>(null);
  const [expandedTools, setExpandedTools] = useState<Set<string>>(new Set());
  const [formData, setFormData] = useState({
    name: '',
    command: '',
    args: '',
    env: '',
    transport_type: 'stdio',
  });
  const [msg, setMsg] = useState('');
  const [msgType, setMsgType] = useState<'success' | 'error'>('success');

  useEffect(() => {
    loadServers();
    loadTools();
  }, []);

  const loadServers = async () => {
    try {
      console.log('[MCP] Calling mcp_list_servers...');
      const result = await invoke<McpServer[]>('mcp_list_servers');
      console.log('[MCP] mcp_list_servers returned:', result);
      setServers(result);
    } catch (e: unknown) {
      console.error('[MCP] Failed to load MCP servers:', e);
      setMsg(`Failed to load MCP servers: ${e}`);
      setMsgType('error');
    } finally {
      console.log('[MCP] Setting loading to false');
      setLoading(false);
    }
  };

  const loadTools = async () => {
    try {
      const result = await invoke<McpTool[]>('mcp_list_tools');
      setTools(result);
    } catch (e: unknown) {
      console.error('Failed to load MCP tools:', e);
    }
  };

  const showMessage = (message: string, type: 'success' | 'error' = 'success') => {
    setMsg(message);
    setMsgType(type);
    setTimeout(() => setMsg(''), 3000);
  };

  const handleStart = async (id: string) => {
    try {
      await invoke('mcp_start_server', { id });
      showMessage('Server started');
      loadServers();
    } catch (e: unknown) {
      showMessage(`Failed to start: ${e}`, 'error');
    }
  };

  const handleStop = async (id: string) => {
    try {
      await invoke('mcp_stop_server', { id });
      showMessage('Server stopped');
      loadServers();
    } catch (e: unknown) {
      showMessage(`Failed to stop: ${e}`, 'error');
    }
  };

  const handleRestart = async (id: string) => {
    try {
      await invoke('mcp_restart_server', { id });
      showMessage('Server restarted');
      loadServers();
    } catch (e: unknown) {
      showMessage(`Failed to restart: ${e}`, 'error');
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await invoke('mcp_toggle_server', { id, enabled });
      showMessage(enabled ? 'Server enabled' : 'Server disabled');
      loadServers();
    } catch (e: unknown) {
      showMessage(`Failed to toggle: ${e}`, 'error');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to remove this MCP server?')) return;
    try {
      await invoke('mcp_remove_server', { id });
      showMessage('Server removed');
      loadServers();
      loadTools();
    } catch (e: unknown) {
      showMessage(`Failed to remove: ${e}`, 'error');
    }
  };

  const handleEdit = (server: McpServer) => {
    setEditingServer(server.id);
    setFormData({
      name: server.name,
      command: server.command,
      args: server.args.join(' '),
      env: server.env ? JSON.stringify(server.env, null, 2) : '',
      transport_type: server.transport_type,
    });
    setShowAddForm(true);
  };

  const handleSubmit = async () => {
    if (!formData.name || !formData.command) {
      showMessage('Name and command are required', 'error');
      return;
    }

    try {
      const args = formData.args.trim() ? formData.args.trim().split(/\s+/) : [];
      const env = formData.env ? JSON.parse(formData.env) : undefined;

      if (editingServer) {
        await invoke('mcp_update_server', {
          id: editingServer,
          config: {
            id: editingServer,
            name: formData.name,
            command: formData.command,
            args,
            env,
            transport_type: formData.transport_type,
            enabled: true,
          },
        });
        showMessage('Server updated');
      } else {
        await invoke('mcp_add_server', {
          config: {
            id: formData.name.toLowerCase().replace(/[^a-z0-9-]/g, '-'),
            name: formData.name,
            command: formData.command,
            args,
            env,
            transport_type: formData.transport_type,
            enabled: true,
          },
        });
        showMessage('Server added');
      }

      setShowAddForm(false);
      setEditingServer(null);
      resetForm();
      loadServers();
      loadTools();
    } catch (e: unknown) {
      showMessage(`Failed to save: ${e}`, 'error');
    }
  };

  const resetForm = () => {
    setFormData({ name: '', command: '', args: '', env: '', transport_type: 'stdio' });
  };

  const toggleToolExpansion = (serverName: string) => {
    const newExpanded = new Set(expandedTools);
    if (newExpanded.has(serverName)) {
      newExpanded.delete(serverName);
    } else {
      newExpanded.add(serverName);
    }
    setExpandedTools(newExpanded);
  };

  const groupedTools = tools.reduce((acc, tool) => {
    if (!acc[tool.server_name]) {
      acc[tool.server_name] = [];
    }
    acc[tool.server_name].push(tool);
    return acc;
  }, {} as Record<string, McpTool[]>);

  if (loading) {
    return <div className="text-[14px] text-[#999] py-8">Loading MCP servers...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-[18px] font-medium text-claude-text">MCP Servers</h3>
        <button
          onClick={() => {
            resetForm();
            setEditingServer(null);
            setShowAddForm(true);
          }}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-claude-btn-hover text-white rounded-lg text-[14px] hover:opacity-90 transition-opacity"
        >
          <Plus size={16} />
          Add Server
        </button>
      </div>

      {msg && (
        <div className={`px-3 py-2 rounded-lg text-[14px] ${
          msgType === 'success' ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'
        }`}>
          {msg}
        </div>
      )}

      {/* Add/Edit Form */}
      {showAddForm && (
        <div className="border border-[#e5e5e5] rounded-xl p-4 space-y-4 bg-[#fafafa]">
          <h4 className="text-[16px] font-medium text-claude-text">
            {editingServer ? 'Edit Server' : 'Add MCP Server'}
          </h4>
          
          <div>
            <label className="block text-[13px] text-[#666] mb-1">Name</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 border border-[#e0e0e0] rounded-lg text-[14px] focus:outline-none focus:ring-2 focus:ring-[#D97757]/30"
              placeholder="my-mcp-server"
            />
          </div>

          <div>
            <label className="block text-[13px] text-[#666] mb-1">Command</label>
            <input
              type="text"
              value={formData.command}
              onChange={(e) => setFormData({ ...formData, command: e.target.value })}
              className="w-full px-3 py-2 border border-[#e0e0e0] rounded-lg text-[14px] focus:outline-none focus:ring-2 focus:ring-[#D97757]/30"
              placeholder="npx"
            />
          </div>

          <div>
            <label className="block text-[13px] text-[#666] mb-1">Arguments</label>
            <input
              type="text"
              value={formData.args}
              onChange={(e) => setFormData({ ...formData, args: e.target.value })}
              className="w-full px-3 py-2 border border-[#e0e0e0] rounded-lg text-[14px] focus:outline-none focus:ring-2 focus:ring-[#D97757]/30"
              placeholder="-y @modelcontextprotocol/server-filesystem /path"
            />
          </div>

          <div>
            <label className="block text-[13px] text-[#666] mb-1">Transport Type</label>
            <select
              value={formData.transport_type}
              onChange={(e) => setFormData({ ...formData, transport_type: e.target.value })}
              className="w-full px-3 py-2 border border-[#e0e0e0] rounded-lg text-[14px] focus:outline-none focus:ring-2 focus:ring-[#D97757]/30"
            >
              <option value="stdio">stdio</option>
              <option value="sse">SSE</option>
              <option value="http">HTTP</option>
            </select>
          </div>

          <div>
            <label className="block text-[13px] text-[#666] mb-1">Environment Variables (JSON)</label>
            <textarea
              value={formData.env}
              onChange={(e) => setFormData({ ...formData, env: e.target.value })}
              className="w-full px-3 py-2 border border-[#e0e0e0] rounded-lg text-[14px] font-mono focus:outline-none focus:ring-2 focus:ring-[#D97757]/30"
              rows={3}
              placeholder='{"API_KEY": "your-key"}'
            />
          </div>

          <div className="flex gap-2">
            <button
              onClick={handleSubmit}
              className="px-4 py-2 bg-claude-btn-hover text-white rounded-lg text-[14px] hover:opacity-90 transition-opacity"
            >
              {editingServer ? 'Update' : 'Add'}
            </button>
            <button
              onClick={() => {
                setShowAddForm(false);
                setEditingServer(null);
                resetForm();
              }}
              className="px-4 py-2 border border-[#e0e0e0] text-[#666] rounded-lg text-[14px] hover:bg-[#f5f5f5] transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Server List */}
      <div className="space-y-3">
        {servers.length === 0 ? (
          <div className="text-center py-12 text-[#999] text-[14px]">
            <Server size={48} className="mx-auto mb-3 opacity-40" />
            <p>No MCP servers configured</p>
            <p className="text-[12px] mt-1">Add a server to extend Claude with external tools</p>
          </div>
        ) : (
          servers.map((server) => (
            <div
              key={server.id}
              className={`border rounded-xl p-4 transition-colors ${
                server.enabled ? 'border-[#e5e5e5]' : 'border-[#e5e5e5] opacity-60'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className={`w-2.5 h-2.5 rounded-full ${
                    server.running ? 'bg-green-500' : server.enabled ? 'bg-gray-400' : 'bg-gray-300'
                  }`} />
                  <div>
                    <h4 className="text-[15px] font-medium text-claude-text">{server.name}</h4>
                    <p className="text-[12px] text-[#999] font-mono mt-0.5">
                      {server.command} {(server.args || []).join(' ')}
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-1.5">
                  <span className="text-[12px] text-[#999] mr-2">
                    {server.tools_count} tools
                    {server.resources_count > 0 && `, ${server.resources_count} resources`}
                  </span>

                  {server.running ? (
                    <button
                      onClick={() => handleStop(server.id)}
                      className="p-1.5 hover:bg-red-50 rounded-lg transition-colors text-red-500"
                      title="Stop"
                    >
                      <Square size={16} />
                    </button>
                  ) : (
                    <button
                      onClick={() => handleStart(server.id)}
                      disabled={!server.enabled}
                      className="p-1.5 hover:bg-green-50 rounded-lg transition-colors text-green-600 disabled:opacity-30"
                      title="Start"
                    >
                      <Play size={16} />
                    </button>
                  )}

                  {server.running && (
                    <button
                      onClick={() => handleRestart(server.id)}
                      className="p-1.5 hover:bg-blue-50 rounded-lg transition-colors text-blue-500"
                      title="Restart"
                    >
                      <RotateCw size={16} />
                    </button>
                  )}

                  <button
                    onClick={() => handleEdit(server)}
                    className="p-1.5 hover:bg-gray-100 rounded-lg transition-colors text-[#666]"
                    title="Edit"
                  >
                    <Edit2 size={14} />
                  </button>

                  <button
                    onClick={() => handleDelete(server.id)}
                    className="p-1.5 hover:bg-red-50 rounded-lg transition-colors text-red-500"
                    title="Delete"
                  >
                    <Trash2 size={14} />
                  </button>

                  <label className="flex items-center gap-1.5 ml-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={server.enabled}
                      onChange={(e) => handleToggle(server.id, e.target.checked)}
                      className="w-4 h-4 rounded border-[#d0d0d0] text-[#D97757] focus:ring-[#D97757]/30"
                    />
                    <span className="text-[12px] text-[#666]">Enabled</span>
                  </label>
                </div>
              </div>

              {server.error && (
                <div className="mt-2 px-3 py-1.5 bg-red-50 text-red-600 rounded-lg text-[12px]">
                  Error: {server.error}
                </div>
              )}
            </div>
          ))
        )}
      </div>

      {/* Tools Section */}
      {tools.length > 0 && (
        <div className="border-t border-[#e5e5e5] pt-6">
          <h3 className="text-[18px] font-medium text-claude-text mb-4 flex items-center gap-2">
            <Wrench size={20} />
            Available MCP Tools ({tools.length})
          </h3>

          <div className="space-y-4">
            {Object.entries(groupedTools).map(([serverName, serverTools]) => (
              <div key={serverName} className="border border-[#e5e5e5] rounded-xl overflow-hidden">
                <button
                  onClick={() => toggleToolExpansion(serverName)}
                  className="w-full flex items-center justify-between px-4 py-3 bg-[#fafafa] hover:bg-[#f5f5f5] transition-colors"
                >
                  <div className="flex items-center gap-2">
                    {expandedTools.has(serverName) ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                    <span className="text-[14px] font-medium text-claude-text">{serverName}</span>
                    <span className="text-[12px] text-[#999]">{serverTools.length} tools</span>
                  </div>
                </button>

                {expandedTools.has(serverName) && (
                  <div className="divide-y divide-[#f0f0f0]">
                    {serverTools.map((tool) => (
                      <div key={tool.name} className="px-4 py-3">
                        <h5 className="text-[14px] font-mono text-claude-text">{tool.name}</h5>
                        {tool.description && (
                          <p className="text-[13px] text-[#666] mt-1">{tool.description}</p>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Quick Add Templates */}
      {!showAddForm && servers.length === 0 && (
        <div className="border border-[#e5e5e5] rounded-xl p-4">
          <h4 className="text-[14px] font-medium text-claude-text mb-3">Quick Add Templates</h4>
          <div className="grid grid-cols-1 gap-2">
            {[
              { name: 'Filesystem', command: 'npx', args: '-y @modelcontextprotocol/server-filesystem .' },
              { name: 'GitHub', command: 'npx', args: '-y @modelcontextprotocol/server-github' },
              { name: 'PostgreSQL', command: 'npx', args: '-y @modelcontextprotocol/server-postgres postgresql://localhost/mydb' },
              { name: 'Puppeteer', command: 'npx', args: '-y @modelcontextprotocol/server-puppeteer' },
            ].map((template) => (
              <button
                key={template.name}
                onClick={() => {
                  setFormData({
                    name: template.name.toLowerCase(),
                    command: template.command,
                    args: template.args,
                    env: '',
                    transport_type: 'stdio',
                  });
                  setShowAddForm(true);
                }}
                className="flex items-center gap-3 px-3 py-2.5 border border-[#e5e5e5] rounded-lg hover:bg-[#f9f9f9] transition-colors text-left"
              >
                <Globe size={16} className="text-[#999]" />
                <div>
                  <span className="text-[14px] font-medium text-claude-text">{template.name}</span>
                  <p className="text-[12px] text-[#999] font-mono">{template.command} {template.args}</p>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default McpSettingsPage;
