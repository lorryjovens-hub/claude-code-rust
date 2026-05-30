import React, { useState, useEffect, useCallback, useRef } from 'react';

export interface ResponsiveLayoutProps {
  sidebar: React.ReactNode;
  main: React.ReactNode;
  rightPanel?: React.ReactNode;
  showRightPanel?: boolean;
  defaultSidebarCollapsed?: boolean;
  onSidebarToggle?: (collapsed: boolean) => void;
}

export function useResponsive() {
  const [isMobile, setIsMobile] = useState(false);
  const [isTablet, setIsTablet] = useState(false);
  const [isDesktop, setIsDesktop] = useState(true);
  const [windowWidth, setWindowWidth] = useState(0);

  useEffect(() => {
    const handleResize = () => {
      const w = window.innerWidth;
      setWindowWidth(w);
      setIsMobile(w < 768);
      setIsTablet(w >= 768 && w < 1024);
      setIsDesktop(w >= 1024);
    };

    handleResize();

    let resizeTimer: ReturnType<typeof setTimeout>;
    const throttledResize = () => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(handleResize, 100);
    };

    window.addEventListener('resize', throttledResize);
    return () => {
      window.removeEventListener('resize', throttledResize);
      clearTimeout(resizeTimer);
    };
  }, []);

  return { isMobile, isTablet, isDesktop, windowWidth };
}

const ResponsiveLayout: React.FC<ResponsiveLayoutProps> = ({
  sidebar,
  main,
  rightPanel,
  showRightPanel = false,
  defaultSidebarCollapsed = false,
  onSidebarToggle,
}) => {
  const { isMobile, isTablet, isDesktop } = useResponsive();
  const [sidebarCollapsed, setSidebarCollapsed] = useState(defaultSidebarCollapsed);
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const touchStartX = useRef(0);

  const toggleSidebar = useCallback(() => {
    const next = !sidebarCollapsed;
    setSidebarCollapsed(next);
    onSidebarToggle?.(next);
  }, [sidebarCollapsed, onSidebarToggle]);

  // Mobile swipe gesture handling
  useEffect(() => {
    if (!isMobile) return;

    const handleTouchStart = (e: TouchEvent) => {
      touchStartX.current = e.touches[0].clientX;
    };

    const handleTouchEnd = (e: TouchEvent) => {
      const deltaX = e.changedTouches[0].clientX - touchStartX.current;
      if (deltaX > 80 && touchStartX.current < 40) {
        setMobileSidebarOpen(true);
      } else if (deltaX < -80 && mobileSidebarOpen) {
        setMobileSidebarOpen(false);
      }
    };

    document.addEventListener('touchstart', handleTouchStart, { passive: true });
    document.addEventListener('touchend', handleTouchEnd, { passive: true });

    return () => {
      document.removeEventListener('touchstart', handleTouchStart);
      document.removeEventListener('touchend', handleTouchEnd);
    };
  }, [isMobile, mobileSidebarOpen]);

  // Close mobile sidebar on escape
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && mobileSidebarOpen) {
        setMobileSidebarOpen(false);
      }
    };

    if (mobileSidebarOpen) {
      document.addEventListener('keydown', handleEscape);
      document.body.style.overflow = 'hidden';
    }

    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = '';
    };
  }, [mobileSidebarOpen]);

  // Mobile overlay
  const renderMobileOverlay = () => {
    if (!isMobile || !mobileSidebarOpen) return null;

    return (
      <div
        className="fixed inset-0 z-40 bg-black/50 animate-fade-in touch-manipulation"
        onClick={() => setMobileSidebarOpen(false)}
      />
    );
  };

  // Mobile sidebar wrapper
  const renderMobileSidebar = () => {
    if (!isMobile) return null;

    return (
      <>
        {renderMobileOverlay()}
        <div
          className={`fixed inset-y-0 left-0 z-50 w-[280px] bg-claude-sidebar transform transition-transform duration-300 ease-out touch-manipulation ${
            mobileSidebarOpen ? 'translate-x-0' : '-translate-x-full'
          }`}
        >
          {sidebar}
        </div>
      </>
    );
  };

  // Desktop sidebar
  const renderDesktopSidebar = () => {
    if (isMobile) return null;

    return (
      <div
        className={`flex-shrink-0 h-full bg-claude-sidebar transition-all duration-300 ease-out overflow-hidden ${
          sidebarCollapsed ? 'w-0' : 'w-[288px]'
        }`}
      >
        {sidebar}
      </div>
    );
  };

  // Right panel
  const renderRightPanel = () => {
    if (!rightPanel || !showRightPanel) return null;

    return (
      <div
        className={`flex-shrink-0 h-full bg-claude-sidebar border-l border-claude-border transition-all duration-300 ease-out ${
          isMobile ? 'w-full' : isTablet ? 'w-[320px]' : 'w-[400px]'
        }`}
      >
        {rightPanel}
      </div>
    );
  };

  // Mobile hamburger button
  const renderMobileMenuButton = () => {
    if (!isMobile) return null;

    return (
      <button
        onClick={() => setMobileSidebarOpen(true)}
        className="fixed top-3 left-3 z-30 p-2 rounded-lg bg-claude-input border border-claude-border text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover transition-colors touch-manipulation"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
          <line x1="3" y1="6" x2="21" y2="6" />
          <line x1="3" y1="12" x2="21" y2="12" />
          <line x1="3" y1="18" x2="21" y2="18" />
        </svg>
      </button>
    );
  };

  return (
    <div className="flex w-full h-full overflow-hidden bg-claude-bg">
      {renderMobileSidebar()}
      {renderDesktopSidebar()}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {renderMobileMenuButton()}
        <div className="flex-1 flex overflow-hidden relative">
          <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
            {main}
          </div>
          {renderRightPanel()}
        </div>
      </div>
    </div>
  );
};

export default ResponsiveLayout;
