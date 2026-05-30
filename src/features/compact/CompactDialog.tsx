import { CompactStatus } from '../../stores/useChatStore';
import { compactConversation, getContextSize } from '../../api';

interface CompactDialogProps {
  activeId: string;
  compactStatus: CompactStatus;
  compactInstruction: string;
  setCompactInstruction: (v: string) => void;
  setCompactStatus: (status: CompactStatus) => void;
  setShowCompactDialog: (show: boolean) => void;
  setContextInfo: (info: { tokens: number; limit: number }) => void;
  loadConversation: (id: string) => Promise<void>;
}

export default function CompactDialog({
  activeId,
  compactStatus,
  compactInstruction,
  setCompactInstruction,
  setCompactStatus,
  setShowCompactDialog,
  setContextInfo,
  loadConversation,
}: CompactDialogProps) {
  const handleCompact = async () => {
    setShowCompactDialog(false);
    if (!activeId || compactStatus.state === 'compacting') return;
    setCompactStatus({ state: 'compacting' });
    try {
      const instruction = compactInstruction.trim() || undefined;
      const result = await compactConversation(activeId, instruction);
      await loadConversation(activeId);
      const newContextInfo = await getContextSize(activeId);
      setContextInfo(newContextInfo);
      setCompactStatus({
        state: 'done',
        message: `Compacted ${result.messagesCompacted} messages, saved ~${result.tokensSaved} tokens`,
      });
      setTimeout(() => setCompactStatus({ state: 'idle' }), 4000);
    } catch (err) {
      console.error('Compact failed:', err);
      setCompactStatus({ state: 'error', message: 'Compaction failed' });
      setTimeout(() => setCompactStatus({ state: 'idle' }), 3000);
    }
  };

  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40"
      onClick={() => setShowCompactDialog(false)}
    >
      <div
        className="bg-claude-bg border border-claude-border rounded-2xl shadow-xl w-[440px] overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="px-5 pt-5 pb-3">
          <h3 className="text-[15px] font-semibold text-claude-text mb-1">
            Compact conversation
          </h3>
          <p className="text-[13px] text-claude-textSecondary leading-snug">
            Summarize the conversation history to free up context space. The
            engine will preserve key decisions and context.
          </p>
        </div>
        <div className="px-5 pb-3">
          <textarea
            className="w-full bg-claude-input border border-claude-border rounded-lg px-3 py-2 text-[13px] text-claude-text placeholder:text-claude-textSecondary/50 outline-none focus:border-claude-textSecondary/40 transition-colors resize-none"
            rows={3}
            placeholder="Optional: add instructions for the summary (e.g. 'preserve all API endpoint details')"
            value={compactInstruction}
            onChange={(e) => setCompactInstruction(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                document.getElementById('compact-confirm-btn')?.click();
              }
            }}
            autoFocus
          />
        </div>
        <div className="flex items-center justify-end gap-2 px-5 pb-4">
          <button
            onClick={() => setShowCompactDialog(false)}
            className="px-3.5 py-1.5 text-[13px] text-claude-textSecondary hover:text-claude-text rounded-lg hover:bg-claude-hover transition-colors"
          >
            Cancel
          </button>
          <button
            id="compact-confirm-btn"
            onClick={handleCompact}
            className="px-3.5 py-1.5 text-[13px] text-white bg-[#C6613F] hover:bg-[#D97757] rounded-lg transition-colors font-medium"
          >
            Compact
          </button>
        </div>
      </div>
    </div>
  );
}
