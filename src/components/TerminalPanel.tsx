import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Terminal, Plus, X, RefreshCw } from 'lucide-react';
import { Terminal as XTerm } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { createTerminal, writeTerminal, resizeTerminal, closeTerminal, streamTerminalOutput } from '../api';
import 'xterm/css/xterm.css';

interface TerminalTab {
  id: string;
  title: string;
  shell: string;
  cwd: string;
  xterm: XTerm | null;
  fitAddon: FitAddon | null;
  cleanupStream: (() => void) | null;
  abortController: AbortController | null;
}

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  tabId: string;
}

const TerminalPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [tabs, setTabs] = useState<TerminalTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [contextMenu, setContextMenu] = useState<ContextMenuState>({ visible: false, x: 0, y: 0, tabId: '' });
  const [renamingTabId, setRenamingTabId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const renameInputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const initializedRef = useRef(false);

  const activeTab = tabs.find(t => t.id === activeTabId) || null;

  const createNewTerminal = useCallback(async () => {
    if (creating) return;
    setCreating(true);
    try {
      const result = await createTerminal();
      const tabId = result.id;
      const controller = new AbortController();

      const xterm = new XTerm({
        cursorBlink: true,
        fontSize: 13,
        fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", Menlo, Monaco, "Courier New", monospace',
        theme: {
          background: '#1a1a2e',
          foreground: '#e0e0e8',
          cursor: '#d97706',
          selectionBackground: '#d9770633',
          black: '#45475a',
          red: '#f38ba8',
          green: '#a6e3a1',
          yellow: '#f9e2af',
          blue: '#89b4fa',
          magenta: '#f5c2e7',
          cyan: '#94e2d5',
          white: '#bac2de',
          brightBlack: '#585b70',
          brightRed: '#f38ba8',
          brightGreen: '#a6e3a1',
          brightYellow: '#f9e2af',
          brightBlue: '#89b4fa',
          brightMagenta: '#f5c2e7',
          brightCyan: '#94e2d5',
          brightWhite: '#a6adc8',
        },
        allowProposedApi: true,
      });

      const fitAddon = new FitAddon();
      xterm.loadAddon(fitAddon);

      const tab: TerminalTab = {
        id: tabId,
        title: 'Terminal',
        shell: 'bash',
        cwd: '~',
        xterm: null,
        fitAddon,
        cleanupStream: null,
        abortController: controller,
      };

      setTabs(prev => [...prev, tab]);
      setActiveTabId(tabId);

      setTimeout(() => {
        const container = document.getElementById(`terminal-${tabId}`);
        if (container) {
          try {
            xterm.open(container);
            fitAddon.fit();

            xterm.focus();

            container.setAttribute('tabindex', '0');
            container.style.outline = 'none';

            container.addEventListener('click', () => {
              xterm.focus();
            });

            xterm.onData(data => {
              writeTerminal(tabId, data).catch(() => {});
            });

            xterm.onResize(({ cols, rows }) => {
              resizeTerminal(tabId, cols, rows).catch(() => {});
            });

            const cleanup = streamTerminalOutput(
              tabId,
              (data) => {
                try {
                  xterm.write(data);
                } catch (err) {
                  console.error('[Terminal] Write error:', err);
                }
              },
              (code) => {
                xterm.writeln(`\r\n\x1b[33m[Process exited with code ${code ?? 0}]\x1b[0m\r\n`);
              },
              (err) => {
                xterm.writeln(`\r\n\x1b[31m[Error: ${err}]\x1b[0m\r\n`);
              },
              controller.signal
            );

            setTabs(prev => prev.map(t => t.id === tabId ? { ...t, xterm, cleanupStream: cleanup } : t));
          } catch (err) {
            console.error('[Terminal] Failed to initialize terminal:', err);
          }
        }
      }, 100);

    } catch (err) {
      console.error('Failed to create terminal:', err);
    } finally {
      setCreating(false);
    }
  }, [creating]);

  useEffect(() => {
    if (!initializedRef.current) {
      initializedRef.current = true;
      createNewTerminal();
    }
  }, [createNewTerminal]);

  useEffect(() => {
    if (activeTabId) {
      setTimeout(() => {
        const tab = tabs.find(t => t.id === activeTabId);
        if (tab?.fitAddon) {
          try {
            tab.fitAddon.fit();
            tab.xterm?.focus();
          } catch (err) {
            console.error('[Terminal] Error focusing terminal on tab switch:', err);
          }
        }
      }, 50);
    }
  }, [activeTabId, tabs.length]);

  const closeTab = useCallback((tabId: string, e?: React.MouseEvent) => {
    e?.stopPropagation();
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return;

    if (tab.cleanupStream) tab.cleanupStream();
    if (tab.abortController) tab.abortController.abort();
    if (tab.xterm) tab.xterm.dispose();
    closeTerminal(tabId).catch(() => {});

    setTabs(prev => prev.filter(t => t.id !== tabId));

    if (activeTabId === tabId) {
      const remaining = tabs.filter(t => t.id !== tabId);
      setActiveTabId(remaining.length > 0 ? remaining[remaining.length - 1].id : null);
    }
  }, [tabs, activeTabId]);

  const switchTab = useCallback((tabId: string) => {
    setActiveTabId(tabId);
  }, []);

  const handleTabContextMenu = useCallback((e: React.MouseEvent, tabId: string) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({
      visible: true,
      x: e.clientX,
      y: e.clientY,
      tabId,
    });
  }, []);

  const handleContextMenuClose = useCallback(() => {
    if (renamingTabId) {
      finishRename();
    }
    setContextMenu({ visible: false, x: 0, y: 0, tabId: '' });
    setRenamingTabId(null);
  }, [renamingTabId]);

  const handleContextMenuCloseTab = useCallback(() => {
    if (contextMenu.tabId) {
      closeTab(contextMenu.tabId);
    }
    handleContextMenuClose();
  }, [contextMenu.tabId, closeTab, handleContextMenuClose]);

  const handleContextMenuRename = useCallback(() => {
    if (contextMenu.tabId) {
      const tab = tabs.find(t => t.id === contextMenu.tabId);
      setRenamingTabId(contextMenu.tabId);
      setRenameValue(tab?.title || 'Terminal');
      setContextMenu(prev => ({ ...prev, visible: false }));
    }
  }, [contextMenu.tabId, tabs]);

  const finishRename = useCallback(() => {
    if (renamingTabId) {
      const newTitle = renameValue.trim() || 'Terminal';
      setTabs(prev => prev.map(t => t.id === renamingTabId ? { ...t, title: newTitle } : t));
      setRenamingTabId(null);
      setRenameValue('');
    }
  }, [renamingTabId, renameValue]);

  const handleRenameKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      finishRename();
    } else if (e.key === 'Escape') {
      setRenamingTabId(null);
      setRenameValue('');
    }
  }, [finishRename]);

  useEffect(() => {
    if (renamingTabId && renameInputRef.current) {
      renameInputRef.current.focus();
      renameInputRef.current.select();
    }
  }, [renamingTabId]);

  useEffect(() => {
    const handleClickOutside = () => {
      if (contextMenu.visible) {
        handleContextMenuClose();
      }
    };
    if (contextMenu.visible) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [contextMenu.visible, handleContextMenuClose]);

  if (tabs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full bg-[#1a1a2e] gap-4">
        <div className="flex flex-col items-center gap-3">
          <Terminal size={32} className="text-[#6c7086]" />
          <span className="text-sm text-[#6c7086]">No terminals open</span>
          <span className="text-xs text-[#585b70]">Press <kbd className="px-1.5 py-0.5 text-[11px] bg-[#16213e] border border-[#2d2d44] rounded text-[#a6adc8] font-mono">Ctrl+`</kbd> to toggle</span>
        </div>
        <button
          onClick={createNewTerminal}
          disabled={creating}
          className="flex items-center gap-2 px-4 py-2 bg-[#313244] hover:bg-[#45475a] text-[#e0e0e8] rounded-lg transition-colors disabled:opacity-50"
        >
          {creating ? <RefreshCw size={16} className="animate-spin" /> : <Plus size={16} />}
          New Terminal
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-[#1a1a2e]">
      <div className="flex items-center bg-[#16213e] border-b border-[#2d2d44] px-2 min-h-[36px] shrink-0">
        <div className="flex items-center gap-1 flex-1 overflow-x-auto scrollbar-hide">
          {tabs.map(tab => (
            <div key={tab.id} className="shrink-0">
              {renamingTabId === tab.id ? (
                <input
                  ref={renameInputRef}
                  type="text"
                  value={renameValue}
                  onChange={(e) => setRenameValue(e.target.value)}
                  onKeyDown={handleRenameKeyDown}
                  onBlur={finishRename}
                  className="w-[100px] px-2 py-1 text-xs bg-[#1a1a2e] text-[#e0e0e8] border border-[#d97706] rounded-md outline-none"
                />
              ) : (
                <button
                  onClick={() => switchTab(tab.id)}
                  onContextMenu={(e) => handleTabContextMenu(e, tab.id)}
                  className={`flex items-center gap-1.5 px-2.5 py-1 text-xs rounded-md transition-colors group shrink-0 ${
                    tab.id === activeTabId
                      ? 'bg-[#1a1a2e] text-[#d97706] border-b-2 border-[#d97706] rounded-b-none'
                      : 'text-[#6c7086] hover:text-[#a6adc8] hover:bg-[#1a1a2e]/50'
                  }`}
                >
                  <Terminal size={12} className={tab.id === activeTabId ? 'text-[#d97706]' : ''} />
                  <span className="max-w-[100px] truncate">{tab.title}</span>
                  <span
                    onClick={(e) => closeTab(tab.id, e)}
                    className="ml-0.5 p-0.5 rounded hover:bg-[#2d2d44] opacity-0 group-hover:opacity-100 transition-opacity"
                  >
                    <X size={10} />
                  </span>
                </button>
              )}
            </div>
          ))}
          <button
            onClick={createNewTerminal}
            disabled={creating}
            className="p-1 text-[#6c7086] hover:text-[#e0e0e8] transition-colors disabled:opacity-50 shrink-0"
            title="New Terminal"
          >
            {creating ? <RefreshCw size={14} className="animate-spin" /> : <Plus size={14} />}
          </button>
        </div>
        <div className="flex items-center gap-1 shrink-0 ml-2">
          <span className="text-[11px] text-[#585b70] bg-[#1a1a2e] px-1.5 py-0.5 rounded font-mono">
            {tabs.length}
          </span>
          <button
            onClick={onClose}
            className="p-1.5 text-[#6c7086] hover:text-[#e0e0e8] transition-colors shrink-0"
            title="Close"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-hidden relative">
        {tabs.map(tab => (
          <div
            key={tab.id}
            ref={containerRef}
            id={`terminal-${tab.id}`}
            className={`absolute inset-0 ${tab.id !== activeTabId ? 'hidden' : ''}`}
          />
        ))}
      </div>

      {contextMenu.visible && (
        <div
          className="fixed z-[100] min-w-[140px] bg-[#16213e] border border-[#2d2d44] rounded-lg shadow-xl py-1 overflow-hidden"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={(e) => e.stopPropagation()}
        >
          <button
            onClick={handleContextMenuRename}
            className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-[#e0e0e8] hover:bg-[#1a1a2e] transition-colors text-left"
          >
            Rename Tab
          </button>
          <button
            onClick={handleContextMenuCloseTab}
            className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-[#f38ba8] hover:bg-[#1a1a2e] transition-colors text-left"
          >
            Close Tab
          </button>
        </div>
      )}
    </div>
  );
};

export default TerminalPanel;