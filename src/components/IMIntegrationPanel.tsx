import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  X, Send, Link2, Unlink, Settings, CheckCircle2, XCircle, Loader2,
  QrCode, KeyRound, Users, MessageSquare, Shield, AlertTriangle,
  ChevronDown, BarChart3, FileText, Copy, Check, RefreshCw, Eye, EyeOff,
  UserCheck, UserX, Unlock, Terminal, Play, Square, Download, BookOpen,
  Zap, Code, Info, Trash2, PlusCircle, Database
} from 'lucide-react';
import { useI18n } from '../hooks/useI18n';
import {
  imAPI, bridgeAPI, ImConnectionInfo, ImPlatformConfig, ImConnectionStatus,
  ImMessageStats, ImPermissionMode, ImPairingRequest, ImErrorLog,
} from '../utils/tauriAPI';
import type { BridgeInstanceInfo, FeishuCredentials } from '../utils/tauriAPI';
import { PlatformIcon } from './BrandIcons';
import { detectBridgePort } from '../api';

interface IMIntegrationPanelProps {
  onClose: () => void;
}

type PlatformKey = 'telegram' | 'feishu' | 'wechat' | 'dingtalk' | 'lark_bridge';
type ConnectionMode = 'qr' | 'manual';
type PanelTab = 'config' | 'stats' | 'permissions' | 'logs';

interface PlatformField {
  key: string;
  label: string;
  placeholder: string;
  type?: string;
}

interface PlatformMeta {
  key: PlatformKey;
  label: string;
  color: string;
  bgColor: string;
  borderColor: string;
  accentColor: string;
  lightBg: string;
  fields: PlatformField[];
}

const PLATFORMS: PlatformMeta[] = [
  {
    key: 'telegram',
    label: 'Telegram',
    color: 'text-[#2AABEE]',
    bgColor: 'bg-[#2AABEE]/5',
    borderColor: 'border-[#2AABEE]/20',
    accentColor: '#2AABEE',
    lightBg: 'bg-[#2AABEE]/10',
    fields: [
      { key: 'token', label: 'Bot Token', placeholder: '123456:ABC-DEF...' },
      { key: 'webhook_url', label: 'Webhook URL (可选)', placeholder: 'https://your-server.com/webhook' },
    ],
  },
  {
    key: 'feishu',
    label: '飞书',
    color: 'text-[#3370FF]',
    bgColor: 'bg-[#3370FF]/5',
    borderColor: 'border-[#3370FF]/20',
    accentColor: '#3370FF',
    lightBg: 'bg-[#3370FF]/10',
    fields: [
      { key: 'webhook_url', label: 'Webhook URL', placeholder: 'https://open.feishu.cn/open-apis/bot/v2/hook/...' },
      { key: 'token', label: 'Token (可选)', placeholder: 'Verification token' },
    ],
  },
  {
    key: 'wechat',
    label: '微信',
    color: 'text-[#07C160]',
    bgColor: 'bg-[#07C160]/5',
    borderColor: 'border-[#07C160]/20',
    accentColor: '#07C160',
    lightBg: 'bg-[#07C160]/10',
    fields: [
      { key: 'webhook_url', label: 'Webhook URL', placeholder: 'https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=...' },
      { key: 'token', label: 'Token (可选)', placeholder: 'CorpSecret / Token' },
    ],
  },
  {
    key: 'dingtalk',
    label: '钉钉',
    color: 'text-[#0089FF]',
    bgColor: 'bg-[#0089FF]/5',
    borderColor: 'border-[#0089FF]/20',
    accentColor: '#0089FF',
    lightBg: 'bg-[#0089FF]/10',
    fields: [
      { key: 'webhook_url', label: 'Webhook URL', placeholder: 'https://oapi.dingtalk.com/robot/send?access_token=...' },
      { key: 'token', label: 'Token (可选)', placeholder: 'Access Token / Secret' },
    ],
  },
  {
    key: 'lark_bridge',
    label: '飞书 Bridge',
    color: 'text-[#3370FF]',
    bgColor: 'bg-[#3370FF]/5',
    borderColor: 'border-[#3370FF]/20',
    accentColor: '#3370FF',
    lightBg: 'bg-[#3370FF]/10',
    fields: [],
  },
];

const STATUS_CONFIG: Record<string, { label: string; color: string; bg: string; border: string; icon: React.ReactNode }> = {
  connected: {
    label: '已连接',
    color: 'text-green-500',
    bg: 'bg-green-500/10',
    border: 'border-green-500/20',
    icon: <CheckCircle2 size={12} />,
  },
  connecting: {
    label: '连接中',
    color: 'text-amber-500',
    bg: 'bg-amber-500/10',
    border: 'border-amber-500/20',
    icon: <Loader2 size={12} className="animate-spin" />,
  },
  disconnected: {
    label: '已断开',
    color: 'text-claude-textSecondary',
    bg: 'bg-claude-hover',
    border: 'border-claude-border',
    icon: <Unlink size={12} />,
  },
  error: {
    label: '连接错误',
    color: 'text-red-500',
    bg: 'bg-red-500/10',
    border: 'border-red-500/20',
    icon: <XCircle size={12} />,
  },
};

const PERMISSION_MODE_CONFIG: Record<ImPermissionMode, { label: string; desc: string; icon: React.ReactNode }> = {
  open: {
    label: '开放模式',
    desc: '任何人都可以使用',
    icon: <Unlock size={14} />,
  },
  whitelist: {
    label: '白名单',
    desc: '仅允许指定用户',
    icon: <Shield size={14} />,
  },
  pairing_code: {
    label: '配对码',
    desc: '需要配对码验证',
    icon: <KeyRound size={14} />,
  },
};

const TAB_CONFIG: Record<PanelTab, { label: string; icon: React.ReactNode }> = {
  config: { label: '配置', icon: <Settings size={13} /> },
  stats: { label: '统计', icon: <BarChart3 size={13} /> },
  permissions: { label: '权限', icon: <Shield size={13} /> },
  logs: { label: '日志', icon: <FileText size={13} /> },
};

