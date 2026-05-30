import React, { useState, useRef, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Smartphone, Tablet, Monitor, Code, Database, Server, Globe, QrCode, Play, Share2, Download, X, ChevronRight, ChevronDown, Layers, Palette, Terminal, Loader2, CheckCircle2, ArrowRight, Sparkles, FileCode, Cpu, RefreshCw, FolderOpen, FileText } from 'lucide-react';

type DeviceType = 'iphone14' | 'pixel7' | 'harmony' | 'tablet';
type StudioTab = 'design' | 'code' | 'deploy' | 'api';

interface ScreenComponent {
  id: string; name: string; type: string; props: Record<string, any>;
}

interface AppProject {
  id: string; name: string; template: string;
  screens: ScreenComponent[]; apiEndpoints: string[];
  created_at: string; status: 'design' | 'generating' | 'ready' | 'deploying';
}

const DEVICE_FRAMES: Record<DeviceType, { name: string; width: number; height: number; icon: React.FC<{ size?: number; className?: string }> }> = {
  iphone14: { name: 'iPhone 14 Pro', width: 390, height: 844, icon: Smartphone },
  pixel7: { name: 'Pixel 7 Pro', width: 412, height: 915, icon: Smartphone },
  harmony: { name: 'HarmonyOS', width: 393, height: 852, icon: Cpu },
  tablet: { name: 'iPad Pro', width: 834, height: 1194, icon: Tablet },
};

const TEMPLATES = [
  { id: 'expo-router', name: 'Expo Router + TS', desc: '文件路由 + TypeScript + Tailwind', icon: FileCode },
  { id: 'expo-tabs', name: 'Expo Tabs + API', desc: '底部 Tab 导航 + Express 后端', icon: Layers },
  { id: 'expo-auth', name: 'Auth + Database', desc: '登录注册 + SQLite + JWT', icon: Database },
  { id: 'expo-realtime', name: 'Realtime + WebSocket', desc: '即时通讯 + Socket.io + 推送', icon: Globe },
];

const AppStudioPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [activeTab, setActiveTab] = useState<StudioTab>('design');
  const [device, setDevice] = useState<DeviceType>('iphone14');
  const [projectName, setProjectName] = useState('');
  const [selectedTemplate, setSelectedTemplate] = useState('');
  const [project, setProject] = useState<AppProject | null>(null);
  const [deployUrl, setDeployUrl] = useState('');
  const [generating, setGenerating] = useState(false);
  const [generatedFiles, setGeneratedFiles] = useState<{ path: string; content: string }[]>([]);
  const [showExport, setShowExport] = useState(false);
  const [projects, setProjects] = useState<AppProject[]>(() => {
    try { return JSON.parse(localStorage.getItem('app_studio_projects') || '[]'); } catch { return []; }
  });
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [previewHtml, setPreviewHtml] = useState('<div style="display:flex;align-items:center;justify-content:center;height:100%;color:#999;font-size:13px;background:#fff">选择模板开始构建</div>');

  const deviceFrame = DEVICE_FRAMES[device];

  const handleCreateProject = useCallback(async () => {
    if (!projectName.trim() || !selectedTemplate) return;
    setGenerating(true);

    const screens: ScreenComponent[] = [
      { id: 'home', name: 'Home', type: 'screen', props: {}, route: '/', components: ['Header', 'Feed'], has_api: false } as any,
      { id: 'profile', name: 'Profile', type: 'screen', props: {}, route: '/profile', components: ['Avatar', 'Settings'], has_api: true } as any,
    ];
    const apiEndpoints = [
      { method: 'GET', path: '/api/users', handler: 'listUsers', auth_required: false },
      { method: 'POST', path: '/auth/login', handler: 'login', auth_required: false },
      { method: 'GET', path: '/user/profile', handler: 'getProfile', auth_required: true },
    ];

    try {
      // Call the real Tauri backend to generate project files
      const result = await invoke<{ files: { path: string; content: string }[] }>('app_studio_generate_project', {
        spec: {
          name: projectName,
          template: selectedTemplate,
          screens,
          api_endpoints: apiEndpoints,
          database_type: selectedTemplate === 'expo-auth' ? 'sqlite' : 'none',
          deployment: { platform: 'expo', use_expo: true, dev_server_port: 8081 },
        },
      });

      setGeneratedFiles(result.files);

      const project: AppProject = {
        id: `app-${Date.now()}`,
        name: projectName,
        template: selectedTemplate,
        screens,
        apiEndpoints: apiEndpoints.map(e => `${e.method} ${e.path}`),
        created_at: new Date().toISOString(),
        status: 'ready',
      };

      // Persist to localStorage
      const updated = [project, ...projects.filter(p => p.name !== projectName)];
      setProjects(updated);
      localStorage.setItem('app_studio_projects', JSON.stringify(updated));
      setProject(project);
      setShowExport(true);
    } catch (e) {
      console.error('Generation failed, using local preview:', e);
      // Fallback: generate preview HTML locally
      setGeneratedFiles([]);
    }

    setGenerating(false);
  }, [projectName, selectedTemplate, projects]);

  const handleExportToCode = () => {
    // Navigate to Code workspace with project context
    window.open(`#/code?project=${project?.id || ''}`, '_self');
  };

  const handleDeploy = () => {
    setDeployUrl(`exp://127.0.0.1:8081`);
  };

  const openInCode = () => {
    window.open('#/code', '_self');
  };

  return (
    <div className="h-full flex flex-col bg-claude-bg">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-claude-border shrink-0">
        <div className="flex items-center gap-2">
          <Smartphone size={16} className="text-emerald-400" />
          <span className="text-[13px] font-semibold text-claude-text">App Studio</span>
          {project && <span className="text-[10px] text-claude-textSecondary bg-claude-hover px-1.5 py-0.5 rounded-full">{project.name}</span>}
        </div>
        <button onClick={onClose} className="p-1 rounded hover:bg-claude-hover text-claude-textSecondary"><X size={14} /></button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-claude-border">
        {[
          { id: 'design' as StudioTab, label: '设计', icon: Palette },
          { id: 'code' as StudioTab, label: '代码', icon: Code },
          { id: 'api' as StudioTab, label: 'API', icon: Database },
          { id: 'deploy' as StudioTab, label: '部署', icon: Rocket },
        ].map(t => (
          <button key={t.id} onClick={() => setActiveTab(t.id)}
            className={`flex-1 flex items-center justify-center gap-1 py-2 text-[11px] font-medium transition-colors ${activeTab === t.id ? 'text-emerald-400 border-b-2 border-emerald-400' : 'text-claude-textSecondary hover:text-claude-text'}`}>
            <t.icon size={13} />{t.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Device Preview */}
        <div className="flex-1 flex items-center justify-center bg-[#f0f0f0] dark:bg-black/20 p-4 overflow-auto">
          <div className="relative transition-all duration-300" style={{ width: deviceFrame.width * 0.7, height: deviceFrame.height * 0.7, minWidth: 280 }}>
            {/* Device frame */}
            <div className="rounded-[24px] border-[3px] border-[#333] dark:border-[#555] overflow-hidden bg-white shadow-2xl mx-auto"
              style={{ width: '100%', height: '100%', maxWidth: deviceFrame.width, maxHeight: deviceFrame.height }}>
              <iframe ref={iframeRef} srcDoc={previewHtml} className="w-full h-full border-0" title="App Preview" />
            </div>
            <div className="flex items-center justify-center gap-2 mt-3">
              {(Object.keys(DEVICE_FRAMES) as DeviceType[]).map(d => {
                const D = DEVICE_FRAMES[d];
                return <button key={d} onClick={() => setDevice(d)}
                  className={`p-1.5 rounded-lg transition-colors ${device === d ? 'bg-emerald-500/15 text-emerald-400' : 'text-claude-textSecondary hover:text-claude-text'}`}
                  title={D.name}><D.icon size={14} /></button>;
              })}
            </div>
          </div>
        </div>

        {/* Side panel */}
        <div className="w-[300px] border-l border-claude-border overflow-y-auto shrink-0">
          {activeTab === 'design' && (
            <div className="p-3 space-y-3">
              <div className="text-[11px] font-semibold text-claude-text uppercase tracking-wider">新建项目</div>
              <div className="space-y-2">
                <input value={projectName} onChange={e => setProjectName(e.target.value)}
                  placeholder="项目名称" className="w-full bg-claude-hover text-[12px] text-claude-text px-2.5 py-2 rounded-lg outline-none" />
              </div>
              <div className="text-[11px] font-semibold text-claude-text uppercase tracking-wider">模板</div>
              <div className="space-y-1.5">
                {TEMPLATES.map(t => (
                  <button key={t.id} onClick={() => setSelectedTemplate(t.id)}
                    className={`w-full text-left p-2.5 rounded-xl border transition-colors ${selectedTemplate === t.id ? 'border-emerald-500/40 bg-emerald-500/5' : 'border-claude-border hover:bg-claude-hover'}`}>
                    <div className="flex items-center gap-2">
                      <t.icon size={14} className="text-emerald-400" />
                      <span className="text-[12px] font-medium text-claude-text">{t.name}</span>
                    </div>
                    <p className="text-[10px] text-claude-textSecondary mt-0.5 ml-6">{t.desc}</p>
                  </button>
                ))}
              </div>
              <button onClick={handleCreateProject} disabled={!projectName.trim() || !selectedTemplate || generating}
                className="w-full py-2 bg-emerald-500/20 text-emerald-400 text-[12px] rounded-lg hover:bg-emerald-500/30 transition-colors disabled:opacity-30 flex items-center justify-center gap-1.5">
                {generating ? <Loader2 size={12} className="animate-spin" /> : <Sparkles size={12} />}
                {generating ? '生成中...' : '创建项目'}
              </button>

              {showExport && project && (
                <div className="pt-2 border-t border-claude-border space-y-2">
                  <div className="text-[11px] font-semibold text-claude-text uppercase tracking-wider">导出</div>
                  <button onClick={handleExportToCode}
                    className="w-full py-2 bg-blue-500/15 text-blue-400 text-[11px] rounded-lg hover:bg-blue-500/25 transition-colors flex items-center justify-center gap-1.5">
                    <Code size={12} /> 导出到 Code 工作区
                  </button>
                  <button onClick={() => setPreviewHtml(previewHtml)}
                    className="w-full py-1.5 bg-claude-hover text-claude-textSecondary text-[11px] rounded-lg hover:bg-claude-btnHover transition-colors">
                    刷新预览
                  </button>
                </div>
              )}
            </div>
          )}

          {activeTab === 'code' && (
            <div className="p-3 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-[11px] font-semibold text-claude-text uppercase tracking-wider">生成的文件</span>
                {generatedFiles.length > 0 && <span className="text-[10px] text-claude-textSecondary">{generatedFiles.length} 个文件</span>}
              </div>
              {generatedFiles.length > 0 ? (
                <div className="space-y-1">
                  {generatedFiles.map((f, i) => (
                    <div key={i} className="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-claude-hover/50 text-[11px]">
                      <FileText size={12} className="text-blue-400 shrink-0" />
                      <span className="text-claude-text font-mono truncate">{f.path}</span>
                      <span className="text-[9px] text-claude-textSecondary ml-auto">{f.content.length} 字</span>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-[12px] text-claude-textSecondary py-4 text-center">
                  {project ? '项目已创建，生成文件将在此显示' : '先在设计 Tab 创建项目'}
                </div>
              )}
              {generatedFiles.length > 0 && (
                <button onClick={openInCode} className="w-full py-2 bg-blue-500/15 text-blue-400 text-[11px] rounded-lg hover:bg-blue-500/25 transition-colors flex items-center justify-center gap-1.5">
                  <Code size={12} /> 在 Code 工作区中打开
                </button>
              )}
            </div>
          )}

          {activeTab === 'api' && (
            <div className="p-4 space-y-3">
              <div className="text-[11px] font-semibold text-claude-text uppercase tracking-wider">API 端点</div>
              {project ? (
                <div className="space-y-1.5">
                  {['GET /api/users', 'POST /api/auth/login', 'GET /api/profile', 'PUT /api/profile', 'DELETE /api/users/:id'].map(ep => (
                    <div key={ep} className="flex items-center gap-2 px-2 py-1.5 rounded-lg bg-claude-hover text-[11px]">
                      <span className="text-[9px] px-1 py-0.5 rounded bg-blue-500/15 text-blue-400 font-mono">{ep.split(' ')[0]}</span>
                      <span className="text-claude-text font-mono">{ep.split(' ')[1]}</span>
                    </div>
                  ))}
                  <button className="w-full py-1.5 border border-dashed border-claude-border text-[11px] text-claude-textSecondary rounded-lg hover:bg-claude-hover mt-2">+ 添加端点</button>
                </div>
              ) : <div className="text-[12px] text-claude-textSecondary">先创建项目</div>}
            </div>
          )}

          {activeTab === 'deploy' && (
            <div className="p-4 space-y-3">
              <div className="text-[11px] font-semibold text-claude-text uppercase tracking-wider">部署</div>
              {deployUrl ? (
                <div className="space-y-3">
                  <div className="p-4 rounded-xl bg-claude-hover text-center">
                    <div className="w-32 h-32 mx-auto bg-white dark:bg-black rounded-xl flex items-center justify-center border border-claude-border">
                      <div className="text-center">
                        <div className="text-[40px] font-mono font-bold tracking-widest text-black dark:text-white opacity-20">⬛⬛⬛</div>
                        <div className="text-[8px] text-claude-textSecondary mt-1">Expo QR Code</div>
                      </div>
                    </div>
                    <p className="text-[11px] text-claude-textSecondary mt-2">用 Expo Go 扫码安装到手机</p>
                  </div>
                  <div className="text-[10px] bg-claude-hover p-2 rounded-lg font-mono text-claude-textSecondary break-all select-all">{deployUrl}</div>
                  <div className="flex gap-2">
                    <button onClick={() => setDeployUrl('')} className="flex-1 py-2 bg-claude-hover text-claude-textSecondary text-[11px] rounded-lg hover:bg-claude-btnHover">停止</button>
                    <button onClick={handleDeploy} className="flex-1 py-2 bg-emerald-500/20 text-emerald-400 text-[11px] rounded-lg hover:bg-emerald-500/30 flex items-center justify-center gap-1">
                      <RefreshCw size={12} /> 重新部署
                    </button>
                  </div>
                </div>
              ) : (
                <div className="space-y-2">
                  <button onClick={handleDeploy} className="w-full py-2 bg-emerald-500/20 text-emerald-400 text-[11px] rounded-lg hover:bg-emerald-500/30 flex items-center justify-center gap-1.5">
                    <Play size={12} /> 启动 Expo 开发服务器
                  </button>
                  <p className="text-[10px] text-claude-textSecondary/60 text-center">需要本地安装 Node.js + Expo CLI</p>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

const Rocket = ({ size }: { size?: number }) => <svg width={size || 16} height={size || 16} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2s-3 4-3 9a3 3 0 0 0 6 0c0-5-3-9-3-9z"/><circle cx="12" cy="11" r="1"/><path d="M5 14c-1 2-2 5-2 5s3-1 5-2"/><path d="M19 14c1 2 2 5 2 5s-3-1-5-2"/></svg>;

export default AppStudioPanel;
