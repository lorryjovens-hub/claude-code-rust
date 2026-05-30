import React, { useState } from 'react';
import { AlertTriangle, ShieldCheck, ShieldX, ShieldPlus, X } from 'lucide-react';

export interface ApprovalInfo {
  id: string;
  tool_name: string;
  action: string;
  risk_level: string;
  description: string;
}

interface PermissionDialogProps {
  open: boolean;
  approval: ApprovalInfo;
  onApprove: (decision: string, reason: string) => void;
  onReject: (reason: string) => void;
  onAlwaysAllow: () => void;
  onClose: () => void;
}

function getRiskColor(riskLevel: string): { bg: string; text: string; border: string; icon: string } {
  switch (riskLevel) {
    case 'critical':
      return {
        bg: 'bg-red-500/15',
        text: 'text-red-500',
        border: 'border-red-500/30',
        icon: 'text-red-500',
      };
    case 'high':
      return {
        bg: 'bg-orange-500/15',
        text: 'text-orange-500',
        border: 'border-orange-500/30',
        icon: 'text-orange-500',
      };
    case 'medium':
      return {
        bg: 'bg-yellow-500/15',
        text: 'text-yellow-600',
        border: 'border-yellow-500/30',
        icon: 'text-yellow-500',
      };
    case 'low':
      return {
        bg: 'bg-green-500/15',
        text: 'text-green-500',
        border: 'border-green-500/30',
        icon: 'text-green-500',
      };
    default:
      return {
        bg: 'bg-claude-textSecondary/15',
        text: 'text-claude-textSecondary',
        border: 'border-claude-border',
        icon: 'text-claude-textSecondary',
      };
  }
}

function getRiskLabel(riskLevel: string): string {
  switch (riskLevel) {
    case 'critical': return '\u6781\u9ad8\u98ce\u9669';
    case 'high': return '\u9ad8\u98ce\u9669';
    case 'medium': return '\u4e2d\u7b49\u98ce\u9669';
    case 'low': return '\u4f4e\u98ce\u9669';
    default: return riskLevel;
  }
}

