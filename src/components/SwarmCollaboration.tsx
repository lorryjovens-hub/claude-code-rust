import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useI18n } from '../hooks/useI18n';
import {
  Brain,
  Users,
  Zap,
  Loader2,
  CheckCircle2,
  XCircle,
  Clock,
  ArrowRight,
  Split,
  GitBranch,
  Activity,
  Target,
  Sparkles,
  ChevronDown,
  ChevronUp,
  Play,
  Pause,
  StopCircle
} from 'lucide-react';
import { multiagentWorkflowStream } from '../api';

export interface SwarmAgent {
  id: string;
  name: string;
  role: string;
  state: 'idle' | 'planning' | 'executing' | 'synthesizing' | 'completed' | 'failed';
  progress: number;
  assignedTask?: string;
  color: string;
  icon: string;
  durationMs: number;
  tokensUsed: number;
}

export interface SubTask {
  id: string;
  description: string;
  agentId?: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  progress: number;
  dependencies: string[];
  output?: string;
}

export interface SwarmSession {
  id: string;
  task: string;
  complexity: number;
  agents: SwarmAgent[];
  subTasks: SubTask[];
  status: 'analyzing' | 'planning' | 'running' | 'paused' | 'completed' | 'failed';
  startTime: number;
  endTime?: number;
  totalDuration: number;
}

const AGENT_ROLES = [
  { role: 'Planner', icon: '', color: '#3B82F6' },
  { role: 'Researcher', icon: '', color: '#8B5CF6' },
  { role: 'Developer', icon: '', color: '#10B981' },
  { role: 'Reviewer', icon: '', color: '#F59E0B' },
  { role: 'Writer', icon: '', color: '#EC4899' },
  { role: 'Architect', icon: '', color: '#06B6D4' },
  { role: 'Analyst', icon: '', color: '#F97316' },
  { role: 'Designer', icon: '', color: '#A855F7' },
];

const COMPLEXITY_THRESHOLD = 7;

function generateAgentId(role: string, index: number): string {
  return `${role.toLowerCase()}_${index}`;
}

function analyzeComplexity(task: string): number {
  let score = 0;
  const length = task.length;
  if (length > 100) score += 2;
  else if (length > 50) score += 1;

  const complexKeywords = ['implement', 'build', 'create', 'design', 'architecture', 'refactor', 'optimize', 'integrate', 'deploy', 'migrate', 'system', 'pipeline', 'workflow', 'automation'];
  const lowerTask = task.toLowerCase();
  for (const keyword of complexKeywords) {
    if (lowerTask.includes(keyword)) score += 1;
  }

  const questionMarks = (task.match(/\?/g) || []).length;
  score += Math.min(questionMarks, 3);

  const andCount = (task.match(/\band\b/gi) || []).length;
  const plusCount = (task.match(/\+/g) || []).length;
  score += Math.min(andCount + plusCount, 4);

  return Math.min(score, 10);
}

function splitTask(task: string, complexity: number): SubTask[] {
  const subTasks: SubTask[] = [];
  const baseId = `task_${Date.now()}`;

  subTasks.push({
    id: `${baseId}_plan`,
    description: 'Analyze requirements and create execution plan',
    status: 'pending' as const,
    progress: 0,
    dependencies: [],
  });

  if (complexity >= 5) {
    subTasks.push({
      id: `${baseId}_research`,
      description: 'Research and gather information for task',
      status: 'pending' as const,
      progress: 0,
      dependencies: [`${baseId}_plan`],
    });
  }

  subTasks.push({
    id: `${baseId}_implement`,
    description: 'Implement core solution based on plan',
    status: 'pending' as const,
    progress: 0,
    dependencies: complexity >= 5 ? [`${baseId}_plan`, `${baseId}_research`] : [`${baseId}_plan`],
  });

  if (complexity >= 6) {
    subTasks.push({
      id: `${baseId}_test`,
      description: 'Test and validate implementation',
      status: 'pending' as const,
      progress: 0,
      dependencies: [`${baseId}_implement`],
    });
  }

  subTasks.push({
    id: `${baseId}_review`,
    description: 'Review and polish final output',
    status: 'pending' as const,
    progress: 0,
    dependencies: complexity >= 6 ? [`${baseId}_test`] : [`${baseId}_implement`],
  });

  if (complexity >= 8) {
    subTasks.push({
      id: `${baseId}_optimize`,
      description: 'Optimize performance and edge cases',
      status: 'pending' as const,
      progress: 0,
      dependencies: [`${baseId}_review`],
    });
  }

  return subTasks;
}

