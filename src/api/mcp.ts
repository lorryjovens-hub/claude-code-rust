import { invoke } from '@tauri-apps/api/core';

export interface McpServerConfig {
  id: string;
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
}

export interface McpServerStatus {
  id: string;
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  enabled: boolean;
  running: boolean;
  pid: number | null;
  tools_count: number;
  resources_count: number;
  error: string | null;
  transport_type: string;
}

export interface McpTool {
  name: string;
  description: string;
  input_schema: unknown;
  server_name: string;
}

export interface McpResource {
  uri: string;
  name: string;
  mime_type: string | null;
}

export interface McpResourceContent {
  uri: string;
  content: string;
  content_type: string;
  metadata?: unknown;
}

export interface McpHealthSummary {
  total: number;
  connected: number;
  disconnected: number;
  errored: number;
  details: Array<{
    id: string;
    status: string;
    latency_ms: number;
  }>;
}

export async function listMcpServers(): Promise<McpServerStatus[]> {
  return invoke('list_mcp_servers');
}

export async function addMcpServer(config: McpServerConfig): Promise<void> {
  return invoke('add_mcp_server', { config });
}

export async function updateMcpServer(id: string, config: McpServerConfig): Promise<void> {
  return invoke('update_mcp_server', { id, config });
}

export async function removeMcpServer(id: string): Promise<void> {
  return invoke('remove_mcp_server', { id });
}

export async function startMcpServer(id: string): Promise<void> {
  return invoke('start_mcp_server', { id });
}

export async function stopMcpServer(id: string): Promise<void> {
  return invoke('stop_mcp_server', { id });
}

export async function restartMcpServer(id: string): Promise<void> {
  return invoke('restart_mcp_server', { id });
}

export async function toggleMcpServer(id: string): Promise<void> {
  return invoke('toggle_mcp_server', { id });
}

export async function setMcpServerEnabled(id: string, enabled: boolean): Promise<void> {
  return invoke('set_mcp_server_enabled', { id, enabled });
}

export async function listMcpTools(): Promise<McpTool[]> {
  return invoke('list_mcp_tools');
}

export async function listMcpResources(): Promise<McpResource[]> {
  return invoke('list_mcp_resources');
}

export async function getMcpTools(serverId: string): Promise<McpTool[]> {
  return invoke('get_mcp_tools', { serverId });
}

export async function getMcpResources(serverId: string): Promise<McpResource[]> {
  return invoke('get_mcp_resources', { serverId });
}

export async function readMcpResource(
  serverId: string,
  uri: string,
  options?: unknown,
): Promise<McpResourceContent> {
  return invoke('read_mcp_resource', { serverId, uri, options });
}

export async function monitorMcpResource(
  serverId: string,
  uri: string,
  enabled: boolean,
): Promise<boolean> {
  return invoke('monitor_mcp_resource', { serverId, uri, enabled });
}

export async function callMcpTool(
  serverId: string,
  toolName: string,
  arguments: unknown,
): Promise<unknown> {
  return invoke('call_mcp_tool', { serverId, toolName, arguments });
}

export async function getMcpHealth(): Promise<McpHealthSummary> {
  return invoke('get_mcp_health');
}

export async function discoverMcpServers(): Promise<McpServerStatus[]> {
  return invoke('discover_mcp_servers');
}

export async function connectMcpServer(serverId: string): Promise<void> {
  return invoke('connect_mcp_server', { serverId });
}

export async function disconnectMcpServer(serverId: string): Promise<void> {
  return invoke('disconnect_mcp_server', { serverId });
}