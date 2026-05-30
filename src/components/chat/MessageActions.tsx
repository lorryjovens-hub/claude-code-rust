import React, { useState, useRef, useCallback } from 'react';
import { RotateCcw, Pencil, Copy, Check, MoreHorizontal, Share2, Trash2 } from 'lucide-react';
import { copyToClipboard } from '../../utils/clipboard';

export interface MessageActionsProps {
  content: string;
  idx: number;
  role: 'user' | 'assistant';
  isCopied: boolean;
  onResend?: (content: string, idx: number) => void;
  onEdit?: (content: string, idx: number) => void;
  onBranch?: (idx: number) => void;
  onCopy: (content: string, idx: number) => void;
  onDelete?: (idx: number) => void;
  onShare?: (idx: number) => void;
  className?: string;
}

const MessageActions: React.FC<MessageActionsProps> = ({
  content,
  idx,
  role,
  isCopied,
  onResend,
  onEdit,
  onBranch,
  onCopy,
  onDelete,
  onShare,
  className,
}) => {
  const [copiedLocal, setCopiedLocal] = useState(false);
  const [showMore, setShowMore] = useState(false);
  const moreMenuRef = useRef<HTMLDivElement>(null);

  const handleCopy = useCallback(async () => {
    try {
      const success = await copyToClipboard(content);
      if (success) {
        setCopiedLocal(true);
        onCopy(content, idx);
        setTimeout(() => setCopiedLocal(false), 2000);
      }
    } catch {
      onCopy(content, idx);
    }
  }, [content, idx, onCopy]);

  const handleResend = useCallback(() => {
    onResend?.(content, idx);
  }, [content, idx, onResend]);

  const handleEdit = useCallback(() => {
    onEdit?.(content, idx);
  }, [content, idx, onEdit]);

  const handleBranch = useCallback(() => {
    onBranch?.(idx);
  }, [idx, onBranch]);

  const handleDelete = useCallback(() => {
    onDelete?.(idx);
  }, [idx, onDelete]);

  const handleShare = useCallback(() => {
    onShare?.(idx);
  }, [idx, onShare]);

  return (
    <div className={`flex items-center gap-1 ${className || ''}`}>
      {/* Primary actions - always visible on hover */}
      <button
        onClick={handleCopy}
        className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
        title="复制"
      >
        {copiedLocal || isCopied ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
      </button>

      {role === 'user' && onEdit && (
        <button
          onClick={handleEdit}
          className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
          title="编辑"
        >
          <Pencil size={14} />
        </button>
      )}

      {role === 'user' && onResend && (
        <button
          onClick={handleResend}
          className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
          title="重新发送"
        >
          <RotateCcw size={14} />
        </button>
      )}

      {role === 'user' && onBranch && (
        <button
          onClick={handleBranch}
          className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
          title="分支对话"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="6" y1="3" x2="6" y2="15" />
            <circle cx="18" cy="6" r="3" />
            <circle cx="6" cy="18" r="3" />
            <path d="M18 9a9 9 0 0 1-9 9" />
          </svg>
        </button>
      )}

      {/* More actions menu */}
      {(onDelete || onShare) && (
        <div className="relative" ref={moreMenuRef}>
          <button
            onClick={() => setShowMore(!showMore)}
            className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
            title="更多操作"
          >
            <MoreHorizontal size={14} />
          </button>

          {showMore && (
            <>
              <div
                className="fixed inset-0 z-40"
                onClick={() => setShowMore(false)}
              />
              <div className="absolute right-0 top-full mt-1 w-40 bg-claude-input border border-claude-border rounded-xl shadow-lg py-1 z-50">
                {onShare && (
                  <button
                    onClick={() => { handleShare(); setShowMore(false); }}
                    className="w-full flex items-center gap-2 px-3 py-2 text-[13px] text-claude-text hover:bg-claude-hover transition-colors"
                  >
                    <Share2 size={14} />
                    <span>分享</span>
                  </button>
                )}
                {onDelete && (
                  <button
                    onClick={() => { handleDelete(); setShowMore(false); }}
                    className="w-full flex items-center gap-2 px-3 py-2 text-[13px] text-red-500 hover:bg-claude-hover transition-colors"
                  >
                    <Trash2 size={14} />
                    <span>删除</span>
                  </button>
                )}
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
};

export default React.memo(MessageActions);
