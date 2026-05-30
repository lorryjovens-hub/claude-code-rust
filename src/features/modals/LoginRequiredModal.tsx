import { useNavigate } from 'react-router-dom';

interface LoginRequiredModalProps {
  onClose: () => void;
}

export default function LoginRequiredModal({ onClose }: LoginRequiredModalProps) {
  const navigate = useNavigate();

  return (
    <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40 backdrop-blur-sm p-4">
      <div className="bg-claude-input border border-claude-border rounded-2xl shadow-xl w-[460px] overflow-hidden">
        <div className="px-6 pt-6 pb-4">
          <h3 className="text-[16px] font-semibold text-claude-text mb-3">需要登录 Claude Rust 账号</h3>
          <p className="text-[14px] text-claude-textSecondary leading-relaxed">
            你当前在 <span className="text-claude-text font-medium">Claude Rust</span> 模式下，需要登录 <span className="font-mono text-claude-text">Claude Rust</span> 账号才能使用。
            <br /><br />
            如果你还没有账号，请先去 <span className="font-mono text-claude-text">Claude Rust</span> 注册一个。
            <br /><br />
            或者你也可以在设置的 General 页面切换到 <span className="text-claude-text font-medium">自部署</span> 模式，用你自己的 API Key。
          </p>
        </div>
        <div className="px-5 pb-5 pt-2 flex flex-col gap-2">
          <button
            onClick={() => {
              onClose();
              navigate('/login');
            }}
            className="w-full px-5 py-2.5 text-[14px] font-medium bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity"
          >
            去登录
          </button>
          <button
            onClick={onClose}
            className="w-full px-5 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text transition-colors"
          >
            取消
          </button>
        </div>
      </div>
    </div>
  );
}
