import React, { useState, useEffect, useCallback, useRef } from 'react';
import { HashRouter, Routes, Route, Navigate, useLocation, useNavigate } from 'react-router-dom';
import { BellRing, Menu, ArrowLeft, ArrowRight, GitBranch, BarChart3, Terminal } from 'lucide-react';
import Sidebar from './components/Sidebar';
import MainContent from './components/MainContent';
import ChatMainContent from './components/ChatMainContent';
import { IconSidebarToggle } from './components/Icons';
import { updateConversation, deleteConversation, exportConversation, getUnreadAnnouncements, markAnnouncementRead, getSystemStatus, detectBridgePort } from './api';
import GitBashRequiredModal from './components/GitBashRequiredModal';
import Auth from './components/Auth';
import Onboarding from './components/Onboarding';
import SettingsPage from './components/SettingsPage';
import AgentPanel from './components/AgentPanel';
import AnalyticsPanel from './components/AnalyticsPanel';
import TerminalPanel from './components/TerminalPanel';
import UpgradePlan from './components/UpgradePlan';
import CodePanel from './components/CodePanel';
import CodeWorkspace from './components/CodeWorkspace';
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
import { ErrorBoundary } from './components/ErrorBoundary';
import ModelsPage from './components/ModelsPage';
import DesignPage from './components/DesignPage';
import MemoryPanel from './components/MemoryPanel';
import DirectoryModal from './components/DirectoryModal';
import PromptSuggestionsPanel from './components/PromptSuggestionsPanel';
import { tauriAPI } from './utils/tauriAPI';
import ChatHeader from './components/ChatHeader';
import Tooltip from './components/ui/Tooltip';
import { useUIStore } from './stores';

const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

