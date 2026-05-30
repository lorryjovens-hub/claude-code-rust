import React, { useState, useEffect, useCallback } from 'react';
import { h5API, H5TokenInfo } from '../utils/tauriAPI';
import QRCode from './QRCode';
import { useI18n } from '../hooks/useI18n';
import { detectBridgePort } from '../api';

interface H5ConnectionViewProps {
  conversationId: string;
}

const H5ConnectionView: React.FC<H5ConnectionViewProps> = ({ conversationId }) => {
  const { t } = useI18n();
  const [isOpen, setIsOpen] = useState(false);
  const [tokens, setTokens] = useState<H5TokenInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [generatedToken, setGeneratedToken] = useState<H5TokenInfo | null>(null);
  const [ttlMinutes, setTtlMinutes] = useState(60);
  const [now, setNow] = useState(Date.now());
  const [bridgePort, setBridgePort] = useState(30085);

  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    detectBridgePort().then(port => setBridgePort(port));
  }, []);

  const loadTokens = useCallback(async () => {
    try {
      const list = await h5API.listTokens(conversationId);
      setTokens(list);
    } catch (err) {
      console.error('Failed to load H5 tokens:', err);
    }
  }, [conversationId]);

  useEffect(() => {
    if (isOpen) {
      loadTokens();
    }
  }, [isOpen, loadTokens]);

  const handleGenerate = async () => {
    setLoading(true);
    setError(null);
    try {
      const token = await h5API.generateToken(conversationId, ttlMinutes);
      setGeneratedToken(token);
      await loadTokens();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleRevoke = async (tokenId: string) => {
    try {
      await h5API.revokeToken(tokenId);
      if (generatedToken?.id === tokenId) {
        setGeneratedToken(null);
      }
      await loadTokens();
    } catch (err) {
      console.error('Failed to revoke token:', err);
    }
  };

  const getAccessUrl = (token: string) =>
    `http://127.0.0.1:${bridgePort}/api/h5/access/${token}`;

  const formatTimeLeft = (expiresAt: string): string => {
    const expires = new Date(expiresAt).getTime();
    const diff = expires - now;
    if (diff <= 0) return t('h5.expired') || '已过期';
    const hours = Math.floor(diff / 3600000);
    const minutes = Math.floor((diff % 3600000) / 60000);
    const seconds = Math.floor((diff % 60000) / 1000);
    if (hours > 0) {
      return `${hours}h ${minutes}m ${seconds}s`;
    }
    if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    }
    return `${seconds}s`;
  };

  const isTokenExpired = (expiresAt: string): boolean => {
    return new Date(expiresAt).getTime() <= now;
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 py-2 rounded-lg text-[13px] font-medium transition-colors bg-blue-500/10 text-blue-400 border border-blue-500/20 hover:bg-blue-500/20"
        title={t('h5.remoteAccess') || '远程访问'}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M18 10l-4-4M8 14l-4 4M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          <path d="M12 7v3l2 2" />
        </svg>
        {t('h5.remoteAccess') || '远程访问'}
      </button>

      {isOpen && (
        <div className="fixed inset-0 z-[90] flex items-start justify-center pt-20 bg-black/40" onClick={() => setIsOpen(false)}>
          <div
            className="bg-claude-bg border border-claude-border rounded-2xl shadow-xl w-[520px] max-h-[80vh] overflow-y-auto animate-fade-in"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between p-5 border-b border-claude-border">
              <h3 className="text-[18px] font-semibold text-claude-text">
                {t('h5.remoteAccess') || '远程访问'}
              </h3>
              <button
                onClick={() => setIsOpen(false)}
                className="p-1.5 rounded-lg text-claude-secondary hover:text-claude-text hover:bg-claude-hover transition-colors"
              >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>

            <div className="p-5 space-y-5">
              <div>
                <p className="text-[13px] text-claude-secondary mb-4">
                  {t('h5.description') || '生成一个临时访问链接，允许外部设备查看和参与此对话。'}
                </p>

                <div className="flex items-center gap-3">
                  <div className="flex-1">
                    <label className="block text-[12px] text-claude-secondary mb-1.5">
                      {t('h5.expiryTime') || '过期时间'}
                    </label>
                    <select
                      value={ttlMinutes}
                      onChange={(e) => setTtlMinutes(Number(e.target.value))}
                      className="w-full px-3 py-2 bg-claude-input border border-claude-border rounded-lg text-claude-text text-[13px] focus:outline-none focus:border-blue-500"
                    >
                      <option value={15}>15 分钟</option>
                      <option value={30}>30 分钟</option>
                      <option value={60}>1 小时</option>
                      <option value={120}>2 小时</option>
                      <option value={360}>6 小时</option>
                      <option value={1440}>24 小时</option>
                    </select>
                  </div>
                  <button
                    onClick={handleGenerate}
                    disabled={loading}
                    className="mt-5 px-5 py-2 rounded-lg text-[13px] font-medium bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {loading ? (t('h5.generating') || '生成中...') : (t('h5.generate') || '生成链接')}
                  </button>
                </div>
              </div>

              {error && (
                <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-[13px]">
                  {error}
                </div>
              )}

              {generatedToken && !isTokenExpired(generatedToken.expires_at) && !generatedToken.is_revoked && (
                <div className="p-4 rounded-xl bg-claude-input border border-blue-500/20 space-y-4">
                  <h4 className="text-[14px] font-medium text-claude-text">
                    {t('h5.accessLink') || '访问链接'}
                  </h4>
                  <QRCode text={getAccessUrl(generatedToken.token)} size={180} />
                  <div className="flex items-center gap-2 pt-2">
                    <span className="text-[12px] text-claude-secondary">
                      {t('h5.expiresIn') || '过期倒计时'}:
                    </span>
                    <span className={`text-[13px] font-mono font-medium ${
                      new Date(generatedToken.expires_at).getTime() - now < 300000
                        ? 'text-red-400'
                        : 'text-green-400'
                    }`}>
                      {formatTimeLeft(generatedToken.expires_at)}
                    </span>
                  </div>
                </div>
              )}

              {tokens.length > 0 && (
                <div>
                  <h4 className="text-[14px] font-medium text-claude-text mb-3">
                    {t('h5.tokenHistory') || '历史令牌'}
                  </h4>
                  <div className="space-y-2">
                    {tokens.map((token) => (
                      <div
                        key={token.id}
                        className={`flex items-center justify-between p-3 rounded-lg border ${
                          token.is_revoked || isTokenExpired(token.expires_at)
                            ? 'border-claude-border bg-claude-hover/30 opacity-50'
                            : 'border-claude-border bg-claude-input'
                        }`}
                      >
                        <div className="flex-1 min-w-0">
                          <div className="text-[12px] text-claude-secondary truncate font-mono">
                            {token.token.substring(0, 16)}...
                          </div>
                          <div className="flex items-center gap-3 mt-1">
                            <span className="text-[11px] text-claude-secondary">
                              {token.is_revoked
                                ? (t('h5.revoked') || '已撤销')
                                : isTokenExpired(token.expires_at)
                                ? (t('h5.expired') || '已过期')
                                : `${t('h5.expiresIn') || '过期倒计时'}: ${formatTimeLeft(token.expires_at)}`}
                            </span>
                            {token.used_at && (
                              <span className="text-[11px] text-claude-secondary">
                                {t('h5.usedAt') || '最近使用'}: {new Date(token.used_at).toLocaleString()}
                              </span>
                            )}
                          </div>
                        </div>
                        {!token.is_revoked && !isTokenExpired(token.expires_at) && (
                          <button
                            onClick={() => handleRevoke(token.id)}
                            className="ml-3 px-3 py-1.5 rounded-lg text-[12px] text-red-400 border border-red-500/20 bg-red-500/10 hover:bg-red-500/20 transition-colors shrink-0"
                          >
                            {t('h5.revoke') || '撤销'}
                          </button>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default H5ConnectionView;
