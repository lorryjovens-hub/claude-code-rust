import React, { useState, useEffect, useCallback, useRef } from 'react';
import { HashRouter, Routes, Route, Navigate, useLocation, useParams, useNavigate } from 'react-router-dom';
import { FileText, ChevronDown, Trash, Pencil, Star, BellRing, Menu, Folder, ArrowLeft, ArrowRight } from 'lucide-react';
import Sidebar from './components/Sidebar';
import MainContent from './components/MainContent';
import { IconSidebarToggle } from './components/Icons';
import { updateConversation, deleteConversation, exportConversation, getUnreadAnnouncements, markAnnouncementRead, getSystemStatus } from './api';
import GitBashRequiredModal from './components/GitBashRequiredModal';
import Auth from './components/Auth';
import Onboarding from './components/Onboarding';
import SettingsPage from './components/SettingsPage';
import UpgradePlan from './components/UpgradePlan';
import DocumentPanel from './components/DocumentPanel';
import ArtifactsPanel from './components/ArtifactsPanel';
import ArtifactsPage from './components/ArtifactsPage';
import DraggableDivider from './components/DraggableDivider';
import { DocumentInfo } from './components/DocumentCard';
import AdminLayout from './components/admin/AdminLayout';
import AdminDashboard from './components/admin/AdminDashboard';
import AdminKeyPool from './components/admin/AdminKeyPool';
import AdminUsers from './components/admin/AdminUsers';
import AdminPlans from './components/admin/AdminPlans';
import AdminRedemption from './components/admin/AdminRedemption';
import AdminModels from './components/admin/AdminModels';
import AdminAnnouncements from './components/admin/AdminAnnouncements';
import ChatsPage from './components/ChatsPage';
import CustomizePage from './components/CustomizePage';
import ProjectsPage from './components/ProjectsPage';
import { tauriAPI } from './utils/tauriAPI';

const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

