import React, { useState, useEffect, useRef, useCallback } from 'react';
import { X, RefreshCw, Download, ExternalLink, ZoomIn, ZoomOut, Maximize2 } from 'lucide-react';
import { previewService, type PreviewContent } from '../services/previewService';
import { getPreviewEventsUrl } from '../services/config';

interface PreviewPanelProps {
  isOpen: boolean;
  onClose: () => void;
  content: string;
  contentType: string;
  previewId?: string;
  title?: string;
}

const PreviewPanel: React.FC<PreviewPanelProps> = ({
  isOpen,
  onClose,
  content,
  contentType,
  previewId,
  title = 'Preview',
}) => {
  const [zoom, setZoom] = useState(100);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [liveContent, setLiveContent] = useState(content);
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const blobUrlRef = useRef<string | null>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  const createBlobUrl = useCallback((content: string, contentType: string) => {
    try {
      const blob = new Blob([content], { type: contentType });
      const url = URL.createObjectURL(blob);
      return url;
    } catch (e) {
      console.error('Failed to create blob URL:', e);
      setError('Failed to render preview');
      return null;
    }
  }, []);

  useEffect(() => {
    if (isOpen && liveContent) {
      setError(null);
      const url = createBlobUrl(liveContent, contentType);
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
      }
      blobUrlRef.current = url;
    }

    return () => {
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
        blobUrlRef.current = null;
      }
    };
  }, [isOpen, liveContent, contentType, createBlobUrl]);

  useEffect(() => {
    setLiveContent(content);
  }, [content]);

  useEffect(() => {
    if (isOpen && previewId) {
      eventSourceRef.current = new EventSource(getPreviewEventsUrl(previewId));

      eventSourceRef.current.onmessage = (event) => {
        try {
          const data: PreviewContent = JSON.parse(event.data);
          setLiveContent(data.content);
          console.log('Preview content updated via SSE');
        } catch (e) {
          console.error('Failed to parse SSE event:', e);
        }
      };

      eventSourceRef.current.onerror = (error) => {
        console.error('SSE connection error:', error);
        eventSourceRef.current?.close();
      };

      return () => {
        if (eventSourceRef.current) {
          eventSourceRef.current.close();
          eventSourceRef.current = null;
        }
      };
    }
  }, [isOpen, previewId]);

  const handleRefresh = useCallback(() => {
    setIsRefreshing(true);
    if (previewId) {
      previewService.getPreview(previewId).then((preview) => {
        if (preview) {
          setLiveContent(preview.content);
        }
        setIsRefreshing(false);
      }).catch(() => {
        setIsRefreshing(false);
      });
    } else {
      setTimeout(() => {
        setIsRefreshing(false);
      }, 500);
    }
  }, [previewId]);

  const handleZoomIn = () => {
    setZoom((prev) => Math.min(prev + 10, 200));
  };

  const handleZoomOut = () => {
    setZoom((prev) => Math.max(prev - 10, 50));
  };

  const handleResetZoom = () => {
    setZoom(100);
  };

  const handleDownload = () => {
    if (liveContent) {
      const blob = new Blob([liveContent], { type: contentType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `preview.${contentType.includes('html') ? 'html' : 'txt'}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  };

  const handleOpenInBrowser = () => {
    if (blobUrlRef.current) {
      window.open(blobUrlRef.current, '_blank');
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      <div className="relative w-[90vw] max-w-6xl h-[85vh] bg-claude-bg rounded-2xl shadow-2xl border border-claude-border flex flex-col overflow-hidden animate-in fade-in zoom-in duration-200">
        <div className="flex items-center justify-between px-4 py-3 bg-claude-surface border-b border-claude-border">
          <div className="flex items-center gap-3">
            <h2 className="text-[14px] font-semibold text-claude-text truncate max-w-md">
              {title}
            </h2>
            <span className="px-2 py-0.5 bg-claude-hover text-claude-textSecondary text-[11px] font-medium rounded">
              {contentType}
            </span>
            {previewId && (
              <span className="px-2 py-0.5 bg-green-500/10 text-green-400 text-[11px] font-medium rounded flex items-center gap-1">
                <span className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                Live
              </span>
            )}
          </div>

          <div className="flex items-center gap-1">
            <div className="flex items-center gap-1 px-2 py-1 bg-claude-hover rounded-lg">
              <button
                onClick={handleZoomOut}
                className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-input rounded transition-colors"
                title="Zoom Out"
              >
                <ZoomOut size={14} />
              </button>
              <span className="px-2 text-[12px] text-claude-textSecondary font-medium min-w-[50px] text-center">
                {zoom}%
              </span>
              <button
                onClick={handleZoomIn}
                className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-input rounded transition-colors"
                title="Zoom In"
              >
                <ZoomIn size={14} />
              </button>
              <button
                onClick={handleResetZoom}
                className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-input rounded transition-colors"
                title="Reset Zoom"
              >
                <Maximize2 size={14} />
              </button>
            </div>

            <div className="h-6 w-px bg-claude-border mx-2" />

            <button
              onClick={handleRefresh}
              disabled={isRefreshing}
              className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-input rounded-lg transition-colors disabled:opacity-50"
              title="Refresh"
            >
              <RefreshCw size={14} className={isRefreshing ? 'animate-spin' : ''} />
            </button>

            <button
              onClick={handleDownload}
              className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-input rounded-lg transition-colors"
              title="Download"
            >
              <Download size={14} />
            </button>

            <button
              onClick={handleOpenInBrowser}
              className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-input rounded-lg transition-colors"
              title="Open in Browser"
            >
              <ExternalLink size={14} />
            </button>

            <div className="h-6 w-px bg-claude-border mx-1" />

            <button
              onClick={onClose}
              className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-input rounded-lg transition-colors"
              title="Close"
            >
              <X size={14} />
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-hidden bg-claude-bg p-4">
          {error ? (
            <div className="flex flex-col items-center justify-center h-full text-claude-textSecondary">
              <div className="w-12 h-12 rounded-xl bg-red-500/10 flex items-center justify-center mb-4">
                <span className="text-red-400 text-xl">!</span>
              </div>
              <p className="text-[14px] font-medium">{error}</p>
            </div>
          ) : blobUrlRef.current ? (
            <div
              className="w-full h-full overflow-auto"
              style={{ transform: `scale(${zoom / 100})`, transformOrigin: 'top left' }}
            >
              <iframe
                ref={iframeRef}
                src={blobUrlRef.current}
                className="w-full h-full border-0 bg-white rounded-xl"
                sandbox="allow-scripts allow-popups"
                title="Preview"
              />
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-claude-textSecondary">
              <div className="w-12 h-12 rounded-xl bg-claude-hover flex items-center justify-center mb-4">
                <RefreshCw size={24} className="animate-spin" />
              </div>
              <p className="text-[14px] font-medium">Loading preview...</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PreviewPanel;