export default function PermissionDialog({
  open,
  approval,
  onApprove,
  onReject,
  onAlwaysAllow,
  onClose,
}: PermissionDialogProps) {
  const [rejectReason, setRejectReason] = useState('');
  const [showRejectInput, setShowRejectInput] = useState(false);
  const [showAlwaysAllowConfirm, setShowAlwaysAllowConfirm] = useState(false);
  const [approveReason, setApproveReason] = useState('');

  if (!open) return null;

  const riskColors = getRiskColor(approval.risk_level);
  const riskLabel = getRiskLabel(approval.risk_level);

  const handleApprove = () => {
    onApprove('approved', approveReason);
    setApproveReason('');
  };

  const handleReject = () => {
    if (showRejectInput) {
      onReject(rejectReason);
      setRejectReason('');
      setShowRejectInput(false);
    } else {
      setShowRejectInput(true);
    }
  };

  const handleAlwaysAllowClick = () => {
    if (!showAlwaysAllowConfirm) {
      setShowAlwaysAllowConfirm(true);
    } else {
      onAlwaysAllow();
      setShowAlwaysAllowConfirm(false);
    }
  };

  const handleClose = () => {
    setRejectReason('');
    setShowRejectInput(false);
    setShowAlwaysAllowConfirm(false);
    setApproveReason('');
    onClose();
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      handleClose();
    }
  };

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50"
      onClick={handleOverlayClick}
    >
      <div
        className="bg-claude-input rounded-2xl shadow-xl w-[460px] animate-fade-in border border-claude-border"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-5 pb-3">
          <div className="flex items-center gap-3">
            <div className={`w-10 h-10 rounded-full ${riskColors.bg} flex items-center justify-center flex-shrink-0`}>
              <AlertTriangle size={20} className={riskColors.icon} />
            </div>
            <div>
              <h3 className="text-[16px] font-semibold text-claude-text">
                {'\u6743\u9650\u5ba1\u6279'}
              </h3>
              <p className="text-[13px] text-claude-textSecondary mt-0.5">
                Permission Approval Required
              </p>
            </div>
          </div>
          <button
            onClick={handleClose}
            className="w-8 h-8 rounded-lg flex items-center justify-center hover:bg-claude-hover transition-colors"
          >
            <X size={16} className="text-claude-textSecondary" />
          </button>
        </div>

        <div className="px-5 pb-5">
          <div className={`rounded-xl border ${riskColors.border} ${riskColors.bg} p-4 mb-4`}>
            <div className="flex items-center gap-2 mb-3">
              <span className={`text-[11px] font-semibold uppercase tracking-wider ${riskColors.text}`}>
                {riskLabel}
              </span>
            </div>

            <div className="space-y-2.5">
              <div className="flex items-center justify-between">
                <span className="text-[12px] text-claude-textSecondary">{'\u5de5\u5177\u540d\u79f0'}</span>
                <span className="text-[13px] font-medium text-claude-text font-mono bg-claude-hover px-2 py-0.5 rounded">
                  {approval.tool_name}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-[12px] text-claude-textSecondary">{'\u64cd\u4f5c\u7c7b\u578b'}</span>
                <span className="text-[13px] font-medium text-claude-text font-mono bg-claude-hover px-2 py-0.5 rounded">
                  {approval.action}
                </span>
              </div>
              <div>
                <span className="text-[12px] text-claude-textSecondary">{'\u63cf\u8ff0'}</span>
                <p className="text-[13px] text-claude-text mt-1 leading-relaxed">
                  {approval.description}
                </p>
              </div>
            </div>
          </div>

          <div className="p-3 rounded-lg bg-claude-hover/50 mb-4">
            <p className="text-[12px] text-claude-textSecondary leading-relaxed">
              {'\u8be5\u64cd\u4f5c\u5c5e\u4e8e\u6f5c\u5728\u98ce\u9669\u64cd\u4f5c\uff0c\u8bf7\u786e\u8ba4\u662f\u5426\u5141\u8bb8\u6267\u884c\u3002'}
            </p>
          </div>

          <div className="flex items-center gap-2.5">
            <button
              onClick={handleApprove}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 text-[13px] font-medium text-white bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
            >
              <ShieldCheck size={16} />
              {'\u5141\u8bb8'}
            </button>

            <button
              onClick={handleAlwaysAllowClick}
              className={`flex items-center justify-center gap-2 px-4 py-2.5 text-[13px] font-medium rounded-lg transition-colors ${
                showAlwaysAllowConfirm
                  ? 'bg-blue-600 text-white hover:bg-blue-700'
                  : 'text-blue-600 border border-blue-600/40 bg-blue-500/10 hover:bg-blue-500/20'
              }`}
            >
              <ShieldPlus size={16} />
              {showAlwaysAllowConfirm ? '\u786e\u8ba4\u59cb\u7ec8\u5141\u8bb8' : '\u59cb\u7ec8\u5141\u8bb8'}
            </button>

            <button
              onClick={handleReject}
              className="flex items-center justify-center gap-2 px-4 py-2.5 text-[13px] font-medium text-red-500 border border-red-500/30 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors"
            >
              <ShieldX size={16} />
              {showRejectInput ? '\u786e\u8ba4\u62d2\u7edd' : '\u62d2\u7edd'}
            </button>
          </div>

          {showAlwaysAllowConfirm && (
            <div className="mt-3 p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
              <p className="text-[12px] text-blue-400 leading-relaxed">
                {'\u59cb\u7ec8\u5141\u8bb8\u540e\uff0c\u8be5\u5de5\u5177\u7684\u540c\u7c7b\u64cd\u4f5c\u5c06\u4e0d\u518d\u9700\u8981\u786e\u8ba4\uff0c\u76f4\u5230\u89c4\u5219\u88ab\u79fb\u9664\u3002'}
              </p>
            </div>
          )}

          {showRejectInput && (
            <div className="mt-3">
              <textarea
                value={rejectReason}
                onChange={(e) => setRejectReason(e.target.value)}
                placeholder={'\u62d2\u7edd\u539f\u56e0\uff08\u53ef\u9009\uff09...'}
                className="w-full px-3 py-2 text-[13px] bg-claude-hover border border-claude-border rounded-lg resize-none text-claude-text placeholder:text-claude-textSecondary/50 focus:outline-none focus:border-red-500/50"
                rows={2}
                autoFocus
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}