const Tooltip = ({ children, text, shortcut }: { children: React.ReactNode; text: string; shortcut?: string }) => {
  const [show, setShow] = useState(false);
  return (
    <div className="relative" onMouseEnter={() => setShow(true)} onMouseLeave={() => setShow(false)}>
      {children}
      {show && (
        <div className="absolute left-1/2 -translate-x-1/2 top-full mt-1.5 z-[200] pointer-events-none">
          <div className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[12px] font-medium whitespace-nowrap bg-[#2a2a2a] text-white dark:bg-[#e8e8e8] dark:text-[#1a1a1a] shadow-lg">
            <span>{text}</span>
            {shortcut && <span className="opacity-60 text-[11px]">{shortcut}</span>}
          </div>
        </div>
      )}
    </div>
  );
};

const ChatHeader = ({
  title,
  showArtifacts,
  documentPanelDoc,
  onOpenArtifacts,
  hasArtifacts,
  onTitleRename
}: {
  title: string;
  showArtifacts: boolean;
  documentPanelDoc: any;
  onOpenArtifacts: () => void;
  hasArtifacts: boolean;
  onTitleRename?: (newTitle: string) => void;
}) => {
  const { id } = useParams();
  const navigate = useNavigate();
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState('');
  const [showMenu, setShowMenu] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node) &&
        buttonRef.current && !buttonRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };
    if (showMenu) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showMenu]);

  const startEditing = () => {
    setEditTitle(title || 'New Chat');
    setIsEditing(true);
    setShowMenu(false);
  };

  const handleDelete = async () => {
    if (!id) return;
    try {
      await deleteConversation(id);
      navigate('/');
      window.dispatchEvent(new CustomEvent('conversationTitleUpdated'));
    } catch (err) {
      console.error('Failed to delete chat:', err);
    }
    setShowMenu(false);
  };

  const handleRenameSubmit = async () => {
    if (!id || !editTitle.trim()) {
      setIsEditing(false);
      return;
    }
    try {
      await updateConversation(id, { title: editTitle });
      onTitleRename?.(editTitle);
      window.dispatchEvent(new CustomEvent('conversationTitleUpdated'));
    } catch (err) {
      console.error('Failed to rename chat:', err);
    } finally {
      setIsEditing(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleRenameSubmit();
    } else if (e.key === 'Escape') {
      setIsEditing(false);
    }
  };

  return (
    <div className="relative flex items-center justify-between px-3 py-2 bg-claude-bg flex-shrink-0 h-[44px] border-b border-claude-border z-40">
      {isEditing ? (
        <input
          type="text"
          value={editTitle}
          onChange={(e) => setEditTitle(e.target.value)}
          onBlur={handleRenameSubmit}
          onKeyDown={handleKeyDown}
          autoFocus
          className="max-w-[60%] px-2 py-1 text-[14px] font-medium text-claude-text bg-claude-input border border-blue-500 rounded-md outline-none shadow-sm"
          style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
        />
      ) : (
        <div className="relative flex items-center gap-1" style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}>
          <button
            onClick={startEditing}
            className="flex items-center px-2 py-1.5 hover:bg-claude-btn-hover rounded-md transition-colors text-[14px] font-medium text-claude-text max-w-[200px] truncate group"
          >
            {title || 'New Chat'}
          </button>
          <button
            ref={buttonRef}
            onClick={() => setShowMenu(!showMenu)}
            className={`p-1 hover:bg-claude-btn-hover rounded-md transition-colors text-claude-textSecondary hover:text-claude-text ${showMenu ? 'bg-claude-btn-hover text-claude-text' : ''}`}
          >
            <ChevronDown size={14} />
          </button>
          {showMenu && (
            <div ref={menuRef} className="absolute top-full left-0 mt-1 z-50 bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_12px_rgba(0,0,0,0.08)] py-1.5 flex flex-col w-[200px]">
              <button className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group">
                <Star size={16} className="text-claude-textSecondary group-hover:text-claude-text" />
                <span className="text-[13px] text-claude-text">Star</span>
              </button>
              <button onClick={(e) => { e.stopPropagation(); startEditing(); }} className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group">
                <Pencil size={16} className="text-claude-textSecondary group-hover:text-claude-text" />
                <span className="text-[13px] text-claude-text">Rename</span>
              </button>
              <div className="h-[1px] bg-claude-border my-1 mx-3" />
              <button onClick={(e) => { e.stopPropagation(); handleDelete(); }} className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group">
                <Trash size={16} className="text-[#B9382C]" />
                <span className="text-[13px] text-[#B9382C]">Delete</span>
              </button>
            </div>
          )}
        </div>
      )}
      <div className="flex items-center gap-1">
        {hasArtifacts && (
          <button onClick={onOpenArtifacts} className={`w-8 h-8 flex items-center justify-center text-claude-textSecondary hover:bg-claude-btn-hover rounded-md transition-colors ${showArtifacts ? 'bg-claude-btn-hover text-claude-text' : ''}`} title="View Artifacts">
            <FileText size={18} strokeWidth={1.5} />
          </button>
        )}
        <button className="px-2 h-8 flex items-center justify-center text-claude-textSecondary hover:text-claude-text transition-colors" title="Open Workspace Folder" onClick={async () => {
          if (!id) return;
          try {
            const res = await fetch(`http://127.0.0.1:30080/api/conversations/${id}`);
            if (!res.ok) return;
            const data = await res.json();
            if (data.workspace_path && tauriAPI.isTauri) {
              tauriAPI.openFolder(data.workspace_path);
            }
          } catch (e) { console.error('Open folder failed:', e); }
        }}>
          <Folder size={17} strokeWidth={1.5} />
        </button>
        <button onClick={async () => {
          if (!id || isExporting) return;
          setIsExporting(true);
          try {
            await exportConversation(id);
          } catch (err) {
            console.error('导出失败', err);
            window.alert(err instanceof Error ? err.message : '导出失败');
          } finally {
            setIsExporting(false);
          }
        }} disabled={isExporting} className="px-3 py-1.5 text-[13px] font-medium text-claude-textSecondary hover:bg-claude-btn-hover rounded-md transition-colors border border-transparent hover:border-claude-border disabled:opacity-50 disabled:cursor-not-allowed">
          {isExporting ? '导出中…' : 'Export'}
        </button>
      </div>
      <div className="absolute top-full left-0 right-0 h-6 bg-gradient-to-b from-claude-bg to-transparent pointer-events-none z-30" />
    </div>
  );
};

