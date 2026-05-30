import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  Monitor,
  Tablet,
  Smartphone,
  RefreshCw,
  ExternalLink,
  Globe,
  FileCode,
  Loader2,
} from 'lucide-react';

interface LivePreviewPanelProps {
  content?: string;
  filePath?: string;
  fileType?: 'html' | 'react' | 'jsx' | 'tsx';
  onRefresh?: () => void;
  projectId?: string;
  initialPrompt?: string;
  designType?: string;
  initialStyle?: string;
  onBack?: () => void;
  onSave?: (content: string) => void;
}

type ViewportMode = 'desktop' | 'tablet' | 'mobile';

const VIEWPORT_WIDTHS: Record<ViewportMode, string> = {
  desktop: '100%',
  tablet: '768px',
  mobile: '375px',
};

function buildPreviewHtml(content: string, fileType?: string): string {
  const isReact = fileType === 'react' || fileType === 'jsx' || fileType === 'tsx';

  if (isReact) {
    return `<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>
  *,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
  body{font-family:system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;line-height:1.5;color:#e0e0e8;background:#1a1a2e;padding:16px}
  #root{min-height:100%}
</style>
</head>
<body>
<div id="root">${content}</div>
</body>
</html>`;
  }

  return `<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>
  *,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
  body{font-family:system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;line-height:1.5;color:#e0e0e8;background:#1a1a2e;padding:16px}
</style>
</head>
<body>
${content}
</body>
</html>`;
}

const LivePreviewPanel: React.FC<LivePreviewPanelProps> = ({
  content,
  filePath,
  fileType,
  onRefresh,
}) => {
  const [viewport, setViewport] = useState<ViewportMode>('desktop');
  const [iframeKey, setIframeKey] = useState(0);
  const [iframeLoading, setIframeLoading] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleRefresh = useCallback(() => {
    setIframeKey(prev => prev + 1);
    setIframeLoading(true);
    onRefresh?.();
  }, [onRefresh]);

  useEffect(() => {
    if (!content) return;

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(() => {
      setIframeKey(prev => prev + 1);
      setIframeLoading(true);
    }, 300);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [content]);

  const handleIframeLoad = useCallback(() => {
    setIframeLoading(false);
  }, []);

  const previewHtml = content ? buildPreviewHtml(content, fileType) : '';

  const handleOpenExternal = useCallback(() => {
    const win = window.open('', '_blank');
    if (win && previewHtml) {
      win.document.write(previewHtml);
      win.document.close();
    }
  }, [previewHtml]);

  return (
    <div className="flex flex-col h-full bg-[#1a1a2e]">
      <div className="flex items-center gap-2 px-3 py-2 bg-[#1a1a2e] border-b border-[#2d2d44] flex-shrink-0">
        <FileCode size={14} className="text-[#8B5CF6] flex-shrink-0" />
        <div className="flex-1 flex items-center gap-1.5 min-w-0">
          <Globe size={12} className="text-[#6b6b8a] flex-shrink-0" />
          <span className="text-[12px] text-[#9999aa] truncate">
            {filePath || '/'}
          </span>
        </div>
        <button
          onClick={handleRefresh}
          className="p-1.5 rounded-md hover:bg-[#2d2d44] text-[#9999aa] hover:text-[#e0e0e8] transition-colors flex-shrink-0"
          title="Refresh preview"
        >
          <RefreshCw
            size={13}
            className={iframeLoading ? 'animate-spin' : ''}
          />
        </button>
        {previewHtml && (
          <button
            onClick={handleOpenExternal}
            className="p-1.5 rounded-md hover:bg-[#2d2d44] text-[#9999aa] hover:text-[#e0e0e8] transition-colors flex-shrink-0"
            title="Open in external window"
          >
            <ExternalLink size={13} />
          </button>
        )}
      </div>

      <div className="flex items-center justify-center gap-1 px-3 py-1.5 bg-[#1a1a2e] border-b border-[#2d2d44] flex-shrink-0">
        {(['desktop', 'tablet', 'mobile'] as ViewportMode[]).map(mode => {
          const Icon =
            mode === 'desktop'
              ? Monitor
              : mode === 'tablet'
                ? Tablet
                : Smartphone;
          const isActive = viewport === mode;
          return (
            <button
              key={mode}
              onClick={() => setViewport(mode)}
              className={`flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                isActive
                  ? 'bg-[#2d2d44] text-[#e0e0e8]'
                  : 'text-[#6b6b8a] hover:text-[#9999aa] hover:bg-[#22223a]'
              }`}
              title={`${
                mode.charAt(0).toUpperCase() + mode.slice(1)
              } view`}
            >
              <Icon size={12} />
              <span className="capitalize">{mode}</span>
            </button>
          );
        })}
      </div>

      <div className="flex-1 relative overflow-auto bg-[#0f0f23]">
        {!content ? (
          <div className="flex flex-col items-center justify-center h-full text-[#6b6b8a] gap-3">
            <Monitor size={40} className="opacity-20" />
            <div className="text-center">
              <p className="text-[14px] font-medium">No preview available</p>
              <p className="text-[12px] mt-1 opacity-60">
                HTML content will render here
              </p>
            </div>
          </div>
        ) : (
          <div className="h-full flex flex-col items-center p-2">
            {iframeLoading && (
              <div className="absolute inset-0 z-10 flex items-center justify-center bg-[#1a1a2e]/60 backdrop-blur-sm">
                <div className="flex flex-col items-center gap-2">
                  <Loader2
                    size={24}
                    className="animate-spin text-[#8B5CF6]"
                  />
                  <span className="text-[12px] text-[#9999aa]">
                    Loading preview...
                  </span>
                </div>
              </div>
            )}

            <div
              className="w-full h-full rounded-lg overflow-hidden border border-[#2d2d44] shadow-lg transition-all duration-300"
              style={{ maxWidth: VIEWPORT_WIDTHS[viewport] }}
            >
              <iframe
                key={iframeKey}
                srcDoc={previewHtml}
                sandbox="allow-scripts"
                className="w-full h-full"
                title="Live Preview"
                onLoad={handleIframeLoad}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default LivePreviewPanel;