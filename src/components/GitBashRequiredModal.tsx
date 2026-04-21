import React, { useState } from 'react';
import { getSystemStatus } from '../api';

interface Props {
  onResolved: () => void;
}

const GIT_DOWNLOAD_URL = 'https://git-scm.com/downloads/win';

const GitBashRequiredModal: React.FC<Props> = ({ onResolved }) => {
  const [checking, setChecking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openDownload = () => {
    import('../utils/tauriAPI').then(m => {
      m.tauriAPI.openExternal(GIT_DOWNLOAD_URL);
    }).catch(() => {
      window.open(GIT_DOWNLOAD_URL, '_blank');
    });
  };

  const recheck = async () => {
    setChecking(true);
    setError(null);
    try {
      const status = await getSystemStatus();
      if (status.gitBash.found) {
        onResolved();
      } else {
        setError('仍未检测到 git-bash。请确认安装完成，或重启应用后再试。');
      }
    } catch (err: any) {
      setError(err?.message || '检测失败');
    } finally {
      setChecking(false);
    }
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="max-w-md w-full mx-4 rounded-2xl bg-claude-bg border border-claude-border shadow-2xl p-7">
        <div className="flex items-start gap-3 mb-4">
          <div className="flex-shrink-0 w-10 h-10 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-amber-600 dark:text-amber-400">
              <path d="M12 9v4" />
              <path d="M12 17h.01" />
              <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z" />
            </svg>
          </div>
          <div className="flex-1">
            <h2 className="text-[17px] font-semibold text-claude-text">需要安装 Git for Windows</h2>
            <p className="text-[13px] text-claude-textSecondary mt-1">Claude Desktop 在 Windows 上需要 git-bash 才能运行内置工具。</p>
          </div>
        </div>

        <div className="rounded-lg bg-claude-hover/50 p-3.5 mb-4">
          <p className="text-[12.5px] text-claude-text leading-relaxed">
            点击下方按钮下载并安装 <span className="font-medium">Git for Windows</span>。安装时使用默认选项即可，安装完成后回到此处点击"重新检测"。
          </p>
        </div>

        {error && (
          <div className="rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-3 mb-4">
            <p className="text-[12.5px] text-red-700 dark:text-red-300">{error}</p>
          </div>
        )}

        <div className="flex flex-col gap-2">
          <button
            onClick={openDownload}
            className="w-full px-4 py-2.5 rounded-lg bg-claude-text text-claude-bg text-[14px] font-medium hover:opacity-90 transition-opacity"
          >
            下载 Git for Windows
          </button>
          <button
            onClick={recheck}
            disabled={checking}
            className="w-full px-4 py-2.5 rounded-lg border border-claude-border text-claude-text text-[14px] font-medium hover:bg-claude-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {checking ? '检测中...' : '我已安装，重新检测'}
          </button>
        </div>

        <p className="text-[11px] text-claude-textSecondary text-center mt-4">
          已安装但未生效？请尝试关闭并重新打开 Claude Desktop。
        </p>
      </div>
    </div>
  );
};

export default GitBashRequiredModal;
