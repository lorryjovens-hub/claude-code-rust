import React, { useState, useEffect } from 'react';
import { FileText, File, X, Code2, Download, FolderOpen, Github } from 'lucide-react';
import { getAttachmentUrl } from '../api';

interface Attachment {
  id: string;
  file_type: string;
  file_name: string;
  mime_type: string;
  file_size?: number;
  line_count?: number;
  source?: string;
  gh_repo?: string;
  gh_ref?: string;
}

import { DocumentInfo } from './DocumentCard';

interface MessageAttachmentsProps {
  attachments: Attachment[];
  onOpenDocument?: (doc: DocumentInfo) => void;
}

const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

// 获取文件扩展名
function getFileExtension(fileName: string | undefined | null): string {
  if (!fileName) return '';
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  return ext.toUpperCase();
}

// 格式化文件大小
function formatFileSize(bytes?: number): string {
  if (!bytes) return '';
  if (bytes > 1024 * 1024) {
    return `${(bytes / 1024 / 1024).toFixed(1)}MB`;
  } else if (bytes > 1024) {
    return `${(bytes / 1024).toFixed(1)}KB`;
  } else {
    return `${bytes}B`;
  }
}

// Open the folder containing a file (Electron only), with deduplication
async function openFileInFolder(fileId: string) {
  if (!isTauri) return false;
  try {
    // Ask bridge server for the local path
    const res = await fetch(`http://127.0.0.1:30080/api/uploads/${encodeURIComponent(fileId)}/path`);
    if (!res.ok) return false;
    const data = await res.json();
    if (data.localPath) {
      const { tauriAPI } = await import('../utils/tauriAPI');
      return await tauriAPI.showItemInFolder(data.localPath);
    }
    if (data.folder) {
      const { tauriAPI } = await import('../utils/tauriAPI');
      return await tauriAPI.openFolder(data.folder);
    }
  } catch (err) {
    console.error('[Attachment] Failed to open folder:', err);
  }
  return false;
}

