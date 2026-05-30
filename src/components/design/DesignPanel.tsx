import React, { useState, useRef, useCallback } from 'react';
import {
  Box, Type, Palette, Layout, Plus, Trash2, Save,
  Download, Upload, Eye, RefreshCw, Layers, Copy, Check,
  RotateCcw
} from 'lucide-react';
import type { DesignCanvasRef } from './DesignCanvas';

interface DesignPanelProps {
  canvasRef: React.RefObject<DesignCanvasRef | null>;
  onSave: (name: string, content: string) => void;
  onApply: (content: string) => void;
  savedDesigns: SavedDesign[];
  onLoadDesign: (design: SavedDesign) => void;
  onDeleteDesign: (id: string) => void;
}

export interface SavedDesign {
  id: string;
  name: string;
  content: string;
  createdAt: number;
  thumbnail?: string;
}

type PanelTab = 'components' | 'styles' | 'layout' | 'settings';

const COMPONENT_CATEGORIES = [
  {
    name: '基础组件',
    items: [
      { id: 'button', label: '按钮', icon: '▢' },
      { id: 'input', label: '输入框', icon: '✎' },
      { id: 'heading', label: '标题', icon: 'H' },
      { id: 'paragraph', label: '段落', icon: '¶' },
      { id: 'divider', label: '分割线', icon: '—' },
    ],
  },
  {
    name: '复合组件',
    items: [
      { id: 'card', label: '卡片', icon: '▣' },
      { id: 'badge', label: '徽章', icon: '◉' },
      { id: 'avatar', label: '头像', icon: '◎' },
      { id: 'image', label: '图片', icon: '▨' },
      { id: 'grid', label: '网格', icon: '⊞' },
    ],
  },
  {
    name: '页面区域',
    items: [
      { id: 'navbar', label: '导航栏', icon: '≡' },
      { id: 'hero', label: 'Hero区域', icon: '★' },
    ],
  },
];

const STYLE_PRESETS = [
  { id: 'modern', name: '现代简约', colors: { primary: '#3B82F6', background: '#ffffff', text: '#1f2937' }, typography: { fontFamily: '-apple-system, BlinkMacSystemFont, sans-serif', fontSize: 14, headingSize: 24 } },
  { id: 'elegant', name: '优雅暗色', colors: { primary: '#8B5CF6', background: '#1a1a2e', text: '#e5e7eb' }, typography: { fontFamily: 'Georgia, serif', fontSize: 15, headingSize: 28 } },
  { id: 'minimal', name: '极简留白', colors: { primary: '#111827', background: '#ffffff', text: '#374151' }, typography: { fontFamily: '-apple-system, sans-serif', fontSize: 16, headingSize: 20 } },
  { id: 'playful', name: '活泼色彩', colors: { primary: '#EC4899', background: '#fdf2f8', text: '#831843' }, typography: { fontFamily: "'Segoe UI', sans-serif", fontSize: 14, headingSize: 26 } },
  { id: 'corporate', name: '企业专业', colors: { primary: '#2563EB', background: '#f8fafc', text: '#334155' }, typography: { fontFamily: 'Arial, sans-serif', fontSize: 14, headingSize: 22 } },
];

const LAYOUT_OPTIONS = [
  { id: 'flex_row', name: 'Flex 横向', description: '水平排列元素' },
  { id: 'flex_col', name: 'Flex 纵向', description: '垂直堆叠元素' },
  { id: 'grid_2', name: '双列网格', description: '2列等宽布局' },
  { id: 'grid_3', name: '三列网格', description: '3列等宽布局' },
  { id: 'centered', name: '居中布局', description: '内容水平垂直居中' },
  { id: 'sidebar', name: '侧边栏布局', description: '左侧边栏 + 主内容区' },
];

const FONT_FAMILIES = [
  '-apple-system, BlinkMacSystemFont, sans-serif',
  'Georgia, "Times New Roman", serif',
  "'Segoe UI', Roboto, sans-serif",
  'Arial, Helvetica, sans-serif',
  "'Courier New', monospace",
];

