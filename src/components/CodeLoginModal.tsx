import React, { useState, useEffect } from 'react';
import { X, Copy, Check, ExternalLink } from 'lucide-react';

interface CodeLoginModalProps {
  isOpen: boolean;
  onClose: () => void;
  username: string;
  password: string;
  url: string;
}

export const CodeLoginModal: React.FC<CodeLoginModalProps> = ({
  isOpen,
  onClose,
  username,
  password,
  url,
}) => {
  const [copiedField, setCopiedField] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      setCopiedField(null);
    }
  }, [isOpen]);

  const copyToClipboard = (text: string, field: string) => {
    navigator.clipboard.writeText(text);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  const handleOpenCode = () => {
    window.open(url, '_blank');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="bg-white dark:bg-[#2f2f2f] rounded-2xl shadow-2xl w-[480px] p-6 animate-fade-in">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-[20px] font-semibold text-gray-900 dark:text-white">
            欢迎使用 Code API
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        <p className="text-[14px] text-gray-600 dark:text-gray-400 mb-6">
          您的 Code API 账号已创建。请保存以下登录信息：
        </p>

        <div className="space-y-4 mb-6">
          {/* Username */}
          <div>
            <label className="block text-[13px] font-medium text-gray-700 dark:text-gray-300 mb-2">
              用户名
            </label>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={username}
                readOnly
                className="flex-1 px-3 py-2 bg-gray-50 dark:bg-[#1a1a1a] border border-gray-200 dark:border-gray-700 rounded-lg text-[14px] text-gray-900 dark:text-white font-mono"
              />
              <button
                onClick={() => copyToClipboard(username, 'username')}
                className="p-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-[#1a1a1a] rounded-lg transition-colors"
                title="复制用户名"
              >
                {copiedField === 'username' ? (
                  <Check size={18} className="text-green-500" />
                ) : (
                  <Copy size={18} />
                )}
              </button>
            </div>
          </div>

          {/* Password */}
          <div>
            <label className="block text-[13px] font-medium text-gray-700 dark:text-gray-300 mb-2">
              密码
            </label>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={password}
                readOnly
                className="flex-1 px-3 py-2 bg-gray-50 dark:bg-[#1a1a1a] border border-gray-200 dark:border-gray-700 rounded-lg text-[14px] text-gray-900 dark:text-white font-mono"
              />
              <button
                onClick={() => copyToClipboard(password, 'password')}
                className="p-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-[#1a1a1a] rounded-lg transition-colors"
                title="复制密码"
              >
                {copiedField === 'password' ? (
                  <Check size={18} className="text-green-500" />
                ) : (
                  <Copy size={18} />
                )}
              </button>
            </div>
          </div>
        </div>

        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-4 mb-6">
          <p className="text-[13px] text-amber-800 dark:text-amber-200">
            ⚠️ 请妥善保存密码，此密码仅显示一次。如遗失，请联系管理员重置。
          </p>
        </div>

        <div className="flex gap-3">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2.5 text-[14px] font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-[#1a1a1a] hover:bg-gray-200 dark:hover:bg-[#252525] rounded-lg transition-colors"
          >
            稍后访问
          </button>
          <button
            onClick={handleOpenCode}
            className="flex-1 px-4 py-2.5 text-[14px] font-medium text-white bg-[#333333] hover:bg-[#1a1a1a] dark:bg-white dark:text-black dark:hover:bg-gray-200 rounded-lg transition-colors flex items-center justify-center gap-2"
          >
            <span>立即登录</span>
            <ExternalLink size={16} />
          </button>
        </div>
      </div>
    </div>
  );
};
