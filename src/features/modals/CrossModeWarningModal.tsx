interface CrossModeWarningData {
  convId: string;
  originalModel: string;
  otherMode: 'clawparrot' | 'selfhosted';
  fallbackModel: string;
}

interface CrossModeWarningModalProps {
  warning: CrossModeWarningData;
  onKeepCrossMode: () => void;
  onSwitchModel: () => void;
  onCancel: () => void;
}

export default function CrossModeWarningModal({
  warning, onKeepCrossMode, onSwitchModel, onCancel,
}: CrossModeWarningModalProps) {
  return (
    <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40 backdrop-blur-sm p-4">
      <div className="bg-claude-input border border-claude-border rounded-2xl shadow-xl w-[460px] overflow-hidden">
        <div className="px-6 pt-6 pb-4">
          <h3 className="text-[16px] font-semibold text-claude-text mb-3">对话模型与当前模式不匹配</h3>
          <p className="text-[14px] text-claude-textSecondary leading-relaxed">
            此对话使用的是 <span className="font-mono text-claude-text">{warning.originalModel}</span>，
            它属于 <span className="text-claude-text font-medium">{warning.otherMode === 'selfhosted' ? '自部署' : 'Clawparrot'}</span> 模式下的模型。
            <br /><br />
            你当前在 <span className="text-claude-text font-medium">{warning.otherMode === 'selfhosted' ? 'Clawparrot' : '自部署'}</span> 模式。
            是要继续在原模式下使用这个模型，还是切换到当前模式下的模型？
          </p>
        </div>
        <div className="px-5 pb-5 pt-2 flex flex-col gap-2">
          <button
            onClick={onKeepCrossMode}
            className="w-full px-5 py-2.5 text-[14px] font-medium text-claude-text border border-claude-border hover:bg-claude-hover rounded-lg transition-colors text-left"
          >
            继续使用 <span className="font-mono">{warning.originalModel}</span>
            <div className="text-[11px] text-claude-textSecondary mt-0.5 font-normal">
              这次和以后都通过 {warning.otherMode === 'selfhosted' ? '自部署' : 'Clawparrot'} 模式发送
            </div>
          </button>
          <button
            onClick={onSwitchModel}
            className="w-full px-5 py-2.5 text-[14px] font-medium bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity text-left"
          >
            切换到 <span className="font-mono">{warning.fallbackModel}</span>
            <div className="text-[11px] opacity-70 mt-0.5 font-normal">
              切完后这个对话会用当前 {warning.otherMode === 'selfhosted' ? 'Clawparrot' : '自部署'} 模式
            </div>
          </button>
          <button
            onClick={onCancel}
            className="w-full px-5 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text transition-colors mt-1"
          >
            取消，先不发送
          </button>
        </div>
      </div>
    </div>
  );
}
