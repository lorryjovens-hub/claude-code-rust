import React from 'react';
import {
  Monitor,
  Smartphone,
  FileText,
  Video,
  Music,
  Palette,
  Eye,
  Zap,
  Star,
  ArrowRight,
} from 'lucide-react';
import type { SkillInfo } from '../../services/previewService';

interface SkillCardProps {
  skill: SkillInfo;
  isSelected: boolean;
  onSelect: (skill: SkillInfo) => void;
  onViewDetail: (skill: SkillInfo) => void;
}

const MODE_CONFIG: Record<string, { icon: React.ElementType; label: string; color: string }> = {
  prototype: { icon: Monitor, label: '原型', color: '#8B5CF6' },
  deck: { icon: FileText, label: '幻灯片', color: '#3B82F6' },
  image: { icon: Palette, label: '图像', color: '#EC4899' },
  video: { icon: Video, label: '视频', color: '#F59E0B' },
  audio: { icon: Music, label: '音频', color: '#10B981' },
  review: { icon: Eye, label: '评审', color: '#EF4444' },
};

const FIDELITY_CONFIG: Record<string, string> = {
  'high-fidelity': '高保真',
  'medium-fidelity': '中保真',
  'low-fidelity': '低保真',
};

const SCENARIO_CONFIG: Record<string, string> = {
  design: '设计',
  marketing: '营销',
  operation: '运营',
  engineering: '工程',
  product: '产品',
  finance: '金融',
  hr: '人力资源',
  sale: '销售',
  personal: '个人',
};

const SkillCard: React.FC<SkillCardProps> = ({ skill, isSelected, onSelect, onViewDetail }) => {
  const od = skill.od_metadata;
  const mode = od?.mode || 'prototype';
  const modeConf = MODE_CONFIG[mode] || MODE_CONFIG.prototype;
  const ModeIcon = modeConf.icon;

  return (
    <div
      className={`relative bg-claude-input border rounded-xl transition-all duration-200 cursor-pointer group overflow-hidden ${
        isSelected
          ? 'border-[#8B5CF6] ring-1 ring-[#8B5CF6]/30 shadow-lg shadow-[#8B5CF6]/10'
          : 'border-claude-border hover:border-claude-textSecondary/40 hover:shadow-md'
      }`}
      onClick={() => onSelect(skill)}
    >
      {od?.featured && od.featured >= 1 && (
        <div className="absolute top-2 right-2 flex items-center gap-1 px-2 py-0.5 bg-amber-500/10 border border-amber-500/20 rounded-full">
          <Star size={10} className="text-amber-400 fill-amber-400" />
          <span className="text-[10px] font-semibold text-amber-400">推荐</span>
        </div>
      )}

      <div className="p-4">
        <div className="flex items-start gap-3 mb-3">
          <div
            className="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0"
            style={{ backgroundColor: modeConf.color + '20' }}
          >
            <ModeIcon size={20} style={{ color: modeConf.color }} />
          </div>
          <div className="min-w-0 flex-1">
            <h3 className="text-[14px] font-semibold text-claude-text truncate">
              {skill.name}
            </h3>
            <p className="text-[12px] text-claude-textSecondary mt-0.5 line-clamp-2 leading-relaxed">
              {skill.description}
            </p>
          </div>
        </div>

        <div className="flex flex-wrap gap-1.5 mb-3">
          <span
            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium"
            style={{ backgroundColor: modeConf.color + '15', color: modeConf.color }}
          >
            {modeConf.label}
          </span>
          {od?.fidelity && FIDELITY_CONFIG[od.fidelity] && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium bg-claude-hover text-claude-textSecondary">
              <Zap size={10} />
              {FIDELITY_CONFIG[od.fidelity]}
            </span>
          )}
          {od?.platform && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium bg-claude-hover text-claude-textSecondary">
              {od.platform === 'mobile' ? <Smartphone size={10} /> : <Monitor size={10} />}
              {od.platform === 'mobile' ? '移动端' : '桌面端'}
            </span>
          )}
          {od?.scenario && SCENARIO_CONFIG[od.scenario] && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium bg-claude-hover text-claude-textSecondary">
              {SCENARIO_CONFIG[od.scenario]}
            </span>
          )}
        </div>

        <div className="flex items-center justify-between pt-2 border-t border-claude-border">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onViewDetail(skill);
            }}
            className="text-[11px] font-medium text-claude-textSecondary hover:text-claude-text transition-colors"
          >
            查看详情
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onSelect(skill);
            }}
            className={`flex items-center gap-1 px-3 py-1.5 rounded-lg text-[12px] font-medium transition-all ${
              isSelected
                ? 'bg-[#8B5CF6] text-white'
                : 'bg-claude-text text-claude-bg hover:opacity-80'
            }`}
          >
            {isSelected ? '已选择' : '使用此技能'}
            <ArrowRight size={12} />
          </button>
        </div>
      </div>
    </div>
  );
};

export default SkillCard;