const DesignPanel: React.FC<DesignPanelProps> = ({
  canvasRef,
  onSave,
  onApply,
  savedDesigns,
  onLoadDesign,
  onDeleteDesign,
}) => {
  const [activeTab, setActiveTab] = useState<PanelTab>('components');
  const [copied, setCopied] = useState(false);
  const [saveName, setSaveName] = useState('');
  const [showSaveDialog, setShowSaveDialog] = useState(false);

  const [styleConfig, setStyleConfig] = useState({
    colors: { primary: '#3B82F6', background: '#ffffff', text: '#1f2937' },
    typography: { fontFamily: '-apple-system, BlinkMacSystemFont, sans-serif', fontSize: 14, headingSize: 24 },
    spacing: { padding: 24 },
    borderRadius: 8,
    shadow: { x: 0, y: 1, blur: 3, spread: 0, color: 'rgba(0,0,0,0.1)' },
  });

  const sendCanvas = useCallback((type: string, payload: any) => {
    canvasRef.current?.sendMessage({ type, payload } as any);
  }, [canvasRef]);

  const insertComponent = (componentId: string) => {
    sendCanvas('insert_component', { name: componentId });
  };

  const applyStylePreset = (presetId: string) => {
    const preset = STYLE_PRESETS.find(p => p.id === presetId);
    if (!preset) return;
    setStyleConfig(prev => ({ ...prev, ...preset }));
    sendCanvas('update_style', preset);
  };

  const applyLayout = (layoutId: string) => {
    sendCanvas('apply_layout', { name: layoutId });
  };

  const updateColor = (key: string, value: string) => {
    setStyleConfig(prev => {
      const next = { ...prev, colors: { ...prev.colors, [key]: value } };
      sendCanvas('update_style', { colors: next.colors });
      return next;
    });
  };

  const updateTypography = (key: string, value: string | number) => {
    setStyleConfig(prev => {
      const next = { ...prev, typography: { ...prev.typography, [key]: value } };
      sendCanvas('update_style', { typography: next.typography });
      return next;
    });
  };

  const updateSpacing = (value: number) => {
    setStyleConfig(prev => {
      const next = { ...prev, spacing: { padding: value } };
      sendCanvas('update_style', { spacing: next.spacing });
      return next;
    });
  };

  const updateBorderRadius = (value: number) => {
    setStyleConfig(prev => {
      const next = { ...prev, borderRadius: value };
      sendCanvas('update_style', { borderRadius: value });
      return next;
    });
  };

  const resetCanvas = () => {
    sendCanvas('reset', {});
    setStyleConfig({
      colors: { primary: '#3B82F6', background: '#ffffff', text: '#1f2937' },
      typography: { fontFamily: '-apple-system, BlinkMacSystemFont, sans-serif', fontSize: 14, headingSize: 24 },
      spacing: { padding: 24 },
      borderRadius: 8,
      shadow: { x: 0, y: 1, blur: 3, spread: 0, color: 'rgba(0,0,0,0.1)' },
    });
  };

  const handleCopyState = () => {
    if (canvasRef.current) {
      const content = canvasRef.current.getPreviewContent();
      navigator.clipboard.writeText(JSON.stringify({ styleConfig, content }));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleSave = () => {
    if (!saveName.trim() || !canvasRef.current) return;
    const content = canvasRef.current.getPreviewContent();
    onSave(saveName.trim(), content);
    setSaveName('');
    setShowSaveDialog(false);
  };

  const tabs = [
    { id: 'components' as PanelTab, label: '组件', icon: Box },
    { id: 'styles' as PanelTab, label: '样式', icon: Palette },
    { id: 'layout' as PanelTab, label: '布局', icon: Layout },
    { id: 'settings' as PanelTab, label: '方案', icon: Save },
  ];

  return (
    <div className="flex flex-col h-full bg-claude-surface border-r border-claude-border">
      <div className="flex-shrink-0 px-3 py-2 border-b border-claude-border">
        <h3 className="text-[13px] font-semibold text-claude-text flex items-center gap-2">
          <Layers size={14} />
          Design Panel
        </h3>
      </div>

      <div className="flex border-b border-claude-border flex-shrink-0">
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 flex items-center justify-center gap-1.5 py-2 text-[12px] font-medium transition-colors ${
              activeTab === tab.id
                ? 'text-claude-accent border-b-2 border-claude-accent -mb-[1px]'
                : 'text-claude-textSecondary hover:text-claude-text'
            }`}
          >
            <tab.icon size={13} />
            {tab.label}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto">
        {activeTab === 'components' && (
          <div className="p-3 space-y-4">
            {COMPONENT_CATEGORIES.map(cat => (
              <div key={cat.name}>
                <h4 className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider mb-2">
                  {cat.name}
                </h4>
                <div className="grid grid-cols-2 gap-1.5">
                  {cat.items.map(item => (
                    <button
                      key={item.id}
                      onClick={() => insertComponent(item.id)}
                      className="flex items-center gap-2 px-3 py-2 rounded-md text-[12px] text-claude-text bg-claude-hover/50 hover:bg-claude-hover transition-colors border border-claude-border/50"
                    >
                      <span className="text-[14px] w-5 text-center">{item.icon}</span>
                      {item.label}
                    </button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        {activeTab === 'styles' && (
          <div className="p-3 space-y-4">
            <div>
              <h4 className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider mb-2">
                风格预设
              </h4>
              <div className="space-y-1">
                {STYLE_PRESETS.map(preset => (
                  <button
                    key={preset.id}
                    onClick={() => applyStylePreset(preset.id)}
                    className="w-full flex items-center gap-3 px-3 py-2 rounded-md text-[12px] hover:bg-claude-hover transition-colors"
                  >
                    <div
                      className="w-6 h-6 rounded-full border border-claude-border flex-shrink-0"
                      style={{ background: `linear-gradient(135deg, ${preset.colors.primary}, ${preset.colors.background})` }}
                    />
                    <span className="text-claude-text">{preset.name}</span>
                  </button>
                ))}
              </div>
            </div>

            <div>
              <h4 className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider mb-2">
                自定义颜色
              </h4>
              <div className="space-y-2">
                {[
                  { key: 'primary', label: '主色调' },
                  { key: 'background', label: '背景色' },
                  { key: 'text', label: '文字色' },
                ].map(({ key, label }) => (
                  <div key={key} className="flex items-center gap-2">
                    <label className="text-[12px] text-claude-textSecondary w-16 flex-shrink-0">{label}</label>
                    <div className="relative flex-1">
                      <input
                        type="color"
                        value={styleConfig.colors[key as keyof typeof styleConfig.colors]}
                        onChange={e => updateColor(key, e.target.value)}
                        className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                      />
                      <div className="flex items-center gap-2 px-2 py-1 rounded border border-claude-border bg-claude-bg">
                        <div
                          className="w-4 h-4 rounded-full border border-gray-300"
                          style={{ backgroundColor: styleConfig.colors[key as keyof typeof styleConfig.colors] }}
                        />
                        <span className="text-[11px] text-claude-textSecondary font-mono">
                          {styleConfig.colors[key as keyof typeof styleConfig.colors]}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div>
              <h4 className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider mb-2">
                排版
              </h4>
              <div className="space-y-2">
                <div>
                  <label className="text-[11px] text-claude-textSecondary block mb-1">字体</label>
                  <select
                    value={styleConfig.typography.fontFamily}
                    onChange={e => updateTypography('fontFamily', e.target.value)}
                    className="w-full px-2 py-1.5 text-[12px] rounded border border-claude-border bg-claude-bg text-claude-text"
                  >
                    {FONT_FAMILIES.map(f => (
                      <option key={f} value={f}>{f.split(',')[0].replace(/'/g, '')}</option>
                    ))}
                  </select>
                </div>
                <div className="flex gap-2">
                  <div className="flex-1">
                    <label className="text-[11px] text-claude-textSecondary block mb-1">正文字号</label>
                    <input
                      type="number"
                      value={styleConfig.typography.fontSize}
                      onChange={e => updateTypography('fontSize', Number(e.target.value))}
                      className="w-full px-2 py-1.5 text-[12px] rounded border border-claude-border bg-claude-bg text-claude-text"
                      min={10}
                      max={24}
                    />
                  </div>
                  <div className="flex-1">
                    <label className="text-[11px] text-claude-textSecondary block mb-1">标题字号</label>
                    <input
                      type="number"
                      value={styleConfig.typography.headingSize}
                      onChange={e => updateTypography('headingSize', Number(e.target.value))}
                      className="w-full px-2 py-1.5 text-[12px] rounded border border-claude-border bg-claude-bg text-claude-text"
                      min={14}
                      max={48}
                    />
                  </div>
                </div>
              </div>
            </div>

            <div>
              <h4 className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider mb-2">
                间距与圆角
              </h4>
              <div className="space-y-2">
                <div>
                  <label className="text-[11px] text-claude-textSecondary block mb-1">
                    内边距: {styleConfig.spacing.padding}px
                  </label>
                  <input
                    type="range"
                    value={styleConfig.spacing.padding}
                    onChange={e => updateSpacing(Number(e.target.value))}
                    className="w-full"
                    min={0}
                    max={80}
                  />
                </div>
                <div>
                  <label className="text-[11px] text-claude-textSecondary block mb-1">
                    圆角: {styleConfig.borderRadius}px
                  </label>
                  <input
                    type="range"
                    value={styleConfig.borderRadius}
                    onChange={e => updateBorderRadius(Number(e.target.value))}
                    className="w-full"
                    min={0}
                    max={24}
                  />
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'layout' && (
          <div className="p-3 space-y-3">
            <h4 className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider mb-2">
              布局模板
            </h4>
            <div className="space-y-1.5">
              {LAYOUT_OPTIONS.map(lo => (
                <button
                  key={lo.id}
                  onClick={() => applyLayout(lo.id)}
                  className="w-full text-left px-3 py-2.5 rounded-md hover:bg-claude-hover transition-colors border border-claude-border/50"
                >
                  <div className="text-[12px] font-medium text-claude-text">{lo.name}</div>
                  <div className="text-[11px] text-claude-textSecondary mt-0.5">{lo.description}</div>
                </button>
              ))}
            </div>

            <div className="pt-3 border-t border-claude-border">
              <button
                onClick={resetCanvas}
                className="w-full flex items-center justify-center gap-2 px-3 py-2 text-[12px] text-red-500 hover:bg-red-50 rounded-md transition-colors"
              >
                <RotateCcw size={13} />
                重置画布
              </button>
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div className="p-3 space-y-4">
            <div className="flex gap-2">
              <button
                onClick={handleCopyState}
                className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-[12px] font-medium bg-claude-hover text-claude-text rounded-md hover:bg-claude-border transition-colors"
              >
                {copied ? <Check size={13} className="text-green-500" /> : <Copy size={13} />}
                {copied ? '已复制' : '复制状态'}
              </button>
              <button
                onClick={() => canvasRef.current?.reloadPreview()}
                className="flex items-center justify-center gap-1.5 px-3 py-2 text-[12px] font-medium bg-claude-hover text-claude-text rounded-md hover:bg-claude-border transition-colors"
              >
                <RefreshCw size={13} />
                刷新
              </button>
            </div>

            <div className="border-t border-claude-border pt-3">
              <div className="flex items-center justify-between mb-2">
                <h4 className="text-[11px] font-semibold text-claude-textSecondary uppercase tracking-wider">
                  已保存方案 ({savedDesigns.length})
                </h4>
              </div>

              {savedDesigns.length === 0 ? (
                <p className="text-[12px] text-claude-textSecondary/60 py-3 text-center">
                  暂无保存的设计方案
                </p>
              ) : (
                <div className="space-y-1.5 max-h-48 overflow-y-auto">
                  {savedDesigns.map(design => (
                    <div
                      key={design.id}
                      className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-claude-hover group"
                    >
                      <div className="flex-1 min-w-0">
                        <div className="text-[12px] text-claude-text truncate">{design.name}</div>
                        <div className="text-[10px] text-claude-textSecondary">
                          {new Date(design.createdAt).toLocaleDateString()}
                        </div>
                      </div>
                      <button
                        onClick={() => onLoadDesign(design)}
                        className="p-1 text-claude-textSecondary hover:text-claude-accent opacity-0 group-hover:opacity-100 transition-all"
                        title="加载方案"
                      >
                        <Download size={12} />
                      </button>
                      <button
                        onClick={() => onDeleteDesign(design.id)}
                        className="p-1 text-claude-textSecondary hover:text-red-500 opacity-0 group-hover:opacity-100 transition-all"
                        title="删除方案"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="border-t border-claude-border pt-3">
              <button
                onClick={() => setShowSaveDialog(true)}
                className="w-full flex items-center justify-center gap-2 px-3 py-2 text-[12px] font-medium bg-claude-accent text-white rounded-md hover:opacity-90 transition-opacity"
              >
                <Save size={13} />
                保存当前方案
              </button>
            </div>

            {showSaveDialog && (
              <div className="border border-claude-border rounded-md p-3 bg-claude-surface space-y-2">
                <input
                  type="text"
                  value={saveName}
                  onChange={e => setSaveName(e.target.value)}
                  placeholder="方案名称..."
                  className="w-full px-2 py-1.5 text-[12px] rounded border border-claude-border bg-claude-input text-claude-text"
                  autoFocus
                  onKeyDown={e => e.key === 'Enter' && handleSave()}
                />
                <div className="flex gap-2">
                  <button
                    onClick={handleSave}
                    className="flex-1 px-3 py-1.5 text-[12px] font-medium bg-claude-accent text-white rounded hover:opacity-90"
                  >
                    保存
                  </button>
                  <button
                    onClick={() => setShowSaveDialog(false)}
                    className="px-3 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text rounded"
                  >
                    取消
                  </button>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default DesignPanel;