import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowLeft, Plus, Trash2, Play, Sparkles, Monitor, Smartphone, FileText, Video, Palette, Zap, Eye, Download, ExternalLink, Loader2, Send, X, Grid3X3 } from 'lucide-react';
import { previewService, skillService, type SkillInfo } from '../services/previewService';
import { SkillGallery, DesignPanel, DesignCanvas } from './design';
import type { DesignCanvasRef, SavedDesign } from './design';
import RemotionMotionPanel from './RemotionMotionPanel';
import LivePreviewPanel from './LivePreviewPanel';

interface DesignProject {
  id: string;
  title: string;
  type: 'prototype' | 'slides' | 'motion' | 'infographic' | 'review';
  prompt: string;
  createdAt: number;
  status: 'draft' | 'generating' | 'completed' | 'failed';
  htmlContent?: string;
  previewUrl?: string;
  skillId?: string;
}

const DESIGN_TYPES = [
  { id: 'prototype', label: '交互原型', description: '高保真 App/Web 原型，可点击交互', icon: Smartphone, color: '#8B5CF6', examples: ['iOS App 原型，4 个核心屏幕', 'SaaS Dashboard 原型'] },
  { id: 'slides', label: '演讲幻灯片', description: 'HTML 幻灯片 + 可编辑 PPTX 导出', icon: FileText, color: '#3B82F6', examples: ['AI 心理学演讲 PPT', '产品发布会幻灯片'] },
  { id: 'motion', label: '时间轴动画', description: 'MP4/GIF 动画，支持 60fps 插帧', icon: Video, color: '#EC4899', examples: ['产品发布 60 秒动画', '品牌 Logo 揭示动画'] },
  { id: 'infographic', label: '信息图/可视化', description: '印刷级排版，数据驱动可视化', icon: Palette, color: '#10B981', examples: ['年度报告信息图', '数据对比可视化'] },
  { id: 'review', label: '5 维专家评审', description: '哲学一致性/视觉层级/细节执行等', icon: Eye, color: '#F59E0B', examples: ['设计稿 5 维度评审', '品牌一致性检查'] },
];

const DESIGN_STYLES = [
  { id: 'pentagram', name: 'Pentagram 信息建筑', desc: '精准网格、大胆排版、信息优先', color: '#1a1a2e' },
  { id: 'field', name: 'Field.io 运动诗学', desc: '粒子系统、流体动画、数字美学', color: '#0f3460' },
  { id: 'kenya-hara', name: 'Kenya Hara 东方极简', desc: '留白、素朴、自然、侘寂美学', color: '#e8e8e8' },
  { id: 'sagmeister', name: 'Sagmeister 实验先锋', desc: '大胆色彩、实验性、打破常规', color: '#e94560' },
  { id: 'apple', name: 'Apple 精致科技', desc: '极致精致、微光、高端感', color: '#16213e' },
];