function assignAgents(subTasks: SubTask[], complexity: number): SwarmAgent[] {
  const agentCount = Math.min(Math.max(complexity, 3), AGENT_ROLES.length);
  const agents: SwarmAgent[] = [];

  for (let i = 0; i < agentCount; i++) {
    const roleConfig = AGENT_ROLES[i % AGENT_ROLES.length];
    agents.push({
      id: generateAgentId(roleConfig.role, i),
      name: `${roleConfig.role} #${i + 1}`,
      role: roleConfig.role,
      state: 'idle' as const,
      progress: 0,
      color: roleConfig.color,
      icon: roleConfig.icon,
      durationMs: 0,
      tokensUsed: 0,
    });
  }

  for (let i = 0; i < subTasks.length; i++) {
    const agentIndex = i % agentCount;
    subTasks[i].agentId = agents[agentIndex].id;
  }

  return agents;
}

const AgentNode: React.FC<{ agent: SwarmAgent; isExpanded: boolean }> = ({ agent, isExpanded }) => {
  const { t } = useI18n();
  const stateColors: Record<string, string> = {
    idle: 'bg-gray-500',
    planning: 'bg-blue-500',
    executing: 'bg-green-500',
    synthesizing: 'bg-purple-500',
    completed: 'bg-emerald-500',
    failed: 'bg-red-500',
  };

  const stateLabels: Record<string, string> = {
    idle: t('swarm.idle'),
    planning: t('swarm.pending'),
    executing: t('swarm.running'),
    synthesizing: 'Synthesizing',
    completed: t('swarm.completed'),
    failed: t('swarm.failed'),
  };

  return (
    <div
      className="relative rounded-xl border transition-all duration-300 overflow-hidden"
      style={{
        borderColor: `${agent.color}40`,
        backgroundColor: `${agent.color}08`,
      }}
    >
      <div className="p-3">
        <div className="flex items-center gap-2">
          <span className="text-lg">{agent.icon}</span>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="text-[13px] font-medium text-claude-text truncate">
                {agent.name}
              </span>
              <span className={`w-2 h-2 rounded-full ${stateColors[agent.state]} ${agent.state === 'executing' ? 'animate-pulse' : ''}`} />
            </div>
            <div className="text-[11px] text-claude-textSecondary mt-0.5">
              {stateLabels[agent.state]}
            </div>
          </div>
        </div>

        {agent.assignedTask && (
          <div className="mt-2 px-2 py-1 rounded bg-black/10 dark:bg-white/5">
            <div className="text-[11px] text-claude-textSecondary truncate">
              {agent.assignedTask}
            </div>
          </div>
        )}

        {agent.state !== 'idle' && agent.state !== 'completed' && agent.state !== 'failed' && (
          <div className="mt-2">
            <div className="h-1 rounded-full bg-claude-border overflow-hidden">
              <div
                className="h-full rounded-full transition-all duration-500"
                style={{ width: `${agent.progress}%`, backgroundColor: agent.color }}
              />
            </div>
            <div className="text-[10px] text-claude-textSecondary mt-1 text-right">
              {Math.round(agent.progress)}%
            </div>
          </div>
        )}

        {isExpanded && (
          <div className="mt-2 pt-2 border-t border-claude-border/50">
            <div className="grid grid-cols-2 gap-2 text-[11px]">
              <div className="text-claude-textSecondary">
                <span className="text-claude-text">{t('swarm.duration')}</span>
                <div>{agent.durationMs > 0 ? `${(agent.durationMs / 1000).toFixed(1)}s` : '-'}</div>
              </div>
              <div className="text-claude-textSecondary">
                <span className="text-claude-text">Tokens</span>
                <div>{agent.tokensUsed > 0 ? agent.tokensUsed.toLocaleString() : '-'}</div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

const TaskNode: React.FC<{ task: SubTask; agents: SwarmAgent[] }> = ({ task, agents }) => {
  const { t } = useI18n();
  const statusIcons: Record<string, React.ReactNode> = {
    pending: <Clock size={12} className="text-gray-400" />,
    in_progress: <Loader2 size={12} className="text-blue-400 animate-spin" />,
    completed: <CheckCircle2 size={12} className="text-emerald-400" />,
    failed: <XCircle size={12} className="text-red-400" />,
  };

  const assignedAgent = task.agentId ? agents.find(a => a.id === task.agentId) : null;

  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-claude-hover/50">
      {statusIcons[task.status]}
      <div className="flex-1 min-w-0">
        <div className="text-[12px] text-claude-text truncate">{task.description}</div>
        {assignedAgent && (
          <div className="text-[10px] text-claude-textSecondary mt-0.5 flex items-center gap-1">
            <span>{assignedAgent.icon}</span>
            <span>{assignedAgent.name}</span>
          </div>
        )}
      </div>
      {task.dependencies.length > 0 && (
        <div className="flex items-center gap-1">
          <GitBranch size={10} className="text-claude-textSecondary" />
          <span className="text-[10px] text-claude-textSecondary">{task.dependencies.length}</span>
        </div>
      )}
    </div>
  );
};

const SwarmVisualization: React.FC<{ agents: SwarmAgent[] }> = ({ agents }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number | undefined>(undefined);
  const particlesRef = useRef<Array<{ x: number; y: number; vx: number; vy: number; targetX: number; targetY: number; color: string; life: number }>>([]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resize = () => {
      const rect = canvas.parentElement?.getBoundingClientRect();
      if (rect) {
        canvas.width = rect.width;
        canvas.height = rect.height;
      }
    };
    resize();
    window.addEventListener('resize', resize);

    const activeAgents = agents.filter(a => a.state === 'executing' || a.state === 'planning' || a.state === 'synthesizing');

    particlesRef.current = activeAgents.map(agent => {
      const angle = Math.random() * Math.PI * 2;
      const radius = 30 + Math.random() * 50;
      const cx = canvas.width / 2;
      const cy = canvas.height / 2;
      return {
        x: cx + Math.cos(angle) * radius,
        y: cy + Math.sin(angle) * radius,
        vx: (Math.random() - 0.5) * 0.5,
        vy: (Math.random() - 0.5) * 0.5,
        targetX: cx + Math.cos(angle) * radius,
        targetY: cy + Math.sin(angle) * radius,
        color: agent.color,
        life: 1,
      };
    });

    const animate = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const cx = canvas.width / 2;
      const cy = canvas.height / 2;

      for (const p of particlesRef.current) {
        p.x += p.vx + (p.targetX - p.x) * 0.02;
        p.y += p.vy + (p.targetY - p.y) * 0.02;
        p.vx += (Math.random() - 0.5) * 0.1;
        p.vy += (Math.random() - 0.5) * 0.1;
        p.vx *= 0.95;
        p.vy *= 0.95;

        ctx.beginPath();
        ctx.arc(p.x, p.y, 3, 0, Math.PI * 2);
        ctx.fillStyle = p.color + '80';
        ctx.fill();

        ctx.beginPath();
        ctx.moveTo(p.x, p.y);
        ctx.lineTo(cx, cy);
        ctx.strokeStyle = p.color + '20';
        ctx.lineWidth = 1;
        ctx.stroke();
      }

      ctx.beginPath();
      ctx.arc(cx, cy, 8, 0, Math.PI * 2);
      ctx.fillStyle = '#3B82F6';
      ctx.fill();
      ctx.beginPath();
      ctx.arc(cx, cy, 12, 0, Math.PI * 2);
      ctx.strokeStyle = '#3B82F640';
      ctx.lineWidth = 2;
      ctx.stroke();

      animationRef.current = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      window.removeEventListener('resize', resize);
      if (animationRef.current) cancelAnimationFrame(animationRef.current);
    };
  }, [agents]);

  return (
    <canvas
      ref={canvasRef}
      className="w-full h-32 rounded-lg"
      style={{ backgroundColor: 'transparent' }}
    />
  );
};

const SwarmCollaboration: React.FC = () => {
  const { t } = useTranslation();
  const [inputTask, setInputTask] = useState('');
  const [session, setSession] = useState<SwarmSession | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());
  const [showVisualization, setShowVisualization] = useState(true);
  const [isPaused, setIsPaused] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);

  const handleAnalyzeTask = useCallback(async () => {
    if (!inputTask.trim()) return;

    console.log('[Swarm] ========== Task Analysis Start ==========');
    console.log('[Swarm] Input task:', inputTask);
    setIsAnalyzing(true);
    await new Promise(resolve => setTimeout(resolve, 1000));

    const complexity = analyzeComplexity(inputTask);
    console.log('[Swarm] Complexity analysis result:', complexity);
    console.log('[Swarm] COMPLEXITY_THRESHOLD:', COMPLEXITY_THRESHOLD);
    console.log('[Swarm] Will use swarm mode:', complexity >= COMPLEXITY_THRESHOLD);
    const isComplex = complexity >= COMPLEXITY_THRESHOLD;

    const subTasks: SubTask[] = isComplex ? splitTask(inputTask, complexity) : [{
      id: `task_${Date.now()}`,
      description: inputTask,
      status: 'pending' as const,
      progress: 0,
      dependencies: [],
    }];

    console.log('[Swarm] Sub-tasks generated:', subTasks.length);
    subTasks.forEach((st, i) => {
      console.log(`[Swarm]   Sub-task [${i}]: ${st.description} (deps: ${st.dependencies.join(', ') || 'none'})`);
    });

    const agents: SwarmAgent[] = isComplex ? assignAgents(subTasks, complexity) : [{
      id: 'single_agent',
      name: 'Single Agent',
      role: 'General',
      state: 'idle' as const,
      progress: 0,
      color: '#3B82F6',
      icon: '',
      durationMs: 0,
      tokensUsed: 0,
    }];

    console.log('[Swarm] Agents assigned:', agents.length);
    agents.forEach((agent, i) => {
      console.log(`[Swarm]   Agent [${i}]: ${agent.name} (${agent.role}) - color: ${agent.color}`);
    });

    const newSession: SwarmSession = {
      id: `session_${Date.now()}`,
      task: inputTask,
      complexity,
      agents,
      subTasks,
      status: 'planning',
      startTime: Date.now(),
      totalDuration: 0,
    };

    console.log('[Swarm] Session created:', newSession.id);
    console.log('[Swarm] ========== Task Analysis Complete ==========');
    setSession(newSession);
    setIsAnalyzing(false);
  }, [inputTask]);

  const handleStartSwarm = useCallback(async () => {
    if (!session) return;

    setSession(prev => prev ? { ...prev, status: 'running' } : null);

    const abortController = new AbortController();

    const agentMap = new Map<string, SwarmAgent>();
    session.agents.forEach(a => agentMap.set(a.id, a));

    multiagentWorkflowStream(
      session.task,
      undefined,
      '',
      (event) => {
        const eventType = event.type as string;
        const agentRole = event.agent_role as string | undefined;
        const phase = event.phase as string | undefined;
        const taskId = event.task_id as string | undefined;
        const message = event.message as string;
        const text = event.text as string | undefined;
        const data = event.data as any;

        if (eventType === 'agent_phase') {
          setSession(prev => {
            if (!prev) return prev;
            const updatedAgents = prev.agents.map(a => {
              if (agentRole && a.role === agentRole) {
                const newState: SwarmAgent['state'] = phase === 'execution' ? 'executing' :
                  phase === 'synthesizing' ? 'synthesizing' :
                  phase === 'complete' ? 'completed' :
                  'planning';
                return { ...a, state: newState, assignedTask: message };
              }
              return a;
            });
            return { ...prev, agents: updatedAgents };
          });
        } else if (eventType === 'agent_task_start' && taskId && data) {
          setSession(prev => {
            if (!prev) return prev;
            const agentId = `agent_${data.agent_role?.toString().toLowerCase() || taskId}`;
            const existingAgent = prev.agents.find(a => a.role === data.agent_role || a.id === agentId);
            const subTaskId = taskId;

            let newSubTasks = prev.subTasks.map(st =>
              st.id === subTaskId ? { ...st, status: 'in_progress' as const, agentId: existingAgent?.id } : st
            );

            const hasSubTask = newSubTasks.some(st => st.id === subTaskId);
            if (!hasSubTask) {
              newSubTasks = [...newSubTasks, {
                id: subTaskId,
                description: data.task_name || message,
                agentId: existingAgent?.id,
                status: 'in_progress' as const,
                progress: 0,
                dependencies: data.dependencies || [],
              }];
            }

            let newAgents = prev.agents.map(a => {
              if (existingAgent && a.id === existingAgent.id) {
                return { ...a, state: 'executing' as const, assignedTask: data.task_name || message };
              }
              return a;
            });

            return { ...prev, agents: newAgents, subTasks: newSubTasks };
          });
        } else if (eventType === 'agent_task_done' && taskId && data) {
          setSession(prev => {
            if (!prev) return prev;
            const newSubTasks = prev.subTasks.map(st =>
              st.id === taskId ? { ...st, status: 'completed' as const, progress: 100, output: data.output || text } : st
            );

            const newAgents = prev.agents.map(a => {
              if (a.assignedTask === data.task_name || a.id === newSubTasks.find(st => st.id === taskId)?.agentId) {
                return {
                  ...a,
                  state: 'completed' as const,
                  progress: 100,
                  durationMs: data.duration_ms || a.durationMs,
                  tokensUsed: a.tokensUsed + (data.output?.length || 0) / 4,
                };
              }
              return a;
            });

            return { ...prev, agents: newAgents, subTasks: newSubTasks };
          });
        } else if (eventType === 'agent_task_failed' && taskId) {
          setSession(prev => {
            if (!prev) return prev;
            const newSubTasks = prev.subTasks.map(st =>
              st.id === taskId ? { ...st, status: 'failed' as const } : st
            );
            return { ...prev, subTasks: newSubTasks };
          });
        } else if (eventType === 'agent_output' && text) {
          setSession(prev => {
            if (!prev) return prev;
            const lastSubTask = prev.subTasks[prev.subTasks.length - 1];
            if (lastSubTask) {
              const newSubTasks = prev.subTasks.map((st, i) =>
                i === prev.subTasks.length - 1
                  ? { ...st, output: (st.output || '') + '\n' + text }
                  : st
              );
              return { ...prev, subTasks: newSubTasks };
            }
            return prev;
          });
        } else if (eventType === 'agent_error') {
          console.error('[Swarm] Agent error:', message);
        }
      },
      () => {
        setSession(prev => prev ? {
          ...prev,
          status: 'completed',
          endTime: Date.now(),
          totalDuration: Date.now() - prev.startTime,
          agents: prev.agents.map(a => ({ ...a, state: 'completed' as const, progress: 100 })),
          subTasks: prev.subTasks.map(st => ({ ...st, status: 'completed' as const, progress: 100 })),
        } : null);
        setIsPaused(false);
      },
      (error) => {
        console.error('[Swarm] Stream error:', error);
        setSession(prev => prev ? { ...prev, status: 'failed' } : null);
      },
      abortController.signal,
    );
  }, [session]);

  const handlePauseSwarm = useCallback(() => {
    setIsPaused(prev => !prev);
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = undefined;
    }
    if (!isPaused) {
      handleStartSwarm();
    }
  }, [isPaused, handleStartSwarm]);

  const handleStopSwarm = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = undefined;
    }
    setSession(prev => prev ? { ...prev, status: 'completed' as const } : null);
    setIsPaused(false);
  }, []);

  useEffect(() => {
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  const toggleAgentExpand = useCallback((agentId: string) => {
    setExpandedAgents(prev => {
      const next = new Set(prev);
      if (next.has(agentId)) next.delete(agentId);
      else next.add(agentId);
      return next;
    });
  }, []);

  const completedAgents = session?.agents.filter(a => a.state === 'completed').length || 0;
  const totalAgents = session?.agents.length || 0;
  const progressPercent = totalAgents > 0 ? (completedAgents / totalAgents) * 100 : 0;

  return (
    <div className="flex flex-col h-full bg-claude-bg">
      <div className="flex-shrink-0 p-4 border-b border-claude-border">
        <div className="flex items-center gap-2 mb-3">
          <Users size={18} className="text-[#3B82F6]" />
          <h2 className="text-[15px] font-semibold text-claude-text">{t('swarm.title')}</h2>
          <Sparkles size={14} className="text-[#A855F7] ml-auto" />
        </div>

        <div className="relative">
          <textarea
            value={inputTask}
            onChange={(e) => setInputTask(e.target.value)}
            placeholder={t('swarm.taskDesc')}
            className="w-full px-3 py-2 text-[13px] bg-claude-input border border-claude-border rounded-lg text-claude-text placeholder:text-claude-textSecondary/50 focus:outline-none focus:border-[#3B82F6] resize-none"
            rows={3}
            disabled={!!session && session.status === 'running'}
          />
          <div className="flex items-center justify-between mt-2">
            <div className="flex items-center gap-2">
              <Brain size={14} className="text-claude-textSecondary" />
              <span className="text-[11px] text-claude-textSecondary">
                {isAnalyzing ? t('swarm.analyzeComplexity') : inputTask ? `Complexity: ${analyzeComplexity(inputTask)}/10` : t('swarm.enterTask')}
              </span>
            </div>
            {!session || session.status === 'completed' || session.status === 'failed' ? (
              <button
                onClick={handleAnalyzeTask}
                disabled={!inputTask.trim() || isAnalyzing}
                className="px-3 py-1.5 text-[12px] font-medium text-white bg-[#3B82F6] hover:bg-[#2563EB] rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
              >
                {isAnalyzing ? <Loader2 size={14} className="animate-spin" /> : <Zap size={14} />}
                {isAnalyzing ? t('swarm.analyzing') : t('swarm.analyzePlan')}
              </button>
            ) : (
              <div className="flex items-center gap-1.5">
                {session.status === 'running' && (
                  <>
                    <button
                      onClick={handlePauseSwarm}
                      className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-md transition-colors"
                      title={isPaused ? t('agent.run') : t('agent.stop')}
                    >
                      {isPaused ? <Play size={14} /> : <Pause size={14} />}
                    </button>
                    <button
                      onClick={handleStopSwarm}
                      className="p-1.5 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-md transition-colors"
                      title={t('agent.stop')}
                    >
                      <StopCircle size={14} />
                    </button>
                  </>
                )}
                {session.status === 'planning' && (
                  <button
                    onClick={handleStartSwarm}
                    className="px-3 py-1.5 text-[12px] font-medium text-white bg-[#10B981] hover:bg-[#059669] rounded-lg transition-colors flex items-center gap-1.5"
                  >
                    <Play size={14} />
                    {t('swarm.startSwarm')}
                  </button>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {session && (
        <div className="flex-1 overflow-y-auto">
          <div className="p-4 space-y-4">
            <div className="flex items-center justify-between px-1">
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-1.5">
                  <Target size={14} className="text-[#3B82F6]" />
                  <span className="text-[12px] text-claude-text">Complexity</span>
                </div>
                <div className="flex items-center gap-1">
                  {Array.from({ length: 10 }).map((_, i) => (
                    <div
                      key={i}
                      className="w-2 h-2 rounded-full transition-colors"
                      style={{
                        backgroundColor: i < session.complexity ? '#3B82F6' : 'rgba(107, 114, 128, 0.2)',
                      }}
                    />
                  ))}
                </div>
                <span className="text-[12px] font-medium text-[#3B82F6]">{session.complexity}/10</span>
              </div>
              <div className="flex items-center gap-1.5">
                <Activity size={14} className="text-claude-textSecondary" />
                <span className="text-[12px] text-claude-textSecondary">
                  {completedAgents}/{totalAgents} agents done
                </span>
              </div>
            </div>

            <div className="h-1 rounded-full bg-claude-border overflow-hidden">
              <div
                className="h-full rounded-full bg-gradient-to-r from-[#3B82F6] to-[#10B981] transition-all duration-500"
                style={{ width: `${progressPercent}%` }}
              />
            </div>

            {showVisualization && session.status === 'running' && (
              <div className="rounded-lg border border-claude-border overflow-hidden">
                <SwarmVisualization agents={session.agents} />
              </div>
            )}

            <div>
              <button
                onClick={() => setShowVisualization(prev => !prev)}
                className="flex items-center gap-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text transition-colors mb-2"
              >
                {showVisualization ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                {showVisualization ? t('common.hide') : t('common.show')} {t('swarm.nodes')} ({totalAgents})
              </button>
              {showVisualization && (
                <div className="grid grid-cols-2 gap-2">
                  {session.agents.map(agent => (
                    <div
                      key={agent.id}
                      onClick={() => toggleAgentExpand(agent.id)}
                      className="cursor-pointer"
                    >
                      <AgentNode agent={agent} isExpanded={expandedAgents.has(agent.id)} />
                    </div>
                  ))}
                </div>
              )}
            </div>

            {session.subTasks.length > 1 && (
              <div>
                <div className="flex items-center gap-1.5 mb-2">
                  <Split size={14} className="text-[#A855F7]" />
                  <span className="text-[12px] font-medium text-claude-text">{t('swarm.taskName')} ({session.subTasks.length})</span>
                </div>
                <div className="space-y-1">
                  {session.subTasks.map((task, index) => (
                    <div key={task.id} className="relative">
                      {index > 0 && (
                        <div className="absolute left-4 -top-1 w-px h-1 bg-claude-border" />
                      )}
                      <TaskNode task={task} agents={session.agents} />
                    </div>
                  ))}
                </div>
              </div>
            )}

            {session.status === 'completed' && (
              <div className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
                <div className="flex items-center gap-2">
                  <CheckCircle2 size={16} className="text-emerald-400" />
                  <span className="text-[13px] font-medium text-emerald-400">{t('swarm.completed')}</span>
                </div>
                <div className="mt-2 text-[12px] text-claude-textSecondary">
                  {t('swarm.duration')}: {((session.endTime || Date.now()) - session.startTime) / 1000}s ·
                  {t('swarm.nodes')}: {totalAgents} ·
                  Tokens: {session.agents.reduce((sum, a) => sum + a.tokensUsed, 0).toLocaleString()}
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {!session && !isAnalyzing && (
        <div className="flex-1 flex items-center justify-center p-6">
          <div className="text-center max-w-xs">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-claude-hover flex items-center justify-center">
              <Users size={28} className="text-[#3B82F6]" />
            </div>
            <h3 className="text-[14px] font-medium text-claude-text mb-2">{t('swarm.title')}</h3>
            <p className="text-[12px] text-claude-textSecondary leading-relaxed">
              {t('swarm.taskDesc')}
            </p>
            <div className="mt-4 space-y-2 text-left">
              {[
                'Automatic complexity analysis',
                'Smart task decomposition',
                'Parallel agent execution',
                'Real-time progress tracking',
              ].map((feature, i) => (
                <div key={i} className="flex items-center gap-2 text-[11px] text-claude-textSecondary">
                  <ArrowRight size={12} className="text-[#3B82F6]" />
                  <span>{feature}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SwarmCollaboration;
