import { invoke } from '@tauri-apps/api/core';

const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

export const tauriAPI = {
  isTauri,

  async getPlatform(): Promise<{ os: string; arch: string; is_electron: boolean }> {
    if (!isTauri) return { os: 'web', arch: 'unknown', is_electron: false };
    return invoke('get_platform');
  },

  async getAppPath(): Promise<string> {
    if (!isTauri) return '';
    return invoke('get_app_path');
  },

  async selectDirectory(): Promise<string | null> {
    if (!isTauri) return null;
    return invoke('select_directory');
  },

  async showItemInFolder(path: string): Promise<void> {
    if (!isTauri) return;
    return invoke('show_item_in_folder', { path });
  },

  async openFolder(path: string): Promise<void> {
    if (!isTauri) return;
    return invoke('open_folder', { path });
  },

  async openExternal(url: string): Promise<void> {
    if (!isTauri) {
      window.open(url, '_blank');
      return;
    }
    return invoke('open_external_url', { url });
  },

  async resizeWindow(width: number, height: number): Promise<void> {
    if (!isTauri) return;
    return invoke('resize_window', { width, height });
  },

  async exportWorkspace(
    workspaceId: string,
    contextMarkdown: string,
    defaultFilename: string
  ): Promise<string> {
    if (!isTauri) return '';
    return invoke('export_workspace', { workspaceId, contextMarkdown, defaultFilename });
  },

  async getSystemStatus(): Promise<{
    platform: string;
    git_bash: { required: boolean; found: boolean; path: string | null };
  }> {
    if (!isTauri) {
      return {
        platform: 'web',
        git_bash: { required: false, found: false, path: null },
      };
    }
    return invoke('get_system_status');
  },

  async executeTool(
    name: string,
    input: any,
    cwd?: string
  ): Promise<any> {
    if (!isTauri) return null;
    return invoke('execute_tool', { name, input, cwd });
  },

  async checkUpdate(): Promise<{ has_update: boolean }> {
    if (!isTauri) return { has_update: false };
    return invoke('check_update');
  },

  async installUpdate(): Promise<void> {
    if (!isTauri) return;
    return invoke('install_update');
  },
};
