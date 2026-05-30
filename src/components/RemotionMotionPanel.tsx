import { getErrorMessage } from '../utils/errorHelpers';
import React, { useState, useEffect, useRef } from 'react';
import {
  Video, Plus, Play, Square, FolderOpen, Download, Eye, Loader2,
  RefreshCw, ExternalLink, Terminal, Check, X, Clock, Film,
  Monitor, Smartphone, FileVideo, Image, Settings, ChevronDown, Zap,
  Sparkles, Send, Wand2, Code, RotateCcw,
} from 'lucide-react';
import {
  remotionService,
  generateRemotionCode,
  writeGeneratedCode,
  type RemotionProject,
  type CompositionInfo,
  type RenderRequest,
  type RenderResponse,
  REMOTION_TEMPLATES,
  RENDER_PRESETS,
  OUTPUT_FORMATS,
} from '../services/remotionService';

// ━━━━━━━━━━━━━━━━━ Types ━━━━━━━━━━━━━━━━━

interface RemotionMotionPanelProps {
  onRenderComplete?: (outputPath: string) => void;
  onOpenInStudio?: (projectPath: string) => void;
  initialPrompt?: string;
}

type PanelView = 'ai-generate' | 'projects' | 'create' | 'compositions' | 'render';

// ━━━━━━━━━━━━━━━━━ COMPONENT ━━━━━━━━━━━━━━━━━

