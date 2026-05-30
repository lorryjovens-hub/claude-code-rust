import React, { useState, useEffect, useCallback } from 'react';
import { Coins, TrendingUp, BarChart3, ChevronDown, ChevronUp, AlertTriangle, Settings, Save } from 'lucide-react';

interface CostSummary {
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  estimated_total_cost: number;
  model_breakdown: Record<string, { input_tokens: number; output_tokens: number; total_tokens: number }>;
}

interface UsageStats {
  daily_usage: number;
  monthly_usage: number;
  daily_budget: number | null;
  monthly_budget: number | null;
  daily_percent: number | null;
  monthly_percent: number | null;
}

interface DailyRecord {
  date: string;
  total_tokens: number;
  total_cost: number;
}

interface DashboardData {
  total_tokens: number;
  total_cost: number;
  session_count: number;
  model_breakdown: Record<string, { total_tokens: number; total_cost: number; session_count: number }>;
  daily_trend: DailyRecord[];
  usage_stats: UsageStats;
}

interface CostTrackerProps {
  conversationId?: string;
  compact?: boolean;
  apiBase?: string;
}

export default function CostTracker({ conversationId, compact = false, apiBase = '/api' }: CostTrackerProps) {
  const [summary, setSummary] = useState<CostSummary | null>(null);
  const [dashboard, setDashboard] = useState<DashboardData | null>(null);
  const [usageStats, setUsageStats] = useState<UsageStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [dailyBudget, setDailyBudget] = useState('');
  const [monthlyBudget, setMonthlyBudget] = useState('');
  const [trendDays, setTrendDays] = useState<7 | 30>(7);
  const [budgetWarning, setBudgetWarning] = useState<string | null>(null);

  const fetchDashboard = useCallback(async () => {
    try {
      const res = await fetch(`${apiBase}/costs/dashboard`);
      if (res.ok) {
        const data = await res.json();
        setDashboard(data);
        setUsageStats(data.usage_stats);
        if (data.usage_stats.daily_budget) {
          setDailyBudget(String(data.usage_stats.daily_budget));
        }
        if (data.usage_stats.monthly_budget) {
          setMonthlyBudget(String(data.usage_stats.monthly_budget));
        }
        if (data.usage_stats.daily_percent && data.usage_stats.daily_percent >= 80) {
          setBudgetWarning(`Daily budget at ${data.usage_stats.daily_percent}%`);
        } else if (data.usage_stats.monthly_percent && data.usage_stats.monthly_percent >= 80) {
          setBudgetWarning(`Monthly budget at ${data.usage_stats.monthly_percent}%`);
        } else {
          setBudgetWarning(null);
        }
      }
    } catch (e) {
      console.error('Failed to load dashboard:', e);
    }
  }, [apiBase]);

  const fetchUsage = useCallback(async () => {
    try {
      const res = await fetch(`${apiBase}/costs/usage`);
      if (res.ok) {
        const data = await res.json();
        setUsageStats(data);
      }
    } catch (e) {
      console.error('Failed to load usage:', e);
    }
  }, [apiBase]);

  useEffect(() => {
    fetchDashboard();
    const interval = setInterval(fetchUsage, 30000);
    return () => clearInterval(interval);
  }, [fetchDashboard, fetchUsage]);

  useEffect(() => {
    if (conversationId) {
      loadCostSummary();
    }
  }, [conversationId]);

  const loadCostSummary = async () => {
    if (!conversationId) return;
    setLoading(true);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<CostSummary>('get_cost_summary', { conversationId });
      setSummary(result);
    } catch (e) {
      console.error('Failed to load cost summary:', e);
    } finally {
      setLoading(false);
    }
  };

  const saveBudget = async () => {
    try {
      const res = await fetch(`${apiBase}/costs/budget`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          daily_budget: dailyBudget ? parseInt(dailyBudget) : 0,
          monthly_budget: monthlyBudget ? parseInt(monthlyBudget) : 0,
        }),
      });
      if (res.ok) {
        const data = await res.json();
        setUsageStats(data);
        setShowSettings(false);
        fetchDashboard();
      }
    } catch (e) {
      console.error('Failed to save budget:', e);
    }
  };

  const formatCost = (cost: number): string => {
    if (cost < 0.001) return '<$0.001';
    if (cost < 0.01) return `$${cost.toFixed(4)}`;
    if (cost < 1) return `$${cost.toFixed(3)}`;
    return `$${cost.toFixed(2)}`;
  };

  const formatTokens = (tokens: number): string => {
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(1)}M`;
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}K`;
    return tokens.toString();
  };

  const getBudgetBarColor = (percent: number | null) => {
    if (!percent) return 'bg-emerald-500';
    if (percent >= 100) return 'bg-red-500';
    if (percent >= 80) return 'bg-amber-500';
    if (percent >= 50) return 'bg-yellow-500';
    return 'bg-emerald-500';
  };

  const getBudgetTextColor = (percent: number | null) => {
    if (!percent) return 'text-emerald-500';
    if (percent >= 100) return 'text-red-500';
    if (percent >= 80) return 'text-amber-500';
    return 'text-emerald-500';
  };

  const maxTrendTokens = dashboard
    ? Math.max(...dashboard.daily_trend.slice(0, trendDays).map(d => d.total_tokens), 1)
    : 1;

  if (compact) {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 text-[11px] text-claude-textSecondary bg-claude-hover/50 rounded-md">
        <Coins size={12} className="text-amber-500" />
        {usageStats ? (
          <span className="font-mono">
            {formatTokens(usageStats.daily_usage)}
            {usageStats.daily_budget && (
              <span className="text-claude-textSecondary">/{formatTokens(usageStats.daily_budget)}</span>
            )}
          </span>
        ) : (
          <span className="animate-pulse">...</span>
        )}
        {budgetWarning && (
          <AlertTriangle size={10} className="text-amber-500" />
        )}
      </div>
    );
  }

  return (
    <div className="border-t border-claude-border pt-2">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between px-2 py-1.5 text-[12px] text-claude-textSecondary hover:text-claude-text transition-colors rounded-md hover:bg-black/5 dark:hover:bg-white/5"
      >
        <div className="flex items-center gap-2">
          <Coins size={14} className="text-amber-500" />
          <span className="font-medium">Cost Dashboard</span>
          {budgetWarning && (
            <AlertTriangle size={12} className="text-amber-500" />
          )}
        </div>
        <div className="flex items-center gap-2">
          {usageStats && (
            <span className="font-mono text-[11px]">
              {formatTokens(usageStats.daily_usage)}
              {usageStats.daily_budget && (
                <span className="text-claude-textSecondary">/{formatTokens(usageStats.daily_budget)}</span>
              )}
            </span>
          )}
          {expanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
        </div>
      </button>

      {expanded && (
        <div className="px-2 py-2 space-y-3 animate-fade-in">
          {budgetWarning && (
            <div className="flex items-center gap-2 px-2 py-1.5 bg-amber-500/10 border border-amber-500/20 rounded-lg text-[11px] text-amber-600 dark:text-amber-400">
              <AlertTriangle size={12} />
              <span>{budgetWarning}</span>
            </div>
          )}

          {usageStats && (
            <div className="space-y-2">
              {usageStats.daily_budget && (
                <div>
                  <div className="flex items-center justify-between text-[10px] mb-0.5">
                    <span className="text-claude-textSecondary">Daily Budget</span>
                    <span className={getBudgetTextColor(usageStats.daily_percent)}>
                      {formatTokens(usageStats.daily_usage)} / {formatTokens(usageStats.daily_budget)}
                      {usageStats.daily_percent !== null && ` (${usageStats.daily_percent}%)`}
                    </span>
                  </div>
                  <div className="h-1.5 bg-claude-hover rounded-full overflow-hidden">
                    <div
                      className={`h-full rounded-full transition-all duration-500 ${getBudgetBarColor(usageStats.daily_percent)}`}
                      style={{ width: `${Math.min(usageStats.daily_percent || 0, 100)}%` }}
                    />
                  </div>
                </div>
              )}
              {usageStats.monthly_budget && (
                <div>
                  <div className="flex items-center justify-between text-[10px] mb-0.5">
                    <span className="text-claude-textSecondary">Monthly Budget</span>
                    <span className={getBudgetTextColor(usageStats.monthly_percent)}>
                      {formatTokens(usageStats.monthly_usage)} / {formatTokens(usageStats.monthly_budget)}
                      {usageStats.monthly_percent !== null && ` (${usageStats.monthly_percent}%)`}
                    </span>
                  </div>
                  <div className="h-1.5 bg-claude-hover rounded-full overflow-hidden">
                    <div
                      className={`h-full rounded-full transition-all duration-500 ${getBudgetBarColor(usageStats.monthly_percent)}`}
                      style={{ width: `${Math.min(usageStats.monthly_percent || 0, 100)}%` }}
                    />
                  </div>
                </div>
              )}
            </div>
          )}

          {dashboard && (
            <>
              <div className="grid grid-cols-3 gap-2">
                <div className="bg-claude-hover/50 rounded-lg p-2">
                  <div className="text-[10px] text-claude-textSecondary mb-0.5">Total Cost</div>
                  <div className="text-[13px] font-semibold text-amber-600 dark:text-amber-400 font-mono">
                    {formatCost(dashboard.total_cost)}
                  </div>
                </div>
                <div className="bg-claude-hover/50 rounded-lg p-2">
                  <div className="text-[10px] text-claude-textSecondary mb-0.5">Total Tokens</div>
                  <div className="text-[13px] font-mono text-claude-text">{formatTokens(dashboard.total_tokens)}</div>
                </div>
                <div className="bg-claude-hover/50 rounded-lg p-2">
                  <div className="text-[10px] text-claude-textSecondary mb-0.5">Sessions</div>
                  <div className="text-[13px] font-mono text-claude-text">{dashboard.session_count}</div>
                </div>
              </div>

              {summary && (
                <div className="grid grid-cols-2 gap-2">
                  <div className="bg-claude-hover/50 rounded-lg p-2">
                    <div className="text-[10px] text-claude-textSecondary mb-0.5">Input Tokens</div>
                    <div className="text-[13px] font-mono text-claude-text">{formatTokens(summary.total_input_tokens)}</div>
                  </div>
                  <div className="bg-claude-hover/50 rounded-lg p-2">
                    <div className="text-[10px] text-claude-textSecondary mb-0.5">Output Tokens</div>
                    <div className="text-[13px] font-mono text-claude-text">{formatTokens(summary.total_output_tokens)}</div>
                  </div>
                </div>
              )}

              {dashboard.daily_trend.length > 0 && (
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <div className="text-[10px] text-claude-textSecondary flex items-center gap-1">
                      <TrendingUp size={10} />
                      Token Trend
                    </div>
                    <div className="flex gap-1">
                      <button
                        onClick={() => setTrendDays(7)}
                        className={`text-[9px] px-1.5 py-0.5 rounded ${trendDays === 7 ? 'bg-claude-hover text-claude-text' : 'text-claude-textSecondary'}`}
                      >
                        7d
                      </button>
                      <button
                        onClick={() => setTrendDays(30)}
                        className={`text-[9px] px-1.5 py-0.5 rounded ${trendDays === 30 ? 'bg-claude-hover text-claude-text' : 'text-claude-textSecondary'}`}
                      >
                        30d
                      </button>
                    </div>
                  </div>
                  <div className="flex items-end gap-[2px] h-12">
                    {dashboard.daily_trend.slice(0, trendDays).reverse().map((record, i) => (
                      <div
                        key={record.date}
                        className="flex-1 bg-emerald-500/60 rounded-t-sm min-w-[3px] transition-all hover:bg-emerald-400/80 group relative"
                        style={{ height: `${(record.total_tokens / maxTrendTokens) * 100}%` }}
                        title={`${record.date}: ${formatTokens(record.total_tokens)}`}
                      >
                        <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 hidden group-hover:block bg-claude-text text-claude-bg text-[9px] px-1 py-0.5 rounded whitespace-nowrap z-10">
                          {record.date.slice(5)}: {formatTokens(record.total_tokens)}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {Object.keys(dashboard.model_breakdown).length > 0 && (
                <div>
                  <div className="text-[10px] text-claude-textSecondary mb-1 flex items-center gap-1">
                    <BarChart3 size={10} />
                    Model Breakdown
                  </div>
                  <div className="space-y-1">
                    {Object.entries(dashboard.model_breakdown)
                      .sort(([, a], [, b]) => b.total_tokens - a.total_tokens)
                      .map(([model, data]) => (
                        <div key={model} className="flex items-center justify-between text-[11px]">
                          <span className="text-claude-textSecondary truncate max-w-[120px]">{model}</span>
                          <div className="flex items-center gap-2">
                            <span className="font-mono text-claude-text">{formatTokens(data.total_tokens)}</span>
                            <span className="text-amber-600 dark:text-amber-400 font-mono">{formatCost(data.total_cost)}</span>
                          </div>
                        </div>
                      ))}
                  </div>
                </div>
              )}
            </>
          )}

          <div>
            <button
              onClick={() => setShowSettings(!showSettings)}
              className="flex items-center gap-1 text-[10px] text-claude-textSecondary hover:text-claude-text transition-colors"
            >
              <Settings size={10} />
              Budget Settings
            </button>
            {showSettings && (
              <div className="mt-2 space-y-2 bg-claude-hover/30 rounded-lg p-2">
                <div>
                  <label className="text-[10px] text-claude-textSecondary block mb-0.5">Daily Token Budget</label>
                  <input
                    type="number"
                    value={dailyBudget}
                    onChange={e => setDailyBudget(e.target.value)}
                    placeholder="No limit"
                    className="w-full bg-claude-hover border border-claude-border rounded px-2 py-1 text-[11px] font-mono text-claude-text focus:outline-none focus:ring-1 focus:ring-amber-500/50"
                  />
                </div>
                <div>
                  <label className="text-[10px] text-claude-textSecondary block mb-0.5">Monthly Token Budget</label>
                  <input
                    type="number"
                    value={monthlyBudget}
                    onChange={e => setMonthlyBudget(e.target.value)}
                    placeholder="No limit"
                    className="w-full bg-claude-hover border border-claude-border rounded px-2 py-1 text-[11px] font-mono text-claude-text focus:outline-none focus:ring-1 focus:ring-amber-500/50"
                  />
                </div>
                <button
                  onClick={saveBudget}
                  className="flex items-center gap-1 px-2 py-1 bg-amber-500/20 text-amber-600 dark:text-amber-400 rounded text-[10px] hover:bg-amber-500/30 transition-colors"
                >
                  <Save size={10} />
                  Save Budget
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
