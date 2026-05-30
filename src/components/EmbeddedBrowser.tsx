import React, { useState, useRef, useCallback } from 'react';
import {
  ArrowLeft,
  ArrowRight,
  RefreshCw,
  ZoomIn,
  ZoomOut,
  Maximize2,
  ExternalLink,
  Maximize,
  Minimize,
  X,
} from 'lucide-react';

interface EmbeddedBrowserProps {
  initialUrl?: string;
  onClose?: () => void;
  className?: string;
}

const ZOOM_STEP = 0.25;
const MIN_ZOOM = 0.25;
const MAX_ZOOM = 2;

const EmbeddedBrowser: React.FC<EmbeddedBrowserProps> = ({
  initialUrl = 'https://example.com',
  onClose,
  className = '',
}) => {
  const [url, setUrl] = useState(initialUrl);
  const [currentUrl, setCurrentUrl] = useState(initialUrl);
  const [zoom, setZoom] = useState(1);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [history, setHistory] = useState<string[]>([initialUrl]);
  const [historyIndex, setHistoryIndex] = useState(0);

  const iframeRef = useRef<HTMLIFrameElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const navigate = useCallback((targetUrl: string) => {
    const normalizedUrl = targetUrl.startsWith('http') ? targetUrl : `https://${targetUrl}`;
    setIsLoading(true);
    setCurrentUrl(normalizedUrl);
    setUrl(normalizedUrl);

    const newHistory = history.slice(0, historyIndex + 1);
    newHistory.push(normalizedUrl);
    setHistory(newHistory);
    setHistoryIndex(newHistory.length - 1);
  }, [history, historyIndex]);

  const handleGo = () => {
    navigate(url);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleGo();
    }
  };

  const handleBack = () => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1;
      setHistoryIndex(newIndex);
      const prevUrl = history[newIndex];
      setCurrentUrl(prevUrl);
      setUrl(prevUrl);
    }
  };

  const handleForward = () => {
    if (historyIndex < history.length - 1) {
      const newIndex = historyIndex + 1;
      setHistoryIndex(newIndex);
      const nextUrl = history[newIndex];
      setCurrentUrl(nextUrl);
      setUrl(nextUrl);
    }
  };

  const handleRefresh = () => {
    if (iframeRef.current) {
      setIsLoading(true);
      iframeRef.current.src = currentUrl;
    }
  };

  const handleZoomIn = () => {
    setZoom(prev => Math.min(prev + ZOOM_STEP, MAX_ZOOM));
  };

  const handleZoomOut = () => {
    setZoom(prev => Math.max(prev - ZOOM_STEP, MIN_ZOOM));
  };

  const handleResetZoom = () => {
    setZoom(1);
  };

  const handleOpenInNewWindow = () => {
    window.open(currentUrl, '_blank', 'noopener,noreferrer');
  };

  const handleToggleFullscreen = async () => {
    if (!document.fullscreenElement) {
      await containerRef.current?.requestFullscreen();
      setIsFullscreen(true);
    } else {
      await document.exitFullscreen();
      setIsFullscreen(false);
    }
  };

  const handleIframeLoad = () => {
    setIsLoading(false);
  };

  const handleIframeError = () => {
    setIsLoading(false);
  };

  return (
    <div
      ref={containerRef}
      className={`flex flex-col bg-claude-bg border border-claude-border rounded-xl overflow-hidden ${className}`}
    >
      {/* Toolbar */}
      <div className="flex items-center gap-1 p-2 border-b border-claude-border bg-claude-input/30">
        {/* Navigation Buttons */}
        <button
          onClick={handleBack}
          disabled={historyIndex <= 0}
          className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="后退"
        >
          <ArrowLeft size={16} />
        </button>
        <button
          onClick={handleForward}
          disabled={historyIndex >= history.length - 1}
          className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="前进"
        >
          <ArrowRight size={16} />
        </button>
        <button
          onClick={handleRefresh}
          className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors"
          title="刷新"
        >
          <RefreshCw size={16} className={isLoading ? 'animate-spin' : ''} />
        </button>

        {/* URL Bar */}
        <div className="flex-1 mx-2">
          <input
            ref={inputRef}
            type="text"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            onKeyDown={handleKeyDown}
            className="w-full px-3 py-1.5 bg-claude-input border border-claude-border rounded-lg text-sm text-claude-text placeholder-claude-textSecondary/50 focus:outline-none focus:border-claude-textSecondary/50 transition-colors"
            placeholder="输入网址..."
          />
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-0.5">
          {/* Zoom Controls */}
          <button
            onClick={handleZoomOut}
            disabled={zoom <= MIN_ZOOM}
            className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="缩小"
          >
            <ZoomOut size={16} />
          </button>
          <button
            onClick={handleResetZoom}
            className="px-2 py-1.5 rounded-lg text-xs text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors min-w-[40px] text-center"
            title="重置缩放"
          >
            {Math.round(zoom * 100)}%
          </button>
          <button
            onClick={handleZoomIn}
            disabled={zoom >= MAX_ZOOM}
            className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="放大"
          >
            <ZoomIn size={16} />
          </button>

          <div className="w-px h-5 bg-claude-border mx-1" />

          {/* Open in New Window */}
          <button
            onClick={handleOpenInNewWindow}
            className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors"
            title="在新窗口中打开"
          >
            <ExternalLink size={16} />
          </button>

          {/* Fullscreen Toggle */}
          <button
            onClick={handleToggleFullscreen}
            className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors"
            title={isFullscreen ? '退出全屏' : '全屏'}
          >
            {isFullscreen ? <Minimize size={16} /> : <Maximize size={16} />}
          </button>

          {/* Close Button */}
          {onClose && (
            <button
              onClick={onClose}
              className="p-1.5 rounded-lg text-claude-textSecondary hover:bg-red-500/20 hover:text-red-400 transition-colors"
              title="关闭"
            >
              <X size={16} />
            </button>
          )}
        </div>
      </div>

      {/* Iframe Container */}
      <div className="flex-1 relative bg-white">
        {/* Loading Indicator */}
        {isLoading && (
          <div className="absolute top-0 left-0 right-0 h-0.5 bg-claude-border overflow-hidden z-10">
            <div
              className="h-full bg-blue-500 animate-[loading_1.5s_ease-in-out_infinite]"
              style={{
                width: '30%',
                background: 'linear-gradient(90deg, transparent, #3b82f6, transparent)',
              }}
            />
          </div>
        )}

        {/* Iframe */}
        <iframe
          ref={iframeRef}
          src={currentUrl}
          onLoad={handleIframeLoad}
          onError={handleIframeError}
          className="w-full h-full border-0"
          style={{
            transform: `scale(${zoom})`,
            transformOrigin: 'top left',
            width: `${100 / zoom}%`,
            height: `${100 / zoom}%`,
          }}
          sandbox="allow-scripts allow-popups allow-forms"
          title="Embedded Browser"
        />

        {/* Empty State */}
        {!currentUrl && (
          <div className="absolute inset-0 flex items-center justify-center bg-claude-bg">
            <div className="text-center">
              <Maximize2 size={48} className="mx-auto text-claude-textSecondary/30 mb-4" />
              <p className="text-claude-textSecondary text-sm">输入网址开始浏览</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default EmbeddedBrowser;