const RemotionMotionPanel: React.FC<RemotionMotionPanelProps> = ({ onRenderComplete, onOpenInStudio, initialPrompt }) => {
  // Project state
  const [projects, setProjects] = useState<RemotionProject[]>([]);
  const [activeProject, setActiveProject] = useState<RemotionProject | null>(null);
  const [compositions, setCompositions] = useState<CompositionInfo[]>([]);

  // View state
  const [view, setView] = useState<PanelView>('ai-generate');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // AI generation state
  const [aiPrompt, setAiPrompt] = useState('');
  const [aiGenerating, setAiGenerating] = useState(false);
  const [aiStreamingText, setAiStreamingText] = useState('');
  const [aiGeneratedCode, setAiGeneratedCode] = useState('');
  const [aiSelectedModel, setAiSelectedModel] = useState('claude-sonnet-4-6');
  const [aiOutputDir, setAiOutputDir] = useState('C:\\Users\\user\\remotion-projects');

  // Create state
  const [newProjectName, setNewProjectName] = useState('');
  const [newProjectDir, setNewProjectDir] = useState('');
  const [selectedTemplate, setSelectedTemplate] = useState('hello-world');
  const [creating, setCreating] = useState(false);

  // Render state
  const [selectedComposition, setSelectedComposition] = useState<string>('');
  const [outputPath, setOutputPath] = useState('');
  const [fps, setFps] = useState(30);
  const [useCustomFps, setUseCustomFps] = useState(false);
  const [rendering, setRendering] = useState(false);
  const [renderResult, setRenderResult] = useState<RenderResponse | null>(null);

  // Studio state
  const [studioPort] = useState(3000);
  const [studioRunning, setStudioRunning] = useState(false);

  // Preset state
  const [selectedPreset, setSelectedPreset] = useState(1); // YouTube 1080p
  const [outputFormat, setOutputFormat] = useState('mp4');

  // ── Load projects on mount ──
  useEffect(() => { loadProjects(); }, []);

  // ── Pre-fill prompt from DesignPage ──
  useEffect(() => {
    if (initialPrompt && initialPrompt.trim()) {
      setAiPrompt(initialPrompt.trim());
      setView('ai-generate');
    }
  }, [initialPrompt]);

  const loadProjects = async () => {
    setLoading(true);
    try {
      const homeDir = aiOutputDir || 'C:\\Users\\user';
      const found = await remotionService.scanProjects(homeDir);
      setProjects(found);
    } catch {
      setProjects([]);
    } finally {
      setLoading(false);
    }
  };

  const getHomeDir = async (): Promise<string> => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const path = await invoke('get_app_path') as string;
      return path.split('\\').slice(0, -1).join('\\') || 'C:\\Users\\user';
    } catch {
      return 'C:\\Users\\user';
    }
  };

  // ━━━━━━━━━━━━━━ AI GENERATION ━━━━━━━━━━━━━━

  const handleAIGenerate = async () => {
    if (!aiPrompt.trim()) return;
    setAiGenerating(true);
    setAiStreamingText('');
    setAiGeneratedCode('');
    setError(null);

    const result = await generateRemotionCode(
      aiPrompt.trim(),
      aiSelectedModel,
      (streaming) => setAiStreamingText(streaming),
    );

    setAiGenerating(false);

    if (!result.success) {
      setError(result.error || 'AI 生成失败');
      return;
    }

    setAiGeneratedCode(result.code || '');
    // Auto-proceed to create project
    if (result.code) {
      await handleCreateFromAI(result.code);
    }
  };

  const handleCreateFromAI = async (code: string) => {
    setError(null);
    setLoading(true);

    // Generate project name from the prompt
    const projectName = 'ai-video-' + Date.now().toString(36);
    const targetDir = aiOutputDir;

    try {
      // 1. Create the Remotion project
      const project = await remotionService.createProject(projectName, targetDir, 'blank');
      setActiveProject(project);

      // 2. Wait for npm install (create-video does this automatically)
      // 3. Write AI-generated code
      const written = await writeGeneratedCode(project.path, code);
      if (!written) {
        setError('代码写入失败');
        setLoading(false);
        return;
      }

      // 4. Refresh project state
      project.has_node_modules = true;
      setProjects(prev => [project, ...prev]);

      // 5. Try to list compositions
      try {
        const comps = await remotionService.listCompositions(project.path);
        setCompositions(comps);
        if (comps.length > 0) {
          setSelectedComposition(comps[0].id);
          setFps(comps[0].fps);
        }
      } catch {
        // Code may need manual tweaking — proceed anyway
        setError('项目已创建，但 composition 列表可能为空。请检查生成的代码。');
      }

      setView('compositions');
      setLoading(false);
      setAiPrompt('');
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : getErrorMessage(e) || '项目创建失败');
      setLoading(false);
    }
  };

  // ━━━━━━━━━━━━━━ MANUAL CREATE ━━━━━━━━━━━━━━

  const handleCreateProject = async () => {
    if (!newProjectName.trim() || !newProjectDir.trim()) return;
    setCreating(true);
    setError(null);
    try {
      const project = await remotionService.createProject(
        newProjectName.trim(), newProjectDir.trim(), selectedTemplate,
      );
      setProjects(prev => [project, ...prev]);
      setActiveProject(project);
      setView('projects');
      setNewProjectName('');
      setNewProjectDir('');
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : getErrorMessage(e) || '创建项目失败');
    } finally { setCreating(false); }
  };

  // ━━━━━━━━━━━━━━ COMPOSITIONS ━━━━━━━━━━━━━━

  const handleLoadCompositions = async (project: RemotionProject) => {
    setLoading(true);
    setError(null);
    try {
      const comps = await remotionService.listCompositions(project.path);
      setCompositions(comps);
      setActiveProject(project);
      if (comps.length > 0) {
        setSelectedComposition(comps[0].id);
        setFps(comps[0].fps);
      }
      setView('compositions');
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : getErrorMessage(e) || '获取 Compositions 失败。请先安装依赖。');
    } finally { setLoading(false); }
  };

  // ━━━━━━━━━━━━━━ STUDIO ━━━━━━━━━━━━━━

  const handleStartStudio = async (project: RemotionProject) => {
    setLoading(true);
    setError(null);
    try {
      await remotionService.startStudio(project.path, studioPort);
      setStudioRunning(true);
      onOpenInStudio?.(project.path);
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : getErrorMessage(e) || '启动 Studio 失败');
    } finally { setLoading(false); }
  };

  // ━━━━━━━━━━━━━━ RENDER ━━━━━━━━━━━━━━

  const handleRender = async () => {
    if (!activeProject || !selectedComposition) return;
    setRendering(true);
    setRenderResult(null);
    setError(null);

    const outputFile = outputPath ||
      `${activeProject.path}\\out\\${selectedComposition}${OUTPUT_FORMATS.find(f => f.id === outputFormat)?.ext || '.mp4'}`;

    const preset = RENDER_PRESETS[selectedPreset];
    const request: RenderRequest = {
      projectPath: activeProject.path,
      compositionId: selectedComposition,
      outputPath: outputFile,
      fps: useCustomFps ? fps : (preset.fps > 0 ? preset.fps : undefined),
    };

    try {
      const result = await remotionService.render(request);
      setRenderResult(result);
      if (result.success) {
        onRenderComplete?.(result.output_file);
      }
    } catch (e: unknown) {
      setError(typeof e === 'string' ? e : getErrorMessage(e) || '渲染失败');
    } finally { setRendering(false); }
  };

  // ━━━━━━━━━━━━━━ STYLES ━━━━━━━━━━━━━━

  const btnBase = "px-3 py-1.5 rounded-lg text-[12px] font-medium transition-colors flex items-center gap-1.5";
  const btnPrimary = `${btnBase} bg-[#8B5CF6] text-white hover:bg-[#7C3AED] disabled:opacity-50`;
  const btnSecondary = `${btnBase} bg-claude-hover text-claude-textSecondary hover:text-claude-text`;
  const btnPink = `${btnBase} bg-[#EC4899] text-white hover:bg-[#DB2777] disabled:opacity-50`;
  const inputClass = "w-full px-3 py-2 bg-claude-input border border-claude-border rounded-lg text-[13px] text-claude-text focus:outline-none focus:border-[#8B5CF6] transition-colors";
  const labelClass = "block text-[12px] text-claude-textSecondary mb-1.5";

  // ━━━━━━━━━━━━━━ RENDER ━━━━━━━━━━━━━━

  return (
    <div className="flex-1 h-full bg-claude-bg flex flex-col overflow-hidden">
      {/* ── Header ── */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border flex-shrink-0">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg flex items-center justify-center bg-[#EC4899]/10">
            <Video size={15} className="text-[#EC4899]" />
          </div>
          <h2 className="text-[14px] font-semibold text-claude-text">Remotion 视频引擎</h2>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => { setView('ai-generate'); setError(null); setAiGeneratedCode(''); }}
            className={`${btnBase} ${view === 'ai-generate' ? 'bg-[#EC4899]/20 text-[#EC4899]' : 'text-claude-textSecondary hover:text-claude-text'}`}
          >
            <Sparkles size={13} />
            AI 生成
          </button>
          <button
            onClick={() => { setView('projects'); setError(null); }}
            className={`${btnBase} ${view === 'projects' ? 'bg-[#8B5CF6]/20 text-[#8B5CF6]' : 'text-claude-textSecondary hover:text-claude-text'}`}
          >
            <FolderOpen size={13} />
            项目
          </button>
          <button
            onClick={() => { setView('create'); setError(null); }}
            className={btnSecondary}
          >
            <Plus size={13} />
            手动创建
          </button>
        </div>
      </div>

      {/* ── Error banner ── */}
      {error && (
        <div className="mx-4 mt-3 px-3 py-2 bg-red-500/10 border border-red-500/20 rounded-lg text-[12px] text-red-400 flex items-start justify-between">
          <span className="flex-1">{error}</span>
          <button onClick={() => setError(null)} className="p-0.5 hover:bg-red-500/20 rounded flex-shrink-0">
            <X size={14} />
          </button>
        </div>
      )}

      {/* ── Content ── */}
      <div className="flex-1 overflow-y-auto p-4">

        {/* ════════════════ AI GENERATE VIEW ════════════════ */}
        {view === 'ai-generate' && (
          <div className="space-y-4">
            <div className="text-center py-4">
              <div className="w-16 h-16 rounded-2xl bg-[#EC4899]/10 flex items-center justify-center mx-auto mb-3">
                <Wand2 size={28} className="text-[#EC4899]" />
              </div>
              <h3 className="text-[15px] font-semibold text-claude-text">AI 视频生成</h3>
              <p className="text-[12px] text-claude-textSecondary mt-1">
                用自然语言描述你想要的动画，Claude 会生成完整的 Remotion 代码并渲染成视频
              </p>
            </div>

            {/* Model selector */}
            <div className="flex items-center gap-2">
              <span className="text-[11px] text-claude-textSecondary">模型:</span>
              <select
                value={aiSelectedModel}
                onChange={e => setAiSelectedModel(e.target.value)}
                className="px-2 py-0.5 bg-claude-input border border-claude-border rounded text-[12px] text-claude-text"
              >
                <option value="claude-sonnet-4-6">Claude Sonnet 4.6</option>
                <option value="claude-opus-4-7">Claude Opus 4.7</option>
                <option value="claude-3-5-sonnet-latest">Claude 3.5 Sonnet</option>
              </select>
              <div className="flex-1" />
              <span className="text-[11px] text-claude-textSecondary">输出目录:</span>
              <input
                value={aiOutputDir}
                onChange={e => setAiOutputDir(e.target.value)}
                className="w-48 px-2 py-0.5 bg-claude-input border border-claude-border rounded text-[12px] text-claude-text"
                placeholder="C:\Users\user\remotion-projects"
              />
            </div>

            {/* Prompt input */}
            <div>
              <label className={labelClass}>描述你想要的动画</label>
              <div className="relative">
                <textarea
                  value={aiPrompt}
                  onChange={e => setAiPrompt(e.target.value)}
                  placeholder={'例如：做一个 5 秒的科技公司 Logo 揭示动画，背景深蓝色，中间一个发光的六边形从 0 缩放到 1 弹出入场，下方文字 "Nexus AI" 从下方淡入上移，使用紫色和青色渐变配色，加上粒子飘散效果'}
                  className={`${inputClass} resize-none`}
                  rows={4}
                  disabled={aiGenerating}
                  onKeyDown={e => {
                    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                      handleAIGenerate();
                    }
                  }}
                />
                <div className="absolute bottom-2 right-2 text-[11px] text-claude-textSecondary">
                  {aiGenerating ? '' : 'Ctrl+Enter 发送'}
                </div>
              </div>
            </div>

            {/* Quick examples */}
            <div className="flex flex-wrap gap-1.5">
              {[
                '5秒 Logo 揭示动画，深蓝背景，金色粒子',
                '10秒 产品宣传片，3D 旋转产品模型',
                '3秒 转场动画，毛玻璃模糊切换效果',
                '30秒 数据可视化，动态图表+数字跳动',
                '8秒 文字打字机效果，打字机光标+音波',
              ].map((ex, i) => (
                <button
                  key={i}
                  onClick={() => setAiPrompt(ex)}
                  className="px-2 py-1 text-[11px] bg-claude-hover text-claude-textSecondary hover:text-claude-text rounded-md transition-colors"
                  disabled={aiGenerating}
                >
                  {ex}
                </button>
              ))}
            </div>

            {/* Generate button */}
            <div className="flex gap-2">
              <button
                onClick={handleAIGenerate}
                disabled={aiGenerating || !aiPrompt.trim()}
                className={`${btnPink} px-4 py-2 text-[13px]`}
              >
                {aiGenerating ? (
                  <><Loader2 size={14} className="animate-spin" /> Claude 正在生成代码...</>
                ) : (
                  <><Sparkles size={14} /> 生成视频</>
                )}
              </button>
              {aiGenerating && (
                <button onClick={() => { setAiGenerating(false); }} className={btnSecondary}>
                  <Square size={12} /> 停止
                </button>
              )}
            </div>

            {/* AI streaming preview */}
            {aiStreamingText && (
              <div className="p-3 bg-claude-input border border-claude-border rounded-lg max-h-64 overflow-y-auto">
                <div className="flex items-center gap-2 mb-2">
                  <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
                  <span className="text-[11px] text-claude-textSecondary">
                    {aiGenerating ? 'Claude 正在生成...' : '生成完成'}
                  </span>
                </div>
                <pre className="text-[11px] text-claude-text font-mono whitespace-pre-wrap">
                  {aiStreamingText.slice(-3000)}
                </pre>
              </div>
            )}

            {/* Generated code preview */}
            {aiGeneratedCode && !aiGenerating && (
              <div className="p-3 bg-green-500/5 border border-green-500/20 rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-[13px] font-medium text-green-400 flex items-center gap-1.5">
                    <Check size={14} />
                    代码已生成 ({aiGeneratedCode.split('\n').length} 行)
                  </span>
                  <button
                    onClick={() => handleCreateFromAI(aiGeneratedCode)}
                    className={btnPink}
                    disabled={loading}
                  >
                    {loading ? <Loader2 size={12} className="animate-spin" /> : <Play size={12} />}
                    创建项目并渲染
                  </button>
                </div>
                <pre className="text-[11px] text-claude-textSecondary font-mono max-h-32 overflow-y-auto whitespace-pre-wrap">
                  {aiGeneratedCode.slice(0, 1500)}
                </pre>
              </div>
            )}
          </div>
        )}

        {/* ════════════════ PROJECTS VIEW ════════════════ */}
        {view === 'projects' && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-[15px] font-semibold text-claude-text">
                项目列表 ({projects.length})
              </h3>
              <button onClick={loadProjects} disabled={loading} className={btnSecondary}>
                <RefreshCw size={13} className={loading ? 'animate-spin' : ''} />
                刷新
              </button>
            </div>

            {loading && projects.length === 0 && (
              <div className="flex items-center justify-center py-12 text-claude-textSecondary">
                <Loader2 size={20} className="animate-spin mr-2" />
                <span className="text-[13px]">扫描 Remotion 项目...</span>
              </div>
            )}

            {!loading && projects.length === 0 && (
              <div className="flex flex-col items-center justify-center py-12 text-claude-textSecondary">
                <div className="w-14 h-14 rounded-2xl bg-claude-hover flex items-center justify-center mb-3">
                  <Video size={24} className="opacity-40" />
                </div>
                <p className="text-[14px] font-medium text-claude-text mb-1">没有找到 Remotion 项目</p>
                <p className="text-[12px] mb-4">
                  使用 <span className="text-[#EC4899]">AI 生成</span> 自动创建，或手动创建
                </p>
                <div className="flex gap-2">
                  <button onClick={() => setView('ai-generate')} className={btnPink}>
                    <Sparkles size={13} />
                    AI 生成
                  </button>
                  <button onClick={() => setView('create')} className={btnPrimary}>
                    <Plus size={13} />
                    手动创建
                  </button>
                </div>
              </div>
            )}

            <div className="space-y-2">
              {projects.map(project => (
                <div key={project.path} className="p-3 bg-claude-input border border-claude-border rounded-lg hover:bg-claude-hover/50 transition-colors">
                  <div className="flex items-center justify-between">
                    <div className="flex-1 min-w-0">
                      <p className="text-[13px] font-medium text-claude-text truncate">{project.name}</p>
                      <p className="text-[11px] text-claude-textSecondary truncate mt-0.5">{project.path}</p>
                      <div className="flex items-center gap-2 mt-1.5">
                        <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${
                          project.has_node_modules ? 'bg-green-500/10 text-green-400' : 'bg-yellow-500/10 text-yellow-400'
                        }`}>
                          {project.has_node_modules ? '就绪' : '需安装依赖'}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-1 flex-shrink-0 ml-3">
                      {!project.has_node_modules && (
                        <button onClick={async () => {
                          try { await remotionService.installDeps(project.path); project.has_node_modules = true; setProjects([...projects]); }
                          catch (e: unknown) { setError(getErrorMessage(e) || '安装失败'); }
                        }} className={btnSecondary} title="安装依赖">
                          <Download size={12} />
                        </button>
                      )}
                      <button onClick={() => handleLoadCompositions(project)} className={btnSecondary} title="Compositions">
                        <Film size={12} />
                      </button>
                      <button onClick={() => handleStartStudio(project)} className={btnSecondary} title="Studio">
                        <Monitor size={12} />
                      </button>
                      <button onClick={() => remotionService.openInEditor(project.path)} className={btnSecondary} title="VS Code">
                        <ExternalLink size={12} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* ════════════════ CREATE VIEW ════════════════ */}
        {view === 'create' && (
          <div className="space-y-4">
            <h3 className="text-[15px] font-semibold text-claude-text">手动创建 Remotion 项目</h3>

            <div>
              <label className={labelClass}>项目名称</label>
              <input value={newProjectName} onChange={e => setNewProjectName(e.target.value)}
                placeholder="my-remotion-video" className={inputClass} />
            </div>

            <div>
              <label className={labelClass}>目标目录</label>
              <input value={newProjectDir} onChange={e => setNewProjectDir(e.target.value)}
                placeholder="C:\Users\user\projects" className={inputClass} />
            </div>

            <div>
              <label className={labelClass}>模板</label>
              <div className="grid grid-cols-2 gap-2">
                {REMOTION_TEMPLATES.map(tpl => (
                  <button key={tpl.id} onClick={() => setSelectedTemplate(tpl.id)}
                    className={`p-3 rounded-lg border text-left transition-all ${
                      selectedTemplate === tpl.id
                        ? 'border-[#8B5CF6] bg-[#8B5CF6]/10'
                        : 'border-claude-border hover:border-claude-textSecondary/30'
                    }`}>
                    <p className="text-[13px] font-medium text-claude-text">{tpl.name}</p>
                    <p className="text-[11px] text-claude-textSecondary mt-0.5">{tpl.description}</p>
                  </button>
                ))}
              </div>
            </div>

            <div className="flex gap-2 pt-2">
              <button onClick={handleCreateProject}
                disabled={creating || !newProjectName.trim() || !newProjectDir.trim()}
                className={btnPrimary}>
                {creating ? <Loader2 size={13} className="animate-spin" /> : <Plus size={13} />}
                {creating ? '创建中...' : '创建项目'}
              </button>
              <button onClick={() => setView('projects')} className={btnSecondary}>取消</button>
            </div>
          </div>
        )}

        {/* ════════════════ COMPOSITIONS VIEW ════════════════ */}
        {view === 'compositions' && activeProject && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-[15px] font-semibold text-claude-text">{activeProject.name}</h3>
                <p className="text-[11px] text-claude-textSecondary mt-0.5">{activeProject.path}</p>
              </div>
              <div className="flex items-center gap-1">
                <button onClick={() => handleStartStudio(activeProject)} className={btnSecondary}>
                  <Play size={12} /> Studio
                </button>
                <button onClick={() => handleLoadCompositions(activeProject)} className={btnSecondary}>
                  <RefreshCw size={12} className={loading ? 'animate-spin' : ''} />
                </button>
              </div>
            </div>

            {compositions.length === 0 ? (
              <div className="p-4 bg-yellow-500/5 border border-yellow-500/20 rounded-lg text-center">
                <p className="text-[13px] text-yellow-400">没有找到 Compositions</p>
                <p className="text-[11px] text-claude-textSecondary mt-1">
                  确认 src/Root.tsx 中包含 &lt;Composition&gt; 注册，然后重新生成。你可以点击 Studio 查看。
                </p>
              </div>
            ) : (
              <>
                <div className="space-y-2">
                  {compositions.map(comp => (
                    <button key={comp.id} onClick={() => { setSelectedComposition(comp.id); setFps(comp.fps); }}
                      className={`w-full p-3 rounded-lg border text-left transition-all ${
                        selectedComposition === comp.id
                          ? 'border-[#8B5CF6] bg-[#8B5CF6]/10'
                          : 'border-claude-border hover:border-claude-textSecondary/30'
                      }`}>
                      <div className="flex items-center justify-between">
                        <p className="text-[13px] font-medium text-claude-text">{comp.id}</p>
                        <span className="text-[10px] text-claude-textSecondary bg-claude-hover px-1.5 py-0.5 rounded">
                          {comp.width}×{comp.height}
                        </span>
                      </div>
                      <div className="flex items-center gap-3 mt-1 text-[11px] text-claude-textSecondary">
                        <span>{comp.duration_in_frames} 帧</span>
                        <span>{comp.fps} fps</span>
                        <span>{(comp.duration_in_frames / comp.fps).toFixed(1)} 秒</span>
                      </div>
                    </button>
                  ))}
                </div>

                {/* ── Render controls ── */}
                <div className="p-4 bg-claude-input border border-claude-border rounded-lg space-y-3">
                  <h4 className="text-[13px] font-semibold text-claude-text flex items-center gap-2">
                    <Settings size={13} /> 渲染设置
                  </h4>

                  <div>
                    <label className={labelClass}>输出预设</label>
                    <div className="flex flex-wrap gap-1.5">
                      {RENDER_PRESETS.map((preset, i) => (
                        <button key={preset.label} onClick={() => setSelectedPreset(i)}
                          className={`px-2.5 py-1 rounded text-[11px] font-medium transition-colors ${
                            selectedPreset === i
                              ? 'bg-[#8B5CF6] text-white'
                              : 'bg-claude-hover text-claude-textSecondary hover:text-claude-text'
                          }`}>{preset.label}</button>
                      ))}
                    </div>
                  </div>

                  <div className="flex items-center gap-3">
                    <label className="flex items-center gap-1.5 text-[12px] text-claude-textSecondary">
                      <input type="checkbox" checked={useCustomFps}
                        onChange={e => setUseCustomFps(e.target.checked)} className="rounded" />
                      自定义 FPS
                    </label>
                    {useCustomFps && (
                      <input type="number" value={fps} onChange={e => setFps(Number(e.target.value))}
                        className="w-20 px-2 py-1 bg-claude-input border border-claude-border rounded text-[12px] text-claude-text"
                        min={1} max={120} />
                    )}
                  </div>

                  <div>
                    <label className={labelClass}>输出格式</label>
                    <div className="flex gap-1.5">
                      {OUTPUT_FORMATS.map(fmt => (
                        <button key={fmt.id} onClick={() => setOutputFormat(fmt.id)}
                          className={`px-3 py-1 rounded text-[11px] font-medium transition-colors ${
                            outputFormat === fmt.id
                              ? 'bg-[#8B5CF6] text-white'
                              : 'bg-claude-hover text-claude-textSecondary hover:text-claude-text'
                          }`}>{fmt.label}</button>
                      ))}
                    </div>
                  </div>

                  <div>
                    <label className={labelClass}>输出路径</label>
                    <input value={outputPath} onChange={e => setOutputPath(e.target.value)}
                      placeholder={`${activeProject.path}\\out\\${selectedComposition}.mp4`}
                      className={inputClass} />
                  </div>

                  <div className="flex gap-2 pt-1">
                    <button onClick={handleRender} disabled={rendering || !selectedComposition} className={btnPink}>
                      {rendering ? <Loader2 size={13} className="animate-spin" /> : <Zap size={13} />}
                      {rendering ? '渲染中...' : '开始渲染'}
                    </button>
                    <button onClick={async () => {
                      if (!activeProject || !selectedComposition) return;
                      try {
                        const stillPath = `${activeProject.path}\\out\\${selectedComposition}_frame.png`;
                        await remotionService.still(activeProject.path, selectedComposition, stillPath, 0);
                      } catch (e: unknown) { setError(getErrorMessage(e) || '截图失败'); }
                    }} className={btnSecondary}>
                      <Image size={13} /> 帧截图
                    </button>
                  </div>

                  {renderResult && (
                    <div className={`p-3 rounded-lg border ${
                      renderResult.success
                        ? 'bg-green-500/5 border-green-500/20'
                        : 'bg-red-500/5 border-red-500/20'
                    }`}>
                      <div className="flex items-center gap-2">
                        {renderResult.success ? <Check size={14} className="text-green-400" /> : <X size={14} className="text-red-400" />}
                        <span className={`text-[13px] font-medium ${renderResult.success ? 'text-green-400' : 'text-red-400'}`}>
                          {renderResult.success ? '渲染完成' : '渲染失败'}
                        </span>
                      </div>
                      {renderResult.success && (
                        <div className="mt-2 text-[12px] text-claude-textSecondary space-y-0.5">
                          <p>输出: {renderResult.output_file}</p>
                          <p>耗时: {renderResult.duration_secs.toFixed(1)} 秒</p>
                        </div>
                      )}
                      {renderResult.error && (
                        <p className="mt-1.5 text-[11px] text-red-400 font-mono whitespace-pre-wrap max-h-24 overflow-y-auto">
                          {renderResult.error}
                        </p>
                      )}
                    </div>
                  )}
                </div>
              </>
            )}
          </div>
        )}

      </div>

      {/* ── Footer ── */}
      <div className="px-4 py-2 border-t border-claude-border bg-claude-surface flex items-center justify-between flex-shrink-0">
        <span className="text-[11px] text-claude-textSecondary">
          {studioRunning ? `Studio: :${studioPort}` : 'Ready'}
        </span>
        <span className="text-[11px] text-claude-textSecondary">
          {projects.length} project{projects.length !== 1 ? 's' : ''}
        </span>
      </div>
    </div>
  );
};

export default RemotionMotionPanel;
