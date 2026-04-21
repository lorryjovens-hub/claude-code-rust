import React, { useState } from 'react';
import { ChevronDown, Check, FileText } from 'lucide-react';

export interface DocumentDraftInfo {
  draftId: string;
  title?: string;
  format?: string;
  preview?: string;
  previewAvailable?: boolean;
  done?: boolean;
}

const labelForFormat = (format?: string) => {
  const fmt = (format || 'markdown').toLowerCase();
  if (fmt === 'markdown') return 'Markdown document';
  if (fmt === 'docx') return 'Word document';
  return `${fmt.toUpperCase()} file`;
};

const summaryForDraft = (draft: DocumentDraftInfo) => {
  if (draft.title?.trim()) return draft.title.trim();
  const preview = (draft.preview || '').trim();
  if (!preview) return 'Creating document...';
  const line = preview.split('\n').find(Boolean) || preview;
  return line.length > 48 ? `${line.slice(0, 48)}...` : line;
};

const DocumentDraftItem: React.FC<{ draft: DocumentDraftInfo }> = ({ draft }) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative pl-8 pb-2">
      <div className="absolute left-0 top-0.5 z-10 bg-[#F8F8F6] text-claude-textSecondary">
        {draft.done ? <Check size={16} /> : <FileText size={16} />}
      </div>

      <div
        className="flex items-center justify-between mb-1 cursor-pointer group select-none py-0.5"
        onClick={() => setIsOpen(!isOpen)}
      >
        <div className="flex flex-col">
          <div className="text-[13px] font-medium text-claude-text group-hover:text-claude-text transition-colors">
            {summaryForDraft(draft)}
          </div>
          <div className="text-[11px] text-claude-textSecondary">
            {draft.done ? 'Document created' : 'Creating document'} · {labelForFormat(draft.format)}
          </div>
        </div>

        <ChevronDown
          size={14}
          className={`transform transition-transform duration-200 text-claude-textSecondary ${isOpen ? 'rotate-180' : ''}`}
        />
      </div>

      {isOpen && (
        <div className="bg-[#F9F9F8] border border-[#E5E5E5] rounded-xl overflow-hidden shadow-sm mt-1">
          <div className="px-3 py-2 border-b border-[#ECEBE7] bg-[#FCFCFB]">
            <div className="text-[12px] font-medium text-claude-text">{draft.title || 'Untitled document'}</div>
            <div className="text-[11px] text-claude-textSecondary">{labelForFormat(draft.format)}</div>
          </div>
          <div className="max-h-[280px] overflow-auto px-3 py-2">
            <pre className="whitespace-pre-wrap break-words text-[12px] leading-5 font-mono text-[#404040]">
              {(draft.previewAvailable ? draft.preview : '') || 'Preparing content...'}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
};

const DocumentCreationProcess: React.FC<{ drafts: DocumentDraftInfo[] }> = ({ drafts }) => {
  if (!drafts || drafts.length === 0) return null;

  return (
    <div className="mb-2 font-sans">
      <div className="flex items-center gap-2 text-claude-textSecondary text-[14px] mb-1">
        <span>{drafts.length > 1 ? 'Creating documents' : 'Creating document'}</span>
      </div>
      <div className="relative pb-2">
        <div className="absolute left-[7.5px] top-2 bottom-2 w-px bg-claude-border" />
        {drafts.map((draft) => (
          <DocumentDraftItem key={draft.draftId} draft={draft} />
        ))}
      </div>
    </div>
  );
};

export default DocumentCreationProcess;
