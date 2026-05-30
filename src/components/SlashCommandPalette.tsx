import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Search, Terminal, BarChart3, Settings, GitBranch, Layers, Zap, HelpCircle, ChevronRight } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface SlashCommand {
  name: string;
  description: string;
  category: string;
}

interface SlashCommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (command: string) => void;
  inputValue: string;
}

const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  general: <Terminal size={14} />,
  context: <GitBranch size={14} />,
  analytics: <BarChart3 size={14} />,
  settings: <Settings size={14} />,
};

const CATEGORY_COLORS: Record<string, string> = {
  general: 'text-blue-500',
  context: 'text-purple-500',
  analytics: 'text-green-500',
  settings: 'text-orange-500',
};

export default function SlashCommandPalette({ isOpen, onClose, onSelect, inputValue }: SlashCommandPaletteProps) {
  const [commands, setCommands] = useState<SlashCommand[]>([]);
  const [filteredCommands, setFilteredCommands] = useState<SlashCommand[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isOpen) {
      loadCommands();
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const loadCommands = async () => {
    setLoading(true);
    try {
      const result = await invoke<SlashCommand[]>('list_slash_commands');
      setCommands(result);
      setFilteredCommands(result);
    } catch (e) {
      console.error('Failed to load slash commands:', e);
      setCommands([]);
      setFilteredCommands([]);
    } finally {
      setLoading(false);
    }
  };

  const filterCommands = useCallback((query: string) => {
    const q = query.toLowerCase().trim();
    if (!q) {
      setFilteredCommands(commands);
    } else {
      setFilteredCommands(
        commands.filter(cmd =>
          cmd.name.toLowerCase().includes(q) ||
          cmd.description.toLowerCase().includes(q)
        )
      );
    }
    setSelectedIndex(0);
  }, [commands]);

  useEffect(() => {
    if (isOpen && inputValue) {
      filterCommands(inputValue);
    }
  }, [inputValue, isOpen, filterCommands]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(prev => Math.min(prev + 1, filteredCommands.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(prev => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filteredCommands[selectedIndex]) {
        onSelect(filteredCommands[selectedIndex].name);
      }
    } else if (e.key === 'Escape') {
      onClose();
    }
  };

  useEffect(() => {
    if (listRef.current) {
      const selectedEl = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedEl) {
        selectedEl.scrollIntoView({ block: 'nearest' });
      }
    }
  }, [selectedIndex]);

  if (!isOpen) return null;

  const groupedCommands = filteredCommands.reduce((acc, cmd) => {
    if (!acc[cmd.category]) acc[cmd.category] = [];
    acc[cmd.category].push(cmd);
    return acc;
  }, {} as Record<string, SlashCommand[]>);

  return (
    <div className="fixed inset-0 z-[200] flex items-start justify-center pt-[20vh] bg-black/30" onClick={onClose}>
      <div
        className="w-[520px] max-h-[400px] bg-white dark:bg-[#2A2928] border border-claude-border rounded-xl shadow-2xl overflow-hidden flex flex-col animate-fade-in"
        onClick={e => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 px-4 py-3 border-b border-claude-border">
          <Search size={16} className="text-claude-textSecondary" />
          <input
            ref={inputRef}
            type="text"
            placeholder="Type a command..."
            value={inputValue}
            onChange={e => filterCommands(e.target.value)}
            onKeyDown={handleKeyDown}
            className="flex-1 bg-transparent text-claude-text placeholder-claude-textSecondary focus:outline-none text-[14px]"
          />
          <kbd className="px-1.5 py-0.5 text-[11px] text-claude-textSecondary bg-claude-hover rounded border border-claude-border">ESC</kbd>
        </div>

        <div ref={listRef} className="flex-1 overflow-y-auto py-1">
          {loading ? (
            <div className="flex items-center justify-center py-8 text-claude-textSecondary text-[13px]">
              Loading commands...
            </div>
          ) : filteredCommands.length === 0 ? (
            <div className="flex items-center justify-center py-8 text-claude-textSecondary text-[13px]">
              No commands found
            </div>
          ) : (
            Object.entries(groupedCommands).map(([category, cmds]) => (
              <div key={category}>
                <div className="px-4 py-1.5 text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider flex items-center gap-1.5">
                  {CATEGORY_ICONS[category] || <Zap size={14} />}
                  {category}
                </div>
                {cmds.map((cmd) => {
                  const globalIndex = filteredCommands.indexOf(cmd);
                  const isSelected = globalIndex === selectedIndex;
                  return (
                    <button
                      key={cmd.name}
                      className={`w-full flex items-center justify-between px-4 py-2 text-left transition-colors ${
                        isSelected
                          ? 'bg-blue-500/10 dark:bg-blue-500/20'
                          : 'hover:bg-black/5 dark:hover:bg-white/5'
                      }`}
                      onClick={() => onSelect(cmd.name)}
                      onMouseEnter={() => setSelectedIndex(globalIndex)}
                    >
                      <div className="flex items-center gap-3 min-w-0">
                        <span className={`font-mono text-[13px] font-medium ${CATEGORY_COLORS[category] || 'text-blue-500'}`}>
                          {cmd.name}
                        </span>
                        <span className="text-[12px] text-claude-textSecondary truncate">
                          {cmd.description}
                        </span>
                      </div>
                      <ChevronRight size={14} className="text-claude-textSecondary flex-shrink-0" />
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        <div className="px-4 py-2 border-t border-claude-border flex items-center gap-4 text-[11px] text-claude-textSecondary">
          <span><kbd className="px-1 py-0.5 bg-claude-hover rounded">↑↓</kbd> Navigate</span>
          <span><kbd className="px-1 py-0.5 bg-claude-hover rounded">Enter</kbd> Select</span>
          <span><kbd className="px-1 py-0.5 bg-claude-hover rounded">Esc</kbd> Close</span>
        </div>
      </div>
    </div>
  );
}
