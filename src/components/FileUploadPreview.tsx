import React from 'react';
import { X, Loader2, Github } from 'lucide-react';

export interface PendingFile {
  id: string;
  file: File;
  fileId?: string;
  fileName: string;
  fileType?: 'image' | 'document' | 'text';
  mimeType: string;
  size: number;
  progress: number;
  status: 'uploading' | 'done' | 'error';
  error?: string;
  previewUrl?: string;
  lineCount?: number;
  source?: 'github';
  ghRepo?: string;
  ghRef?: string;
}

interface FileUploadPreviewProps {
  files: PendingFile[];
  onRemove: (id: string) => void;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
}

const FileUploadPreview: React.FC<FileUploadPreviewProps> = ({ files, onRemove }) => {
  if (files.length === 0) return null;

  return (
    <>
      <style>{`
        .custom-scrollbar-horizontal::-webkit-scrollbar {
          height: 6px;
        }
        .custom-scrollbar-horizontal::-webkit-scrollbar-track {
          background: transparent;
        }
        .custom-scrollbar-horizontal::-webkit-scrollbar-thumb {
          background: rgba(0, 0, 0, 0.1);
          border-radius: 10px;
        }
        .dark .custom-scrollbar-horizontal::-webkit-scrollbar-thumb {
          background: rgba(255, 255, 255, 0.1);
        }
        .custom-scrollbar-horizontal::-webkit-scrollbar-thumb:hover {
          background: rgba(0, 0, 0, 0.2);
        }
        .dark .custom-scrollbar-horizontal::-webkit-scrollbar-thumb:hover {
          background: rgba(255, 255, 255, 0.2);
        }
      `}</style>
      <div className="flex flex-nowrap gap-3 px-4 pt-3 pb-2 overflow-x-auto overflow-y-hidden custom-scrollbar-horizontal">
        {files.map((f) => {
          const isImage = !!f.previewUrl && f.mimeType.startsWith('image/');
          const isGithub = f.source === 'github';
          const ext = f.fileName.split('.').pop()?.toUpperCase() || '?';
          const shortError = f.error ? (f.error.length > 26 ? `${f.error.slice(0, 26)}...` : f.error) : '上传失败';

          return (
            <div
              key={f.id}
            className={`relative group/file flex-shrink-0 w-28 h-28 rounded-xl border border-gray-200 hover:border-gray-300 dark:border-[#5B5B56] dark:hover:border-gray-400 overflow-hidden transition-all ${
              isImage ? '' : 'bg-white dark:bg-claude-input p-3 flex flex-col justify-between'
            }`}
          >
            {isGithub ? (
              <>
                <div className="min-w-0">
                  <div className="text-[13px] font-medium text-claude-text leading-tight break-words line-clamp-2" title={f.ghRepo || f.fileName}>
                    {f.ghRepo || f.fileName}
                  </div>
                  <div className="text-[11px] text-claude-textSecondary mt-1">
                    {f.status === 'uploading' ? (
                      <span className="flex items-center gap-1">
                        <Loader2 size={10} className="animate-spin" />
                        {f.progress}%
                      </span>
                    ) : f.status === 'error' ? (
                      <span className="text-red-500" title={f.error}>{shortError}</span>
                    ) : (
                      f.ghRef || 'main'
                    )}
                  </div>
                </div>
                <div className="self-start flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium border border-gray-200 dark:border-[#5B5B56] bg-gray-50 dark:bg-claude-input rounded text-claude-textSecondary uppercase">
                  <Github size={10} />
                  GITHUB
                </div>
              </>
            ) : isImage && f.previewUrl ? (
              <>
                <img
                  src={f.previewUrl}
                  alt={f.fileName}
                  className="w-full h-full object-cover"
                />
                {f.status !== 'done' && (
                  <div
                    className={`absolute inset-0 flex items-center justify-center text-[11px] px-2 text-center ${
                      f.status === 'error' ? 'bg-red-600/80 text-white' : 'bg-black/45 text-white'
                    }`}
                    title={f.error}
                  >
                    {f.status === 'uploading' ? (
                      <span className="flex items-center gap-1">
                        <Loader2 size={12} className="animate-spin" />
                        {f.progress}%
                      </span>
                    ) : (
                      shortError
                    )}
                  </div>
                )}
              </>
            ) : (
              <>
                <div className="min-w-0">
                  <div className="text-[13px] font-medium text-claude-text truncate" title={f.fileName}>
                    {f.fileName}
                  </div>
                  <div className="text-[11px] text-claude-textSecondary mt-0.5">
                    {f.status === 'uploading' ? (
                      <span className="flex items-center gap-1">
                        <Loader2 size={10} className="animate-spin" />
                        {f.progress}%
                      </span>
                    ) : f.status === 'error' ? (
                      <span className="text-red-500" title={f.error}>{shortError}</span>
                    ) : (
                      f.lineCount !== undefined ? `${f.lineCount} lines` : formatSize(f.size)
                    )}
                  </div>
                </div>
                
                <div className="self-start px-1.5 py-0.5 text-[10px] font-medium border border-gray-200 dark:border-[#5B5B56] bg-gray-50 dark:bg-claude-input rounded text-claude-textSecondary uppercase">
                  {ext}
                </div>
              </>
            )}

            <button
              onClick={() => onRemove(f.id)}
              className="absolute top-1 right-1 w-6 h-6 bg-black/50 hover:bg-black/70 text-white rounded-full flex items-center justify-center opacity-0 group-hover/file:opacity-100 transition-opacity backdrop-blur-sm"
            >
              <X size={14} />
            </button>
          </div>
        );
      })}
      </div>
    </>
  );
};

export default FileUploadPreview;