const DesignPage = () => {
  const navigate = useNavigate();
  const [projects, setProjects] = useState<DesignProject[]>([]);
  const [showNewProject, setShowNewProject] = useState(false);
  const [selectedType, setSelectedType] = useState<string>('');
  const [inputPrompt, setInputPrompt] = useState('');
  const [selectedStyle, setSelectedStyle] = useState<string>('');
  const [generating, setGenerating] = useState(false);
  const [previewProject, setPreviewProject] = useState<DesignProject | null>(null);
  const [designSkills, setDesignSkills] = useState<SkillInfo[]>([]);
  const [selectedSkill, setSelectedSkill] = useState<string>('');
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const canvasRef = useRef<DesignCanvasRef>(null);
  const [previewContent, setPreviewContent] = useState<string>('');
  const [isPreviewLoading, setIsPreviewLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'projects' | 'gallery' | 'live'>('projects');
  const [savedDesigns, setSavedDesigns] = useState<SavedDesign[]>([]);
  const [liveMode, setLiveMode] = useState(false);
  const [remotionMode, setRemotionMode] = useState(false);
  const [remotionProjectPath, setRemotionProjectPath] = useState<string>('');
  const [lastMotionPrompt, setLastMotionPrompt] = useState('');

  useEffect(() => {
    loadProjects();
    loadDesignSkills();
    loadSavedDesigns();
  }, []);

  useEffect(() => {
    if (previewProject?.htmlContent) {
      loadPreviewContent(previewProject.id);
    }
  }, [previewProject]);

  const loadProjects = () => {
    try {
      const saved = localStorage.getItem('design_projects');
      setProjects(saved ? JSON.parse(saved) : []);
    } catch {
      setProjects([]);
    }
  };

  const loadDesignSkills = async () => {
    try {
      const skills = await skillService.listDesignSkills();
      setDesignSkills(skills);
    } catch (error) {
      console.error('Failed to load design skills:', error);
    }
  };

  const loadSavedDesigns = () => {
    try {
      const saved = localStorage.getItem('design_saved_designs');
      setSavedDesigns(saved ? JSON.parse(saved) : []);
    } catch {
      setSavedDesigns([]);
    }
  };

  const handleSaveDesign = (name: string, content: string) => {
    const newDesign: SavedDesign = {
      id: Date.now().toString(),
      name,
      content,
      createdAt: Date.now(),
    };
    const updated = [newDesign, ...savedDesigns];
    localStorage.setItem('design_saved_designs', JSON.stringify(updated));
    setSavedDesigns(updated);
  };

  const handleLoadDesign = (design: SavedDesign) => {
    if (canvasRef.current) {
      canvasRef.current.sendMessage({ type: 'set_content', payload: { html: extractHtml(design.content) } });
      setLiveMode(true);
      setActiveTab('live');
    }
  };

  const handleDeleteDesign = (id: string) => {
    const updated = savedDesigns.filter(d => d.id !== id);
    localStorage.setItem('design_saved_designs', JSON.stringify(updated));
    setSavedDesigns(updated);
  };

  const handleApplyDesign = (content: string) => {
    if (previewProject) {
      previewService.setPreview(previewProject.id, content, 'text/html');
    }
  };

  const saveProjects = (newProjects: DesignProject[]) => {
    localStorage.setItem('design_projects', JSON.stringify(newProjects));
    setProjects(newProjects);
  };

  const loadPreviewContent = async (projectId: string) => {
    setIsPreviewLoading(true);
    try {
      const preview = await previewService.getPreview(projectId);
      if (preview) {
        setPreviewContent(preview.content);
      } else if (previewProject?.htmlContent?.startsWith('/')) {
        const response = await fetch(previewProject.htmlContent);
        setPreviewContent(await response.text());
      }
    } catch (error) {
      console.error('Failed to load preview content:', error);
    } finally {
      setIsPreviewLoading(false);
    }
  };

  const handleGenerate = async () => {
    if (!inputPrompt.trim() || !selectedType) return;

    const project: DesignProject = {
      id: Date.now().toString(),
      title: inputPrompt.slice(0, 30) + (inputPrompt.length > 30 ? '...' : ''),
      type: selectedType as DesignProject['type'],
      prompt: inputPrompt,
      createdAt: Date.now(),
      status: 'generating',
      skillId: selectedSkill || undefined,
    };

    const updated = [project, ...projects];
    saveProjects(updated);

    // Motion type uses Remotion engine
    if (selectedType === 'motion') {
      const completedProject: DesignProject = { ...project, status: 'completed', htmlContent: '' };
      const finalProjects = updated.map(p => p.id === project.id ? completedProject : p);
      saveProjects(finalProjects);
      setLastMotionPrompt(inputPrompt);
      setShowNewProject(false);
      setInputPrompt('');
      setSelectedType('');
      setSelectedStyle('');
      setSelectedSkill('');
      setGenerating(false);
      setRemotionMode(true);
      return;
    }

    // All other types: open LivePreviewPanel with real AI generation
    const readyProject: DesignProject = { ...project, status: 'completed' };
    const finalProjects = updated.map(p => p.id === project.id ? readyProject : p);
    saveProjects(finalProjects);
    setShowNewProject(false);
    setInputPrompt('');
    setSelectedType('');
    setSelectedStyle('');
    setSelectedSkill('');
    setGenerating(false);
    setPreviewProject(readyProject);
  };

  const handleDeleteProject = (id: string) => {
    if (!confirm('确定删除此设计项目？')) return;
    const updated = projects.filter(p => p.id !== id);
    saveProjects(updated);
    if (previewProject?.id === id) setPreviewProject(null);
  };

  const handlePreview = (project: DesignProject) => {
    setPreviewProject(project);
  };

  const handleGallerySkillSelect = (skill: SkillInfo) => {
    const od = skill.od_metadata;
    if (od?.mode) {
      const modeMap: Record<string, string> = {
        prototype: 'prototype',
        deck: 'slides',
        image: 'infographic',
        video: 'motion',
        review: 'review',
      };
      const mappedType = modeMap[od.mode] || 'prototype';
      setSelectedType(mappedType);
    }
    setSelectedSkill(skill.id);
    setSelectedStyle('');
  };

  const handleGallerySkillGenerate = (skill: SkillInfo) => {
    setSelectedSkill(skill.id);
    const od = skill.od_metadata;
    if (od?.mode) {
      const modeMap: Record<string, string> = {
        prototype: 'prototype',
        deck: 'slides',
        image: 'infographic',
        video: 'motion',
        review: 'review',
      };
      setSelectedType(modeMap[od.mode] || 'prototype');
    }
    if (od?.example_prompt) {
      setInputPrompt(od.example_prompt);
    }
    setShowNewProject(true);
    setActiveTab('projects');
  };

  const getTypeIcon = (type: string) => DESIGN_TYPES.find(t => t.id === type)?.icon || Sparkles;
  const getTypeColor = (type: string) => DESIGN_TYPES.find(t => t.id === type)?.color || '#8B5CF6';

  if (liveMode) {
    return (
      <div className="flex-1 h-full bg-claude-bg flex flex-col">
        <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border bg-claude-surface flex-shrink-0">
          <div className="flex items-center gap-3">
            <button onClick={() => { setLiveMode(false); setActiveTab('projects'); }} className="flex items-center gap-1.5 text-claude-textSecondary hover:text-claude-text transition-colors">
              <ArrowLeft size={16} />
              <span className="text-[14px]">Back</span>
            </button>
            <div className="h-4 w-px bg-claude-border" />
            <h2 className="text-[14px] font-medium text-claude-text">Live Design</h2>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => canvasRef.current?.sendMessage({ type: 'reset', payload: {} })}
              className="px-3 py-1.5 text-[13px] font-medium text-claude-textSecondary hover:text-claude-text flex items-center gap-1.5 transition-colors"
            >
              <Download size={14} />
              Export
            </button>
          </div>
        </div>

        <div className="flex-1 flex overflow-hidden">
          <div className="w-[260px] flex-shrink-0 border-r border-claude-border bg-claude-surface overflow-hidden">
            <DesignPanel
              canvasRef={canvasRef}
              onSave={handleSaveDesign}
              onApply={handleApplyDesign}
              savedDesigns={savedDesigns}
              onLoadDesign={handleLoadDesign}
              onDeleteDesign={handleDeleteDesign}
            />
          </div>

          <div className="flex-1 bg-claude-bg p-4">
            <div className="w-full h-full rounded-xl border border-claude-border overflow-hidden bg-white shadow-sm">
              <DesignCanvas ref={canvasRef} />
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (remotionMode) {
    return (
      <div className="flex-1 h-full bg-claude-bg flex flex-col">
        <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border bg-claude-surface flex-shrink-0">
          <div className="flex items-center gap-3">
            <button onClick={() => { setRemotionMode(false); setPreviewProject(null); }} className="flex items-center gap-1.5 text-claude-textSecondary hover:text-claude-text transition-colors">
              <ArrowLeft size={16} />
              <span className="text-[14px]">Back to Design</span>
            </button>
            <div className="h-4 w-px bg-claude-border" />
            <div className="flex items-center gap-2">
              <div className="w-5 h-5 rounded flex items-center justify-center bg-[#EC4899]/10">
                <Video size={12} className="text-[#EC4899]" />
              </div>
              <h2 className="text-[14px] font-medium text-claude-text">Remotion 视频动画</h2>
            </div>
          </div>
        </div>
        <RemotionMotionPanel
          initialPrompt={lastMotionPrompt}
          onRenderComplete={(outputPath) => {
            console.log('[Design] Remotion render complete:', outputPath);
          }}
          onOpenInStudio={(projectPath) => {
            console.log('[Design] Remotion Studio opened for:', projectPath);
          }}
        />
      </div>
    );
  }

  if (previewProject) {
    const isMotionProject = previewProject.type === 'motion';

    return (
      <div className="flex-1 h-full bg-claude-bg flex flex-col">
        <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border bg-claude-surface flex-shrink-0">
          <div className="flex items-center gap-3">
            <button onClick={() => setPreviewProject(null)} className="flex items-center gap-1.5 text-claude-textSecondary hover:text-claude-text transition-colors">
              <ArrowLeft size={16} />
              <span className="text-[14px]">Back</span>
            </button>
            <div className="h-4 w-px bg-claude-border" />
            <h2 className="text-[14px] font-medium text-claude-text">{previewProject.title}</h2>
          </div>
          <div className="flex items-center gap-2">
            {isMotionProject && (
              <button
                onClick={() => setRemotionMode(true)}
                className="px-3 py-1.5 text-[13px] font-medium bg-[#EC4899] text-white rounded-md flex items-center gap-1.5 hover:opacity-90 transition-opacity"
              >
                <Video size={14} />
                Open Remotion Studio
              </button>
            )}
            <button
              onClick={() => { setLiveMode(true); setActiveTab('live'); }}
              className="px-3 py-1.5 text-[13px] font-medium bg-claude-accent text-white rounded-md flex items-center gap-1.5 hover:opacity-90 transition-opacity"
            >
              <Palette size={14} />
              Live Design
            </button>
            <button className="px-3 py-1.5 text-[13px] font-medium text-claude-textSecondary hover:text-claude-text flex items-center gap-1.5 transition-colors">
              <Download size={14} />
              Export
            </button>
            <button className="px-3 py-1.5 text-[13px] font-medium text-claude-textSecondary hover:text-claude-text flex items-center gap-1.5 transition-colors">
              <ExternalLink size={14} />
              Open in Browser
            </button>
          </div>
        </div>

        <div className="flex-1 bg-claude-bg p-4">
          {isMotionProject ? (
            <div className="w-full h-full rounded-xl border border-claude-border overflow-hidden bg-white">
              <RemotionMotionPanel
                initialPrompt={previewProject.prompt}
                onRenderComplete={(outputPath) => {
                  console.log('[Design] Remotion render complete:', outputPath);
                }}
                onOpenInStudio={(projectPath) => {
                  console.log('[Design] Remotion Studio opened for:', projectPath);
                }}
              />
            </div>
          ) : (
            <LivePreviewPanel
              key={previewProject.id}
              projectId={previewProject.id}
              initialPrompt={previewProject.prompt}
              designType={(previewProject.type === 'review' ? 'review' : previewProject.type === 'slides' ? 'slides' : previewProject.type === 'infographic' ? 'infographic' : 'prototype')}
              initialStyle={selectedStyle}
              onBack={() => setPreviewProject(null)}
              onSave={(content) => {
                handleSaveDesign(previewProject.title, content);
              }}
            />
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
      <div className="max-w-[900px] mx-auto px-4 py-8 md:px-8 md:py-12">
        <button onClick={() => navigate('/')} className="flex items-center gap-1.5 text-claude-textSecondary hover:text-claude-text transition-colors mb-6">
          <ArrowLeft size={16} />
          <span className="text-[14px]">Back</span>
        </button>

        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="font-[Spectral] text-[32px] text-claude-text" style={{ fontWeight: 500, WebkitTextStroke: '0.5px currentColor' }}>Claude Design</h1>
            <p className="text-[14px] text-claude-textSecondary mt-1">用 HTML 做高保真原型、幻灯片、动画与设计评审</p>
          </div>
          <div className="flex items-center gap-2">
            <div className="flex bg-claude-hover rounded-lg p-0.5">
              <button
                onClick={() => setActiveTab('projects')}
                className={`px-3 py-1.5 rounded-md text-[13px] font-medium transition-colors ${
                  activeTab === 'projects'
                    ? 'bg-claude-surface text-claude-text shadow-sm'
                    : 'text-claude-textSecondary hover:text-claude-text'
                }`}
              >
                项目
              </button>
              <button
                onClick={() => setActiveTab('gallery')}
                className={`px-3 py-1.5 rounded-md text-[13px] font-medium transition-colors ${
                  activeTab === 'gallery'
                    ? 'bg-claude-surface text-claude-text shadow-sm'
                    : 'text-claude-textSecondary hover:text-claude-text'
                }`}
              >
                技能画廊
              </button>
              <button
                onClick={() => { setLiveMode(true); setActiveTab('live'); }}
                className={`px-3 py-1.5 rounded-md text-[13px] font-medium transition-colors flex items-center gap-1.5 ${
                  activeTab === 'live'
                    ? 'bg-claude-surface text-claude-text shadow-sm'
                    : 'text-claude-textSecondary hover:text-claude-text'
                }`}
              >
                <Palette size={13} />
                Live Design
              </button>
            </div>
            <button onClick={() => { setSelectedType('motion'); setRemotionMode(true); }} className="flex items-center gap-2 px-3 py-1.5 bg-[#EC4899] text-white hover:bg-[#DB2777] rounded-lg transition-colors font-medium" style={{ fontSize: '14px' }}>
              <Video size={16} />
              Remotion Studio
            </button>
            <button onClick={() => { setShowNewProject(true); setActiveTab('projects'); }} className="flex items-center gap-2 px-3.5 py-1.5 bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity font-medium" style={{ fontSize: '14px' }}>
              <Plus size={16} />
              New Design
            </button>
          </div>
        </div>

        {activeTab === 'gallery' ? (
          <SkillGallery onSkillSelect={handleGallerySkillSelect} onGenerateWithSkill={handleGallerySkillGenerate} />
        ) : (
          <>
            {showNewProject && (
          <div className="mb-8 p-5 bg-claude-input border border-claude-border rounded-xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-[15px] font-semibold text-claude-text">创建新设计</h3>
              <button onClick={() => { setShowNewProject(false); setInputPrompt(''); setSelectedType(''); setSelectedStyle(''); setSelectedSkill(''); }} className="p-1 hover:bg-claude-hover rounded transition-colors">
                <X size={16} className="text-claude-textSecondary" />
              </button>
            </div>

            <div className="mb-5">
              <label className="block text-[13px] text-claude-textSecondary mb-2">设计类型</label>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                {DESIGN_TYPES.map(type => {
                  const Icon = type.icon;
                  return (
                    <button key={type.id} onClick={() => setSelectedType(type.id)} className={`flex items-center gap-2.5 p-3 rounded-lg border transition-all text-left ${selectedType === type.id ? 'border-[#8B5CF6] bg-[#8B5CF6]/10' : 'border-claude-border hover:border-claude-textSecondary/30'}`}>
                      <div className="w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0" style={{ backgroundColor: type.color + '20' }}>
                        <Icon size={16} style={{ color: type.color }} />
                      </div>
                      <div>
                        <p className="text-[13px] font-medium text-claude-text">{type.label}</p>
                        <p className="text-[11px] text-claude-textSecondary">{type.description}</p>
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>

            <div className="mb-5">
              <label className="block text-[13px] text-claude-textSecondary mb-2">设计风格（可选）</label>
              <div className="flex flex-wrap gap-2">
                {DESIGN_STYLES.map(style => (
                  <button key={style.id} onClick={() => setSelectedStyle(style.id === selectedStyle ? '' : style.id)} className={`px-3 py-1.5 rounded-lg text-[12px] font-medium transition-colors ${selectedStyle === style.id ? 'bg-claude-text text-claude-bg' : 'bg-claude-hover text-claude-textSecondary hover:text-claude-text'}`}>
                    {style.name}
                  </button>
                ))}
              </div>
            </div>

            {designSkills.length > 0 && (
              <div className="mb-5">
                <label className="block text-[13px] text-claude-textSecondary mb-2">设计技能（可选）</label>
                <div className="flex flex-wrap gap-2">
                  {designSkills.map(skill => (
                    <button key={skill.id} onClick={() => setSelectedSkill(skill.id === selectedSkill ? '' : skill.id)} className={`px-3 py-1.5 rounded-lg text-[12px] font-medium transition-colors ${selectedSkill === skill.id ? 'bg-blue-500 text-white' : 'bg-claude-hover text-claude-textSecondary hover:text-claude-text'}`}>
                      {skill.name}
                    </button>
                  ))}
                </div>
              </div>
            )}

            <div className="mb-4">
              <label className="block text-[13px] text-claude-textSecondary mb-1.5">设计描述</label>
              <div className="relative">
                <textarea value={inputPrompt} onChange={(e) => setInputPrompt(e.target.value)} placeholder="描述你想要的设计，例如：做一个 AI 番茄钟 iOS 原型，4 个核心屏幕要真能点击" className="w-full px-3 py-2.5 bg-transparent border border-claude-border rounded-lg text-[14px] text-claude-text focus:outline-none focus:border-blue-500 resize-none" rows={3} onKeyDown={(e) => { if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) handleGenerate(); }} />
                <div className="absolute bottom-2 right-2 text-[11px] text-claude-textSecondary">⌘ + Enter 发送</div>
              </div>
            </div>

            {selectedType && (
              <div className="mb-4">
                <label className="block text-[12px] text-claude-textSecondary mb-1.5">快速示例</label>
                <div className="flex flex-wrap gap-2">
                  {DESIGN_TYPES.find(t => t.id === selectedType)?.examples.map((ex, i) => (
                    <button key={i} onClick={() => setInputPrompt(ex)} className="px-2.5 py-1 bg-claude-hover text-claude-textSecondary hover:text-claude-text rounded-md text-[12px] transition-colors">
                      {ex}
                    </button>
                  ))}
                </div>
              </div>
            )}

            <div className="flex justify-end gap-3">
              <button onClick={() => { setShowNewProject(false); setInputPrompt(''); setSelectedType(''); setSelectedStyle(''); setSelectedSkill(''); }} className="px-4 py-2 text-[14px] font-medium text-claude-text hover:bg-claude-hover rounded-lg transition-colors">
                Cancel
              </button>
              <button onClick={handleGenerate} disabled={generating || !inputPrompt.trim() || !selectedType} className="px-4 py-2 text-[14px] font-medium text-white bg-[#333333] hover:bg-[#1a1a1a] dark:bg-[#FFFFFF] dark:text-black dark:hover:bg-[#e5e5e5] rounded-lg transition-colors disabled:opacity-50 flex items-center gap-2">
                {generating ? (<><Loader2 size={14} className="animate-spin" />Generating...</>) : (<><Sparkles size={14} />Generate</>)}
              </button>
            </div>
          </div>
        )}

        {projects.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-claude-textSecondary">
            <div className="w-16 h-16 rounded-2xl bg-claude-hover flex items-center justify-center mb-4">
              <Sparkles size={24} className="opacity-40" />
            </div>
            <p className="text-[15px] font-medium text-claude-text mb-1">还没有设计项目</p>
            <p className="text-[13px] mb-4">点击 "New Design" 创建第一个设计</p>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3 mt-6 max-w-[600px]">
              {DESIGN_TYPES.map(type => {
                const Icon = type.icon;
                return (
                  <div key={type.id} className="p-3 bg-claude-input border border-claude-border rounded-lg text-center">
                    <div className="w-8 h-8 rounded-lg flex items-center justify-center mx-auto mb-2" style={{ backgroundColor: type.color + '20' }}>
                      <Icon size={16} style={{ color: type.color }} />
                    </div>
                    <p className="text-[13px] font-medium text-claude-text">{type.label}</p>
                    <p className="text-[11px] text-claude-textSecondary mt-0.5">{type.description}</p>
                  </div>
                );
              })}
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {projects.map(project => {
              const Icon = getTypeIcon(project.type);
              const color = getTypeColor(project.type);
              return (
                <div key={project.id} className="flex items-center justify-between p-4 bg-claude-input border border-claude-border rounded-xl hover:bg-claude-hover transition-colors group">
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0" style={{ backgroundColor: color + '20' }}>
                      <Icon size={18} style={{ color }} />
                    </div>
                    <div>
                      <h3 className="text-[14px] font-medium text-claude-text">{project.title}</h3>
                      <p className="text-[12px] text-claude-textSecondary mt-0.5">
                        {DESIGN_TYPES.find(t => t.id === project.type)?.label} · {new Date(project.createdAt).toLocaleDateString('zh-CN')}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-0.5 rounded text-[11px] font-medium ${project.status === 'completed' ? 'bg-green-500/10 text-green-400' : project.status === 'generating' ? 'bg-yellow-500/10 text-yellow-400' : project.status === 'failed' ? 'bg-red-500/10 text-red-400' : 'bg-claude-hover text-claude-textSecondary'}`}>
                      {project.status === 'completed' ? '完成' : project.status === 'generating' ? '生成中' : project.status === 'failed' ? '失败' : '草稿'}
                    </span>
                    {project.status === 'completed' && (
                      <button onClick={() => handlePreview(project)} className="px-2.5 py-1.5 text-[12px] font-medium text-claude-text hover:bg-claude-hover rounded-md transition-colors flex items-center gap-1">
                        <Play size={12} />
                        预览
                      </button>
                    )}
                    <button onClick={() => handleDeleteProject(project.id)} className="w-8 h-8 rounded-lg flex items-center justify-center text-claude-textSecondary hover:text-[#B9382C] hover:bg-[#B9382C]/10 transition-colors opacity-0 group-hover:opacity-100">
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
          </>
        )}
      </div>
    </div>
  );
};

export default DesignPage;

function extractHtml(fullDoc: string): string {
  const rootMatch = fullDoc.match(/id="design-canvas-root">([\s\S]*?)<\/div>\s*(?:<script|<style|$)/);
  if (rootMatch) return rootMatch[1].trim();
  const bodyMatch = fullDoc.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  if (bodyMatch) return bodyMatch[1].trim();
  return fullDoc;
}