const IMIntegrationPanel: React.FC<IMIntegrationPanelProps> = ({ onClose }) => {
  const { t } = useI18n();
  const [connections, setConnections] = useState<ImConnectionInfo[]>([]);
  const [connectionStatuses, setConnectionStatuses] = useState<Record<string, ImConnectionStatus>>({});
  const [messageStats, setMessageStats] = useState<Record<string, ImMessageStats>>({});
  const [permissionModes, setPermissionModes] = useState<Record<string, ImPermissionMode>>({});
  const [pairingRequests, setPairingRequests] = useState<Record<string, ImPairingRequest[]>>({});
  const [errorLogs, setErrorLogs] = useState<Record<string, ImErrorLog[]>>({});
  const [loading, setLoading] = useState<Record<string, boolean>>({});
  const [expandedPlatform, setExpandedPlatform] = useState<PlatformKey | null>(null);
  const [activeTab, setActiveTab] = useState<Record<string, PanelTab>>({});
  const [connectionMode, setConnectionMode] = useState<Record<string, ConnectionMode>>({});
  const [configForms, setConfigForms] = useState<Record<PlatformKey, Record<string, string>>>({
    telegram: { token: '', webhook_url: '' },
    feishu: { token: '', webhook_url: '' },
    wechat: { token: '', webhook_url: '' },
    dingtalk: { token: '', webhook_url: '' },
    lark_bridge: {},
  });
  const [sendForms, setSendForms] = useState<Record<PlatformKey, { chatId: string; message: string }>>({
    telegram: { chatId: '', message: '' },
    feishu: { chatId: '', message: '' },
    wechat: { chatId: '', message: '' },
    dingtalk: { chatId: '', message: '' },
    lark_bridge: { chatId: '', message: '' },
  });
  const [sending, setSending] = useState<Record<string, boolean>>({});
  const [qrCodeUrl, setQrCodeUrl] = useState<Record<string, string>>({});
  const [pairingCode, setPairingCode] = useState<Record<string, string>>({});
  const [bridgePort, setBridgePort] = useState(30085);
  const [bridgeInstances, setBridgeInstances] = useState<BridgeInstanceInfo[]>([]);
  const [bridgeLoading, setBridgeLoading] = useState(false);
  const [bridgeActionLoading, setBridgeActionLoading] = useState(false);
  const [bridgeCredentials, setBridgeCredentials] = useState<FeishuCredentials | null>(null);
  const [credentialLoading, setCredentialLoading] = useState(false);
  const [qrAuthUrl, setQrAuthUrl] = useState<string | null>(null);
  const [qrAuthLoading, setQrAuthLoading] = useState(false);
  const [qrAuthStatus, setQrAuthStatus] = useState<string>('idle'); // idle | scanning | completed | failed
  const [feishuConnected, setFeishuConnected] = useState(false);
  const [bridgeFeatures, setBridgeFeatures] = useState<any>(null);
  const [bridgeSessions, setBridgeSessions] = useState<any[]>([]);
  const [bridgeFeaturesLoading, setBridgeFeaturesLoading] = useState(false);
  const [bridgeLogs, setBridgeLogs] = useState<string[]>([]);

  useEffect(() => {
    detectBridgePort().then(port => setBridgePort(port));
  }, []);

  // Detect lark-channel-bridge on mount
  useEffect(() => {
    loadBridgeStatus();
    loadCredentials();
    loadBridgeFeatures();
    loadBridgeSessions();
  }, []);
  const [copied, setCopied] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<string | null>(null);
  const [showAllLogs, setShowAllLogs] = useState<Record<string, boolean>>({});
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const loadBridgeStatus = useCallback(async () => {
    setBridgeLoading(true);
    try {
      const instances = await bridgeAPI.detect();
      setBridgeInstances(instances);
    } catch (e) {
      console.error('Bridge detect failed:', e);
      setBridgeInstances([]);
    } finally {
      setBridgeLoading(false);
    }
  }, []);

  const handleBridgeStart = useCallback(async () => {
    setBridgeActionLoading(true);
    setError(null);
    try {
      await bridgeAPI.start();
      await new Promise(r => setTimeout(r, 3000));
      await loadBridgeStatus();
    } catch (e: unknown) {
      setError(`Bridge start failed: ${e?.toString() || 'Unknown error'}`);
    } finally {
      setBridgeActionLoading(false);
    }
  }, [loadBridgeStatus]);

  const handleBridgeStop = useCallback(async (inst: BridgeInstanceInfo) => {
    setBridgeActionLoading(true);
    setError(null);
    try {
      await bridgeAPI.stop(inst.app_id || undefined);
      await new Promise(r => setTimeout(r, 2000));
      await loadBridgeStatus();
    } catch (e: unknown) {
      setError(`Bridge stop failed: ${e?.toString() || 'Unknown error'}`);
    } finally {
      setBridgeActionLoading(false);
    }
  }, [loadBridgeStatus]);

  // Detect bridge credentials on mount
  const loadCredentials = useCallback(async () => {
    setCredentialLoading(true);
    try {
      const creds = await bridgeAPI.getCredentials();
      setBridgeCredentials(creds);
    } catch (e) {
      console.error('Credential detect failed:', e);
      setBridgeCredentials(null);
    } finally {
      setCredentialLoading(false);
    }
  }, []);

  const handleQrAuthStart = useCallback(async () => {
    setQrAuthLoading(true);
    setQrAuthStatus('scanning');
    setError(null);
    try {
      const url = await bridgeAPI.startAuth();
      setQrAuthUrl(url);
      // Poll for completion
      const poll = setInterval(async () => {
        try {
          const creds = await bridgeAPI.completeAuth();
          if (creds) {
            setBridgeCredentials(creds);
            setQrAuthStatus('completed');
            clearInterval(poll);
          }
        } catch {}
      }, 3000);
      // Timeout after 10 minutes
      setTimeout(() => { clearInterval(poll); setQrAuthStatus('failed'); }, 600000);
    } catch (e: unknown) {
      setError(`Auth failed: ${e?.toString() || 'Unknown error'}`);
      setQrAuthStatus('failed');
    } finally {
      setQrAuthLoading(false);
    }
  }, []);

  const loadBridgeFeatures = useCallback(async () => {
    setBridgeFeaturesLoading(true);
    try {
      const resp = await fetch(`http://127.0.0.1:${bridgePort}/api/feishu-bridge/features`);
      const data = await resp.json();
      if (data.status === 'ok') setBridgeFeatures(data.features);
    } catch (e) {
      console.error('Bridge features load failed:', e);
    } finally {
      setBridgeFeaturesLoading(false);
    }
  }, [bridgePort]);

  const loadBridgeSessions = useCallback(async () => {
    try {
      const resp = await fetch(`http://127.0.0.1:${bridgePort}/api/feishu-bridge/sessions`);
      const data = await resp.json();
      if (data.status === 'ok') setBridgeSessions(data.sessions || []);
    } catch (e) {
      console.error('Bridge sessions load failed:', e);
    }
  }, [bridgePort]);

  const handleBridgeInstall = useCallback(async () => {
    setBridgeActionLoading(true);
    setError(null);
    try {
      const resp = await fetch(`http://127.0.0.1:${bridgePort}/api/feishu-bridge/install`, { method: 'POST' });
      const data = await resp.json();
      setBridgeLogs(prev => [...prev, data.message].slice(-20));
      await loadBridgeStatus();
    } catch (e: unknown) {
      setError(`Bridge install failed: ${e?.toString() || 'Unknown error'}`);
    } finally {
      setBridgeActionLoading(false);
    }
  }, [bridgePort, loadBridgeStatus]);

  const handleBridgeCommand = useCallback(async (command: string) => {
    try {
      const resp = await fetch(`http://127.0.0.1:${bridgePort}/api/feishu-bridge/command`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command }),
      });
      const data = await resp.json();
      setBridgeLogs(prev => [...prev, `[${command}] ${data.message}`].slice(-20));
      await loadBridgeStatus();
      if (command === 'status') await loadBridgeSessions();
    } catch (e) {
      console.error('Bridge command failed:', e);
    }
  }, [bridgePort, loadBridgeStatus, loadBridgeSessions]);

  const handleSessionCleanup = useCallback(async () => {
    try {
      const resp = await fetch(`http://127.0.0.1:${bridgePort}/api/feishu-bridge/sessions/cleanup`, { method: 'POST' });
      const data = await resp.json();
      setBridgeLogs(prev => [...prev, data.message].slice(-20));
      await loadBridgeSessions();
    } catch (e) {
      console.error('Session cleanup failed:', e);
    }
  }, [bridgePort, loadBridgeSessions]);

  const loadConnections = useCallback(async () => {
    try {
      const conns = await imAPI.listConnections();
      setConnections(conns);
      for (const conn of conns) {
        const key = conn.platform as PlatformKey;
        setConfigForms(prev => ({
          ...prev,
          [key]: {
            webhook_url: conn.config.webhook_url || '',
            token: conn.config.token || '',
          },
        }));
      }
    } catch (e) {
      console.error('[IM] Failed to load connections:', e);
    }
  }, []);

  const loadConnectionStatuses = useCallback(async () => {
    try {
      const statuses: Record<string, ImConnectionStatus> = {};
      for (const platform of PLATFORMS) {
        const status = await imAPI.getConnectionStatus(platform.key);
        statuses[platform.key] = status;
      }
      setConnectionStatuses(statuses);
    } catch (e) {
      console.error('[IM] Failed to load connection statuses:', e);
    }
  }, []);

  const loadMessageStats = useCallback(async () => {
    try {
      const stats = await imAPI.getMessageStats();
      const statsMap: Record<string, ImMessageStats> = {};
      for (const stat of stats) {
        statsMap[stat.platform] = stat;
      }
      setMessageStats(statsMap);
    } catch (e) {
      console.error('[IM] Failed to load message stats:', e);
    }
  }, []);

  const loadPermissionModes = useCallback(async () => {
    try {
      const modes: Record<string, ImPermissionMode> = {};
      for (const platform of PLATFORMS) {
        const result = await imAPI.getPermissionMode(platform.key);
        modes[platform.key] = result.mode;
      }
      setPermissionModes(modes);
    } catch (e) {
      console.error('[IM] Failed to load permission modes:', e);
    }
  }, []);

  const loadPairingRequests = useCallback(async () => {
    try {
      const requests: Record<string, ImPairingRequest[]> = {};
      for (const platform of PLATFORMS) {
        const reqs = await imAPI.getPendingPairingRequests(platform.key);
        requests[platform.key] = reqs;
      }
      setPairingRequests(requests);
    } catch (e) {
      console.error('[IM] Failed to load pairing requests:', e);
    }
  }, []);

  const loadErrorLogs = useCallback(async () => {
    try {
      const logs = await imAPI.getErrorLogs();
      const logsMap: Record<string, ImErrorLog[]> = {};
      for (const log of logs) {
        if (!logsMap[log.platform]) logsMap[log.platform] = [];
        logsMap[log.platform].push(log);
      }
      setErrorLogs(logsMap);
    } catch (e) {
      console.error('[IM] Failed to load error logs:', e);
    }
  }, []);

  useEffect(() => {
    loadConnections();
    loadConnectionStatuses();
    loadMessageStats();
    loadPermissionModes();
    loadPairingRequests();
    loadErrorLogs();

    pollingRef.current = setInterval(() => {
      loadConnectionStatuses();
      loadMessageStats();
      loadPairingRequests();
    }, 10000);

    return () => {
      if (pollingRef.current) clearInterval(pollingRef.current);
    };
  }, [loadConnections, loadConnectionStatuses, loadMessageStats, loadPermissionModes, loadPairingRequests, loadErrorLogs]);

  const getConnection = (platform: string): ImConnectionInfo | undefined => {
    return connections.find(c => c.platform === platform);
  };

  const getConnectionStatus = (platform: string): ImConnectionStatus | undefined => {
    return connectionStatuses[platform];
  };

  const isConnected = (platform: string): boolean => {
    const status = getConnectionStatus(platform);
    return status?.status === 'connected';
  };

  const handleConnect = async (platform: PlatformKey) => {
    const form = configForms[platform];
    setLoading(prev => ({ ...prev, [platform]: true }));
    setError(null);
    try {
      const config: ImPlatformConfig = {
        webhook_url: form.webhook_url || '',
        token: form.token || '',
      };
      await imAPI.connectPlatform(platform, config);
      await loadConnections();
      await loadConnectionStatuses();
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '连接失败'}`);
    } finally {
      setLoading(prev => ({ ...prev, [platform]: false }));
    }
  };

  const handleDisconnect = async (platform: PlatformKey) => {
    setLoading(prev => ({ ...prev, [platform]: true }));
    setError(null);
    try {
      await imAPI.disconnectPlatform(platform);
      await loadConnections();
      await loadConnectionStatuses();
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '断开失败'}`);
    } finally {
      setLoading(prev => ({ ...prev, [platform]: false }));
    }
  };

  const handleSendMessage = async (platform: PlatformKey) => {
    const form = sendForms[platform];
    if (!form.chatId.trim() || !form.message.trim()) return;
    setSending(prev => ({ ...prev, [platform]: true }));
    setError(null);
    try {
      await imAPI.sendMessage(platform, form.chatId, form.message);
      setSendForms(prev => ({
        ...prev,
        [platform]: { chatId: prev[platform].chatId, message: '' },
      }));
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '发送失败'}`);
    } finally {
      setSending(prev => ({ ...prev, [platform]: false }));
    }
  };

  const handleGenerateQr = async (platform: PlatformKey) => {
    setLoading(prev => ({ ...prev, [`${platform}_qr`]: true }));
    try {
      const result = await imAPI.generateQrCode(platform);
      setQrCodeUrl(prev => ({ ...prev, [platform]: result.qr_url }));
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '生成二维码失败'}`);
    } finally {
      setLoading(prev => ({ ...prev, [`${platform}_qr`]: false }));
    }
  };

  const handleSetPermissionMode = async (platform: PlatformKey, mode: ImPermissionMode) => {
    setLoading(prev => ({ ...prev, [`${platform}_perm`]: true }));
    try {
      await imAPI.setPermissionMode(platform, mode);
      setPermissionModes(prev => ({ ...prev, [platform]: mode }));
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '设置权限模式失败'}`);
    } finally {
      setLoading(prev => ({ ...prev, [`${platform}_perm`]: false }));
    }
  };

  const handleGeneratePairingCode = async (platform: PlatformKey) => {
    setLoading(prev => ({ ...prev, [`${platform}_pair`]: true }));
    try {
      const result = await imAPI.generatePairingCode(platform, 'current_user');
      setPairingCode(prev => ({ ...prev, [platform]: result.code }));
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '生成配对码失败'}`);
    } finally {
      setLoading(prev => ({ ...prev, [`${platform}_pair`]: false }));
    }
  };

  const handleApproveRequest = async (platform: PlatformKey, userId: string) => {
    setLoading(prev => ({ ...prev, [`${platform}_approve_${userId}`]: true }));
    try {
      await imAPI.approvePairingRequest(platform, userId);
      await loadPairingRequests();
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '批准请求失败'}`);
    } finally {
      setLoading(prev => ({ ...prev, [`${platform}_approve_${userId}`]: false }));
    }
  };

  const handleRejectRequest = async (platform: PlatformKey, userId: string) => {
    setLoading(prev => ({ ...prev, [`${platform}_reject_${userId}`]: true }));
    try {
      await imAPI.rejectPairingRequest(platform, userId);
      await loadPairingRequests();
    } catch (e: unknown) {
      setError(`${platform}: ${e?.toString() || '拒绝请求失败'}`);
    } finally {
      setLoading(prev => ({ ...prev, [`${platform}_reject_${userId}`]: false }));
    }
  };

  const handleConfigChange = (platform: PlatformKey, fieldKey: string, value: string) => {
    setConfigForms(prev => ({
      ...prev,
      [platform]: { ...prev[platform], [fieldKey]: value },
    }));
  };

  const handleSendFormChange = (platform: PlatformKey, field: 'chatId' | 'message', value: string) => {
    setSendForms(prev => ({
      ...prev,
      [platform]: { ...prev[platform], [field]: value },
    }));
  };

  const toggleExpand = (platform: PlatformKey) => {
    setExpandedPlatform(prev => (prev === platform ? null : platform));
  };

  const setTab = (platform: PlatformKey, tab: PanelTab) => {
    setActiveTab(prev => ({ ...prev, [platform]: tab }));
  };

  const setMode = (platform: PlatformKey, mode: ConnectionMode) => {
    setConnectionMode(prev => ({ ...prev, [platform]: mode }));
  };

  const copyToClipboard = async (text: string, key: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(prev => ({ ...prev, [key]: true }));
      setTimeout(() => setCopied(prev => ({ ...prev, [key]: false })), 2000);
    } catch (e) {
      console.error('Copy failed:', e);
    }
  };

  const renderStatusBadge = (platformKey: PlatformKey) => {
    if (platformKey === 'lark_bridge') {
      const running = bridgeInstances.some(i => i.running);
      return (
        <span className={`flex items-center gap-1 text-[11px] font-medium px-2 py-0.5 rounded-full border ${
          running
            ? 'bg-green-500/10 text-green-500 border-green-500/20'
            : 'bg-claude-hover text-claude-textSecondary border-claude-border'
        }`}>
          {running ? <CheckCircle2 size={12} /> : <Unlink size={12} />}
          {running ? '已连接' : '未运行'}
        </span>
      );
    }
    const status = getConnectionStatus(platformKey);
    const statusKey = status?.status || 'disconnected';
    const config = STATUS_CONFIG[statusKey] || STATUS_CONFIG.disconnected;

    return (
      <span className={`flex items-center gap-1 text-[11px] font-medium px-2 py-0.5 rounded-full ${config.bg} ${config.color} ${config.border} border`}>
        {config.icon}
        {config.label}
      </span>
    );
  };

  const renderConfigTab = (platformMeta: PlatformMeta) => {
    const platformKey = platformMeta.key;
    const connected = isConnected(platformKey);
    const isLoading = loading[platformKey] || false;
    const mode = connectionMode[platformKey] || 'manual';
    const qrLoading = loading[`${platformKey}_qr`] || false;

    return (
      <div className="space-y-3 animate-fade-in">
        <div className="flex bg-claude-hover rounded-lg p-0.5">
          <button
              onClick={() => setMode(platformKey, 'qr')}
            className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md text-[11px] font-medium transition-all ${
              mode === 'qr'
                ? 'bg-claude-bg text-claude-text shadow-sm'
                : 'text-claude-textSecondary hover:text-claude-text'
            }`}
          >
            <QrCode size={12} />
            扫码接入
          </button>
          <button
            onClick={() => setMode(platformKey, 'manual')}
            className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md text-[11px] font-medium transition-all ${
              mode === 'manual'
                ? 'bg-claude-bg text-claude-text shadow-sm'
                : 'text-claude-textSecondary hover:text-claude-text'
            }`}
          >
            <KeyRound size={12} />
            手动配置
          </button>
        </div>

        {mode === 'qr' ? (
          <div className="space-y-3">
            <div className="flex flex-col items-center gap-3 py-3">
              {qrCodeUrl[platformKey] ? (
                <div className="relative">
                  <div className="w-40 h-40 rounded-xl border-2 border-claude-border bg-white p-2 flex items-center justify-center">
                    <QrCode size={120} className="text-claude-text" />
                  </div>
                  <div className="absolute -bottom-1 -right-1 w-6 h-6 rounded-full bg-claude-bg border border-claude-border flex items-center justify-center">
                    <PlatformIcon platform={platformKey} size={14} />
                  </div>
                </div>
              ) : (
                <div className="w-40 h-40 rounded-xl border-2 border-dashed border-claude-border bg-claude-hover flex flex-col items-center justify-center gap-2">
                  <QrCode size={32} className="text-claude-textSecondary/40" />
                  <span className="text-[10px] text-claude-textSecondary">点击生成二维码</span>
                </div>
              )}
              <button
                onClick={() => handleGenerateQr(platformKey)}
                disabled={qrLoading}
                className="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-claude-hover border border-claude-border text-claude-text text-[11px] font-medium hover:bg-claude-btnHover transition-colors disabled:opacity-50"
              >
                {qrLoading ? <Loader2 size={12} className="animate-spin" /> : <RefreshCw size={12} />}
                生成二维码
              </button>
              {qrCodeUrl[platformKey] && (
                <p className="text-[10px] text-claude-textSecondary text-center">
                  请使用{platformMeta.label}扫描二维码完成授权
                </p>
              )}
            </div>
          </div>
        ) : (
          <div className="space-y-2.5">
            {platformMeta.fields.map((field: PlatformField) => (
              <div key={field.key}>
                <label className="block text-[11px] font-medium text-claude-textSecondary mb-1.5">
                  {field.label}
                </label>
                <input
                  type={field.type || 'text'}
                  value={configForms[platformKey][field.key] || ''}
                  onChange={e => handleConfigChange(platformKey, field.key, e.target.value)}
                  placeholder={field.placeholder}
                  disabled={connected}
                  className="w-full px-3 py-2 rounded-lg bg-claude-bg border border-claude-border text-[12px] text-claude-text placeholder:text-claude-textSecondary/40 focus:outline-none focus:border-blue-500/50 disabled:opacity-50 transition-colors"
                />
              </div>
            ))}

            <div className="flex gap-2 pt-1">
              {connected ? (
                <button
                  onClick={() => handleDisconnect(platformKey)}
                  disabled={isLoading}
                  className="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-[12px] font-medium hover:bg-red-500/20 transition-colors disabled:opacity-50"
                >
                  <Unlink size={12} />
                  断开连接
                </button>
              ) : (
                <button
                  onClick={() => handleConnect(platformKey)}
                  disabled={isLoading}
                  className="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-400 text-[12px] font-medium hover:bg-blue-500/20 transition-colors disabled:opacity-50"
                >
                  <Link2 size={12} />
                  连接
                </button>
              )}
            </div>
          </div>
        )}

        {connected && (
          <div className="space-y-2.5 pt-2 border-t border-claude-border/30">
            <div className="text-[11px] font-medium text-claude-textSecondary mb-1">发送消息</div>
            <input
              type="text"
              value={sendForms[platformKey].chatId}
              onChange={e => handleSendFormChange(platformKey, 'chatId', e.target.value)}
              placeholder="Chat ID"
              className="w-full px-3 py-2 rounded-lg bg-claude-bg border border-claude-border text-[12px] text-claude-text placeholder:text-claude-textSecondary/40 focus:outline-none focus:border-blue-500/50 transition-colors"
            />
            <div className="flex gap-2">
              <input
                type="text"
                value={sendForms[platformKey].message}
                onChange={e => handleSendFormChange(platformKey, 'message', e.target.value)}
                placeholder="输入消息内容..."
                className="flex-1 px-3 py-2 rounded-lg bg-claude-bg border border-claude-border text-[12px] text-claude-text placeholder:text-claude-textSecondary/40 focus:outline-none focus:border-blue-500/50 transition-colors"
                onKeyDown={e => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    handleSendMessage(platformKey);
                  }
                }}
              />
              <button
                onClick={() => handleSendMessage(platformKey)}
                disabled={sending[platformKey] || !sendForms[platformKey].chatId.trim() || !sendForms[platformKey].message.trim()}
                className="flex items-center justify-center w-9 h-9 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-400 hover:bg-blue-500/20 transition-colors disabled:opacity-50"
              >
                {sending[platformKey] ? <Loader2 size={14} className="animate-spin" /> : <Send size={14} />}
              </button>
            </div>
          </div>
        )}
      </div>
    );
  };

  const renderStatsTab = (platformMeta: PlatformMeta) => {
    const platformKey = platformMeta.key;
    const stats = messageStats[platformKey];

    return (
      <div className="space-y-3 animate-fade-in">
        <div className="grid grid-cols-2 gap-2">
          <div className={`rounded-lg border ${platformMeta.borderColor} ${platformMeta.bgColor} p-3`}>
            <div className="flex items-center gap-1.5 mb-1.5">
              <MessageSquare size={12} className={platformMeta.color} />
              <span className="text-[10px] text-claude-textSecondary font-medium">今日消息</span>
            </div>
            <span className="text-[20px] font-bold text-claude-text">{stats?.today_messages ?? 0}</span>
          </div>
          <div className={`rounded-lg border ${platformMeta.borderColor} ${platformMeta.bgColor} p-3`}>
            <div className="flex items-center gap-1.5 mb-1.5">
              <Users size={12} className={platformMeta.color} />
              <span className="text-[10px] text-claude-textSecondary font-medium">活跃用户</span>
            </div>
            <span className="text-[20px] font-bold text-claude-text">{stats?.active_users ?? 0}</span>
          </div>
        </div>
        <div className={`rounded-lg border ${platformMeta.borderColor} ${platformMeta.bgColor} p-3`}>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5">
              <BarChart3 size={12} className={platformMeta.color} />
              <span className="text-[10px] text-claude-textSecondary font-medium">总消息数</span>
            </div>
            <span className="text-[16px] font-semibold text-claude-text">{stats?.total_messages ?? 0}</span>
          </div>
        </div>
        {stats?.updated_at && (
          <p className="text-[10px] text-claude-textSecondary/60 text-right">
            更新于 {new Date(stats.updated_at).toLocaleTimeString()}
          </p>
        )}
      </div>
    );
  };

  const renderPermissionsTab = (platformMeta: PlatformMeta) => {
    const platformKey = platformMeta.key;
    const currentMode = permissionModes[platformKey] || 'open';
    const permLoading = loading[`${platformKey}_perm`] || false;
    const pairLoading = loading[`${platformKey}_pair`] || false;
    const requests = pairingRequests[platformKey] || [];
    const code = pairingCode[platformKey];

    return (
      <div className="space-y-3 animate-fade-in">
        <div className="space-y-1.5">
          <label className="text-[11px] font-medium text-claude-textSecondary">权限模式</label>
          <div className="space-y-1.5">
            {(Object.keys(PERMISSION_MODE_CONFIG) as ImPermissionMode[]).map(mode => {
              const config = PERMISSION_MODE_CONFIG[mode];
              const isActive = currentMode === mode;
              return (
                <button
                  key={mode}
                  onClick={() => handleSetPermissionMode(platformKey, mode)}
                  disabled={permLoading}
                  className={`w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg border text-left transition-all ${
                    isActive
                      ? `${platformMeta.borderColor} ${platformMeta.bgColor}`
                      : 'border-claude-border bg-claude-bg hover:bg-claude-hover'
                  }`}
                >
                  <div className={`${isActive ? platformMeta.color : 'text-claude-textSecondary'}`}>
                    {config.icon}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className={`text-[12px] font-medium ${isActive ? 'text-claude-text' : 'text-claude-textSecondary'}`}>
                      {config.label}
                    </div>
                    <div className="text-[10px] text-claude-textSecondary/70">{config.desc}</div>
                  </div>
                  {isActive && <CheckCircle2 size={14} className={platformMeta.color} />}
                </button>
              );
            })}
          </div>
        </div>

        {currentMode === 'pairing_code' && (
          <div className="space-y-2 pt-2 border-t border-claude-border/30">
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-medium text-claude-textSecondary">配对码</label>
              <button
                onClick={() => handleGeneratePairingCode(platformKey)}
                disabled={pairLoading}
                className="flex items-center gap-1 px-2 py-1 rounded-md bg-claude-hover border border-claude-border text-claude-text text-[10px] hover:bg-claude-btnHover transition-colors disabled:opacity-50"
              >
                {pairLoading ? <Loader2 size={10} className="animate-spin" /> : <RefreshCw size={10} />}
                生成
              </button>
            </div>
            {code && (
              <div className="flex items-center gap-2">
                <div className={`flex-1 px-3 py-2 rounded-lg ${platformMeta.lightBg} border ${platformMeta.borderColor} text-center`}>
                  <span className="text-[18px] font-mono font-bold tracking-widest text-claude-text">{code}</span>
                </div>
                <button
                  onClick={() => copyToClipboard(code, `${platformKey}_pair`)}
                  className="p-2 rounded-lg bg-claude-hover border border-claude-border text-claude-textSecondary hover:text-claude-text transition-colors"
                >
                  {copied[`${platformKey}_pair`] ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
                </button>
              </div>
            )}
          </div>
        )}

        {requests.length > 0 && (
          <div className="space-y-2 pt-2 border-t border-claude-border/30">
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-medium text-claude-textSecondary">待审核请求</label>
              <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-amber-500/10 text-amber-500 font-medium">
                {requests.length}
              </span>
            </div>
            <div className="space-y-1.5 max-h-40 overflow-y-auto sidebar-scroll">
              {requests.map(req => (
                <div key={req.id} className="flex items-center gap-2 px-2.5 py-2 rounded-lg bg-claude-hover border border-claude-border/50">
                  <div className="w-6 h-6 rounded-full bg-claude-accent/20 flex items-center justify-center flex-shrink-0">
                    <Users size={12} className="text-claude-accent" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-[11px] font-medium text-claude-text truncate">{req.user_name || req.user_id}</div>
                    <div className="text-[9px] text-claude-textSecondary/60">
                      {new Date(req.created_at).toLocaleString()}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() => handleApproveRequest(platformKey, req.user_id)}
                      disabled={loading[`${platformKey}_approve_${req.user_id}`]}
                      className="p-1 rounded-md bg-green-500/10 text-green-500 hover:bg-green-500/20 transition-colors disabled:opacity-50"
                    >
                      {loading[`${platformKey}_approve_${req.user_id}`] ? <Loader2 size={12} className="animate-spin" /> : <UserCheck size={12} />}
                    </button>
                    <button
                      onClick={() => handleRejectRequest(platformKey, req.user_id)}
                      disabled={loading[`${platformKey}_reject_${req.user_id}`]}
                      className="p-1 rounded-md bg-red-500/10 text-red-500 hover:bg-red-500/20 transition-colors disabled:opacity-50"
                    >
                      {loading[`${platformKey}_reject_${req.user_id}`] ? <Loader2 size={12} className="animate-spin" /> : <UserX size={12} />}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  };

  const renderLogsTab = (platformMeta: PlatformMeta) => {
    const platformKey = platformMeta.key;
    const logs = errorLogs[platformKey] || [];
    const showAll = showAllLogs[platformKey] || false;
    const displayLogs = showAll ? logs : logs.slice(0, 3);

    return (
      <div className="space-y-2 animate-fade-in">
        {logs.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-6 gap-2">
            <CheckCircle2 size={24} className="text-green-500/40" />
            <span className="text-[11px] text-claude-textSecondary">暂无错误日志</span>
          </div>
        ) : (
          <>
            <div className="space-y-1.5 max-h-48 overflow-y-auto sidebar-scroll">
              {displayLogs.map(log => (
                <div key={log.id} className="px-2.5 py-2 rounded-lg bg-red-500/5 border border-red-500/10">
                  <div className="flex items-start gap-1.5">
                    <AlertTriangle size={11} className="text-red-400 mt-0.5 flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <div className="text-[11px] text-red-400 font-medium truncate">{log.error}</div>
                      {log.details && (
                        <div className="text-[10px] text-claude-textSecondary/60 mt-0.5 line-clamp-2">{log.details}</div>
                      )}
                      <div className="text-[9px] text-claude-textSecondary/40 mt-1">
                        {new Date(log.created_at).toLocaleString()}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
            {logs.length > 3 && (
              <button
                onClick={() => setShowAllLogs(prev => ({ ...prev, [platformKey]: !prev[platformKey] }))}
                className="w-full flex items-center justify-center gap-1 py-1.5 text-[10px] text-claude-textSecondary hover:text-claude-text transition-colors"
              >
                {showAll ? <EyeOff size={10} /> : <Eye size={10} />}
                {showAll ? '收起' : `查看全部 ${logs.length} 条`}
              </button>
            )}
          </>
        )}
      </div>
    );
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border flex-shrink-0">
        <div className="flex items-center gap-2.5">
          <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-blue-500/20 to-indigo-500/20 flex items-center justify-center">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" className="text-blue-400">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
              <path d="M8 9h8"/>
              <path d="M8 13h5"/>
            </svg>
          </div>
          <span className="text-[14px] font-medium text-claude-text">{t('sidebar.imIntegration')}</span>
        </div>
        <button
          onClick={onClose}
          className="p-1.5 rounded-lg hover:bg-claude-hover text-claude-textSecondary hover:text-claude-text transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-3 sidebar-scroll">
        {error && (
          <div className="px-3 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-[12px] flex items-center justify-between animate-fade-in">
            <span>{error}</span>
            <button onClick={() => setError(null)} className="text-red-400 hover:text-red-300 p-0.5">
              <X size={12} />
            </button>
          </div>
        )}

        {PLATFORMS.map(platform => {
          const conn = getConnection(platform.key);
          const connected = isConnected(platform.key);
          const isLoading = loading[platform.key] || false;
          const isExpanded = expandedPlatform === platform.key;
          const currentTab = activeTab[platform.key] || 'config';

          return (
            <div
              key={platform.key}
              className={`rounded-xl border transition-all duration-300 overflow-hidden ${
                connected
                  ? `${platform.borderColor} ${platform.bgColor}`
                  : 'border-claude-border bg-claude-bg'
              }`}
            >
              <div
                className="flex items-center justify-between px-4 py-3.5 cursor-pointer hover:bg-claude-hover/30 transition-colors"
                onClick={() => toggleExpand(platform.key)}
              >
                <div className="flex items-center gap-3">
                  <PlatformIcon platform={platform.key} size={28} />
                  <div className="flex flex-col gap-0.5">
                    <span className="text-[13px] font-semibold text-claude-text">{platform.label}</span>
                    {renderStatusBadge(platform.key)}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {isLoading && <Loader2 size={14} className="animate-spin text-blue-400" />}
                  <ChevronDown
                    size={14}
                    className={`text-claude-textSecondary transition-transform duration-300 ${isExpanded ? 'rotate-180' : ''}`}
                  />
                </div>
              </div>

              <div
                className={`transition-all duration-300 ease-in-out ${
                  isExpanded ? 'max-h-[800px] opacity-100' : 'max-h-0 opacity-0'
                } overflow-hidden`}
              >
                <div className="px-4 pb-4 border-t border-claude-border/40">
                  <div className="flex gap-0.5 py-2 -mx-1 overflow-x-auto">
                    {(Object.keys(TAB_CONFIG) as PanelTab[]).map(tab => {
                      const tabConfig = TAB_CONFIG[tab];
                      const isActive = currentTab === tab;
                      return (
                        <button
                          key={tab}
                          onClick={(e) => {
                            e.stopPropagation();
                            setTab(platform.key, tab);
                          }}
                          className={`flex items-center gap-1 px-2.5 py-1.5 rounded-md text-[11px] font-medium whitespace-nowrap transition-all ${
                            isActive
                              ? `${platform.bgColor} ${platform.color} border ${platform.borderColor}`
                              : 'text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover'
                          }`}
                        >
                          {tabConfig.icon}
                          {tabConfig.label}
                        </button>
                      );
                    })}
                  </div>

                  <div className="pt-1">
                    {platform.key === 'lark_bridge' ? (
                      <div className="space-y-3 animate-fade-in pt-2">
                        {/* Bridge instance status */}
                        {bridgeLoading ? (
                          <div className="flex items-center justify-center py-8 text-claude-textSecondary">
                            <Loader2 size={16} className="animate-spin mr-2" />
                            <span className="text-[12px]">检测 bridge 中...</span>
                          </div>
                        ) : (
                          <>
                            {/* Feishu credential status */}
                            {credentialLoading ? (
                              <div className="flex items-center justify-center py-4 text-[12px] text-claude-textSecondary">
                                <Loader2 size={14} className="animate-spin mr-2" />
                                检测飞书凭证中...
                              </div>
                            ) : bridgeCredentials ? (
                              <div className="rounded-xl border border-green-500/20 bg-green-500/5 p-4 space-y-2">
                                <div className="flex items-center gap-2">
                                  <CheckCircle2 size={16} className="text-green-500" />
                                  <span className="text-[13px] font-semibold text-claude-text">飞书凭证已就绪</span>
                                </div>
                                <div className="text-[11px] text-claude-textSecondary space-y-1">
                                  <div>App ID: <span className="font-mono text-claude-text">{bridgeCredentials.app_id}</span></div>
                                  <div>租户: {bridgeCredentials.tenant}</div>
                                </div>
                                <button
                                  className="flex items-center gap-1.5 px-4 py-2 bg-[#3370FF] text-white text-[12px] font-medium rounded-lg hover:bg-[#3370FF]/90 transition-colors"
                                  onClick={() => {
                                    imAPI.connectPlatform('feishu', {
                                      webhook_url: '',
                                      token: '',
                                      extra: {
                                        app_id: bridgeCredentials.app_id,
                                        app_secret: bridgeCredentials.app_secret,
                                      },
                                    }).then(() => {
                                      setFeishuConnected(true);
                                      loadConnectionStatuses();
                                    }).catch((e: any) => setError(`连接失败: ${e?.toString() || '未知错误'}`));
                                  }}
                                >
                                  <Send size={14} />
                                  使用此凭证连接飞书
                                </button>
                              </div>
                            ) : qrAuthStatus === 'scanning' ? (
                              <div className="flex flex-col items-center gap-3 py-6">
                                <QrCode size={64} className="text-[#3370FF]" />
                                <span className="text-[12px] text-claude-textSecondary text-center max-w-[220px]">
                                  {qrAuthUrl
                                    ? '在飞书 App 中打开此链接完成授权'
                                    : '正在生成授权二维码...'}
                                </span>
                                {qrAuthUrl && (
                                  <div className="text-[10px] bg-claude-hover px-3 py-2 rounded-lg break-all text-claude-textSecondary font-mono text-center max-w-full">
                                    {qrAuthUrl}
                                  </div>
                                )}
                                {qrAuthLoading && <Loader2 size={16} className="animate-spin text-blue-400" />}
                              </div>
                            ) : qrAuthStatus === 'completed' ? (
                              <div className="rounded-xl border border-green-500/20 bg-green-500/5 p-4 text-center space-y-2">
                                <CheckCircle2 size={24} className="text-green-500 mx-auto" />
                                <span className="text-[13px] text-claude-text font-semibold">授权完成</span>
                              </div>
                            ) : (
                              <div className="flex flex-col items-center gap-3 py-6">
                                <span className="text-[12px] text-claude-textSecondary text-center">
                                  未检测到飞书凭证
                                </span>
                                <span className="text-[11px] text-claude-textSecondary/60 text-center max-w-[240px]">
                                  需要先在飞书开放平台创建应用，或使用已有 bridge 的凭证
                                </span>
                                <button
                                  onClick={handleQrAuthStart}
                                  disabled={qrAuthLoading}
                                  className="flex items-center gap-1.5 px-4 py-2 bg-[#3370FF] text-white text-[12px] font-medium rounded-lg hover:bg-[#3370FF]/90 transition-colors disabled:opacity-50"
                                >
                                  <QrCode size={14} />
                                  {qrAuthLoading ? '连接中...' : '扫码授权飞书'}
                                </button>
                              </div>
                            )}

                            {/* Bridge instance management */}
                            {bridgeInstances.length > 0 ? bridgeInstances.map((inst, idx) => (
                              <div key={idx} className="rounded-xl border border-green-500/20 bg-green-500/5 p-4 space-y-2.5 mt-2">
                                <div className="flex items-center justify-between">
                                  <div className="flex items-center gap-2">
                                    <span className="w-2 h-2 rounded-full bg-green-500" />
                                    <span className="text-[13px] font-semibold text-claude-text">{inst.bot_name || 'Lark Bridge'}</span>
                                  </div>
                                  <span className="text-[10px] text-claude-textSecondary bg-claude-hover px-2 py-0.5 rounded-full">{inst.version || '-'}</span>
                                </div>
                                <div className="grid grid-cols-2 gap-2 text-[11px] text-claude-textSecondary">
                                  <div><span className="opacity-60">PID</span><div className="font-mono text-claude-text">{inst.pid || '-'}</div></div>
                                  <div><span className="opacity-60">App ID</span><div className="font-mono text-claude-text truncate">{inst.app_id || '-'}</div></div>
                                  <div>
                                    <span className="opacity-60">状态</span>
                                    <div className="flex items-center gap-1">
                                      <span className="w-1.5 h-1.5 rounded-full bg-green-500" />
                                      <span className="text-green-500">运行中</span>
                                    </div>
                                  </div>
                                  <div><span className="opacity-60">启动</span><div className="text-claude-text">{inst.started_at ? new Date(inst.started_at).toLocaleString() : '-'}</div></div>
                                </div>
                                <div className="flex gap-2 pt-1">
                                  <button onClick={() => handleBridgeStop(inst)} disabled={bridgeActionLoading}
                                    className="flex items-center gap-1 px-3 py-1.5 text-[11px] text-red-400 border border-red-400/30 rounded-lg hover:bg-red-500/10 transition-colors disabled:opacity-50"
                                  >{bridgeActionLoading ? <Loader2 size={12} className="animate-spin" /> : <Unlink size={12} />}停止</button>
                                  <button onClick={loadBridgeStatus}
                                    className="flex items-center gap-1 px-3 py-1.5 text-[11px] text-claude-textSecondary border border-claude-border rounded-lg hover:bg-claude-hover transition-colors"
                                  ><RefreshCw size={12} />刷新</button>
                                </div>
                              </div>
                            )) : (
                              <div className="flex flex-col items-center gap-2 py-4 mt-2">
                                <span className="text-[11px] text-claude-textSecondary">
                                  (bridge 后台管理进程未运行)
                                </span>
                                <button onClick={handleBridgeStart} disabled={bridgeActionLoading}
                                  className="text-[11px] text-blue-400 hover:underline"
                                >启动 bridge</button>
                            </div>
                          )}

                          {/* feishu-claude-code-bridge Enhanced Features */}
                          {bridgeFeatures && (
                            <div className="rounded-xl border border-blue-500/20 bg-blue-500/5 p-4 space-y-3 mt-2">
                              <div className="flex items-center gap-2">
                                <Zap size={14} className="text-blue-400" />
                                <span className="text-[13px] font-semibold text-claude-text">Bridge 增强功能</span>
                              </div>
                              <div className="grid gap-2">
                                {bridgeFeatures.streaming_cards && (
                                  <div className="flex items-center gap-2 text-[11px] text-claude-textSecondary">
                                    <span className={`w-1.5 h-1.5 rounded-full ${bridgeFeatures.streaming_cards.enabled ? 'bg-green-500' : 'bg-gray-400'}`} />
                                    <span>流式卡片: AI 输出实时推送到飞书卡片</span>
                                  </div>
                                )}
                                {bridgeFeatures.group_sessions && (
                                  <div className="flex items-center gap-2 text-[11px] text-claude-textSecondary">
                                    <span className={`w-1.5 h-1.5 rounded-full ${bridgeFeatures.group_sessions.enabled ? 'bg-green-500' : 'bg-gray-400'}`} />
                                    <span>一事一群: 会话自动创建独立飞书群</span>
                                  </div>
                                )}
                                {bridgeFeatures.document_export && (
                                  <div className="flex items-center gap-2 text-[11px] text-claude-textSecondary">
                                    <span className={`w-1.5 h-1.5 rounded-full ${bridgeFeatures.document_export.enabled ? 'bg-green-500' : 'bg-gray-400'}`} />
                                    <span>文档导出: AI 对话导出为飞书文档</span>
                                  </div>
                                )}
                              </div>
                            </div>
                          )}

                          {bridgeFeatures?.slash_commands && (
                            <div className="rounded-xl border border-purple-500/20 bg-purple-500/5 p-4 space-y-2.5 mt-2">
                              <div className="flex items-center gap-2">
                                <Terminal size={14} className="text-purple-400" />
                                <span className="text-[13px] font-semibold text-claude-text">斜杠命令</span>
                              </div>
                              <div className="space-y-1">
                                {bridgeFeatures.slash_commands.commands.map((cmd: any, i: number) => (
                                  <div key={i} className="flex items-center gap-2 text-[11px]">
                                    <code className="px-1.5 py-0.5 bg-claude-hover rounded text-[11px] font-mono text-purple-400">{cmd.cmd}</code>
                                    <span className="text-claude-textSecondary">{cmd.desc}</span>
                                  </div>
                                ))}
                              </div>
                            </div>
                          )}

                          {bridgeSessions.length > 0 && (
                            <div className="rounded-xl border border-green-500/20 bg-green-500/5 p-4 space-y-2.5 mt-2">
                              <div className="flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                  <Database size={14} className="text-green-400" />
                                  <span className="text-[13px] font-semibold text-claude-text">活跃会话</span>
                                </div>
                                <span className="text-[10px] text-claude-textSecondary bg-claude-hover px-2 py-0.5 rounded-full">{bridgeSessions.length} 个</span>
                              </div>
                              <div className="space-y-2 max-h-32 overflow-y-auto">
                                {bridgeSessions.slice(0, 5).map((s: any, i: number) => (
                                  <div key={i} className="flex items-center justify-between text-[11px] p-2 rounded-lg bg-claude-hover/50">
                                    <div className="flex-1 min-w-0">
                                      <div className="text-claude-text truncate">{s.title}</div>
                                      <div className="text-claude-textSecondary/60 text-[10px]">{s.message_count} 条消息</div>
                                    </div>
                                    <span className={`px-1.5 py-0.5 rounded-full text-[10px] ${s.status === 'active' ? 'bg-green-500/10 text-green-400' : 'bg-gray-500/10 text-gray-400'}`}>{s.status}</span>
                                  </div>
                                ))}
                              </div>
                              <div className="flex gap-2 pt-1">
                                <button onClick={loadBridgeSessions} className="flex items-center gap-1 text-[10px] text-claude-textSecondary/60 hover:text-claude-textSecondary"><RefreshCw size={10} /> 刷新</button>
                                <button onClick={handleSessionCleanup} className="flex items-center gap-1 text-[10px] text-red-400/60 hover:text-red-400"><Trash2 size={10} /> 清理过期</button>
                              </div>
                            </div>
                          )}

                          <div className="flex gap-2 mt-2 flex-wrap">
                            <button onClick={() => handleBridgeCommand('status')} disabled={bridgeActionLoading}
                              className="flex items-center gap-1 px-3 py-1.5 text-[11px] text-blue-400 border border-blue-400/30 rounded-lg hover:bg-blue-500/10 transition-colors"
                            ><Info size={12} />状态</button>
                            <button onClick={handleBridgeInstall} disabled={bridgeActionLoading}
                              className="flex items-center gap-1 px-3 py-1.5 text-[11px] text-green-400 border border-green-400/30 rounded-lg hover:bg-green-500/10 transition-colors"
                            ><Download size={12} />安装</button>
                            <button onClick={loadBridgeFeatures} disabled={bridgeFeaturesLoading}
                              className="flex items-center gap-1 px-3 py-1.5 text-[11px] text-purple-400 border border-purple-400/30 rounded-lg hover:bg-purple-500/10 transition-colors"
                            ><Zap size={12} />功能</button>
                          </div>

                          {bridgeLogs.length > 0 && (
                            <div className="rounded-xl border border-claude-border p-3 space-y-1.5 mt-2">
                              <div className="flex items-center gap-1.5">
                                <BookOpen size={12} className="text-claude-textSecondary" />
                                <span className="text-[11px] font-medium text-claude-textSecondary">操作日志</span>
                              </div>
                              <div className="space-y-1 max-h-24 overflow-y-auto">
                                {bridgeLogs.map((log, i) => (
                                  <div key={i} className="text-[10px] font-mono text-claude-textSecondary/60 px-2 py-0.5 bg-claude-hover/30 rounded">{log}</div>
                                ))}
                              </div>
                            </div>
                          )}
                        </>
                      )}
                    </div>
                ) : (
                      <>
                        {currentTab === 'config' && renderConfigTab(platform)}
                        {currentTab === 'stats' && renderStatsTab(platform)}
                        {currentTab === 'permissions' && renderPermissionsTab(platform)}
                        {currentTab === 'logs' && renderLogsTab(platform)}
                      </>
                    )}
                  </div>
                </div>
              </div>
            </div>
          );
        })}

        <div className="mt-4 pt-3 border-t border-claude-border/50">
          <div className="text-[11px] text-claude-textSecondary/60 space-y-1.5">
            <p className="font-medium text-claude-textSecondary/80">Webhook 接收地址：</p>
            <code className="block px-3 py-2 rounded-lg bg-claude-bg border border-claude-border text-[10px] text-claude-textSecondary break-all">
              http://127.0.0.1:{bridgePort}/api/im/webhook/&#123;platform&#125;
            </code>
            <p className="mt-1">将此地址配置到对应 IM 平台的 Webhook 回调中即可接收消息。</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default IMIntegrationPanel;
