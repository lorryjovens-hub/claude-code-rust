import React, { useState, useEffect } from 'react';
import {
  BarChart3,
  TrendingUp,
  Layers,
  Target,
  Monitor,
  Smartphone,
  Zap,
  Brain,
  Loader2,
  Sparkles,
  Clock,
  Shield,
} from 'lucide-react';
import { skillService, type DesignStats } from '../../services/previewService';

const MODE_LABELS: Record<string, string> = {
  prototype: '原型',
  deck: '幻灯片',
  image: '图像',
  video: '视频',
  audio: '音频',
  review: '评审',
};

const MODE_COLORS: Record<string, string> = {
  prototype: '#8B5CF6',
  deck: '#F59E0B',
  image: '#10B981',
  video: '#EF4444',
  audio: '#06B6D4',
  review: '#3B82F6',
};

const SCENARIO_LABELS: Record<string, string> = {
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

const SCENARIO_COLORS: Record<string, string> = {
  design: '#8B5CF6',
  marketing: '#F59E0B',
  engineering: '#3B82F6',
  product: '#10B981',
  finance: '#06B6D4',
  hr: '#EC4899',
  sale: '#EF4444',
  personal: '#6366F1',
  operation: '#F97316',
};

const MOCK_STATS: DesignStats = {
  total_design_skills: 10,
  featured_skills: 3,
  by_mode: { prototype: 6, deck: 1, image: 1, video: 1, review: 1 },
  by_scenario: { design: 1, marketing: 2, engineering: 2, product: 1, finance: 1, hr: 1, sale: 1, personal: 1 },
  by_fidelity: { high: 5, medium: 3 },
  by_platform: { desktop: 8, mobile: 2 },
  today_usage: { today_messages: 42, today_conversations: 8, today_tokens: 156800, date: new Date().toISOString().slice(0, 10) },
  caveman: { total_segments: 128, tokens_saved: 24150, total_tokens_processed: 98500, avg_compression_ratio: 0.245 },
};

interface SkillStatsPanelProps {
  totalSkills: number;
}

const SkillStatsPanel: React.FC<SkillStatsPanelProps> = ({ totalSkills }) => {
  const [stats, setStats] = useState<DesignStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [usingMock, setUsingMock] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      setLoading(true);
      try {
        const result = await skillService.getDesignStats();
        if (!cancelled) {
          if (result.total_design_skills > 0 || Object.keys(result.by_mode).length > 0) {
            setStats(result);
            setUsingMock(false);
          } else {
            setStats(MOCK_STATS);
            setUsingMock(true);
          }
          setLoading(false);
        }
      } catch {
        if (!cancelled) {
          setStats(MOCK_STATS);
          setUsingMock(true);
          setLoading(false);
        }
      }
    };
    load();
    return () => { cancelled = true; };
  }, []);

  const displayStats = stats || MOCK_STATS;

  if (loading) {
    return (
      <div className="w-[260px] flex-shrink-0 border-r border-claude-border bg-claude-surface/30 flex items-center justify-center">
        <div className="flex flex-col items-center gap-2 text-claude-textSecondary">
          <Loader2 size={20} className="animate-spin opacity-40" />
          <span className="text-[11px]">加载统计...</span>
        </div>
      </div>
    );
  }

  const maxModeValue = Math.max(1, ...Object.values(displayStats.by_mode));

  return (
    <div className="w-[260px] flex-shrink-0 border-r border-claude-border bg-claude-surface/20 flex flex-col overflow-y-auto">
      <div className="px-3.5 pt-3 pb-2">
        <div className="flex items-center gap-2 mb-2">
          <BarChart3 size={14} className="text-[#8B5CF6]" />
          <span className="text-[12px] font-semibold text-claude-text">设计统计</span>
          {usingMock && (
            <span className="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20">
              MOCK
            </span>
          )}
        </div>
      </div>

      <div className="px-3.5 pb-3 space-y-3">
        <div className="grid grid-cols-2 gap-1.5">
          <div className="bg-claude-hover rounded-lg p-2.5">
            <div className="flex items-center gap-1 mb-0.5">
              <Layers size={11} className="text-[#8B5CF6]" />
              <span className="text-[10px] text-claude-textSecondary">技能总数</span>
            </div>
            <p className="text-[16px] font-semibold text-claude-text">{displayStats.total_design_skills}</p>
          </div>
          <div className="bg-claude-hover rounded-lg p-2.5">
            <div className="flex items-center gap-1 mb-0.5">
              <Sparkles size={11} className="text-amber-400" />
              <span className="text-[10px] text-claude-textSecondary">推荐</span>
            </div>
            <p className="text-[16px] font-semibold text-claude-text">{displayStats.featured_skills}</p>
          </div>
        </div>

        <div className="bg-claude-hover rounded-lg p-2.5 space-y-1.5">
          <div className="flex items-center gap-1.5 mb-0.5">
            <Zap size={11} className="text-[#10B981]" />
            <span className="text-[10px] font-medium text-claude-textSecondary">今日用量</span>
            <span className="text-[9px] text-claude-textSecondary/60 ml-auto">{displayStats.today_usage.date}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-[10px] text-claude-textSecondary">消息</span>
            <span className="text-[12px] font-medium text-claude-text">{displayStats.today_usage.today_messages}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-[10px] text-claude-textSecondary">会话</span>
            <span className="text-[12px] font-medium text-claude-text">{displayStats.today_usage.today_conversations}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-[10px] text-claude-textSecondary">Token</span>
            <span className="text-[12px] font-medium text-claude-text">{displayStats.today_usage.today_tokens.toLocaleString()}</span>
          </div>
        </div>

        <div>
          <div className="flex items-center gap-1.5 mb-2">
            <Target size={11} className="text-[#8B5CF6]" />
            <span className="text-[10px] font-medium text-claude-textSecondary">按模式</span>
          </div>
          <div className="space-y-1">
            {Object.entries(displayStats.by_mode)
              .sort(([, a], [, b]) => b - a)
              .map(([mode, count]) => (
                <div key={mode} className="flex items-center gap-1.5">
                  <span className="text-[10px] text-claude-textSecondary w-12 text-right flex-shrink-0">
                    {MODE_LABELS[mode] || mode}
                  </span>
                  <div className="flex-1 h-3.5 bg-claude-hover rounded-full overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all duration-500"
                      style={{
                        width: `${(count / maxModeValue) * 100}%`,
                        backgroundColor: MODE_COLORS[mode] || '#8B5CF6',
                      }}
                    />
                  </div>
                  <span className="text-[10px] font-medium text-claude-text w-4 text-right flex-shrink-0">{count}</span>
                </div>
              ))}
          </div>
        </div>

        <div>
          <div className="flex items-center gap-1.5 mb-2">
            <TrendingUp size={11} className="text-[#10B981]" />
            <span className="text-[10px] font-medium text-claude-textSecondary">按场景</span>
          </div>
          <div className="flex flex-wrap gap-1">
            {Object.entries(displayStats.by_scenario)
              .sort(([, a], [, b]) => b - a)
              .map(([scenario, count]) => (
                <span
                  key={scenario}
                  className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[9px] font-medium"
                  style={{
                    backgroundColor: (SCENARIO_COLORS[scenario] || '#8B5CF6') + '18',
                    color: SCENARIO_COLORS[scenario] || '#8B5CF6',
                  }}
                >
                  {SCENARIO_LABELS[scenario] || scenario}
                  <span className="opacity-60">{count}</span>
                </span>
              ))}
          </div>
        </div>

        <div>
          <div className="flex items-center gap-1.5 mb-2">
            <Brain size={11} className="text-[#06B6D4]" />
            <span className="text-[10px] font-medium text-claude-textSecondary">Caveman RTK</span>
          </div>
          <div className="grid grid-cols-2 gap-1.5">
            <div className="bg-claude-hover rounded-lg p-2">
              <span className="text-[9px] text-claude-textSecondary">记忆片段</span>
              <p className="text-[13px] font-semibold text-claude-text">{displayStats.caveman.total_segments}</p>
            </div>
            <div className="bg-claude-hover rounded-lg p-2">
              <span className="text-[9px] text-claude-textSecondary">节省 Token</span>
              <p className="text-[13px] font-semibold text-[#10B981]">{displayStats.caveman.tokens_saved.toLocaleString()}</p>
            </div>
            <div className="bg-claude-hover rounded-lg p-2 col-span-2">
              <div className="flex items-center justify-between">
                <span className="text-[9px] text-claude-textSecondary">处理 / 压缩比</span>
                <div className="flex items-center gap-2">
                  <span className="text-[11px] text-claude-text">{displayStats.caveman.total_tokens_processed.toLocaleString()}</span>
                  <span className="text-[9px] px-1 py-0.5 rounded bg-[#06B6D4]/10 text-[#06B6D4]">
                    {(displayStats.caveman.avg_compression_ratio * 100).toFixed(1)}%
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div>
          <div className="flex items-center gap-1.5 mb-2">
            <Monitor size={11} className="text-claude-textSecondary" />
            <span className="text-[10px] font-medium text-claude-textSecondary">平台</span>
          </div>
          <div className="flex items-center gap-3">
            {Object.entries(displayStats.by_platform).map(([platform, count]) => (
              <div key={platform} className="flex items-center gap-1">
                {platform === 'mobile' ? (
                  <Smartphone size={11} className="text-claude-textSecondary" />
                ) : (
                  <Monitor size={11} className="text-claude-textSecondary" />
                )}
                <span className="text-[10px] text-claude-textSecondary">
                  {platform === 'mobile' ? '移动' : '桌面'}:
                </span>
                <span className="text-[11px] font-medium text-claude-text">{count}</span>
              </div>
            ))}
          </div>
        </div>

        {usingMock && (
          <div className="flex items-center gap-1.5 text-[9px] text-amber-400/70">
            <Shield size={10} />
            <span>使用本地模拟数据</span>
          </div>
        )}
      </div>
    </div>
  );
};

export default SkillStatsPanel;
