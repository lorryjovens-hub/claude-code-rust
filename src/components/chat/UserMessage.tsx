import React, { useState, useRef } from 'react';
import { RotateCcw, Pencil, Copy, Check } from 'lucide-react';
import { formatMessageTime, extractTextContent } from '../../utils/messageHelpers';
import MessageAttachments from '../MessageAttachments';
import MessageExportActions from './MessageExportActions';
import { DocumentInfo } from '../DocumentCard';

export interface UserMessageProps {
  msg: any;
  idx: number;
  messagesLength: number;
  isEditing: boolean;
  editingContent: string;
  isCopied: boolean;
  isExpanded: boolean;
  onSetEditingContent: (v: string) => void;
  onEditCancel: () => void;
  onEditSave: () => void;
  onResend: (content: string, idx: number) => void;
  onEdit: (content: string, idx: number) => void;
  onCopy: (content: string, idx: number) => void;
  onBranch: (idx: number) => void;
  onToggleExpand: (idx: number) => void;
  onOpenDocument?: (doc: DocumentInfo) => void;
  messageContentRefs: React.MutableRefObject<Map<number, HTMLDivElement>>;
  t: (key: string, params?: Record<string, string | number>) => string;
}

const UserMessage: React.FC<UserMessageProps> = ({
  msg,
  idx,
  messagesLength,
  isEditing,
  editingContent,
  isCopied,
  isExpanded,
  onSetEditingContent,
  onEditCancel,
  onEditSave,
  onResend,
  onEdit,
  onCopy,
  onBranch,
  onToggleExpand,
  onOpenDocument,
  messageContentRefs,
  t,
}) => {
  const displayText = extractTextContent(msg.content);

  if (isEditing) {
    return (
      <div className="w-full bg-[#F0EEE7] dark:bg-claude-btnHover rounded-xl p-3 border border-black/5 dark:border-white/10">
        <div className="bg-white dark:bg-black/20 rounded-lg border border-black/10 dark:border-white/10 focus-within:ring-2 focus-within:ring-blue-500/20 focus-within:border-blue-500 transition-all p-3">
          <textarea
            className="w-full bg-transparent text-claude-text outline-none resize-none text-[16px] leading-relaxed font-sans font-[350] block"
            value={editingContent}
            onChange={(e) => {
              onSetEditingContent(e.target.value);
              e.target.style.height = 'auto';
              e.target.style.height = e.target.scrollHeight + 'px';
            }}
            onKeyDown={(e) => { if (e.key === 'Escape') onEditCancel(); }}
            ref={(el) => {
              if (el) {
                el.style.height = 'auto';
                el.style.height = el.scrollHeight + 'px';
                el.focus();
              }
            }}
            style={{ minHeight: '60px' }}
          />
        </div>
        <div className="flex items-start justify-between mt-3 px-1 gap-4">
          <div className="flex items-start gap-2 text-claude-textSecondary text-[13px] leading-tight pt-1">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="mt-0.5 shrink-0">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="16" x2="12" y2="12" />
              <line x1="12" y1="8" x2="12.01" y2="8" />
            </svg>
            <span>{t('chat.editingMessageWarning')}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <button
              onClick={onEditCancel}
              className="px-3 py-1.5 text-[13px] font-medium text-claude-text bg-white dark:bg-claude-bg border border-black/10 dark:border-white/10 hover:bg-gray-50 dark:hover:bg-claude-hover rounded-lg transition-colors"
            >
              {t('chat.cancelEdit')}
            </button>
            <button
              onClick={onEditSave}
              disabled={!editingContent.trim() || editingContent === msg.content}
              className="px-3 py-1.5 text-[13px] font-medium text-white bg-claude-text hover:bg-claude-textSecondary rounded-lg transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            >
              {t('chat.saveEdit')}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-end">
      {msg.attachments && msg.attachments.length > 0 && (
        <div className="max-w-[85%] w-fit mb-1">
          <MessageAttachments attachments={msg.attachments} onOpenDocument={onOpenDocument} />
        </div>
      )}
      {(!msg.attachments || msg.attachments.length === 0) && msg.has_attachments === 1 && (
        <div className="max-w-[85%] w-fit mb-1">
          <div className="bg-[#F0EEE7] dark:bg-claude-btnHover text-claude-textSecondary px-3.5 py-2 text-[14px] rounded-2xl font-sans italic">
            Files attached
          </div>
        </div>
      )}
      {displayText && displayText.trim() !== '' && (
        <div className="max-w-[85%] w-fit relative">
          <div
            className="bg-[#F0EEE7] dark:bg-claude-btnHover text-claude-text px-3.5 py-2.5 text-[16px] leading-relaxed font-sans font-[350] whitespace-pre-wrap break-words relative overflow-hidden"
            style={{
              maxHeight: isExpanded ? 'none' : '300px',
              borderRadius: (() => {
                const el = messageContentRefs.current.get(idx);
                const isOverflow = el && el.scrollHeight > 300;
                return isOverflow ? '16px 16px 0 0' : '16px';
              })(),
            }}
            ref={(el) => { if (el) messageContentRefs.current.set(idx, el); }}
          >
            {(() => {
              try {
                const text = extractTextContent(msg.content);
                if (!text) return '';
                const skillMatch = text.match(/^\/([a-zA-Z0-9_-]+)(\s|$)/);
                if (skillMatch) {
                  const slug = skillMatch[1];
                  const rest = text.slice(skillMatch[0].length);
                  return (
                    <>
                      <span className="text-[#4B9EFA] font-medium">/{slug}</span>
                      {rest ? ' ' + rest : ''}
                    </>
                  );
                }
                return text;
              } catch {
                return extractTextContent(msg.content) || '';
              }
            })()}
            {!isExpanded && (() => {
              const el = messageContentRefs.current.get(idx);
              return el && el.scrollHeight > 300;
            })() && (
              <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-[#F0EEE7] dark:from-claude-btnHover to-transparent pointer-events-none" />
            )}
          </div>
          {(() => {
            const el = messageContentRefs.current.get(idx);
            const isOverflow = el && el.scrollHeight > 300;
            if (!isOverflow) return null;
            return (
              <div className="bg-[#F0EEE7] dark:bg-claude-btnHover rounded-b-2xl px-3.5 pb-3 pt-1 -mt-[1px] relative" style={{ borderTopLeftRadius: 0, borderTopRightRadius: 0 }}>
                <button onClick={() => onToggleExpand(idx)} className="text-[13px] text-claude-textSecondary hover:text-claude-text transition-colors">
                  {isExpanded ? 'Show less' : 'Show more'}
                </button>
              </div>
            );
          })()}
        </div>
      )}
      <div className="flex items-center gap-1.5 mt-1.5 pr-1">
        {msg.created_at && (
          <span className="text-[12px] text-claude-textSecondary mr-1">{formatMessageTime(msg.created_at)}</span>
        )}
        <div className="flex items-center gap-0.5 overflow-hidden transition-all duration-200 ease-in-out max-w-0 opacity-0 group-hover:max-w-[280px] group-hover:opacity-100">
          <button onClick={() => onResend(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="重新发送"><RotateCcw size={14} /></button>
          <button onClick={() => onEdit(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="编辑"><Pencil size={14} /></button>
          <button onClick={() => onBranch(idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="分支"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg></button>
          <button onClick={() => onCopy(msg.content, idx)} className="p-1 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded transition-colors" title="复制">
            {isCopied ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
          </button>
          <div className="ml-0.5 pl-0.5 border-l border-black/10 dark:border-white/10">
            <MessageExportActions
              content={msg.content}
              role="user"
              timestamp={msg.created_at ? formatMessageTime(msg.created_at) : undefined}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default React.memo(UserMessage);
