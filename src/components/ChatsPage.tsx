import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { getConversations, deleteConversation, updateConversation, getProjects, Project } from '../api';
import { Search, Plus, MoreHorizontal, Star, Pencil, Trash, X, Minus, Check } from 'lucide-react';
import { createPortal } from 'react-dom';
import { IconProjects } from './Icons';

const RenameModal = ({ isOpen, onClose, onSave, initialTitle }: { isOpen: boolean; onClose: () => void; onSave: (newTitle: string) => void; initialTitle: string; }) => {
  const [title, setTitle] = useState(initialTitle);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setTitle(initialTitle);
      setTimeout(() => {
        if (inputRef.current) {
          inputRef.current.focus();
          inputRef.current.select();
        }
      }, 50);
    }
  }, [isOpen, initialTitle]);

  if (!isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/40" onClick={onClose}>
      <div
        className="bg-claude-input rounded-2xl shadow-xl w-[400px] p-6 animate-fade-in"
        onClick={e => e.stopPropagation()}
      >
        <h3 className="text-[18px] font-semibold text-claude-text mb-4">Rename chat</h3>
        <input
          ref={inputRef}
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              if (title.trim()) onSave(title.trim());
            } else if (e.key === 'Escape') {
              onClose();
            }
          }}
          className="w-full px-3 py-2 bg-transparent border border-claude-border rounded-lg text-claude-text focus:outline-none focus:border-blue-500 mb-6 text-[15px]"
        />
        <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-[14px] font-medium text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={() => {
              if (title.trim()) onSave(title.trim());
            }}
            disabled={!title.trim()}
            className="px-4 py-2 text-[14px] font-medium text-white bg-[#333333] hover:bg-[#1a1a1a] dark:bg-[#FFFFFF] dark:text-black dark:hover:bg-[#e5e5e5] rounded-lg transition-colors disabled:opacity-50"
          >
            Save
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
};