const AttachmentCard: React.FC<{ attachment: Attachment; onClick: () => void }> = ({ attachment, onClick }) => {
  if (!attachment || !attachment.id) return null;
  const [thumbnailUrl, setThumbnailUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const isImage = attachment.file_type === 'image' || (attachment.mime_type?.startsWith('image/') ?? false);
  const isGithub = attachment.source === 'github' || attachment.file_type === 'github';

  if (isGithub) {
    const repo = attachment.gh_repo || attachment.file_name || 'github';
    const ref = attachment.gh_ref || 'main';
    return (
      <div
        className="relative w-28 h-28 rounded-xl border border-gray-200 dark:border-[#5B5B56] bg-white dark:bg-claude-input p-3 flex flex-col justify-between overflow-hidden shadow-sm"
        title={repo}
      >
        <div className="min-w-0">
          <div className="text-[13px] font-medium text-claude-text leading-tight break-words line-clamp-2">
            {repo}
          </div>
          <div className="text-[11px] text-claude-textSecondary mt-1">
            {ref}
          </div>
        </div>
        <div className="self-start flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium border border-gray-200 dark:border-[#5B5B56] bg-gray-50 dark:bg-claude-input rounded text-claude-textSecondary uppercase">
          <Github size={10} />
          GITHUB
        </div>
      </div>
    );
  }

  useEffect(() => {
    if (isImage) {
      if (isTauri) {
        // In Electron, use bridge server to serve raw file
        setThumbnailUrl(`http://127.0.0.1:30080/api/uploads/${encodeURIComponent(attachment.id)}/raw`);
      } else {
        const url = getAttachmentUrl(attachment.id);
        const token = localStorage.getItem('auth_token');
        setThumbnailUrl(`${url}${url.includes('?') ? '&' : '?'}token=${token}`);
      }
      setLoading(false);
      return;
    }

    setLoading(false);
  }, [attachment.id, isImage]);

  // 图片卡片
  if (isImage) {
    return (
      <div
        className="relative w-28 h-28 rounded-xl overflow-hidden cursor-pointer border border-gray-200 hover:border-gray-300 dark:border-[#5B5B56] dark:hover:border-gray-400 group bg-white dark:bg-claude-input shadow-sm hover:opacity-90 transition-all"
        onClick={onClick}
      >
        <img
          src={thumbnailUrl || ''}
          alt={attachment.file_name}
          className="w-full h-full object-cover"
        />
        {/* Folder icon overlay for Electron */}
        {isTauri && (
          <div className="absolute bottom-1 right-1 bg-black/40 rounded-md p-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <FolderOpen size={12} className="text-white" />
          </div>
        )}
      </div>
    );
  }

  // 文档卡片
  const ext = getFileExtension(attachment.file_name || '');

  return (
    <div
      className="relative w-28 h-28 rounded-xl cursor-pointer border border-gray-200 hover:border-gray-300 dark:border-[#5B5B56] dark:hover:border-gray-400 group overflow-hidden transition-all bg-white dark:bg-claude-input p-3 flex flex-col justify-between shadow-sm"
      onClick={onClick}
      title={`${attachment.file_name}${isTauri ? '\nClick to open in folder' : ''}`}
    >
      {loading ? (
        <div className="flex items-center justify-center h-full">
          <div className="animate-spin rounded-full h-5 w-5 border-2 border-gray-300 border-t-blue-500" />
        </div>
      ) : (
        <>
          <div className="min-w-0">
            <div className="text-[13px] font-medium text-claude-text truncate" title={attachment.file_name || 'file'}>
              {attachment.file_name || 'file'}
            </div>
            <div className="text-[11px] text-claude-textSecondary mt-0.5">
              {attachment.line_count ? `${attachment.line_count} lines` : (formatFileSize(attachment.file_size) || '文件')}
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div className="px-1.5 py-0.5 text-[10px] font-medium border border-gray-200 dark:border-[#5B5B56] bg-gray-50 dark:bg-claude-input rounded text-claude-textSecondary uppercase">
              {ext}
            </div>
            {isTauri && (
              <FolderOpen size={12} className="text-claude-textSecondary opacity-0 group-hover:opacity-100 transition-opacity" />
            )}
          </div>
        </>
      )}
    </div>
  );
};

const MessageAttachments: React.FC<MessageAttachmentsProps> = ({ attachments, onOpenDocument }) => {
  const [lightboxUrl, setLightboxUrl] = useState<string | null>(null);

  if (!attachments || attachments.length === 0) return null;

  const handleClick = async (att: Attachment) => {
    // Electron mode: open file in system file explorer
    if (isTauri) {
      const opened = await openFileInFolder(att.id);
      if (opened) return;
      // If open failed (file not found), fall through to other handlers
    }

    const url = getAttachmentUrl(att.id);
    const token = localStorage.getItem('auth_token');

    // 图片：打开灯箱
    if (att.file_type === 'image' || (att.mime_type?.startsWith('image/') ?? false)) {
      if (isTauri) {
        setLightboxUrl(`http://127.0.0.1:30080/api/uploads/${encodeURIComponent(att.id)}/raw`);
      } else {
        setLightboxUrl(`${url}${url.includes('?') ? '&' : '?'}token=${token}`);
      }
      return;
    }

    // 代码/文本文件：尝试在右侧面板打开
    const textExtensions = ['SH', 'MD', 'PY', 'JS', 'TXT', 'HTML', 'CSS', 'JSON', 'XML', 'YAML', 'TS', 'TSX', 'JSX', 'JAVA', 'CPP', 'C', 'H', 'CS', 'GO', 'RS', 'RB', 'PHP', 'SQL', 'VUE', 'SVELTE', 'LUA'];
    const ext = getFileExtension(att.file_name);

    if (onOpenDocument && (textExtensions.includes(ext) || (att.mime_type?.startsWith('text/') ?? false))) {
      try {
        // Fetch content
        const fetchUrl = isTauri
          ? `http://127.0.0.1:30080/api/uploads/${encodeURIComponent(att.id)}/raw`
          : `${url}${url.includes('?') ? '&' : '?'}token=${token}`;
        const res = await fetch(fetchUrl);
        if (res.ok) {
          const content = await res.text();
          onOpenDocument({
            id: att.id,
            title: att.file_name,
            filename: att.file_name,
            url: url,
            content: content,
            format: 'markdown',
          });
          return;
        }
      } catch (err) {
        console.error('Failed to fetch file content', err);
      }
    }

    // 默认：下载文件
    if (!isTauri) {
      window.open(url, '_blank');
    }
  };

  return (
    <>
      <div className="flex flex-wrap gap-2 mb-2">
        {attachments.filter(att => att && att.id).map((att) => (
          <AttachmentCard
            key={att.id}
            attachment={att}
            onClick={() => handleClick(att)}
          />
        ))}
      </div>

      {/* 灯箱 */}
      {lightboxUrl && (
        <div
          className="fixed inset-0 z-[100] bg-black/80 flex items-center justify-center backdrop-blur-sm"
          onClick={() => setLightboxUrl(null)}
        >
          <button
            className="absolute top-4 right-4 text-white/70 hover:text-white transition-colors bg-black/20 hover:bg-black/40 rounded-full p-2"
            onClick={() => setLightboxUrl(null)}
          >
            <X size={24} />
          </button>
          <img
            src={lightboxUrl}
            alt="preview"
            className="max-w-[95vw] max-h-[95vh] object-contain rounded-lg shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          />
        </div>
      )}
    </>
  );
};

export default React.memo(MessageAttachments);
