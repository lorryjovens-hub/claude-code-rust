import React, { useState, useRef, useEffect } from 'react';
import { Copy, Check, Download, FileText, File, FileCode } from 'lucide-react';
import { downloadMessageAsText, ExportFormat } from '../../utils/documentExport';
import { extractTextContent } from '../../utils/messageHelpers';

interface MessageExportActionsProps {
  content: unknown;
  role: string;
  timestamp?: string;
  onCopy?: () => void;
}

type ExportAction = { format: ExportFormat; label: string; icon: React.ReactNode };

const exportActions: ExportAction[] = [
  { format: 'doc', label: '导出 Word', icon: <File size={14} /> },
  { format: 'md', label: '导出 MD', icon: <FileCode size={14} /> },
  { format: 'txt', label: '导出 TXT', icon: <FileText size={14} /> },
];

const MessageExportActions: React.FC<MessageExportActionsProps> = ({ content, role, timestamp, onCopy }) => {
  const [exportMenuOpen, setExportMenuOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setExportMenuOpen(false);
      }
    };
    if (exportMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [exportMenuOpen]);

  const handleCopy = async () => {
    const text = extractTextContent(content);
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      const ta = document.createElement('textarea');
      ta.value = text;
      ta.style.position = 'fixed';
      ta.style.left = '-9999px';
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
    onCopy?.();
  };

  const handleExport = (format: ExportFormat) => {
    downloadMessageAsText(content, format, role, timestamp);
    setExportMenuOpen(false);
  };

  const text = extractTextContent(content);
  if (!text || !text.trim()) return null;

  return (
    <div className="flex items-center gap-0.5" ref={menuRef}>
      <button
        onClick={handleCopy}
        className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors"
        title="复制内容"
      >
        {copied ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
      </button>

      <div className="relative">
        <button
          onClick={() => setExportMenuOpen(!exportMenuOpen)}
          className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors"
          title="导出文档"
        >
          <Download size={14} />
        </button>

        {exportMenuOpen && (
          <div className="absolute bottom-full right-0 mb-1.5 bg-white dark:bg-[#2a2a2a] border border-black/10 dark:border-white/10 rounded-lg shadow-xl py-1.5 min-w-[140px] z-50 animate-scale-in origin-bottom-right">
            {exportActions.map((action) => (
              <button
                key={action.format}
                onClick={() => handleExport(action.format)}
                className="w-full flex items-center gap-2.5 px-3 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors text-left"
              >
                {action.icon}
                <span>{action.label}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default React.memo(MessageExportActions);