import React, { useState, useEffect, useRef, useCallback } from 'react';
import { X, RefreshCw, Monitor, Smartphone, Maximize2, Minimize2, Loader2 } from 'lucide-react';

interface PreviewContent {
  html: string;
  full: string;
  conversationId: string;
}

const LivePreviewRenderer: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [content, setContent] = useState<PreviewContent | null>(null);
  const [viewport, setViewport] = useState<'desktop' | 'mobile'>('desktop');
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const contentRef = useRef<string>('');

  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail as PreviewContent;
      if (!detail || !detail.html) return;
      contentRef.current = detail.html;
      setContent(detail);
      setIsGenerating(true);
      // Stop "generating" after 2s of no updates
      clearTimeout((window as any).__livePreviewTimer);
      (window as any).__livePreviewTimer = setTimeout(() => setIsGenerating(false), 2000);
    };
    window.addEventListener('live-preview', handler as EventListener);
    return () => window.removeEventListener('live-preview', handler as EventListener);
  }, []);

  useEffect(() => {
    if (!content?.html || !iframeRef.current) return;
    const blob = new Blob([content.html], { type: 'text/html;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    iframeRef.current.src = url;
    return () => URL.revokeObjectURL(url);
  }, [content?.html]);

  if (!content) {
    return (
      <div className="h-full flex flex-col bg-claude-bg">
        <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border">
          <span className="text-[13px] font-semibold text-claude-text flex items-center gap-2">
            <Monitor size={14} className="text-blue-400" />
            实时预览
          </span>
          <button onClick={onClose} className="p-1 rounded hover:bg-claude-hover text-claude-textSecondary"><X size={14} /></button>
        </div>
        <div className="flex-1 flex items-center justify-center text-claude-textSecondary text-[12px] px-6 text-center leading-relaxed">
          AI 生成 HTML 代码时，实时预览将自动显示在这里
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-claude-bg">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border">
        <div className="flex items-center gap-2">
          <Monitor size={14} className="text-blue-400" />
          <span className="text-[13px] font-semibold text-claude-text">实时预览</span>
          {isGenerating && (
            <span className="flex items-center gap-1 text-[10px] text-amber-400">
              <Loader2 size={10} className="animate-spin" />
              生成中
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button onClick={() => setViewport('desktop')} className={`p-1 rounded ${viewport === 'desktop' ? 'bg-claude-hover text-claude-text' : 'text-claude-textSecondary hover:text-claude-text'}`}>
            <Monitor size={13} />
          </button>
          <button onClick={() => setViewport('mobile')} className={`p-1 rounded ${viewport === 'mobile' ? 'bg-claude-hover text-claude-text' : 'text-claude-textSecondary hover:text-claude-text'}`}>
            <Smartphone size={13} />
          </button>
          <button onClick={() => setIsFullscreen(!isFullscreen)} className="p-1 rounded text-claude-textSecondary hover:text-claude-text">
            {isFullscreen ? <Minimize2 size={13} /> : <Maximize2 size={13} />}
          </button>
          <button onClick={onClose} className="p-1 rounded hover:bg-claude-hover text-claude-textSecondary"><X size={14} /></button>
        </div>
      </div>

      {/* Preview area with animated border when generating */}
      <div className={`flex-1 relative m-3 rounded-xl overflow-hidden ${isGenerating ? 'preview-generating-border' : 'border border-claude-border'}`}>
        <iframe
          ref={iframeRef}
          className="w-full h-full bg-white"
          style={viewport === 'mobile' ? { maxWidth: '375px', margin: '0 auto' } : {}}
          title="Live Preview"
          sandbox="allow-scripts"
        />
        {isGenerating && (
          <div className="absolute bottom-3 right-3 px-2 py-1 rounded-full bg-amber-500/20 backdrop-blur-sm border border-amber-500/30 text-[10px] text-amber-400 flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping" />
            流式更新中
          </div>
        )}
      </div>
    </div>
  );
};

export default LivePreviewRenderer;