const Layout = () => {
  const [unreadAnnouncements, setUnreadAnnouncements] = useState<Array<{ id: number; title: string; content: string; created_at: string; updated_at?: string }>>([]);
  const [activeAnnouncementId, setActiveAnnouncementId] = useState<number | null>(null);
  const [isMarkingAnnouncementRead, setIsMarkingAnnouncementRead] = useState(false);
  const isSidebarCollapsed = useUIStore(s => s.sidebarCollapsed);
  const setIsSidebarCollapsed = useUIStore(s => s.setSidebarCollapsed);
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  const [newChatKey, setNewChatKey] = useState(0);
  const [authChecked, setAuthChecked] = useState(false);
  const [authValid, setAuthValid] = useState(true);

  // Check auth validity on mount
  useEffect(() => {
    if (!isTauri) {
      setAuthChecked(true);
      setAuthValid(true);
      return;
    }
    const mode = localStorage.getItem('user_mode');
    // selfhosted mode: always valid (user brings their own key)
    if (!mode || mode === 'selfhosted') {
      setAuthChecked(true);
      setAuthValid(true);
      return;
    }
    // clawparrot mode: need gateway credentials
    const hasGatewayKey = !!(localStorage.getItem('ANTHROPIC_API_KEY') && localStorage.getItem('gateway_user'));
    setAuthValid(hasGatewayKey);
    setAuthChecked(true);
  }, []);
  const setShowSettings = useUIStore(s => s.setShowSettings);
  const setShowUpgrade = useUIStore(s => s.setShowUpgrade);
  const setShowAgentPanel = useUIStore(s => s.setShowAgentPanel);
  const setShowAnalyticsPanel = useUIStore(s => s.setShowAnalyticsPanel);
  const showSettings = useUIStore(s => s.showSettings);
  const showUpgrade = useUIStore(s => s.showUpgrade);
  const showAgentPanel = useUIStore(s => s.showAgentPanel);
  const showAnalyticsPanel = useUIStore(s => s.showAnalyticsPanel);
  const showTerminal = useUIStore(s => s.showTerminalPanel);
  const setShowTerminal = useUIStore(s => s.setShowTerminalPanel);
  const terminalHeight = useUIStore(s => s.terminalPanelHeight);
  const setTerminalHeight = useUIStore(s => s.setTerminalPanelHeight);
  const showOnboarding = useUIStore(s => s.showOnboarding);
  const setShowOnboarding = useUIStore(s => s.setShowOnboarding);
  const [needsGitBash, setNeedsGitBash] = useState(false);
  const [showDirectoryModal, setShowDirectoryModal] = useState(false);
  const [showPromptSuggestions, setShowPromptSuggestions] = useState(true);

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
      tauriAPI.showMainWindow();
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
      const next = [...trimmed, fullPath];
      setNavIndex(next.length - 1);
      return next;
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
    if (documentPanelDoc || showArtifacts) {
      setDocumentPanelDoc(null);
      setShowArtifacts(false);
      setIsSidebarCollapsed(sidebarWasCollapsedRef.current);
    }
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
    if (documentPanelDoc || showArtifacts) {
      setDocumentPanelDoc(null);
      setShowArtifacts(false);
      setIsSidebarCollapsed(sidebarWasCollapsedRef.current);
    }
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

  const [zoomLevel, setZoomLevel] = useState(() => {
    const saved = localStorage.getItem('app_zoom');
    return saved ? parseFloat(saved) : 1;
  });
  const [showZoomIndicator, setShowZoomIndicator] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === '=' || e.key === '+')) {
        e.preventDefault();
        setZoomLevel(prev => {
          const next = Math.min(prev + 0.1, 2.0);
          localStorage.setItem('app_zoom', next.toString());
          return next;
        });
        setShowZoomIndicator(true);
        setTimeout(() => setShowZoomIndicator(false), 1500);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '-') {
        e.preventDefault();
        setZoomLevel(prev => {
          const next = Math.max(prev - 0.1, 0.5);
          localStorage.setItem('app_zoom', next.toString());
          return next;
        });
        setShowZoomIndicator(true);
        setTimeout(() => setShowZoomIndicator(false), 1500);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '0') {
        e.preventDefault();
        setZoomLevel(1);
        localStorage.setItem('app_zoom', '1');
        setShowZoomIndicator(true);
        setTimeout(() => setShowZoomIndicator(false), 1500);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '`') {
        e.preventDefault();
        setShowTerminal(!showTerminal);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  if (needsGitBash) {
    return <GitBashRequiredModal onResolved={() => setNeedsGitBash(false)} />;
  }

  if (showOnboarding) {
    return <Onboarding onComplete={() => {
      setShowOnboarding(false);
      if (!isTauri) { setAuthValid(true); return; }
      const mode = localStorage.getItem('user_mode');
      if (mode === 'selfhosted') {
        setAuthValid(true);
        return;
      }
      const hasGatewayKey = !!(localStorage.getItem('ANTHROPIC_API_KEY') && localStorage.getItem('gateway_user'));
      if (hasGatewayKey) {
        setAuthValid(true);
      } else {
        setAuthValid(false);
      }
    }} />;
  }

  if (!authChecked) {
    return (
      <div className="flex items-center justify-center w-full h-screen bg-claude-bg">
        <div className="text-claude-textSecondary text-[14px]">Loading...</div>
      </div>
    );
  }
  if (!authValid) {
    return <Auth />;
  }

  return (
    <>
      <div className="relative flex w-full h-screen overflow-hidden bg-claude-bg font-sans antialiased" style={{ zoom: zoomLevel }}>
        {showZoomIndicator && (
          <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[300] px-4 py-2 bg-claude-tooltipBg text-claude-tooltipText rounded-lg shadow-lg text-[13px] font-medium flex items-center gap-2 transition-opacity duration-300">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
            {Math.round(zoomLevel * 100)}%
            <button onClick={() => { setZoomLevel(1); localStorage.setItem('app_zoom', '1'); }} className="ml-1 px-1.5 py-0.5 text-[11px] rounded bg-white/20 dark:bg-black/10 hover:bg-white/30 dark:hover:bg-black/20 transition-colors">Reset</button>
          </div>
        )}
        <div className="absolute top-0 left-0 w-full z-50 flex items-center select-none pointer-events-none bg-claude-bg border-b border-claude-border transition-all duration-300" style={{ WebkitAppRegion: 'drag', height: `${titleBarHeight}px` } as React.CSSProperties}>
          <div className="h-full flex items-center pr-2 gap-0.5" style={{ pointerEvents: 'auto', WebkitAppRegion: 'no-drag', paddingLeft: isMac ? '78px' : '4px' } as React.CSSProperties}>
            <Tooltip content="Menu"><button onClick={() => { }} className="p-2 hover:bg-black/5 dark:hover:bg-white/5 rounded-md text-claude-textSecondary hover:text-claude-text transition-colors"><Menu size={18} className="opacity-80" /></button></Tooltip>
            <Tooltip content={isSidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}><button onClick={() => setIsSidebarCollapsed(!isSidebarCollapsed)} className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-md text-claude-textSecondary hover:text-claude-text transition-colors"><IconSidebarToggle size={26} className="dark:invert transition-[filter] duration-200" /></button></Tooltip>
            {canGoBack ? (
              <Tooltip content="Back"><button onClick={handleNavBack} className="p-1.5 rounded-md transition-colors hover:bg-black/5 dark:hover:bg-white/5 text-claude-navActive"><ArrowLeft size={16} strokeWidth={1.5} /></button></Tooltip>
            ) : (<span className="p-1.5 text-claude-navDisabled"><ArrowLeft size={16} strokeWidth={1.5} /></span>)}
            {canGoForward ? (
              <Tooltip content="Forward"><button onClick={handleNavForward} className="p-1.5 rounded-md transition-colors hover:bg-black/5 dark:hover:bg-white/5 text-claude-navActive"><ArrowRight size={16} strokeWidth={1.5} /></button></Tooltip>
            ) : (<span className="p-1.5 text-claude-navDisabled"><ArrowRight size={16} strokeWidth={1.5} /></span>)}
            <Tooltip content="Agent Worktree (Ctrl+Shift+A)"><button onClick={() => { setShowAgentPanel(true); setShowAnalyticsPanel(false); }} className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-md text-claude-textSecondary hover:text-claude-text transition-colors"><GitBranch size={16} strokeWidth={1.5} /></button></Tooltip>
            <Tooltip content="使用统计"><button onClick={() => { setShowAnalyticsPanel(true); setShowAgentPanel(false); }} className="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-md text-claude-textSecondary hover:text-claude-text transition-colors"><BarChart3 size={16} strokeWidth={1.5} /></button></Tooltip>
            <Tooltip content="终端 (Ctrl+`)"><button onClick={() => setShowTerminal(!showTerminal)} className={`p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-md transition-colors ${showTerminal ? 'text-[#C6613F]' : 'text-claude-textSecondary hover:text-claude-text'}`}><Terminal size={16} strokeWidth={1.5} /></button></Tooltip>
          </div>
        </div>
        <Sidebar isCollapsed={isSidebarCollapsed} toggleSidebar={() => setIsSidebarCollapsed(!isSidebarCollapsed)} refreshTrigger={refreshTrigger} onNewChatClick={handleNewChat} onOpenSettings={() => { setShowSettings(true); setShowUpgrade(false); }} onOpenUpgrade={() => { setShowUpgrade(true); setShowSettings(false); }} onOpenDirectory={() => setShowDirectoryModal(true)} onCloseOverlays={() => { setShowSettings(false); setShowUpgrade(false); }} tunerConfig={tunerConfig} setTunerConfig={setTunerConfig} titleBarHeight={titleBarHeight} />
        <div className="flex-1 flex flex-col h-full min-w-0 overflow-hidden relative" style={{ paddingTop: `${titleBarHeight}px` }}>
          {isChatMode && !showSettings && !showUpgrade && location.pathname !== '/chats' && location.pathname !== '/customize' && location.pathname !== '/projects' && location.pathname !== '/artifacts' && (
            <ChatHeader title={currentChatTitle} showArtifacts={showArtifacts} documentPanelDoc={documentPanelDoc} onOpenArtifacts={handleOpenArtifacts} hasArtifacts={artifacts.length > 0} onTitleRename={handleTitleChange} />
          )}
          <div className="flex-1 flex overflow-hidden relative" ref={contentContainerRef}>
            <div className="flex-1 flex flex-col h-full min-w-0">
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
              ) : location.pathname === '/models' ? (
                <ModelsPage />
              ) : location.pathname === '/design' ? (
                <DesignPage />
              ) : location.pathname === '/memory' ? (
                <MemoryPanel />
              ) : location.pathname === '/code' ? (
                <MainContent />
              ) : (
                <ChatMainContent onNewChat={refreshSidebar} resetKey={newChatKey} tunerConfig={tunerConfig} onOpenDocument={handleOpenDocument} onArtifactsUpdate={handleArtifactsUpdate} onOpenArtifacts={handleOpenArtifacts} onTitleChange={handleTitleChange} onChatModeChange={handleChatModeChange} />
              )}
            </div>
            <div className={`h-full bg-claude-bg transition-all duration-300 ease-out flex z-20 relative ${(documentPanelDoc || showArtifacts) ? 'border-l border-claude-border' : ''} ${!(documentPanelDoc || showArtifacts) ? 'pointer-events-none' : ''}`} style={{ width: documentPanelDoc ? `${documentPanelWidth}%` : showArtifacts ? '360px' : '0px', opacity: (documentPanelDoc || showArtifacts) ? 1 : 0, overflow: 'hidden' }}>
              {documentPanelDoc && (
                <div className="absolute left-0 top-0 bottom-0 h-full z-50">
                  <DraggableDivider onResize={setDocumentPanelWidth} containerRef={contentContainerRef} />
                </div>
              )}
              <div className={`w-full h-full flex relative min-w-0 overflow-hidden`}>
                {(documentPanelDoc || showArtifacts) && (
                  <>
                    {documentPanelDoc ? (
                      <CodePanel document={documentPanelDoc} onClose={handleCloseDocument} />
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
      {showAgentPanel && <AgentPanel onClose={() => setShowAgentPanel(false)} />}
      {showAnalyticsPanel && <AnalyticsPanel onClose={() => setShowAnalyticsPanel(false)} />}
      {showTerminal && (
        <div
          className="fixed bottom-0 left-0 right-0 z-[80] border-t border-claude-border bg-[#1e1e2e] shadow-2xl"
          style={{ height: `${terminalHeight}px` }}
        >
          <TerminalPanel onClose={() => setShowTerminal(false)} />
        </div>
      )}
      <DirectoryModal isOpen={showDirectoryModal} onClose={() => setShowDirectoryModal(false)} />
      {showPromptSuggestions && (
        <div className="fixed bottom-20 left-1/2 -translate-x-1/2 z-[100]">
          <PromptSuggestionsPanel onSelectPrompt={(prompt) => {
            sessionStorage.setItem('prefill_input', prompt);
            setShowPromptSuggestions(false);
            handleNewChat();
            window.location.hash = '#/';
          }} />
        </div>
      )}
    </>
  );
};

const App = () => {
  return (
    <ErrorBoundary>
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
          <Route path="/models" element={<Layout />} />
          <Route path="/design" element={<Layout />} />
          <Route path="/memory" element={<Layout />} />
          <Route path="/code" element={<Layout />} />
          <Route path="/chat/:id" element={<Layout />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </HashRouter>
    </ErrorBoundary>
  );
};

export default App;
