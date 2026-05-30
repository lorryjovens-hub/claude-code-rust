import { getErrorMessage } from '../utils/errorHelpers';
import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Monitor, X, Camera, MousePointer, Keyboard, ArrowUpDown,
  Trash2, ChevronDown, ChevronUp, Shield, ShieldAlert, ShieldCheck
} from 'lucide-react';
import { computerUseAPI } from '../utils/tauriAPI';

interface ComputerUsePanelProps {
  onClose: () => void;
}

interface ActionLog {
  id: string;
  type: 'screenshot' | 'click' | 'type' | 'key' | 'scroll';
  description: string;
  timestamp: number;
  success: boolean;
  error?: string;
}

type PermissionMode = 'auto' | 'confirm' | 'disabled';

const ComputerUsePanel: React.FC<ComputerUsePanelProps> = ({ onClose }) => {

  const [enabled, setEnabled] = useState(true);
  const [permissionMode, setPermissionMode] = useState<PermissionMode>('confirm');
  const [screenshotData, setScreenshotData] = useState<string | null>(null);
  const [screenInfo, setScreenInfo] = useState<{ width: number; height: number } | null>(null);
  const [clickCoord, setClickCoord] = useState<{ x: number; y: number } | null>(null);
  const [mouseButton, setMouseButton] = useState<'left' | 'right' | 'middle'>('left');
  const [inputText, setInputText] = useState('');
  const [keyCombo, setKeyCombo] = useState('');
  const [scrollAmount, setScrollAmount] = useState(3);
  const [actionLogs, setActionLogs] = useState<ActionLog[]>([]);
  const [loading, setLoading] = useState(false);
  const [showLogs, setShowLogs] = useState(true);
  const screenshotRef = useRef<HTMLImageElement>(null);

  const addLog = useCallback((log: Omit<ActionLog, 'id' | 'timestamp'>) => {
    setActionLogs(prev => [
      { ...log, id: Date.now().toString() + Math.random(), timestamp: Date.now() },
      ...prev.slice(0, 99),
    ]);
  }, []);

  const takeScreenshot = useCallback(async () => {
    if (!enabled) return;
    setLoading(true);
    try {
      const result = await computerUseAPI.screenshot();
      setScreenshotData(`data:image/jpeg;base64,${result.base64}`);
      setScreenInfo({ width: result.width, height: result.height });
      addLog({ type: 'screenshot', description: `Screenshot ${result.width}x${result.height}`, success: true });
    } catch (e: unknown) {
      addLog({ type: 'screenshot', description: 'Screenshot failed', success: false, error: getErrorMessage(e) || String(e) });
    } finally {
      setLoading(false);
    }
  }, [enabled, addLog]);

  const handleScreenshotClick = useCallback((e: React.MouseEvent<HTMLImageElement>) => {
    if (!screenshotRef.current || !screenInfo) return;
    const rect = screenshotRef.current.getBoundingClientRect();
    const scaleX = screenInfo.width / rect.width;
    const scaleY = screenInfo.height / rect.height;
    const x = Math.round((e.clientX - rect.left) * scaleX);
    const y = Math.round((e.clientY - rect.top) * scaleY);
    setClickCoord({ x, y });
  }, [screenInfo]);

  const doClick = useCallback(async () => {
    if (!clickCoord || !enabled) return;
    if (permissionMode === 'disabled') return;
    setLoading(true);
    try {
      await computerUseAPI.mouseClick(clickCoord.x, clickCoord.y, mouseButton);
      addLog({ type: 'click', description: `${mouseButton} click (${clickCoord.x}, ${clickCoord.y})`, success: true });
    } catch (e: unknown) {
      addLog({ type: 'click', description: `Click failed (${clickCoord.x}, ${clickCoord.y})`, success: false, error: getErrorMessage(e) || String(e) });
    } finally {
      setLoading(false);
    }
  }, [clickCoord, mouseButton, enabled, permissionMode, addLog]);

  const doType = useCallback(async () => {
    if (!inputText || !enabled) return;
    if (permissionMode === 'disabled') return;
    setLoading(true);
    try {
      await computerUseAPI.keyboardType(inputText);
      addLog({ type: 'type', description: `Type: "${inputText.substring(0, 40)}${inputText.length > 40 ? '...' : ''}"`, success: true });
      setInputText('');
    } catch (e: unknown) {
      addLog({ type: 'type', description: 'Type text failed', success: false, error: getErrorMessage(e) || String(e) });
    } finally {
      setLoading(false);
    }
  }, [inputText, enabled, permissionMode, addLog]);

  const doKey = useCallback(async () => {
    if (!keyCombo || !enabled) return;
    if (permissionMode === 'disabled') return;
    setLoading(true);
    try {
      const keys = keyCombo.split('+').map(k => k.trim());
      for (const key of keys) {
        await computerUseAPI.keyboardKey(key);
      }
      addLog({ type: 'key', description: `Key: ${keyCombo}`, success: true });
      setKeyCombo('');
    } catch (e: unknown) {
      addLog({ type: 'key', description: `Key press failed: ${keyCombo}`, success: false, error: getErrorMessage(e) || String(e) });
    } finally {
      setLoading(false);
    }
  }, [keyCombo, enabled, permissionMode, addLog]);

  const doScroll = useCallback(async (direction: 'up' | 'down') => {
    if (!enabled) return;
    if (permissionMode === 'disabled') return;
    setLoading(true);
    try {
      const dy = direction === 'up' ? scrollAmount : -scrollAmount;
      await computerUseAPI.mouseScroll(0, dy);
      addLog({ type: 'scroll', description: `Scroll ${direction} ${scrollAmount}`, success: true });
    } catch (e: unknown) {
      addLog({ type: 'scroll', description: `Scroll failed`, success: false, error: getErrorMessage(e) || String(e) });
    } finally {
      setLoading(false);
    }
  }, [enabled, permissionMode, scrollAmount, addLog]);

  const clearLogs = useCallback(() => setActionLogs([]), []);

  useEffect(() => {
    computerUseAPI.getScreenInfo().then(info => {
      setScreenInfo({ width: info.width, height: info.height });
    }).catch(() => {});
  }, []);

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
  };

  const logTypeIcon = (type: ActionLog['type']) => {
    switch (type) {
      case 'screenshot': return <Camera size={12} />;
      case 'click': return <MousePointer size={12} />;
      case 'type': return <Keyboard size={12} />;
      case 'key': return <Keyboard size={12} />;
      case 'scroll': return <ArrowUpDown size={12} />;
    }
  };

  const permIcon = () => {
    switch (permissionMode) {
      case 'auto': return <ShieldCheck size={14} className="text-green-400" />;
      case 'confirm': return <Shield size={14} className="text-yellow-400" />;
      case 'disabled': return <ShieldAlert size={14} className="text-red-400" />;
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border flex-shrink-0">
        <div className="flex items-center gap-2">
          <Monitor size={16} className="text-blue-400" />
          <span className="text-[14px] font-medium text-claude-text">Computer Use</span>
        </div>
        <div className="flex items-center gap-2">
          <label className="flex items-center gap-1.5 cursor-pointer">
            <span className="text-[11px] text-claude-textSecondary">ON/OFF</span>
            <div
              className={`w-8 h-4 rounded-full relative transition-colors ${enabled ? 'bg-blue-500' : 'bg-gray-600'}`}
              onClick={() => setEnabled(!enabled)}
            >
              <div className={`absolute top-0.5 w-3 h-3 rounded-full bg-white transition-transform ${enabled ? 'translate-x-4' : 'translate-x-0.5'}`} />
            </div>
          </label>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-claude-hover text-claude-textSecondary hover:text-claude-text transition-colors"
          >
            <X size={16} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {!enabled ? (
          <div className="flex flex-col items-center justify-center h-full text-claude-textSecondary">
            <ShieldAlert size={48} className="mb-3 opacity-30" />
            <p className="text-[13px]">Computer Use is disabled</p>
          </div>
        ) : (
          <div className="p-3 space-y-3">
            <div className="border border-claude-border rounded-lg overflow-hidden">
              <div className="flex items-center justify-between px-3 py-2 bg-claude-hover/30">
                <span className="text-[12px] font-medium text-claude-text">Screenshot</span>
                <button
                  onClick={takeScreenshot}
                  disabled={loading}
                  className="flex items-center gap-1 px-2 py-1 text-[11px] rounded bg-blue-500/20 text-blue-400 hover:bg-blue-500/30 disabled:opacity-50 transition-colors"
                >
                  <Camera size={12} />
                  Capture
                </button>
              </div>
              <div
                className="relative bg-black/40 min-h-[160px] flex items-center justify-center cursor-crosshair"
                onClick={handleScreenshotClick}
              >
                {screenshotData ? (
                  <img
                    ref={screenshotRef}
                    src={screenshotData}
                    alt="Screenshot"
                    className="w-full h-auto max-h-[400px] object-contain"
                    draggable={false}
                  />
                ) : (
                  <div className="text-claude-textSecondary text-[12px] flex flex-col items-center gap-2">
                    <Monitor size={32} className="opacity-30" />
                    <span>Click "Capture" to take a screenshot</span>
                  </div>
                )}
                {clickCoord && (
                  <div
                    className="absolute pointer-events-none"
                    style={{
                      left: `${(clickCoord.x / (screenInfo?.width || 1920)) * 100}%`,
                      top: `${(clickCoord.y / (screenInfo?.height || 1080)) * 100}%`,
                      transform: 'translate(-50%, -50%)',
                    }}
                  >
                    <div className="w-4 h-4 border-2 border-red-500 rounded-full animate-ping opacity-75" />
                    <div className="w-4 h-4 border-2 border-red-500 rounded-full absolute top-0 left-0" />
                  </div>
                )}
              </div>
              {clickCoord && (
                <div className="flex items-center justify-between px-3 py-2 bg-claude-hover/20 text-[11px]">
                  <span className="text-claude-textSecondary">
                    Position: ({clickCoord.x}, {clickCoord.y})
                  </span>
                  <div className="flex items-center gap-2">
                    <select
                      value={mouseButton}
                      onChange={e => setMouseButton(e.target.value as any)}
                      className="bg-transparent text-claude-text text-[11px] border border-claude-border rounded px-1 py-0.5"
                    >
                      <option value="left">Left</option>
                      <option value="right">Right</option>
                      <option value="middle">Middle</option>
                    </select>
                    <button
                      onClick={doClick}
                      disabled={loading}
                      className="flex items-center gap-1 px-2 py-0.5 rounded bg-green-500/20 text-green-400 hover:bg-green-500/30 disabled:opacity-50 text-[11px]"
                    >
                      <MousePointer size={11} />
                      Click
                    </button>
                  </div>
                </div>
              )}
            </div>

            <div className="border border-claude-border rounded-lg overflow-hidden">
              <div className="px-3 py-2 bg-claude-hover/30">
                <span className="text-[12px] font-medium text-claude-text">Keyboard</span>
              </div>
              <div className="p-3 space-y-2">
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={inputText}
                    onChange={e => setInputText(e.target.value)}
                    onKeyDown={e => { if (e.key === 'Enter') doType(); }}
                    placeholder="Type text..."
                    className="flex-1 bg-transparent text-claude-text text-[12px] border border-claude-border rounded px-2 py-1.5 placeholder:text-claude-textSecondary/50 focus:outline-none focus:border-blue-500/50"
                  />
                  <button
                    onClick={doType}
                    disabled={loading || !inputText}
                    className="px-3 py-1.5 text-[11px] rounded bg-blue-500/20 text-blue-400 hover:bg-blue-500/30 disabled:opacity-50"
                  >
                    Type
                  </button>
                </div>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={keyCombo}
                    onChange={e => setKeyCombo(e.target.value)}
                    onKeyDown={e => { if (e.key === 'Enter') doKey(); }}
                    placeholder="Key combo (e.g. ctrl+c, enter, alt+tab)"
                    className="flex-1 bg-transparent text-claude-text text-[12px] border border-claude-border rounded px-2 py-1.5 placeholder:text-claude-textSecondary/50 focus:outline-none focus:border-blue-500/50"
                  />
                  <button
                    onClick={doKey}
                    disabled={loading || !keyCombo}
                    className="px-3 py-1.5 text-[11px] rounded bg-purple-500/20 text-purple-400 hover:bg-purple-500/30 disabled:opacity-50"
                  >
                    Press
                  </button>
                </div>
                <div className="flex flex-wrap gap-1">
                  {['enter', 'escape', 'tab', 'backspace', 'delete', 'ctrl+c', 'ctrl+v', 'ctrl+a', 'alt+tab'].map(k => (
                    <button
                      key={k}
                      onClick={() => { setKeyCombo(k); }}
                      className="px-2 py-0.5 text-[10px] rounded bg-claude-hover text-claude-textSecondary hover:text-claude-text transition-colors"
                    >
                      {k}
                    </button>
                  ))}
                </div>
              </div>
            </div>

            <div className="border border-claude-border rounded-lg overflow-hidden">
              <div className="px-3 py-2 bg-claude-hover/30">
                <span className="text-[12px] font-medium text-claude-text">Scroll</span>
              </div>
              <div className="p-3 flex items-center gap-3">
                <button
                  onClick={() => doScroll('up')}
                  disabled={loading}
                  className="flex items-center gap-1 px-3 py-1.5 text-[11px] rounded bg-claude-hover text-claude-text hover:bg-claude-hover/80 disabled:opacity-50"
                >
                  <ArrowUpDown size={12} />
                  Up
                </button>
                <button
                  onClick={() => doScroll('down')}
                  disabled={loading}
                  className="flex items-center gap-1 px-3 py-1.5 text-[11px] rounded bg-claude-hover text-claude-text hover:bg-claude-hover/80 disabled:opacity-50"
                >
                  <ArrowUpDown size={12} className="rotate-180" />
                  Down
                </button>
                <div className="flex items-center gap-1">
                  <span className="text-[11px] text-claude-textSecondary">Amount:</span>
                  <input
                    type="number"
                    value={scrollAmount}
                    onChange={e => setScrollAmount(Math.max(1, parseInt(e.target.value) || 1))}
                    className="w-12 bg-transparent text-claude-text text-[11px] border border-claude-border rounded px-1 py-0.5 text-center focus:outline-none focus:border-blue-500/50"
                  />
                </div>
              </div>
            </div>

            <div className="border border-claude-border rounded-lg overflow-hidden">
              <div
                className="flex items-center justify-between px-3 py-2 bg-claude-hover/30 cursor-pointer"
                onClick={() => setShowLogs(!showLogs)}
              >
                <span className="text-[12px] font-medium text-claude-text">Action Log ({actionLogs.length})</span>
                <div className="flex items-center gap-2">
                  {actionLogs.length > 0 && (
                    <button
                      onClick={e => { e.stopPropagation(); clearLogs(); }}
                      className="p-0.5 rounded hover:bg-claude-hover text-claude-textSecondary hover:text-red-400 transition-colors"
                    >
                      <Trash2 size={12} />
                    </button>
                  )}
                  {showLogs ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                </div>
              </div>
              {showLogs && (
                <div className="max-h-[200px] overflow-y-auto">
                  {actionLogs.length === 0 ? (
                    <div className="px-3 py-4 text-[11px] text-claude-textSecondary text-center">
                      No actions yet
                    </div>
                  ) : (
                    actionLogs.map(log => (
                      <div
                        key={log.id}
                        className={`flex items-start gap-2 px-3 py-1.5 border-t border-claude-border/50 text-[11px] ${
                          log.success ? 'text-claude-textSecondary' : 'text-red-400'
                        }`}
                      >
                        <span className="mt-0.5 flex-shrink-0">{logTypeIcon(log.type)}</span>
                        <span className="flex-1 break-all">{log.description}</span>
                        <span className="text-claude-textSecondary/50 flex-shrink-0">{formatTime(log.timestamp)}</span>
                      </div>
                    ))
                  )}
                </div>
              )}
            </div>

            <div className="border border-claude-border rounded-lg overflow-hidden">
              <div className="px-3 py-2 bg-claude-hover/30">
                <div className="flex items-center gap-1.5">
                  {permIcon()}
                  <span className="text-[12px] font-medium text-claude-text">Permission Mode</span>
                </div>
              </div>
              <div className="p-3">
                <div className="flex gap-2">
                  {([
                    { mode: 'auto' as PermissionMode, label: 'Auto', desc: 'Execute without confirmation', color: 'green' },
                    { mode: 'confirm' as PermissionMode, label: 'Confirm', desc: 'Ask before each action', color: 'yellow' },
                    { mode: 'disabled' as PermissionMode, label: 'Disabled', desc: 'Block all actions', color: 'red' },
                  ]).map(opt => (
                    <button
                      key={opt.mode}
                      onClick={() => setPermissionMode(opt.mode)}
                      className={`flex-1 px-2 py-2 rounded text-[11px] border transition-colors ${
                        permissionMode === opt.mode
                          ? `border-${opt.color}-500/50 bg-${opt.color}-500/10 text-${opt.color}-400`
                          : 'border-claude-border text-claude-textSecondary hover:text-claude-text'
                      }`}
                    >
                      <div className="font-medium">{opt.label}</div>
                      <div className="text-[9px] opacity-70 mt-0.5">{opt.desc}</div>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ComputerUsePanel;
