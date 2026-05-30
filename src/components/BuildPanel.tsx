import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { Play, Square, CheckCircle, XCircle, Loader2, Package, Cog, Terminal } from 'lucide-react';

interface BuildPanelProps {
  workspacePath?: string;
  onBuildComplete?: (success: boolean, output: string) => void;
}

type BuildStatus = 'idle' | 'building' | 'success' | 'failed';

interface ProjectTypeInfo {
  type: string;
  label: string;
  keyFiles: string[];
  buildCommand: string;
  icon: React.ReactNode;
}

const PROJECT_TYPES: ProjectTypeInfo[] = [
  {
    type: 'npm',
    label: 'Node.js',
    keyFiles: ['package.json'],
    buildCommand: 'npm install && npm run build',
    icon: <Package size={14} />,
  },
  {
    type: 'cargo',
    label: 'Rust',
    keyFiles: ['Cargo.toml'],
    buildCommand: 'cargo build --release',
    icon: <Cog size={14} />,
  },
  {
    type: 'pip',
    label: 'Python',
    keyFiles: ['pyproject.toml', 'requirements.txt', 'setup.py'],
    buildCommand: 'pip install -r requirements.txt',
    icon: <Terminal size={14} />,
  },
  {
    type: 'go',
    label: 'Go',
    keyFiles: ['go.mod'],
    buildCommand: 'go build ./...',
    icon: <Terminal size={14} />,
  },
  {
    type: 'make',
    label: 'Make',
    keyFiles: ['Makefile'],
    buildCommand: 'make',
    icon: <Cog size={14} />,
  },
  {
    type: 'maven',
    label: 'Maven',
    keyFiles: ['pom.xml'],
    buildCommand: 'mvn package',
    icon: <Package size={14} />,
  },
  {
    type: 'gradle',
    label: 'Gradle',
    keyFiles: ['build.gradle', 'build.gradle.kts'],
    buildCommand: 'gradle build',
    icon: <Package size={14} />,
  },
  {
    type: 'cmake',
    label: 'CMake',
    keyFiles: ['CMakeLists.txt'],
    buildCommand: 'cmake --build build',
    icon: <Cog size={14} />,
  },
];

const SIMULATED_BUILD_LINES: Record<string, string[]> = {
  npm: [
    '\x1b[36m$ npm install\x1b[0m',
    '\x1b[90mnpm WARN deprecated some-dep@1.0.0: Use v2 instead\x1b[0m',
    'added 142 packages in 3.2s',
    '',
    '\x1b[36m$ npm run build\x1b[0m',
    '\x1b[90m> project@1.0.0 build\x1b[0m',
    '\x1b[90m> vite build\x1b[0m',
    '\x1b[32mvite v6.0.0 building for production...\x1b[0m',
    '\x1b[32m✓ 142 modules transformed.\x1b[0m',
    '\x1b[90mdist/index.html    0.45 kB │ gzip: 0.30 kB\x1b[0m',
    '\x1b[90mdist/assets/app.js  142.10 kB │ gzip: 45.20 kB\x1b[0m',
    '\x1b[90mdist/assets/style.css  12.30 kB │ gzip: 3.10 kB\x1b[0m',
    '\x1b[32m✓ built in 2.14s\x1b[0m',
  ],
  cargo: [
    '\x1b[36m$ cargo build --release\x1b[0m',
    '\x1b[32m   Compiling serde v1.0.200\x1b[0m',
    '\x1b[32m   Compiling tokio v1.38.0\x1b[0m',
    '\x1b[32m   Compiling project-core v0.1.0\x1b[0m',
    '\x1b[32m   Compiling project-app v0.1.0\x1b[0m',
    '\x1b[32m    Finished release [optimized] target(s) in 45.32s\x1b[0m',
  ],
  pip: [
    '\x1b[36m$ pip install -r requirements.txt\x1b[0m',
    '\x1b[90mCollecting requests>=2.28.0\x1b[0m',
    '\x1b[90m  Downloading requests-2.32.3-py3-none-any.whl (63 kB)\x1b[0m',
    '\x1b[90mCollecting flask>=3.0.0\x1b[0m',
    '\x1b[90m  Downloading flask-3.1.0-py3-none-any.whl (102 kB)\x1b[0m',
    '\x1b[32mSuccessfully installed requests-2.32.3 flask-3.1.0 click-8.1.7\x1b[0m',
  ],
  go: [
    '\x1b[36m$ go build ./...\x1b[0m',
    '\x1b[90minternal/pkg/config\x1b[0m',
    '\x1b[90minternal/pkg/database\x1b[0m',
    '\x1b[90mcmd/server\x1b[0m',
    '\x1b[90mcmd/cli\x1b[0m',
  ],
  make: [
    '\x1b[36m$ make\x1b[0m',
    '\x1b[32mgcc -c -o obj/main.o src/main.c\x1b[0m',
    '\x1b[32mgcc -c -o obj/utils.o src/utils.c\x1b[0m',
    '\x1b[32mgcc -o build/app obj/main.o obj/utils.o\x1b[0m',
    '\x1b[32mBuild complete.\x1b[0m',
  ],
  maven: [
    '\x1b[36m$ mvn package\x1b[0m',
    '\x1b[90m[INFO] Scanning for projects...\x1b[0m',
    '\x1b[90m[INFO] Building project 1.0.0\x1b[0m',
    '\x1b[90m[INFO] --- maven-compiler-plugin:3.13.0:compile ---\x1b[0m',
    '\x1b[90m[INFO] Compiling 24 source files to target/classes\x1b[0m',
    '\x1b[32m[INFO] BUILD SUCCESS\x1b[0m',
    '\x1b[90m[INFO] Total time: 8.421 s\x1b[0m',
  ],
  gradle: [
    '\x1b[36m$ gradle build\x1b[0m',
    '\x1b[90m> Task :compileJava\x1b[0m',
    '\x1b[90m> Task :processResources\x1b[0m',
    '\x1b[90m> Task :classes\x1b[0m',
    '\x1b[90m> Task :jar\x1b[0m',
    '\x1b[90m> Task :assemble\x1b[0m',
    '\x1b[32mBUILD SUCCESSFUL in 4s\x1b[0m',
  ],
  cmake: [
    '\x1b[36m$ cmake --build build\x1b[0m',
    '\x1b[90m[ 25%] Building CXX object CMakeFiles/app.dir/main.cpp.o\x1b[0m',
    '\x1b[90m[ 50%] Building CXX object CMakeFiles/app.dir/utils.cpp.o\x1b[0m',
    '\x1b[90m[ 75%] Building CXX object CMakeFiles/app.dir/config.cpp.o\x1b[0m',
    '\x1b[90m[100%] Linking CXX executable app\x1b[0m',
    '\x1b[32mBuild finished\x1b[0m',
  ],
};

