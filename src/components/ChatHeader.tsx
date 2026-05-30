import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { FileText, ChevronDown, Trash, Pencil, Star, Folder, Download } from 'lucide-react';
import { updateConversation, deleteConversation, exportConversationAs, detectBridgePort, type ExportFormat } from '../api';
import { tauriAPI } from '../utils/tauriAPI';

interface ChatHeaderProps {
  title: string;
  showArtifacts: boolean;
  documentPanelDoc: any;
  onOpenArtifacts: () => void;
  hasArtifacts: boolean;
  onTitleRename?: (newTitle: string) => void;
}

function FormatItem({ label, icon, onClick }: { label: string; icon: string; format?: string; onClick: () => void }) {
  return (
    <button onClick={onClick} className="flex items-center gap-2 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors">
      <span className="text-[14px]">{icon}</span>
      <span className="text-[13px] text-claude-text">{label}</span>
    </button>
  );
}

export default function ChatHeader({
  title,
  showArtifacts,
  onOpenArtifacts,
  hasArtifacts,
  onTitleRename
}: ChatHeaderProps) {
  const { id } = useParams();
  const navigate = useNavigate();
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState('');
  const [showMenu, setShowMenu] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node) &&
        buttonRef.current && !buttonRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };
    if (showMenu) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showMenu]);

  const startEditing = () => {
    setEditTitle(title || 'New Chat');
    setIsEditing(true);
    setShowMenu(false);
  };

  const handleDelete = async () => {
    if (!id) return;
    try {
      await deleteConversation(id);
      navigate('/');
      window.dispatchEvent(new CustomEvent('conversationTitleUpdated'));
    } catch (err) {
      console.error('Failed to delete chat:', err);
    }
    setShowMenu(false);
  };

  const handleRenameSubmit = async () => {
    if (!id || !editTitle.trim()) { setIsEditing(false); return; }
    try {
      await updateConversation(id, { title: editTitle });
      onTitleRename?.(editTitle);
      window.dispatchEvent(new CustomEvent('conversationTitleUpdated'));
    } catch (err) {
      console.error('Failed to rename chat:', err);
    } finally {
      setIsEditing(false);
    }
  };

  return (
    <div className="relative flex items-center justify-between px-3 py-2 bg-claude-bg flex-shrink-0 h-[44px] border-b border-claude-border z-40">
      {isEditing ? (
        <input
          type="text" value={editTitle}
          onChange={(e) => setEditTitle(e.target.value)}
          onBlur={handleRenameSubmit}
          onKeyDown={(e) => { if (e.key === 'Enter') handleRenameSubmit(); else if (e.key === 'Escape') setIsEditing(false); }}
          autoFocus
          className="max-w-[60%] px-2 py-1 text-[14px] font-medium text-claude-text bg-claude-input border border-blue-500 rounded-md outline-none shadow-sm"
          style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
        />
      ) : (
        <div className="relative flex items-center gap-1" style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}>
          <button onClick={startEditing} className="flex items-center px-2 py-1.5 hover:bg-claude-btn-hover rounded-md transition-colors text-[14px] font-medium text-claude-text max-w-[200px] truncate group">
            {title || 'New Chat'}
          </button>
          <button ref={buttonRef} onClick={() => setShowMenu(!showMenu)} className={`p-1 hover:bg-claude-btn-hover rounded-md transition-colors text-claude-textSecondary hover:text-claude-text ${showMenu ? 'bg-claude-btn-hover text-claude-text' : ''}`}>
            <ChevronDown size={14} />
          </button>
          {showMenu && (
            <div ref={menuRef} className="absolute top-full left-0 mt-1 z-50 bg-claude-input border border-claude-border rounded-xl shadow-[0_4px_12px_rgba(0,0,0,0.08)] py-1.5 flex flex-col w-[200px]">
              <button className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group">
                <Star size={16} className="text-claude-textSecondary group-hover:text-claude-text" />
                <span className="text-[13px] text-claude-text">Star</span>
              </button>
              <button onClick={(e) => { e.stopPropagation(); startEditing(); }} className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group">
                <Pencil size={16} className="text-claude-textSecondary group-hover:text-claude-text" />
                <span className="text-[13px] text-claude-text">Rename</span>
              </button>
              <div className="h-[1px] bg-claude-border my-1 mx-3" />
              <button onClick={(e) => { e.stopPropagation(); handleDelete(); }} className="flex items-center gap-3 px-3 py-2 hover:bg-claude-hover text-left w-full transition-colors group">
                <Trash size={16} className="text-[#B9382C]" />
                <span className="text-[13px] text-[#B9382C]">Delete</span>
              </button>
            </div>
          )}
        </div>
      )}
      <div className="flex items-center gap-1">
        {hasArtifacts && (
          <button onClick={onOpenArtifacts} className={`w-8 h-8 flex items-center justify-center text-claude-textSecondary hover:bg-claude-btn-hover rounded-md transition-colors ${showArtifacts ? 'bg-claude-btn-hover text-claude-text' : ''}`} title="View Artifacts">
            <FileText size={18} strokeWidth={1.5} />
          </button>
        )}
        <button className="px-2 h-8 flex items-center justify-center text-claude-textSecondary hover:text-claude-text transition-colors" title="Open Workspace Folder" onClick={async () => {
          if (!id) return;
          try {
            const port = await detectBridgePort();
            const res = await fetch(`http://127.0.0.1:${port}/api/conversations/${id}`);
            if (!res.ok) return;
            const data = await res.json();
            if (data.workspace_path && tauriAPI.isTauri) tauriAPI.openFolder(data.workspace_path);
          } catch (e) { console.error('Open folder failed:', e); }
        }}>
          <Folder size={17} strokeWidth={1.5} />
        </button>
        <button onClick={async () => {
          if (!id || isExporting) return;
          setIsExporting(true);
          try { await exportConversationAs(id, 'markdown'); } catch (err) {
            window.alert(err instanceof Error ? err.message : '导出失败');
          } finally { setIsExporting(false); }
        }} disabled={isExporting} className="px-3 py-1.5 text-[13px] font-medium text-claude-textSecondary hover:bg-claude-btn-hover rounded-md transition-colors border border-transparent hover:border-claude-border disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5 group relative">
          <Download size={14} />
          <span>{isExporting ? '导出中…' : 'Export'}</span>
          <ChevronDown size={10} className="opacity-50" />
          <div className="absolute top-full right-0 mt-1 z-50 bg-claude-input border border-claude-border rounded-lg shadow-[0_4px_12px_rgba(0,0,0,0.15)] py-1 flex flex-col w-[180px] opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto transition-opacity">
            <FormatItem label="Markdown (.md)" format="markdown" icon="📝" onClick={async () => {
              if (!id || isExporting) return;
              setIsExporting(true);
              try { await exportConversationAs(id, 'markdown'); } catch (err) {
                window.alert(err instanceof Error ? err.message : '导出失败');
              } finally { setIsExporting(false); }
            }} />
            <FormatItem label="JSON (.json)" format="json" icon="📋" onClick={async () => {
              if (!id || isExporting) return;
              setIsExporting(true);
              try { await exportConversationAs(id, 'json'); } catch (err) {
                window.alert(err instanceof Error ? err.message : '导出失败');
              } finally { setIsExporting(false); }
            }} />
            <FormatItem label="HTML (.html)" format="html" icon="🌐" onClick={async () => {
              if (!id || isExporting) return;
              setIsExporting(true);
              try { await exportConversationAs(id, 'html'); } catch (err) {
                window.alert(err instanceof Error ? err.message : '导出失败');
              } finally { setIsExporting(false); }
            }} />
          </div>
        </button>
      </div>
      <div className="absolute top-full left-0 right-0 h-6 bg-gradient-to-b from-claude-bg to-transparent pointer-events-none z-30" />
    </div>
  );
}