const Layout = () => {
  const [unreadAnnouncements, setUnreadAnnouncements] = useState<Array<{ id: number; title: string; content: string; created_at: string; updated_at?: string }>>([]);
  const [activeAnnouncementId, setActiveAnnouncementId] = useState<number | null>(null);
  const [isMarkingAnnouncementRead, setIsMarkingAnnouncementRead] = useState(false);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  const [newChatKey, setNewChatKey] = useState(0);
  const [authChecked, setAuthChecked] = useState(true);
  const [authValid, setAuthValid] = useState(() => {
    if (!isTauri) return true;
    const mode = localStorage.getItem('user_mode');
    const hasGatewayKey = !!(localStorage.getItem('ANTHROPIC_API_KEY') && localStorage.getItem('gateway_user'));
    return !(mode === 'clawparrot' && !hasGatewayKey);
  });
  const [showSettings, setShowSettings] = useState(false);
  const [showUpgrade, setShowUpgrade] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(() => !localStorage.getItem('onboarding_done'));
  const [needsGitBash, setNeedsGitBash] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const check = async () => {
      try {
        const status = await getSystemStatus();
        if (cancelled) return;
        if (status.gitBash.required && !status.gitBash.found) {
          setNeedsGitBash(true);
        }
      } catch {
        if (!cancelled) setTimeout(check, 1500);
      }
    };
    check();
    return () => { cancelled = true; };
  }, []);

  const [documentPanelDoc, setDocumentPanelDoc] = useState<DocumentInfo | null>(null);
  const [showArtifacts, setShowArtifacts] = useState(false);
  const [artifacts, setArtifacts] = useState<DocumentInfo[]>([]);
  const [documentPanelWidth, setDocumentPanelWidth] = useState(50);
  const [isChatMode, setIsChatMode] = useState(false);
  const [currentChatTitle, setCurrentChatTitle] = useState('');
  const sidebarWasCollapsedRef = useRef(false);
  const contentContainerRef = useRef<HTMLDivElement>(null);

  const [isMac, setIsMac] = useState(false);
  useEffect(() => {
    if (tauriAPI.isTauri) {
      tauriAPI.getPlatform().then((p) => setIsMac(p.os === 'darwin'));
    }
  }, []);

  const [titleBarHeight, setTitleBarHeight] = useState(44);

  const location = useLocation();
  const navigate = useNavigate();

  const [navHistory, setNavHistory] = useState<string[]>([location.pathname + location.search + location.hash]);
  const [navIndex, setNavIndex] = useState(0);
  const isNavAction = useRef(false);

  useEffect(() => {
    const fullPath = location.pathname + location.search;
    if (isNavAction.current) {
      isNavAction.current = false;
      return;
    }
    setNavHistory(prev => {
      const trimmed = prev.slice(0, navIndex + 1);
      if (trimmed[trimmed.length - 1] === fullPath) return trimmed;
      return [...trimmed, fullPath];
    });
    setNavIndex(prev => {
      const trimmed = navHistory.slice(0, prev + 1);
      if (trimmed[trimmed.length - 1] === fullPath) return prev;
      return trimmed.length;
    });
  }, [location.pathname, location.search]);

  const canGoBack = navIndex > 0;
  const canGoForward = navIndex < navHistory.length - 1;

  const handleNavBack = () => {
    if (!canGoBack) return;
    isNavAction.current = true;
    const newIndex = navIndex - 1;
    setNavIndex(newIndex);
    navigate(navHistory[newIndex]);
  };

  const handleNavForward = () => {
    if (!canGoForward) return;
    isNavAction.current = true;
    const newIndex = navIndex + 1;
    setNavIndex(newIndex);
    navigate(navHistory[newIndex]);
  };

  useEffect(() => {
    setShowSettings(false);
    setShowUpgrade(false);
    setDocumentPanelDoc(null);
    setShowArtifacts(false);
  }, [location.pathname]);

  useEffect(() => {
    const handler = () => { setShowUpgrade(true); setShowSettings(false); };
    window.addEventListener('open-upgrade', handler);
    return () => window.removeEventListener('open-upgrade', handler);
  }, []);

  useEffect(() => {
    if (!isTauri) return;
    const mode = localStorage.getItem('user_mode');
    const hasGatewayKey = !!(localStorage.getItem('ANTHROPIC_API_KEY') && localStorage.getItem('gateway_user'));
    setAuthValid(!(mode === 'clawparrot' && !hasGatewayKey));
  }, [isTauri]);

  const loadUnreadAnnouncements = useCallback(async () => {
    try {
      const data = await getUnreadAnnouncements();
      setUnreadAnnouncements(Array.isArray(data?.announcements) ? data.announcements : []);
    } catch (err) {
      console.error('Failed to fetch announcements:', err);
    }
  }, []);

  useEffect(() => {
    if (!authValid) return;
    loadUnreadAnnouncements();
    const intervalId = window.setInterval(() => { loadUnreadAnnouncements(); }, 15000);
    const handleFocus = () => { loadUnreadAnnouncements(); };
    const handleVisibilityChange = () => { if (document.visibilityState === 'visible') { loadUnreadAnnouncements(); } };
    window.addEventListener('focus', handleFocus);
    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => {
      window.clearInterval(intervalId);
      window.removeEventListener('focus', handleFocus);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [authValid, loadUnreadAnnouncements]);

  useEffect(() => {
    if (unreadAnnouncements.length === 0) {
      if (activeAnnouncementId !== null) setActiveAnnouncementId(null);
      return;
    }
    if (activeAnnouncementId === null || !unreadAnnouncements.some(item => item.id === activeAnnouncementId)) {
      setActiveAnnouncementId(unreadAnnouncements[0].id);
    }
  }, [unreadAnnouncements, activeAnnouncementId]);

  const activeAnnouncement = unreadAnnouncements.find(item => item.id === activeAnnouncementId) || null;

  const handleAnnouncementRead = useCallback(async () => {
    if (!activeAnnouncement || isMarkingAnnouncementRead) return;
    setIsMarkingAnnouncementRead(true);
    try {
      await markAnnouncementRead(activeAnnouncement.id);
      setUnreadAnnouncements(prev => prev.filter(item => item.id !== activeAnnouncement.id));
    } catch (err: any) {
      alert(err?.message || '公告已读失败，请稍后重试');
    } finally {
      setIsMarkingAnnouncementRead(false);
    }
  }, [activeAnnouncement, isMarkingAnnouncementRead]);

  const refreshSidebar = () => { setRefreshTrigger(prev => prev + 1); };

  const handleNewChat = () => {
    setNewChatKey(prev => prev + 1);
    setRefreshTrigger(prev => prev + 1);
    setShowSettings(false);
    setShowUpgrade(false);
    setDocumentPanelDoc(null);
    setShowArtifacts(false);
  };

  const handleOpenDocument = useCallback((doc: DocumentInfo) => {
    if (!documentPanelDoc && !showArtifacts) {
      sidebarWasCollapsedRef.current = isSidebarCollapsed;
    }
    setShowArtifacts(false);
    setIsSidebarCollapsed(true);
    setDocumentPanelDoc(doc);
  }, [isSidebarCollapsed, documentPanelDoc, showArtifacts]);

  const handleCloseDocument = useCallback(() => {
    setDocumentPanelDoc(null);
    if (!showArtifacts) {
      setIsSidebarCollapsed(sidebarWasCollapsedRef.current);
    }
  }, [showArtifacts]);

  const handleArtifactsUpdate = useCallback((docs: DocumentInfo[]) => { setArtifacts(docs); }, []);

  const handleOpenArtifacts = useCallback(() => {
    if (showArtifacts) {
      setShowArtifacts(false);
      if (!documentPanelDoc) {
        setIsSidebarCollapsed(sidebarWasCollapsedRef.current);
      }
      return;
    }
    if (!documentPanelDoc) {
      sidebarWasCollapsedRef.current = isSidebarCollapsed;
    }
    setIsSidebarCollapsed(true);
    setShowArtifacts(true);
    setDocumentPanelDoc(null);
  }, [isSidebarCollapsed, documentPanelDoc, showArtifacts]);

  const handleCloseArtifacts = useCallback(() => {
    setShowArtifacts(false);
    setIsSidebarCollapsed(sidebarWasCollapsedRef.current);
  }, []);

  const handleChatModeChange = useCallback((isChat: boolean) => { setIsChatMode(isChat); }, []);
  const handleTitleChange = useCallback((title: string) => { setCurrentChatTitle(title); }, []);

  const [tunerConfig, setTunerConfig] = useState({
    sidebarWidth: 288, recentsMt: 24, profilePy: 10, profilePx: 12, mainContentWidth: 773, mainContentMt: -100,
    inputRadius: 24, welcomeSize: 46, welcomeMb: 34, recentsFontSize: 14, recentsItemPy: 7, recentsPl: 6,
    userAvatarSize: 36, userNameSize: 15, headerPy: 0, toggleSize: 28, toggleAbsRight: 10, toggleAbsTop: 11, toggleAbsLeft: 8,
  });

  if (needsGitBash) {
    return <GitBashRequiredModal onResolved={() => setNeedsGitBash(false)} />;
  }

  if (showOnboarding) {
    return <Onboarding onComplete={() => {
      setShowOnboarding(false);
      if (!isTauri) { setAuthValid(true); return; }
      const mode = localStorage.getItem('user_mode');
      const hasGatewayKey = !!(localStorage.getItem('ANTHROPIC_API_KEY') && localStorage.getItem('gateway_user'));
      setAuthValid(!(mode === 'clawparrot' && !hasGatewayKey));
    }} />;
  }

  if (!authChecked) return null;
  if (!authValid) return <Navigate to="/login" replace />;

  return (
    <>
      <div className="relative flex w-full h-screen overflow-hidden bg-claude-bg font-sans antialiased">
        <div className="absolute top-0 left-0 w-full z-50 flex items-center select-none pointer-events-none bg-claude-bg border-b border-claude-border transition-all duration-300" style={{ WebkitAppRegion: 'drag', height: `${titleBarHeight}px` } as React.CSSProperties}>
          <div className="h-full flex items-center pr-2 gap-0.5" style={{ pointerEvents: 'auto', WebkitAppRegion: 'no-drag', paddingLeft: isMac ? '78px' : '4px' } as React.CSSProperties}>
            <Tooltip text="Menu"><button onClick={() => { }} className="p-2 hover:bg-black/5 dark:hover:bg-white/5 rounded-md text-claude-textSecondary hover:text-claude-text transition-colors"><Menu size={18} className="opacity-80" /></button></Tooltip>
            <Tooltip text={isSidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}><button onClick={() => setIsSidebarCollapsed(!isSidebarCollapsed)} className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-md text-claude-textSecondary hover:text-claude-text transition-colors"><IconSidebarToggle size={26} className="dark:invert transition-[filter] duration-200" /></button></Tooltip>
            {canGoBack ? (
              <Tooltip text="Back"><button onClick={handleNavBack} className="p-1.5 rounded-md transition-colors hover:bg-black/5 dark:hover:bg-white/5" style={{ color: '#73726C' }}><ArrowLeft size={16} strokeWidth={1.5} /></button></Tooltip>
            ) : (<span className="p-1.5" style={{ color: '#B7B5B0' }}><ArrowLeft size={16} strokeWidth={1.5} /></span>)}
            {canGoForward ? (
              <Tooltip text="Forward"><button onClick={handleNavForward} className="p-1.5 rounded-md transition-colors hover:bg-black/5 dark:hover:bg-white/5" style={{ color: '#73726C' }}><ArrowRight size={16} strokeWidth={1.5} /></button></Tooltip>
            ) : (<span className="p-1.5" style={{ color: '#B7B5B0' }}><ArrowRight size={16} strokeWidth={1.5} /></span>)}
          </div>
        </div>
        <Sidebar isCollapsed={isSidebarCollapsed} toggleSidebar={() => setIsSidebarCollapsed(!isSidebarCollapsed)} refreshTrigger={refreshTrigger} onNewChatClick={handleNewChat} onOpenSettings={() => { setShowSettings(true); setShowUpgrade(false); }} onOpenUpgrade={() => { setShowUpgrade(true); setShowSettings(false); }} onCloseOverlays={() => { setShowSettings(false); setShowUpgrade(false); }} tunerConfig={tunerConfig} setTunerConfig={setTunerConfig} />
        <div className="flex-1 flex flex-col h-full min-w-0 overflow-hidden relative" style={{ paddingTop: `${titleBarHeight}px` }}>
          <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[100] flex items-center rounded-xl p-0.5 pointer-events-auto" style={{ backgroundColor: 'var(--bg-mode-tabs)' as React.CSSProperties['backgroundColor'] }}>
            <Tooltip text="Chat" shortcut="Ctrl+1"><button className="px-3.5 py-1 text-[13px] font-medium rounded-[10px] text-claude-text shadow-sm transition-colors" style={{ backgroundColor: 'var(--bg-mode-tab-active)', fontFamily: 'Inter, system-ui, -apple-system, sans-serif', letterSpacing: '0.01em' }}>Chat</button></Tooltip>
            <Tooltip text="Cowork" shortcut="Ctrl+2"><button className="px-3.5 py-1 text-[13px] font-medium rounded-[10px] text-claude-textSecondary hover:text-claude-text transition-colors" style={{ fontFamily: 'Inter, system-ui, -apple-system, sans-serif', letterSpacing: '0.01em' }}>Cowork</button></Tooltip>
            <Tooltip text="Code" shortcut="Ctrl+3"><button className="px-3.5 py-1 text-[13px] font-medium rounded-[10px] text-claude-textSecondary hover:text-claude-text transition-colors" style={{ fontFamily: 'Inter, system-ui, -apple-system, sans-serif', letterSpacing: '0.01em' }}>Code</button></Tooltip>
          </div>
          {isChatMode && (showArtifacts && !documentPanelDoc) && !showSettings && !showUpgrade && (
            <ChatHeader title={currentChatTitle} showArtifacts={showArtifacts} documentPanelDoc={documentPanelDoc} onOpenArtifacts={handleOpenArtifacts} hasArtifacts={artifacts.length > 0} onTitleRename={handleTitleChange} />
          )}
          <div className="flex-1 flex overflow-hidden relative" ref={contentContainerRef}>
            <div className="flex-1 flex flex-col h-full min-w-0">
              {isChatMode && (!showArtifacts || documentPanelDoc) && !showSettings && !showUpgrade && location.pathname !== '/chats' && location.pathname !== '/customize' && location.pathname !== '/projects' && location.pathname !== '/artifacts' && (
                <ChatHeader title={currentChatTitle} showArtifacts={showArtifacts} documentPanelDoc={documentPanelDoc} onOpenArtifacts={handleOpenArtifacts} hasArtifacts={artifacts.length > 0} onTitleRename={handleTitleChange} />
              )}
              {showSettings ? (
                <SettingsPage onClose={() => setShowSettings(false)} />
              ) : showUpgrade ? (
                <UpgradePlan onClose={() => setShowUpgrade(false)} />
              ) : location.pathname === '/chats' ? (
                <ChatsPage />
              ) : location.pathname === '/customize' ? (
                <CustomizePage onCreateWithClaude={() => {
                  sessionStorage.setItem('prefill_input', '让我们一起使用你的 skill-creator skill 来创建一个 skill 吧。请先问我这个 skill 应该做什么。');
                  handleNewChat();
                  window.location.hash = '#/';
                }} />
              ) : location.pathname === '/projects' ? (
                <ProjectsPage />
              ) : location.pathname === '/artifacts' ? (
                <ArtifactsPage onTryPrompt={(prompt) => {
                  if (prompt === '__remix__') sessionStorage.setItem('artifact_prompt', '__remix__');
                  else sessionStorage.setItem('artifact_prompt', prompt);
                  handleNewChat();
                  window.location.hash = '#/';
                }} />
              ) : (
                <MainContent onNewChat={refreshSidebar} resetKey={newChatKey} tunerConfig={tunerConfig} onOpenDocument={handleOpenDocument} onArtifactsUpdate={handleArtifactsUpdate} onOpenArtifacts={handleOpenArtifacts} onTitleChange={handleTitleChange} onChatModeChange={handleChatModeChange} />
              )}
            </div>
            <div className={`h-full bg-claude-bg transition-all duration-300 ease-out flex z-20 relative ${(documentPanelDoc || showArtifacts) ? 'border-l border-claude-border' : ''}`} style={{ width: documentPanelDoc ? `${documentPanelWidth}%` : showArtifacts ? '360px' : '0px', opacity: (documentPanelDoc || showArtifacts) ? 1 : 0, overflow: 'hidden' }}>
              {documentPanelDoc && (
                <div className="absolute left-0 top-0 bottom-0 h-full z-50">
                  <DraggableDivider onResize={setDocumentPanelWidth} containerRef={contentContainerRef} />
                </div>
              )}
              <div className={`w-full h-full flex relative min-w-0 overflow-hidden`}>
                {(documentPanelDoc || showArtifacts) && (
                  <>
                    {documentPanelDoc ? (
                      <DocumentPanel document={documentPanelDoc} onClose={handleCloseDocument} />
                    ) : (
                      <ArtifactsPanel documents={artifacts} onClose={handleCloseArtifacts} onOpenDocument={handleOpenDocument} />
                    )}
                  </>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
      {activeAnnouncement && (
        <div className="fixed inset-0 z-[120] flex items-center justify-center bg-black/45 px-4">
          <div className="w-full max-w-2xl rounded-2xl bg-white dark:bg-[#1F1F1F] shadow-2xl border border-black/5 dark:border-white/10">
            <div className="flex items-center gap-3 px-6 py-5 border-b border-gray-100 dark:border-white/10">
              <div className="w-10 h-10 rounded-full bg-blue-50 text-blue-600 dark:bg-blue-500/15 dark:text-blue-300 flex items-center justify-center shrink-0"><BellRing size={20} /></div>
              <div className="min-w-0">
                <h3 className="text-[18px] font-semibold text-gray-900 dark:text-white break-words">{activeAnnouncement.title}</h3>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">系统公告 · {activeAnnouncement.created_at?.slice(0, 16).replace('T', ' ') || ''}</p>
              </div>
            </div>
            <div className="px-6 py-5">
              <div className="max-h-[50vh] overflow-y-auto whitespace-pre-wrap break-words text-[15px] leading-7 text-gray-700 dark:text-gray-200">{activeAnnouncement.content}</div>
              <div className="mt-4 text-xs text-gray-500 dark:text-gray-400">点击右下角“已读”后，后续将不再重复弹出这条公告。</div>
            </div>
            <div className="flex items-center justify-between px-6 py-4 border-t border-gray-100 dark:border-white/10">
              <div className="text-xs text-gray-400 dark:text-gray-500">{unreadAnnouncements.length > 1 ? `还有 ${unreadAnnouncements.length - 1} 条未读公告` : '暂无其他未读公告'}</div>
              <button onClick={handleAnnouncementRead} disabled={isMarkingAnnouncementRead} className="px-5 py-2.5 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed">{isMarkingAnnouncementRead ? '处理中...' : '已读'}</button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};

const App = () => {
  return (
    <HashRouter>
      <Routes>
        <Route path="/login" element={<Auth />} />
        <Route path="/admin" element={<AdminLayout />}>
          <Route index element={<AdminDashboard />} />
          <Route path="keys" element={<AdminKeyPool />} />
          <Route path="models" element={<AdminModels />} />
          <Route path="users" element={<AdminUsers />} />
          <Route path="announcements" element={<AdminAnnouncements />} />
          <Route path="plans" element={<AdminPlans />} />
          <Route path="redemption" element={<AdminRedemption />} />
        </Route>
        <Route path="/" element={<Layout />} />
        <Route path="/chats" element={<Layout />} />
        <Route path="/customize" element={<Layout />} />
        <Route path="/projects" element={<Layout />} />
        <Route path="/artifacts" element={<Layout />} />
        <Route path="/chat/:id" element={<Layout />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </HashRouter>
  );
};

export default App;
