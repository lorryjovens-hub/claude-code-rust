import React, { useState, useEffect } from 'react';
import { FolderOpen } from 'lucide-react';
import ClaudeLogo from './ClaudeLogo';

interface OnboardingProps {
  onComplete: () => void;
}

const Onboarding: React.FC<OnboardingProps> = ({ onComplete }) => {
  const [step, setStep] = useState(0);
  const [direction, setDirection] = useState(1); // 1 = forward, -1 = back
  const [animating, setAnimating] = useState(false);

  // Step 0: Theme
  const [theme, setTheme] = useState<'system' | 'light' | 'dark'>(() => {
    return (localStorage.getItem('theme') as any) || 'system';
  });

  // Step 1: Identity
  const [mode, setMode] = useState<'selfhosted' | 'clawparrot' | null>(null);

  // Step 2: Workspace
  const [workspace, setWorkspace] = useState('');

  useEffect(() => {
    // Fetch default workspace path from bridge-server
    fetch('http://127.0.0.1:30080/api/workspace-config')
      .then(r => r.json())
      .then(data => { if (data.defaultDir) setWorkspace(data.defaultDir); })
      .catch(() => {});
    // Shrink window for onboarding
    import('../utils/tauriAPI').then(m => {
      m.tauriAPI.resizeWindow(600, 520);
    }).catch(() => {});
  }, []);

  // Apply theme live
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') root.classList.add('dark');
    else if (theme === 'light') root.classList.remove('dark');
    else {
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) root.classList.add('dark');
      else root.classList.remove('dark');
    }
    localStorage.setItem('theme', theme);
  }, [theme]);

  const goTo = (next: number) => {
    if (animating || next === step) return;
    setDirection(next > step ? 1 : -1);
    setAnimating(true);
    setTimeout(() => { setStep(next); setAnimating(false); }, 300);
  };

  const handleBrowse = async () => {
    const { tauriAPI } = await import('../utils/tauriAPI');
    if (tauriAPI.isTauri) {
      const dir = await tauriAPI.selectDirectory();
      if (dir) setWorkspace(dir);
    }
  };

  const handleFinish = () => {
    // Restore full window size
    import('../utils/tauriAPI').then(m => {
      m.tauriAPI.resizeWindow(1300, 780);
    }).catch(() => {});

    localStorage.setItem('theme', theme);
    localStorage.setItem('user_mode', mode || 'selfhosted');
    localStorage.setItem('onboarding_done', 'true');
    if (mode === 'selfhosted') {
      // Self-hosted: clear any gateway keys, user will configure in settings
      localStorage.removeItem('ANTHROPIC_API_KEY');
      localStorage.removeItem('ANTHROPIC_BASE_URL');
      localStorage.removeItem('gateway_user');
      localStorage.removeItem('auth_token');
    } else if (mode === 'clawparrot') {
      try { import('../utils/tauriAPI').then(m => m.tauriAPI.openExternal('http://127.0.0.1:30090/register')).catch(() => {}); } catch {}
    }
    if (workspace) {
      localStorage.setItem('workspace_path', workspace);
      // Save to bridge-server config (takes effect on next launch)
      fetch('http://127.0.0.1:30080/api/workspace-config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ dir: workspace }),
      }).catch(() => {});
    }
    onComplete();
  };

  const canContinue = step === 0 ? true : step === 1 ? mode !== null : true;

  // Use same static ClaudeLogo as the welcome screen

  const themeCards = [
    { id: 'system' as const, label: '跟随系统',
      preview: (
        <div className="w-full h-[80px] rounded-lg overflow-hidden flex">
          <div className="flex-1 bg-[#F8F8F6] flex items-end p-2"><div className="w-full h-3 rounded bg-[#E8E5DE]"/></div>
          <div className="flex-1 bg-[#2A2A28] flex items-end p-2"><div className="w-full h-3 rounded bg-[#3A3A38]"/></div>
        </div>
      )
    },
    { id: 'light' as const, label: 'Light',
      preview: (
        <div className="w-full h-[80px] rounded-lg bg-[#F8F8F6] flex flex-col justify-end p-2 gap-1.5">
          <div className="w-[70%] h-2.5 rounded bg-[#E8E5DE]"/>
          <div className="w-[45%] h-2.5 rounded bg-[#E8E5DE]"/>
        </div>
      )
    },
    { id: 'dark' as const, label: 'Dark',
      preview: (
        <div className="w-full h-[80px] rounded-lg bg-[#1A1A18] flex flex-col justify-end p-2 gap-1.5">
          <div className="w-[70%] h-2.5 rounded bg-[#2E2E2C]"/>
          <div className="w-[45%] h-2.5 rounded bg-[#2E2E2C]"/>
        </div>
      )
    },
  ];

  return (
    <div className="fixed inset-0 z-[999] bg-claude-bg flex flex-col select-none overflow-hidden">
      <style>{`
        @keyframes onboarding-fade-in {
          from { opacity: 0; transform: translateY(16px); }
          to { opacity: 1; transform: translateY(0); }
        }
        @keyframes onboarding-fade-out-left {
          from { opacity: 1; transform: translateX(0); }
          to { opacity: 0; transform: translateX(-40px); }
        }
        @keyframes onboarding-fade-out-right {
          from { opacity: 1; transform: translateX(0); }
          to { opacity: 0; transform: translateX(40px); }
        }
        .onboarding-step-enter {
          animation: onboarding-fade-in 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards;
        }
        .onboarding-step-exit-left {
          animation: onboarding-fade-out-left 0.25s cubic-bezier(0.4, 0, 1, 1) forwards;
        }
        .onboarding-step-exit-right {
          animation: onboarding-fade-out-right 0.25s cubic-bezier(0.4, 0, 1, 1) forwards;
        }
        .onboarding-card {
          transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
        }
        .onboarding-card:hover {
          transform: translateY(-2px);
        }
        .onboarding-card-selected {
          box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.25), 0 8px 24px -8px rgba(59, 130, 246, 0.1);
        }
        .onboarding-glow {
          background: radial-gradient(ellipse 500px 350px at 50% 35%, rgba(0, 0, 0, 0.015) 0%, transparent 70%);
        }
        .dark .onboarding-glow {
          background: radial-gradient(ellipse 500px 350px at 50% 35%, rgba(255, 255, 255, 0.02) 0%, transparent 70%);
        }
      `}</style>

      {/* Title bar drag zone */}
      <div className="absolute top-0 left-0 right-0 h-[44px] z-10" style={{ WebkitAppRegion: 'drag' } as React.CSSProperties} />

      {/* Subtle background glow */}
      <div className="absolute inset-0 onboarding-glow pointer-events-none" />

      {/* Main content */}
      <div className="flex-1 flex flex-col items-center justify-center px-6 relative">
        {/* Logo + title */}
        <div className="flex flex-col items-center mb-10">
          <div className="w-[42px] h-[42px] mb-4">
            <ClaudeLogo color="#D97757" maxScale={0.15} />
          </div>
          <h1
            className="text-[15px] tracking-[0.12em] uppercase text-claude-textSecondary/60 font-medium"
          >
            欢迎使用 Claude Desktop
          </h1>
        </div>

        {/* Step content with animation */}
        <div className="w-full max-w-[560px] min-h-[280px] flex items-start justify-center">
          <div
            key={step}
            className={animating
              ? (direction > 0 ? 'onboarding-step-exit-left' : 'onboarding-step-exit-right')
              : 'onboarding-step-enter'
            }
            style={{ width: '100%' }}
          >
            {/* ───── Step 0: Theme ───── */}
            {step === 0 && (
              <div className="flex flex-col items-center">
                <h2 className="text-[24px] font-semibold text-claude-text tracking-[-0.02em] mb-1.5">
                  选择外观
                </h2>
                <p className="text-[14px] text-claude-textSecondary mb-7">
                  选择界面主题，之后可以在设置中随时更改。
                </p>
                <div className="flex gap-3 w-full max-w-[420px]">
                  {themeCards.map(t => (
                    <button
                      key={t.id}
                      onClick={() => setTheme(t.id)}
                      className={`onboarding-card flex-1 flex flex-col gap-2.5 p-3 rounded-xl border transition-all ${
                        theme === t.id
                          ? 'border-blue-500/60 onboarding-card-selected bg-claude-bg'
                          : 'border-claude-border/60 hover:border-claude-textSecondary/20 bg-claude-bg'
                      }`}
                    >
                      {t.preview}
                      <span className={`text-[13px] font-medium text-center ${
                        theme === t.id ? 'text-blue-500' : 'text-claude-textSecondary'
                      }`}>
                        {t.label}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* ───── Step 1: Identity ───── */}
            {step === 1 && (
              <div className="flex flex-col items-center">
                <h2 className="text-[24px] font-semibold text-claude-text tracking-[-0.02em] mb-1.5">
                  选择模式
                </h2>
                <p className="text-[14px] text-claude-textSecondary mb-7">
                  选择 API 来源，之后可以在设置中随时切换。
                </p>

                <div className="flex gap-3 w-full">
                  {/* Self-hosted */}
                  <button
                    onClick={() => setMode('selfhosted')}
                    className={`onboarding-card flex-1 flex flex-col p-5 rounded-xl border text-left transition-all ${
                      mode === 'selfhosted'
                        ? 'border-blue-500/60 onboarding-card-selected bg-claude-bg'
                        : 'border-claude-border/60 hover:border-claude-textSecondary/20 bg-claude-bg'
                    }`}
                  >
                    <div className={`w-9 h-9 rounded-lg flex items-center justify-center mb-3 transition-colors ${
                      mode === 'selfhosted' ? 'bg-blue-500/10' : 'bg-claude-hover'
                    }`}>
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={mode === 'selfhosted' ? '#3b82f6' : 'currentColor'} strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" className="text-claude-textSecondary">
                        <rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><circle cx="6" cy="6" r="1" fill="currentColor"/><circle cx="6" cy="18" r="1" fill="currentColor"/>
                      </svg>
                    </div>
                    <span className="text-[15px] font-semibold text-claude-text mb-1">自行部署</span>
                    <span className="text-[12.5px] text-claude-textSecondary/70 leading-relaxed">
                      使用自己的 API Key，可接入 Claude、GPT、GLM、Gemini 等任意兼容模型。
                    </span>
                  </button>

                  {/* Clawparrot */}
                  <button
                    onClick={() => setMode('clawparrot')}
                    className={`onboarding-card flex-1 flex flex-col p-5 rounded-xl border text-left transition-all ${
                      mode === 'clawparrot'
                        ? 'border-blue-500/60 onboarding-card-selected bg-claude-bg'
                        : 'border-claude-border/60 hover:border-claude-textSecondary/20 bg-claude-bg'
                    }`}
                  >
                    <div className={`w-9 h-9 rounded-lg flex items-center justify-center mb-3 transition-colors ${
                      mode === 'clawparrot' ? 'bg-blue-500/10' : 'bg-claude-hover'
                    }`}>
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={mode === 'clawparrot' ? '#3b82f6' : 'currentColor'} strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" className="text-claude-textSecondary">
                        <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
                      </svg>
                    </div>
                    <span className="text-[15px] font-semibold text-claude-text mb-1">Claude Rust 托管</span>
                    <span className="text-[12.5px] text-claude-textSecondary/70 leading-relaxed">
                      Claude Rust API 代理服务 — 价格实惠，即开即用，无需配置。
                    </span>
                  </button>
                </div>

                {/* Mode hints */}
                {mode === 'selfhosted' && (
                  <p className="mt-5 text-[12.5px] text-claude-textSecondary/60 text-center" style={{ animation: 'onboarding-fade-in 0.3s ease' }}>
                    进入应用后，可在设置中添加多个模型渠道和 API Key。
                  </p>
                )}
                {mode === 'clawparrot' && (
                  <p className="mt-5 text-[12.5px] text-claude-textSecondary/60 text-center" style={{ animation: 'onboarding-fade-in 0.3s ease' }}>
                    支持 Opus 4.6、Sonnet 4.6、Haiku 4.5 等模型，继续后将跳转到 Clawparrot 网站登录并选购套餐。
                  </p>
                )}
              </div>
            )}

            {/* ───── Step 2: Workspace ───── */}
            {step === 2 && (
              <div className="flex flex-col items-center">
                <h2 className="text-[24px] font-semibold text-claude-text tracking-[-0.02em] mb-1.5">
                  工作目录
                </h2>
                <p className="text-[14px] text-claude-textSecondary mb-7">
                  Claude 将在此文件夹中读写项目文件。
                </p>

                <div className="w-full max-w-[440px]">
                  <button
                    onClick={handleBrowse}
                    className="onboarding-card w-full flex items-center gap-3 p-4 rounded-xl border border-claude-border/60 hover:border-claude-textSecondary/20 bg-claude-bg text-left transition-all"
                    title="点击浏览"
                  >
                    <div className="w-10 h-10 rounded-lg bg-claude-hover flex items-center justify-center flex-shrink-0">
                      <FolderOpen size={18} className="text-claude-textSecondary" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-[13.5px] text-claude-text truncate">
                        {workspace || '选择文件夹...'}
                      </div>
                      <div className="text-[11.5px] text-claude-textSecondary/50 mt-0.5">
                        点击选择其他路径
                      </div>
                    </div>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-claude-textSecondary/40 flex-shrink-0">
                      <polyline points="9 18 15 12 9 6"/>
                    </svg>
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Bottom navigation */}
      <div className="px-8 pb-8 flex items-center justify-between max-w-[640px] w-full mx-auto">
        <button
          onClick={() => goTo(step - 1)}
          className={`text-[13.5px] text-claude-textSecondary hover:text-claude-text transition-colors py-2 px-3 rounded-lg hover:bg-claude-hover ${step === 0 ? 'invisible' : ''}`}
        >
          上一步
        </button>

        {/* Progress */}
        <div className="flex items-center gap-1.5">
          {[0, 1, 2].map(i => (
            <button
              key={i}
              onClick={() => i < step && goTo(i)}
              className={`h-1.5 rounded-full transition-all duration-500 ease-out ${
                i === step ? 'w-6 bg-claude-text' : i < step ? 'w-1.5 bg-claude-text/25 cursor-pointer hover:bg-claude-text/40' : 'w-1.5 bg-claude-border'
              }`}
            />
          ))}
        </div>

        <button
          onClick={() => { if (step < 2) goTo(step + 1); else handleFinish(); }}
          disabled={!canContinue}
          className={`text-[13.5px] font-medium py-2 px-5 rounded-lg transition-all duration-200 ${
            canContinue
              ? 'text-white bg-[#333] dark:bg-[#e0e0e0] dark:text-[#1a1a1a] hover:bg-[#444] dark:hover:bg-[#ccc] shadow-sm hover:shadow'
              : 'text-claude-textSecondary/40 bg-claude-border/50 cursor-not-allowed'
          }`}
        >
          {step === 2 ? '开始使用' : '继续'}
        </button>
      </div>
    </div>
  );
};

export default Onboarding;