const ChatsPage = () => {
  const navigate = useNavigate();
  const [chats, setChats] = useState<any[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [activeMenuId, setActiveMenuId] = useState<string | null>(null);
  const [menuPosition, setMenuPosition] = useState<{ top: number, left: number } | null>(null);
  const [showRenameModal, setShowRenameModal] = useState(false);
  const [renameChatId, setRenameChatId] = useState<string | null>(null);
  const [renameInitialTitle, setRenameInitialTitle] = useState('');

  // Selection Mode State
  const [isSelectionMode, setIsSelectionMode] = useState(false);
  const [selectedChatIds, setSelectedChatIds] = useState<Set<string>>(new Set());

  // Add to project state
  const [showProjectPicker, setShowProjectPicker] = useState(false);
  const [projectList, setProjectList] = useState<Project[]>([]);

  const menuRef = useRef<HTMLDivElement>(null);
  const projectPickerRef = useRef<HTMLDivElement>(null);
  const projectBtnRef = useRef<HTMLButtonElement>(null);
  const [pickerPos, setPickerPos] = useState<{ top: number; left: number } | null>(null);

  useEffect(() => {
    // Inject Spectral font for the title
    const link = document.createElement('link');
    link.href = 'https://fonts.googleapis.com/css2?family=Spectral:ital,wght@0,300;0,400;0,500;0,600;0,700;0,800;1,300&display=swap';
    link.rel = 'stylesheet';
    document.head.appendChild(link);

    fetchChats();

    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setActiveMenuId(null);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);

    return () => {
      document.head.removeChild(link);
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const fetchChats = async () => {
    try {
      const data = await getConversations();
      if (Array.isArray(data)) {
        setChats(data);
      }
    } catch (e) {
      console.error("Failed to fetch chats", e);
    } finally {
      setLoading(false);
    }
  };

  const handleMenuClick = (e: React.MouseEvent, chat: any) => {
    e.stopPropagation();
    const rect = e.currentTarget.getBoundingClientRect();
    setMenuPosition({
      top: rect.bottom + 4,
      left: rect.right - 200, // Align right edge roughly
    });
    setActiveMenuId(chat.id);
  };

  const handleDeleteChat = async (id: string) => {
    try {
      await deleteConversation(id);
      setChats(chats.filter(c => c.id !== id));
      setActiveMenuId(null);
    } catch (err) {
      console.error(err);
    }
  };

  const handleRenameClick = (chat: any) => {
    setRenameChatId(chat.id);
    setRenameInitialTitle(chat.title || 'New Chat');
    setShowRenameModal(true);
    setActiveMenuId(null);
  };

  const handleRenameSubmit = async (newTitle: string) => {
    if (!renameChatId) return;
    try {
      setChats(chats.map(c => c.id === renameChatId ? { ...c, title: newTitle } : c));
      await updateConversation(renameChatId, { title: newTitle });
    } catch (err) {
      console.error('Failed to rename chat:', err);
      fetchChats();
    }
    setShowRenameModal(false);
    setRenameChatId(null);
  };

  // Selection Mode Handlers
  const toggleSelectionMode = () => {
    setIsSelectionMode(!isSelectionMode);
    setSelectedChatIds(new Set());
  };

  const toggleChatSelection = (id: string) => {
    const newSelected = new Set(selectedChatIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedChatIds(newSelected);
  };

  const handleSelectAll = () => {
    if (selectedChatIds.size === filteredChats.length) {
      setSelectedChatIds(new Set());
    } else {
      setSelectedChatIds(new Set(filteredChats.map(c => c.id)));
    }
  };

  const handleDeleteSelected = async () => {
    if (selectedChatIds.size === 0) return;
    if (!confirm(`Delete ${selectedChatIds.size} chats?`)) return;

    try {
      await Promise.all(Array.from(selectedChatIds).map(id => deleteConversation(id)));
      setChats(prev => prev.filter(c => !selectedChatIds.has(c.id)));
      setSelectedChatIds(new Set());
      setIsSelectionMode(false);
    } catch (err) {
      console.error('Failed to delete selected chats:', err);
    }
  };

  const handleAddToProject = async () => {
    if (selectedChatIds.size === 0) return;
    try {
      const projects = await getProjects();
      if (projects.length === 0) {
        alert('暂无项目，请先在 Projects 页面创建一个项目');
        return;
      }
      setProjectList(projects);
      if (projectBtnRef.current) {
        const rect = projectBtnRef.current.getBoundingClientRect();
        setPickerPos({ top: rect.bottom + 6, left: rect.left });
      }
      setShowProjectPicker(true);
    } catch (err) {
      console.error('Failed to load projects:', err);
    }
  };

  const handleMoveToProject = async (projectId: string) => {
    try {
      await Promise.all(
        Array.from(selectedChatIds).map(id =>
          updateConversation(id, { project_id: projectId })
        )
      );
      // Remove moved chats from the list (they're now project chats)
      setChats(prev => prev.filter(c => !selectedChatIds.has(c.id)));
      setSelectedChatIds(new Set());
      setIsSelectionMode(false);
      setShowProjectPicker(false);
    } catch (err) {
      console.error('Failed to move chats to project:', err);
    }
  };

  const filteredChats = chats.filter(chat =>
    (chat.title || 'New Chat').toLowerCase().includes(searchQuery.toLowerCase())
  );

  const formatTimeAgo = (dateStr: string) => {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    const now = new Date();
    const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diffInSeconds < 60) return 'just now';
    if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)} minutes ago`;
    if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)} hours ago`;
    if (diffInSeconds < 604800) return `${Math.floor(diffInSeconds / 86400)} days ago`;
    return date.toLocaleDateString();
  };

  return (
    <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
      <div className="max-w-[800px] mx-auto px-4 py-8 md:px-8 md:py-12">
        <div className="flex items-center justify-between mb-8">
          <h1
            className="font-[Spectral] text-[32px] text-claude-text"
            style={{
              fontWeight: 500,
              WebkitTextStroke: '0.5px currentColor'
            }}
          >
            Chats
          </h1>
          <button
            onClick={() => navigate('/')}
            className="flex items-center gap-2 px-3.5 py-1.5 bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity font-medium"
            style={{ fontSize: '14px' }}
          >
            <Plus size={16} strokeWidth={2.5} />
            New chat
          </button>
        </div>

        <div className="relative mb-6">
          <div className="absolute inset-y-0 left-3 flex items-center pointer-events-none">
            <Search className="h-5 w-5 text-claude-textSecondary opacity-80" />
          </div>
          <input
            type="text"
            placeholder="Search your chats..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-3 bg-white dark:bg-claude-input border border-gray-200 dark:border-claude-border rounded-xl text-claude-text placeholder-claude-textSecondary focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all text-[15px]"
          />
        </div>

        {/* Action Bar / Selection Bar */}
        {isSelectionMode ? (
          <div className="flex items-center justify-between mb-4 bg-transparent animate-fade-in h-8">
            <div className="flex items-center gap-4">
              <button
                onClick={handleSelectAll}
                className="flex items-center justify-center w-5 h-5 rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors"
              >
                {selectedChatIds.size > 0 && selectedChatIds.size === filteredChats.length ? (
                  <Check size={14} strokeWidth={3} />
                ) : selectedChatIds.size > 0 ? (
                  <Minus size={14} strokeWidth={3} />
                ) : (
                  <div className="w-5 h-5 border border-gray-300 rounded bg-white dark:bg-claude-input"></div>
                )}
              </button>
              <span className="text-[13px] text-claude-textSecondary">
                {selectedChatIds.size} selected
              </span>
              <div className="flex items-center gap-2 ml-2">
                <button
                  ref={projectBtnRef}
                  className="p-1 text-claude-textSecondary hover:text-claude-text transition-colors"
                  title="Add to project"
                  onClick={handleAddToProject}
                >
                  <IconProjects size={24} className="dark:invert" />
                </button>
                {showProjectPicker && pickerPos && createPortal(
                  <>
                    <div className="fixed inset-0 z-[90]" onClick={() => setShowProjectPicker(false)} />
                    <div ref={projectPickerRef} className="fixed w-[240px] bg-white dark:bg-[#2A2928] border border-claude-border rounded-xl shadow-lg py-1.5 z-[100]" style={{ top: pickerPos.top, left: pickerPos.left }}>
                      <div className="px-3 py-2 text-[12px] font-medium text-claude-textSecondary border-b border-claude-border">
                        移动到项目
                      </div>
                      {projectList.map(p => (
                        <button
                          key={p.id}
                          onClick={() => handleMoveToProject(p.id)}
                          className="w-full text-left px-3 py-2.5 text-[14px] text-claude-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors truncate"
                        >
                          {p.name}
                        </button>
                      ))}
                    </div>
                  </>,
                  document.body
                )}
                <button
                  onClick={handleDeleteSelected}
                  className="p-1 text-[#B9382C] hover:opacity-80 transition-opacity"
                  title="Delete selected"
                >
                  <Trash size={18} />
                </button>
              </div>
            </div>
            <button
              onClick={toggleSelectionMode}
              className="text-claude-textSecondary hover:text-claude-text transition-colors"
            >
              <X size={20} />
            </button>
          </div>
        ) : (
          <div className="flex items-center gap-2 mb-4 text-[13px] text-claude-textSecondary h-8">
            <span>{chats.length} chats with Claude</span>
            <button onClick={toggleSelectionMode} className="text-blue-600 hover:underline">Select</button>
          </div>
        )}

        <div className="space-y-0">
          {loading ? (
            <div className="py-8 text-center text-claude-textSecondary">Loading chats...</div>
          ) : filteredChats.length === 0 ? (
            <div className="py-8 text-center text-claude-textSecondary">No chats found</div>
          ) : (
            filteredChats.map((chat) => {
              const isSelected = selectedChatIds.has(chat.id);
              return (
                <div
                  key={chat.id}
                  onClick={() => {
                    if (isSelectionMode) {
                      toggleChatSelection(chat.id);
                    } else {
                      navigate(`/chat/${chat.id}`);
                    }
                  }}
                  className={`
                    group relative py-4 border-b border-[#DAD9D4] dark:border-claude-border px-4 -mx-4 cursor-pointer transition-colors flex items-center
                    ${isSelected ? 'bg-[#EBF1F5] dark:bg-blue-900/20' : 'hover:bg-[#F5F4ED] dark:hover:bg-claude-hover'}
                  `}
                >
                  {/* Checkbox (Always visible in selection mode, or on hover) */}
                  <div
                    className={`
                      absolute left-4 transition-all duration-200 flex items-center justify-center
                      ${isSelectionMode ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}
                    `}
                    onClick={(e) => {
                      e.stopPropagation();
                      if (isSelectionMode) {
                        toggleChatSelection(chat.id);
                      } else {
                        // Enter selection mode and select this one
                        setIsSelectionMode(true);
                        setSelectedChatIds(new Set([chat.id]));
                      }
                    }}
                  >
                    {isSelected ? (
                      <div className="w-5 h-5 rounded bg-blue-600 flex items-center justify-center text-white">
                        <Check size={14} strokeWidth={3} />
                      </div>
                    ) : (
                      <div className="w-5 h-5 border border-gray-300 rounded bg-white dark:bg-claude-input hover:border-gray-400"></div>
                    )}
                  </div>

                  <div className={`flex-1 transition-all duration-200 ${isSelectionMode ? 'pl-8' : 'pl-0 group-hover:pl-8'}`}>
                    <div className="flex justify-between items-baseline mb-1">
                      <h3 className="text-[16px] font-medium text-claude-text truncate pr-4 flex-1">
                        {chat.title || 'Untitled'}
                      </h3>
                    </div>
                    <div className="text-[13px] text-claude-textSecondary">
                      Last message {formatTimeAgo(chat.updated_at || chat.created_at)}
                    </div>
                  </div>

                  {/* Right menu dots (Only visible in normal mode) */}
                  {!isSelectionMode && (
                    <button
                      onClick={(e) => handleMenuClick(e, chat)}
                      className={`absolute right-4 p-1.5 rounded-md text-claude-textSecondary hover:text-claude-text hover:bg-black/5 dark:hover:bg-white/10 transition-all ${activeMenuId === chat.id ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}`}
                    >
                      <MoreHorizontal size={20} />
                    </button>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Context Menu */}
      {activeMenuId && menuPosition && !isSelectionMode && (
        <div
          ref={menuRef}
          className="fixed z-50 bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_12px_rgba(0,0,0,0.08)] py-1.5 flex flex-col w-[200px]"
          style={{ top: menuPosition.top, left: menuPosition.left }}
        >
          <button className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group">
            <Star size={16} className="text-claude-textSecondary group-hover:text-claude-text" />
            <span className="text-[13px] text-claude-text">Star</span>
          </button>
          <button
            onClick={() => handleRenameClick(chats.find(c => c.id === activeMenuId))}
            className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group"
          >
            <Pencil size={16} className="text-claude-textSecondary group-hover:text-claude-text" />
            <span className="text-[13px] text-claude-text">Rename</span>
          </button>
          <div className="h-[1px] bg-claude-border my-1 mx-3" />
          <button
            onClick={() => handleDeleteChat(activeMenuId)}
            className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group"
          >
            <Trash size={16} className="text-[#B9382C]" />
            <span className="text-[13px] text-[#B9382C]">Delete</span>
          </button>
        </div>
      )}

      {/* Rename Modal */}
      <RenameModal
        isOpen={showRenameModal}
        onClose={() => {
          setShowRenameModal(false);
          setRenameChatId(null);
        }}
        onSave={handleRenameSubmit}
        initialTitle={renameInitialTitle}
      />
    </div>
  );
};

export default ChatsPage;
