import React, { useState, useEffect, useCallback } from 'react';
import { h5API, H5TokenInfo } from '../utils/tauriAPI';
import QRCode from './QRCode';
import { useI18n } from '../hooks/useI18n';
import { detectBridgePort } from '../api';

interface H5AccessPanelProps {
  conversationId: string;
  onOpenChange?: (open: boolean) => void;
}

const EXPIRY_OPTIONS = [
  { value: 15, label: '15 分钟' },
  { value: 30, label: '30 分钟' },
  { value: 60, label: '1 小时' },
  { value: 120, label: '2 小时' },
  { value: 360, label: '6 小时' },
  { value: 1440, label: '24 小时' },
];

const H5AccessPanel: React.FC<H5AccessPanelProps> = ({ conversationId, onOpenChange }) => {
  const { t } = useI18n();
  const [isOpen, setIsOpen] = useState(false);
  const [tokens, setTokens] = useState<H5TokenInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [generatedToken, setGeneratedToken] = useState<H5TokenInfo | null>(null);
  const [ttlMinutes, setTtlMinutes] = useState(60);
  const [now, setNow] = useState(Date.now());
  const [copiedUrl, setCopiedUrl] = useState(false);
  const [bridgePort, setBridgePort] = useState(30085);

  console.log('[H5AccessPanel] Rendered, conversationId:', conversationId, 'isOpen:', isOpen);

  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    detectBridgePort().then(port => setBridgePort(port));
  }, []);

  const loadTokens = useCallback(async () => {
    console.log('[H5AccessPanel] loadTokens called, conversationId:', conversationId);
    try {
      const list = await h5API.listTokens(conversationId);
      console.log('[H5AccessPanel] loadTokens success, count:', list.length, list);
      setTokens(list);
    } catch (err) {
      console.error('[H5AccessPanel] loadTokens FAILED:', err);
    }
  }, [conversationId]);

  useEffect(() => {
    console.log('[H5AccessPanel] isOpen useEffect, isOpen:', isOpen);
    if (isOpen) {
      loadTokens();
    }
  }, [isOpen, loadTokens]);

  const handleOpenChange = (open: boolean) => {
    console.log('[H5AccessPanel] handleOpenChange:', open);
    setIsOpen(open);
    onOpenChange?.(open);
    if (!open) {
      setGeneratedToken(null);
      setError(null);
    }
  };

  const handleGenerate = async () => {
    console.log('[H5AccessPanel] handleGenerate START, conversationId:', conversationId, 'ttlMinutes:', ttlMinutes);
    setLoading(true);
    setError(null);
    try {
      const token = await h5API.generateToken(conversationId, ttlMinutes);
      console.log('[H5AccessPanel] handleGenerate SUCCESS, token:', token);
      setGeneratedToken(token);
      await loadTokens();
    } catch (err) {
      console.error('[H5AccessPanel] handleGenerate FAILED:', err);
      setError(String(err));
    } finally {
      setLoading(false);
      console.log('[H5AccessPanel] handleGenerate END');
    }
  };

  const handleRevoke = async (tokenId: string) => {
    console.log('[H5AccessPanel] handleRevoke, tokenId:', tokenId);
    try {
      await h5API.revokeToken(tokenId);
      console.log('[H5AccessPanel] handleRevoke SUCCESS');
      if (generatedToken?.id === tokenId) {
        setGeneratedToken(null);
      }
      await loadTokens();
    } catch (err) {
      console.error('[H5AccessPanel] handleRevoke FAILED:', err);
    }
  };

  const handleCleanup = async () => {
    console.log('[H5AccessPanel] handleCleanup START');
    try {
      await h5API.cleanupExpiredTokens();
      console.log('[H5AccessPanel] handleCleanup SUCCESS');
      await loadTokens();
    } catch (err) {
      console.error('[H5AccessPanel] handleCleanup FAILED:', err);
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

  const handleCopyUrl = async (url: string) => {
    try {
      await navigator.clipboard.writeText(url);
      setCopiedUrl(true);
      setTimeout(() => setCopiedUrl(false), 2000);
    } catch {
      // fallback not needed
    }
  };

  const activeTokens = tokens.filter(
    (t) => !t.is_revoked && !isTokenExpired(t.expires_at)
  );
  const expiredTokens = tokens.filter(
    (t) => t.is_revoked || isTokenExpired(t.expires_at)
  );

  return (
    <div className="relative">
      <button
        onClick={() => handleOpenChange(!isOpen)}
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
        <div
          className="fixed inset-0 z-[90] flex items-start justify-center pt-20 bg-black/40"
          onClick={() => handleOpenChange(false)}
        >
          <div
            className="bg-claude-bg border border-claude-border rounded-2xl shadow-xl w-[520px] max-h-[80vh] overflow-y-auto animate-fade-in"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between p-5 border-b border-claude-border">
              <div>
                <h3 className="text-[18px] font-semibold text-claude-text">
                  {t('h5.remoteAccess') || '远程访问'}
                </h3>
                <p className="text-[12px] text-claude-secondary mt-0.5">
                  {t('h5.subtitle') || '生成临时链接，允许外部设备查看对话'}
                </p>
              </div>
              <button
                onClick={() => handleOpenChange(false)}
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
                <label className="block text-[12px] text-claude-secondary mb-2">
                  {t('h5.expiryTime') || '过期时间'}
                </label>
                <div className="flex items-center gap-3">
                  <select
                    value={ttlMinutes}
                    onChange={(e) => setTtlMinutes(Number(e.target.value))}
                    className="flex-1 px-3 py-2 bg-claude-input border border-claude-border rounded-lg text-claude-text text-[13px] focus:outline-none focus:border-blue-500"
                  >
                    {EXPIRY_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                  <button
                    onClick={handleGenerate}
                    disabled={loading}
                    className="px-5 py-2 rounded-lg text-[13px] font-medium bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
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
                  <div className="flex items-center justify-between">
                    <h4 className="text-[14px] font-medium text-claude-text">
                      {t('h5.accessLink') || '访问链接'}
                    </h4>
                    <div className="flex items-center gap-1.5">
                      <span className="text-[11px] text-green-400 flex items-center gap-1">
                        <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse" />
                        活跃
                      </span>
                    </div>
                  </div>

                  <div className="flex justify-center py-2">
                    <QRCode text={getAccessUrl(generatedToken.token)} size={180} />
                  </div>

                  <div className="flex items-center gap-2">
                    <code className="flex-1 px-3 py-2 bg-claude-bg border border-claude-border rounded-lg text-[12px] text-claude-secondary truncate font-mono">
                      {getAccessUrl(generatedToken.token)}
                    </code>
                    <button
                      onClick={() => handleCopyUrl(getAccessUrl(generatedToken.token))}
                      className={`px-3 py-2 rounded-lg text-[12px] font-medium transition-colors shrink-0 ${
                        copiedUrl
                          ? 'bg-green-500/20 text-green-400 border border-green-500/30'
                          : 'bg-blue-500/20 text-blue-400 border border-blue-500/30 hover:bg-blue-500/30'
                      }`}
                    >
                      {copiedUrl ? '已复制' : '复制'}
                    </button>
                  </div>

                  <div className="flex items-center justify-between pt-2 border-t border-claude-border">
                    <div className="flex items-center gap-3">
                      <span className="text-[12px] text-claude-secondary">
                        {t('h5.expiresIn') || '剩余时间'}:
                      </span>
                      <span
                        className={`text-[13px] font-mono font-medium ${
                          new Date(generatedToken.expires_at).getTime() - now < 300000
                            ? 'text-red-400'
                            : 'text-green-400'
                        }`}
                      >
                        {formatTimeLeft(generatedToken.expires_at)}
                      </span>
                    </div>
                    <button
                      onClick={() => handleRevoke(generatedToken.id)}
                      className="px-3 py-1.5 rounded-lg text-[12px] text-red-400 border border-red-500/20 bg-red-500/10 hover:bg-red-500/20 transition-colors"
                    >
                      {t('h5.revoke') || '撤销'}
                    </button>
                  </div>
                </div>
              )}

              {tokens.length > 0 && (
                <div>
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-[14px] font-medium text-claude-text">
                      {t('h5.activeTokens') || '活跃令牌'} ({activeTokens.length})
                    </h4>
                    {expiredTokens.length > 0 && (
                      <button
                        onClick={handleCleanup}
                        className="px-3 py-1 rounded-lg text-[12px] text-claude-secondary border border-claude-border hover:bg-claude-hover transition-colors"
                      >
                        清理过期令牌 ({expiredTokens.length})
                      </button>
                    )}
                  </div>

                  <div className="space-y-2">
                    {tokens
                      .filter((t) => !t.is_revoked && !isTokenExpired(t.expires_at))
                      .map((token) => (
                        <div
                          key={token.id}
                          className="flex items-center justify-between p-3 rounded-lg border border-claude-border bg-claude-input"
                        >
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="w-1.5 h-1.5 rounded-full bg-green-400" />
                              <span className="text-[12px] text-claude-secondary truncate font-mono">
                                {token.token.substring(0, 16)}...
                              </span>
                            </div>
                            <div className="flex items-center gap-3 mt-1 ml-3.5">
                              <span className="text-[11px] text-green-400 font-medium">
                                {formatTimeLeft(token.expires_at)}
                              </span>
                              {token.used_at && (
                                <span className="text-[11px] text-claude-secondary">
                                  {t('h5.lastUsed') || '最近使用'}: {new Date(token.used_at).toLocaleTimeString()}
                                </span>
                              )}
                            </div>
                          </div>
                          <div className="flex items-center gap-2 ml-3 shrink-0">
                            <button
                              onClick={() => handleCopyUrl(getAccessUrl(token.token))}
                              className="px-2.5 py-1.5 rounded-lg text-[11px] text-blue-400 border border-blue-500/20 bg-blue-500/10 hover:bg-blue-500/20 transition-colors"
                            >
                              复制链接
                            </button>
                            <button
                              onClick={() => handleRevoke(token.id)}
                              className="px-2.5 py-1.5 rounded-lg text-[11px] text-red-400 border border-red-500/20 bg-red-500/10 hover:bg-red-500/20 transition-colors"
                            >
                              {t('h5.revoke') || '撤销'}
                            </button>
                          </div>
                        </div>
                      ))}
                  </div>
                </div>
              )}

              {tokens.length === 0 && !generatedToken && (
                <div className="py-8 text-center">
                  <svg
                    className="mx-auto mb-3 text-claude-secondary opacity-40"
                    width="48"
                    height="48"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M18 10l-4-4M8 14l-4 4M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    <path d="M12 7v3l2 2" />
                  </svg>
                  <p className="text-[13px] text-claude-secondary">
                    {t('h5.noTokens') || '还没有生成过访问令牌'}
                  </p>
                  <p className="text-[12px] text-claude-secondary mt-1 opacity-60">
                    {t('h5.noTokensHint') || '选择过期时间并点击"生成链接"开始'}
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default H5AccessPanel;