const ANIMATION_DELAY_MS = 180;

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function parseAnsiToHtml(text: string): string {
  const escaped = escapeHtml(text);
  let result = escaped;
  result = result.replace(/\x1b\[32m/g, '<span style="color: #a6e3a1">');
  result = result.replace(/\x1b\[36m/g, '<span style="color: #89dceb">');
  result = result.replace(/\x1b\[90m/g, '<span style="color: #585b70">');
  result = result.replace(/\x1b\[31m/g, '<span style="color: #f38ba8">');
  result = result.replace(/\x1b\[33m/g, '<span style="color: #f9e2af">');
  result = result.replace(/\x1b\[0m/g, '</span>');
  const openCount = (result.match(/<span/g) || []).length;
  const closeCount = (result.match(/<\/span>/g) || []).length;
  for (let i = closeCount; i < openCount; i++) {
    result += '</span>';
  }
  return result;
}

const BuildPanel: React.FC<BuildPanelProps> = ({ workspacePath, onBuildComplete }) => {
  const [detectedType, setDetectedType] = useState<string | null>(null);
  const [buildStatus, setBuildStatus] = useState<BuildStatus>('idle');
  const [buildLog, setBuildLog] = useState<string[]>([]);
  const [customCommand, setCustomCommand] = useState('');
  const [elapsedMs, setElapsedMs] = useState(0);
  const [showCustomInput, setShowCustomInput] = useState(false);
  const logContainerRef = useRef<HTMLDivElement>(null);
  const buildStartRef = useRef<number>(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const abortRef = useRef(false);

  const projectInfo = useMemo(() => {
    if (!detectedType) return null;
    return PROJECT_TYPES.find(p => p.type === detectedType) || null;
  }, [detectedType]);

  useEffect(() => {
    if (!workspacePath) {
      setDetectedType(null);
      return;
    }

    const detectProjectType = async () => {
      try {
        setDetectedType(null);
        const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;
        if (isTauri) {
          const { invoke } = await import('@tauri-apps/api/core');
          for (const pt of PROJECT_TYPES) {
            for (const keyFile of pt.keyFiles) {
              const filePath = `${workspacePath.replace(/\\/g, '/')}/${keyFile}`;
              try {
                const exists = await invoke('file_exists', { path: filePath });
                if (exists) {
                  setDetectedType(pt.type);
                  return;
                }
              } catch {
                continue;
              }
            }
          }
        } else {
          await Promise.all(
            PROJECT_TYPES.map(async (pt) => {
              for (const keyFile of pt.keyFiles) {
                try {
                  const url = `${workspacePath}/${keyFile}`;
                  const res = await fetch(url, { method: 'HEAD' });
                  if (res.ok) {
                    setDetectedType(pt.type);
                    throw new Error('DONE_DETECT');
                  }
                } catch (e: unknown) {
                  if (e instanceof Error && e.message === 'DONE_DETECT') throw e;
                  continue;
                }
              }
            })
          ).catch((e: unknown) => {
            if (e instanceof Error && e.message !== 'DONE_DETECT') throw e;
          });
        }
      } catch {
        setDetectedType(null);
      }
    };

    detectProjectType();
  }, [workspacePath]);

  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [buildLog]);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, []);

  const runBuild = useCallback(async () => {
    const command = showCustomInput ? customCommand.trim() : projectInfo?.buildCommand;
    if (!command) return;

    abortRef.current = false;
    setBuildStatus('building');
    setBuildLog([]);
    setElapsedMs(0);
    buildStartRef.current = Date.now();

    timerRef.current = setInterval(() => {
      setElapsedMs(Date.now() - buildStartRef.current);
    }, 100);

    const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;

    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        setBuildLog(prev => [...prev, `\x1b[36m$ ${command}\x1b[0m`, '']);

        const result = await invoke('execute_build_command', {
          command,
          cwd: workspacePath || '.',
        });

        if (timerRef.current) clearInterval(timerRef.current);
        setElapsedMs(Date.now() - buildStartRef.current);

        const output = typeof result === 'string' ? result : '';
        const lines = output.split('\n');
        setBuildLog(prev => [...prev, ...lines]);

        setBuildStatus('success');
        onBuildComplete?.(true, output);
      } catch (err: unknown) {
        if (timerRef.current) clearInterval(timerRef.current);
        setElapsedMs(Date.now() - buildStartRef.current);

        const errorMsg = err instanceof Error ? err.message : String(err);
        setBuildLog(prev => [...prev, '', `\x1b[31mError: ${errorMsg}\x1b[0m`]);

        setBuildStatus('failed');
        onBuildComplete?.(false, errorMsg);
      }
      return;
    }

    const simulateLines = projectInfo?.type
      ? SIMULATED_BUILD_LINES[projectInfo.type] || [
          `\x1b[36m$ ${command}\x1b[0m`,
          '\x1b[32mBuild completed successfully.\x1b[0m',
        ]
      : [
          `\x1b[36m$ ${command}\x1b[0m`,
          '\x1b[32mBuild completed successfully.\x1b[0m',
        ];

    setBuildLog([`\x1b[36m$ ${command}\x1b[0m`, '']);

    for (let i = 0; i < simulateLines.length; i++) {
      if (abortRef.current) break;
      await new Promise(resolve => setTimeout(resolve, ANIMATION_DELAY_MS));
      setBuildLog(prev => [...prev, simulateLines[i]]);
    }

    if (timerRef.current) clearInterval(timerRef.current);
    setElapsedMs(Date.now() - buildStartRef.current);

    if (!abortRef.current) {
      const fullOutput = simulateLines.join('\n');
      setBuildStatus('success');
      onBuildComplete?.(true, fullOutput);
    }
  }, [showCustomInput, customCommand, projectInfo, workspacePath, onBuildComplete]);

  const stopBuild = useCallback(() => {
    abortRef.current = true;
    if (timerRef.current) clearInterval(timerRef.current);
    setElapsedMs(Date.now() - buildStartRef.current);
    setBuildLog(prev => [...prev, '', '\x1b[33mBuild cancelled by user.\x1b[0m']);
    setBuildStatus('failed');
    onBuildComplete?.(false, 'Build cancelled');
  }, [onBuildComplete]);

  const formatTime = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    const seconds = (ms / 1000).toFixed(1);
    return `${seconds}s`;
  };

  const renderStatusBadge = () => {
    switch (buildStatus) {
      case 'idle':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-[11px] font-medium rounded-full bg-[#313244] text-[#6c7086]">
            <div className="w-1.5 h-1.5 rounded-full bg-[#6c7086]" />
            Idle
          </span>
        );
      case 'building':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-[11px] font-medium rounded-full bg-[#1e1e32] text-[#f9e2af]">
            <Loader2 size={12} className="animate-spin text-[#f9e2af]" />
            Building
          </span>
        );
      case 'success':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-[11px] font-medium rounded-full bg-[#1a2e1a] text-[#a6e3a1]">
            <CheckCircle size={12} className="text-[#a6e3a1]" />
            Success · {formatTime(elapsedMs)}
          </span>
        );
      case 'failed':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-[11px] font-medium rounded-full bg-[#2e1a1a] text-[#f38ba8]">
            <XCircle size={12} className="text-[#f38ba8]" />
            Failed · {formatTime(elapsedMs)}
          </span>
        );
    }
  };

  const canBuild = buildStatus !== 'building' && (showCustomInput ? customCommand.trim().length > 0 : !!projectInfo);

  return (
    <div className="flex flex-col h-full bg-[#1a1a2e]">
      <div className="flex items-center justify-between px-4 py-2.5 bg-[#16213e] border-b border-[#2d2d44] shrink-0">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <Package size={16} className="text-[#d97706]" />
            <span className="text-[13px] font-semibold text-[#e0e0e8]">Build</span>
          </div>
          {projectInfo && !showCustomInput && (
            <span className="inline-flex items-center gap-1.5 px-2 py-0.5 text-[11px] font-medium rounded-md bg-[#d977061a] text-[#d97706] border border-[#d9770633]">
              {projectInfo.icon}
              {projectInfo.label}
            </span>
          )}
          {showCustomInput && (
            <span className="inline-flex items-center gap-1.5 px-2 py-0.5 text-[11px] font-medium rounded-md bg-[#3b82f61a] text-[#3b82f6] border border-[#3b82f633]">
              <Cog size={14} />
              Custom
            </span>
          )}
          {renderStatusBadge()}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => {
              setShowCustomInput(!showCustomInput);
              setCustomCommand('');
              setBuildStatus('idle');
              setBuildLog([]);
            }}
            disabled={buildStatus === 'building'}
            className={`p-1.5 rounded-md text-[#6c7086] hover:text-[#e0e0e8] hover:bg-[#1a1a2e] transition-colors disabled:opacity-40 disabled:cursor-not-allowed ${showCustomInput ? 'bg-[#1a1a2e] text-[#e0e0e8]' : ''}`}
            title="Custom command"
          >
            <Terminal size={15} />
          </button>
        </div>
      </div>

      {showCustomInput && (
        <div className="px-4 py-2 bg-[#16213e] border-b border-[#2d2d44] shrink-0">
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={customCommand}
              onChange={e => setCustomCommand(e.target.value)}
              placeholder="Enter custom build command..."
              disabled={buildStatus === 'building'}
              className="flex-1 px-3 py-1.5 text-[13px] bg-[#1a1a2e] text-[#e0e0e8] border border-[#2d2d44] rounded-md font-mono placeholder-[#585b70] focus:outline-none focus:border-[#d97706] transition-colors disabled:opacity-40"
              onKeyDown={e => {
                if (e.key === 'Enter' && canBuild) runBuild();
              }}
            />
          </div>
        </div>
      )}

      <div className="flex items-center justify-between px-4 py-2 shrink-0">
        {projectInfo && !showCustomInput && (
          <span className="text-[11px] text-[#585b70] font-mono">{projectInfo.buildCommand}</span>
        )}
        {(!projectInfo && !showCustomInput) && (
          <span className="text-[11px] text-[#585b70]">
            {workspacePath ? 'No recognized project detected' : 'Select a workspace to begin'}
          </span>
        )}
        <div className="flex items-center gap-2">
          {buildStatus === 'building' ? (
            <button
              onClick={stopBuild}
              className="flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium rounded-md bg-[#f38ba81a] text-[#f38ba8] border border-[#f38ba833] hover:bg-[#f38ba826] transition-colors"
            >
              <Square size={12} fill="currentColor" />
              Stop
            </button>
          ) : (
            <button
              onClick={runBuild}
              disabled={!canBuild}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium rounded-md transition-colors ${
                canBuild
                  ? 'bg-[#d97706] text-[#1a1a2e] hover:bg-[#f59e0b]'
                  : 'bg-[#313244] text-[#585b70] cursor-not-allowed'
              }`}
            >
              <Play size={12} fill="currentColor" />
              Build
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-hidden px-4 pb-4 pt-1 min-h-0">
        <div
          ref={logContainerRef}
          className="h-full w-full bg-[#0d1117] border border-[#2d2d44] rounded-lg overflow-y-auto p-4 font-mono text-[13px] leading-[1.7] whitespace-pre-wrap break-all"
        >
          {buildLog.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-[#585b70] select-none">
              <Terminal size={28} className="mb-3 opacity-40" />
              <span className="text-[12px]">Build output will appear here</span>
            </div>
          ) : (
            buildLog.map((line, i) => (
              <div
                key={i}
                dangerouslySetInnerHTML={{ __html: parseAnsiToHtml(line) || '&nbsp;' }}
                className="min-h-[1.7em]"
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
};

export default BuildPanel;