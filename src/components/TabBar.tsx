import React, { useState, useRef, useEffect, useCallback } from 'react';
import { X, Plus, Pin, PinOff, ChevronLeft, ChevronRight } from 'lucide-react';
import { useTabStore, TabItem } from '../stores/useTabStore';
import { useI18n } from '../hooks/useI18n';
import claudeImg from '../assets/icons/claude.png';

const TabBar: React.FC<{
  onNewChat: () => void;
  rightActions?: React.ReactNode;
}> = ({ onNewChat, rightActions }) => {
  const { t } = useI18n();
  const { openTabs, activeTabId, switchTab, closeTab, renameTab, pinTab, clearTabUnread } = useTabStore();
  const [scrollPos, setScrollPos] = useState(0);
  const [showLeftArrow, setShowLeftArrow] = useState(false);
  const [showRightArrow, setShowRightArrow] = useState(false);
  const [editingTabId, setEditingTabId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');
  const [contextMenu, setContextMenu] = useState<{ tabId: string; x: number; y: number } | null>(null);

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const editingInputRef = useRef<HTMLInputElement>(null);

  const activeTab = openTabs.find(t => t.id === activeTabId);

  useEffect(() => {
    if (editingTabId && editingInputRef.current) {
      editingInputRef.current.focus();
      editingInputRef.current.select();
    }
  }, [editingTabId]);

  const checkScrollArrows = useCallback(() => {
    const el = scrollContainerRef.current;
    if (!el) return;
    setShowLeftArrow(el.scrollLeft > 0);
    setShowRightArrow(el.scrollLeft < el.scrollWidth - el.clientWidth - 1);
  }, []);

  useEffect(() => {
    checkScrollArrows();
    const el = scrollContainerRef.current;
    el?.addEventListener('scroll', checkScrollArrows);
    return () => el?.removeEventListener('scroll', checkScrollArrows);
  }, [checkScrollArrows, openTabs]);

  const scroll = (direction: 'left' | 'right') => {
    const el = scrollContainerRef.current;
    if (!el) return;
    const scrollAmount = 200;
    el.scrollBy({
      left: direction === 'left' ? -scrollAmount : scrollAmount,
      behavior: 'smooth',
    });
  };

  const handleTabClick = (tab: TabItem) => {
    switchTab(tab.id);
    clearTabUnread(tab.id);
    if (tab.conversationId) {
      window.dispatchEvent(new CustomEvent('tabSwitched', { detail: { conversationId: tab.conversationId } }));
    }
  };

  const handleCloseTab = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation();
    closeTab(tabId);
    if (editingTabId === tabId) setEditingTabId(null);
  };

  const handleStartRename = (tabId: string) => {
    const tab = openTabs.find(t => t.id === tabId);
    if (tab) {
      setEditingTabId(tabId);
      setEditingTitle(tab.title);
    }
    setContextMenu(null);
  };

  const handleSaveRename = () => {
    if (editingTabId && editingTitle.trim()) {
      renameTab(editingTabId, editingTitle.trim());
    }
    setEditingTabId(null);
  };

  const handlePinToggle = (tabId: string) => {
    const tab = openTabs.find(t => t.id === tabId);
    if (tab) {
      pinTab(tabId, !tab.pinned);
    }
    setContextMenu(null);
  };

  const handleCloseOtherTabs = (tabId: string) => {
    const tab = openTabs.find(t => t.id === tabId);
    if (!tab) return;
    const toClose = openTabs.filter(t => t.id !== tabId && !t.pinned);
    toClose.forEach(t => closeTab(t.id));
    setContextMenu(null);
  };

  const handleContextMenu = (e: React.MouseEvent, tabId: string) => {
    e.preventDefault();
    setContextMenu({ tabId, x: e.clientX, y: e.clientY });
  };

  useEffect(() => {
    const handleClickOutside = () => setContextMenu(null);
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  }, []);

  if (openTabs.length === 0) return null;

  const sortedTabs = [...openTabs].sort((a, b) => {
    if (a.pinned && !b.pinned) return -1;
    if (!a.pinned && b.pinned) return 1;
    return b.lastActiveAt - a.lastActiveAt;
  });

  return (
    <>
      <div className="relative flex items-center h-10 bg-claude-bg border-b border-claude-border">
        {showLeftArrow && (
          <button
            onClick={() => scroll('left')}
            className="flex-shrink-0 w-7 h-full flex items-center justify-center bg-claude-bg/90 hover:bg-claude-hover text-claude-textSecondary transition-colors z-10"
          >
            <ChevronLeft size={16} />
          </button>
        )}

        <div
          ref={scrollContainerRef}
          className="flex-1 flex items-center overflow-x-auto scrollbar-none gap-0.5 px-1"
          style={{ scrollbarWidth: 'none', msOverflowStyle: 'none' }}
        >
          {sortedTabs.map(tab => {
            const isActive = tab.id === activeTabId;
            const isEditing = tab.id === editingTabId;

            return (
              <div
                key={tab.id}
                onClick={() => handleTabClick(tab)}
                onContextMenu={(e) => handleContextMenu(e, tab.id)}
                className={`
                  group relative flex items-center gap-1.5 h-8 px-3 rounded-md cursor-pointer
                  transition-all duration-150 select-none min-w-0 flex-shrink-0
                  ${isActive
                    ? 'bg-claude-hover text-claude-text'
                    : 'text-claude-textSecondary hover:bg-claude-hover/50 hover:text-claude-text'
                  }
                `}
                style={{ maxWidth: '200px', minWidth: '100px' }}
              >
                {tab.pinned && (
                  <Pin size={12} className="flex-shrink-0 text-claude-textSecondary opacity-60" />
                )}

                {!tab.pinned && (
                  <img
                    src={claudeImg}
                    alt=""
                    className="flex-shrink-0 w-3.5 h-3.5 dark:invert opacity-70"
                  />
                )}

                {isEditing ? (
                  <input
                    ref={editingInputRef}
                    type="text"
                    value={editingTitle}
                    onChange={(e) => setEditingTitle(e.target.value)}
                    onBlur={handleSaveRename}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleSaveRename();
                      if (e.key === 'Escape') setEditingTabId(null);
                    }}
                    onClick={(e) => e.stopPropagation()}
                    className="flex-1 min-w-0 bg-transparent text-claude-text text-sm outline-none border-none p-0"
                  />
                ) : (
                  <span className="flex-1 min-w-0 truncate text-sm leading-none">
                    {tab.title || tab.firstMessage?.slice(0, 30) || 'New Chat'}
                  </span>
                )}

                {tab.unreadCount && tab.unreadCount > 0 && !isEditing && (
                  <span className="flex-shrink-0 w-4 h-4 rounded-full bg-blue-500 text-white text-[10px] flex items-center justify-center font-medium">
                    {tab.unreadCount > 9 ? '9+' : tab.unreadCount}
                  </span>
                )}

                <button
                  onClick={(e) => handleCloseTab(e, tab.id)}
                  className={`
                    flex-shrink-0 w-4 h-4 flex items-center justify-center rounded-sm
                    transition-all duration-100
                    ${isActive || true ? 'opacity-0 group-hover:opacity-100 hover:bg-claude-border' : 'opacity-0 group-hover:opacity-100 hover:bg-claude-border'}
                  `}
                >
                  <X size={12} />
                </button>
              </div>
            );
          })}
        </div>

        {showRightArrow && (
          <button
            onClick={() => scroll('right')}
            className="flex-shrink-0 w-7 h-full flex items-center justify-center bg-claude-bg/90 hover:bg-claude-hover text-claude-textSecondary transition-colors z-10"
          >
            <ChevronRight size={16} />
          </button>
        )}

        {rightActions}

        <button
          onClick={onNewChat}
          className="flex-shrink-0 w-8 h-full flex items-center justify-center text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover transition-colors"
        >
          <Plus size={16} />
        </button>
      </div>

      {contextMenu && (
        <div
          className="fixed z-[200] bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_12px_rgba(0,0,0,0.15)] py-1.5 w-[180px]"
          style={{ top: contextMenu.y, left: contextMenu.x }}
        >
          <button
            onClick={() => handleStartRename(contextMenu.tabId)}
            className="flex items-center gap-2.5 px-3 py-1.5 text-[13px] text-claude-text hover:bg-claude-hover w-full transition-colors"
          >
            <IconPencil size={14} className="text-claude-textSecondary" />
            {t('sidebar.rename') || '重命名'}
          </button>
          <button
            onClick={() => handlePinToggle(contextMenu.tabId)}
            className="flex items-center gap-2.5 px-3 py-1.5 text-[13px] text-claude-text hover:bg-claude-hover w-full transition-colors"
          >
            {openTabs.find(t => t.id === contextMenu.tabId)?.pinned ? (
              <PinOff size={14} className="text-claude-textSecondary" />
            ) : (
              <Pin size={14} className="text-claude-textSecondary" />
            )}
            {openTabs.find(t => t.id === contextMenu.tabId)?.pinned ? (t('common.unpin') || '取消置顶') : (t('common.pin') || '置顶')}
          </button>
          <div className="h-[1px] bg-claude-border my-1 mx-2" />
          <button
            onClick={() => handleCloseOtherTabs(contextMenu.tabId)}
            className="flex items-center gap-2.5 px-3 py-1.5 text-[13px] text-claude-text hover:bg-claude-hover w-full transition-colors"
          >
            <X size={14} className="text-claude-textSecondary" />
            {t('tabs.closeOthers') || '关闭其他标签'}
          </button>
        </div>
      )}
    </>
  );
};

const IconPencil = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
    <path d="m15 5 4 4" />
  </svg>
);

export default TabBar;
