import React, { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../hooks/useI18n';
import { BarChart3, TrendingUp, MessageSquare, Zap, AlertTriangle, Mic, Hash, FileUp, Activity, Calendar, ChevronLeft, ChevronRight, X } from 'lucide-react';
import { getAnalyticsSummary, getAnalyticsRange, getAnalyticsEventCounts, getAnalyticsRecentEvents } from '../api';

interface DailyStats {
  date: string;
  messages_sent: number;
  conversations_created: number;
  tokens_input: number;
  tokens_output: number;
  tools_executed: number;
  errors: number;
  voice_inputs: number;
  slash_commands: number;
  files_uploaded: number;
}

interface UsageSummary {
  total_days: number;
  total_messages: number;
  total_conversations: number;
  total_tokens_input: number;
  total_tokens_output: number;
  total_tools: number;
  total_errors: number;
  total_voice_inputs: number;
  avg_daily_messages: number;
  avg_daily_tokens: number;
  streak_days: number;
  most_active_day: string;
  most_used_model: string;
}

interface EventTypeCount {
  event_type: string;
  count: number;
}

interface AnalyticsEvent {
  id: string;
  event_type: string;
  timestamp: string;
  properties: Record<string, any>;
  session_id: string | null;
}

const PERIOD_OPTIONS = [
  { label: '7 天', days: 7 },
  { label: '30 天', days: 30 },
  { label: '90 天', days: 90 },
];

function formatNumber(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
  if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
  return n.toString();
}

function MiniBarChart({ data, dataKey, color = '#C6613F' }: { data: DailyStats[]; dataKey: keyof DailyStats; color?: string }) {
  if (data.length === 0) return <div className="h-24 flex items-center justify-center text-xs text-claude-textSecondary">{/* no data */}</div>;
  const values = data.map(d => d[dataKey] as number);
  const max = Math.max(...values, 1);

  return (
    <div className="flex items-end gap-px h-24">
      {data.map((d, i) => {
        const val = d[dataKey] as number;
        const h = Math.max((val / max) * 100, 2);
        return (
          <div key={i} className="flex-1 relative group" style={{ height: '100%' }}>
            <div
              className="absolute bottom-0 w-full rounded-t-sm transition-all"
              style={{ height: `${h}%`, backgroundColor: color, opacity: 0.8 }}
            />
            <div className="absolute bottom-full mb-1 left-1/2 -translate-x-1/2 hidden group-hover:block bg-claude-avatar text-claude-avatarText text-xs px-2 py-1 rounded whitespace-nowrap z-10">
              {d.date}: {val}
            </div>
          </div>
        );
      })}
    </div>
  );
}

const AnalyticsPanel = ({ onClose }: { onClose: () => void }) => {
  const { t } = useI18n();
  const [tab, setTab] = useState<'overview' | 'trends' | 'events'>('overview');
  const [period, setPeriod] = useState(30);
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [dailyStats, setDailyStats] = useState<DailyStats[]>([]);
  const [eventCounts, setEventCounts] = useState<EventTypeCount[]>([]);
  const [recentEvents, setRecentEvents] = useState<AnalyticsEvent[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const end = new Date();
      const start = new Date();
      start.setDate(start.getDate() - period);
      const from = start.toISOString().slice(0, 10);
      const to = end.toISOString().slice(0, 10);

      const [sumRes, rangeRes, countsRes, eventsRes] = await Promise.all([
        getAnalyticsSummary(period),
        getAnalyticsRange(from, to),
        getAnalyticsEventCounts(period),
        getAnalyticsRecentEvents(30),
      ]);

      if (sumRes.success) setSummary(sumRes.summary);
      if (rangeRes.success) setDailyStats(rangeRes.stats || []);
      if (countsRes.success) setEventCounts(countsRes.counts || []);
      if (eventsRes.success) setRecentEvents(eventsRes.events || []);
    } catch (_) {}
    setLoading(false);
  }, [period]);

  useEffect(() => { refresh(); }, [refresh]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const statCards = summary ? [
    { label: t('chat.thread'), value: formatNumber(summary.total_messages), icon: MessageSquare, color: 'text-blue-500' },
    { label: t('sidebar.chats'), value: formatNumber(summary.total_conversations), icon: BarChart3, color: 'text-green-500' },
    { label: t('analytics.inputTokens'), value: formatNumber(summary.total_tokens_input), icon: TrendingUp, color: 'text-purple-500' },
    { label: t('analytics.outputTokens'), value: formatNumber(summary.total_tokens_output), icon: TrendingUp, color: 'text-orange-500' },
    { label: t('analytics.requests'), value: formatNumber(summary.total_tools), icon: Zap, color: 'text-yellow-500' },
    { label: t('analytics.errors'), value: formatNumber(summary.total_errors), icon: AlertTriangle, color: 'text-red-500' },
    { label: t('sidebar.computerUse'), value: formatNumber(summary.total_voice_inputs ?? 0), icon: Mic, color: 'text-pink-500' },
    { label: t('analytics.running'), value: summary.streak_days.toString(), icon: Activity, color: 'text-cyan-500' },
  ] : [];

  const labelMap: Record<string, string> = {
    message_sent: t('chat.send'),
    conversation_created: t('chat.newChat'),
    tokens_used: t('analytics.totalTokens'),
    tool_executed: t('analytics.requests'),
    error: t('analytics.failed'),
    voice_input: t('sidebar.computerUse'),
    slash_command: 'Slash Command',
    file_uploaded: t('agent.upload'),
  };

  const eventColorMap: Record<string, string> = {
    message_sent: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300',
    conversation_created: 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300',
    tokens_used: 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300',
    tool_executed: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300',
    error: 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300',
    voice_input: 'bg-pink-100 text-pink-700 dark:bg-pink-900 dark:text-pink-300',
    slash_command: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900 dark:text-indigo-300',
    file_uploaded: 'bg-cyan-100 text-cyan-700 dark:bg-cyan-900 dark:text-cyan-300',
  };

  return (
    <div className="fixed inset-0 z-[200] bg-black/40 flex items-center justify-center" onClick={onClose}>
      <div
        className="bg-claude-input border border-claude-border rounded-2xl shadow-2xl w-[680px] max-h-[85vh] overflow-hidden flex flex-col"
        onClick={e => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-claude-border">
          <div className="flex items-center gap-3">
            <BarChart3 size={22} className="text-[#C6613F]" />
            <h2 className="text-lg font-semibold text-claude-text">{t('analytics.title')}</h2>
          </div>
          <div className="flex items-center gap-2">
            {PERIOD_OPTIONS.map(opt => (
              <button
                key={opt.days}
                onClick={() => setPeriod(opt.days)}
                className={`px-3 py-1 text-xs rounded-full transition-colors ${
                  period === opt.days
                    ? 'bg-[#C6613F] text-white'
                    : 'bg-claude-hover text-claude-textSecondary hover:bg-claude-btn-hover'
                }`}
              >
                {opt.label}
              </button>
            ))}
            <button onClick={refresh} className="ml-2 p-1.5 rounded-md hover:bg-claude-hover text-claude-textSecondary" title={t('analytics.refresh')}>
              <Activity size={16} />
            </button>
            <button onClick={onClose} className="p-1.5 rounded-md hover:bg-claude-hover text-claude-textSecondary" title={t('common.close')}>
              <X size={16} />
            </button>
          </div>
        </div>

        <div className="flex border-b border-claude-border">
          {([['overview', t('analytics.overview')], ['trends', t('analytics.trends')], ['events', t('analytics.eventFlow')]] as const).map(([key, label]) => (
            <button
              key={key}
              onClick={() => setTab(key)}
              className={`flex-1 py-2.5 text-sm font-medium transition-colors ${
                tab === key
                  ? 'text-claude-text border-b-2 border-[#C6613F]'
                  : 'text-claude-textSecondary hover:text-claude-text'
              }`}
            >
              {label}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          {loading ? (
            <div className="flex items-center justify-center py-12 text-claude-textSecondary">
              <Activity size={20} className="animate-spin mr-2" /> {t('common.loading')}
            </div>
          ) : tab === 'overview' ? (
            <div className="space-y-6">
              <div className="grid grid-cols-4 gap-3">
                {statCards.map((card, i) => (
                  <div key={i} className="bg-claude-hover rounded-lg p-3">
                    <div className="flex items-center gap-2 mb-1">
                      <card.icon size={14} className={card.color} />
                      <span className="text-xs text-claude-textSecondary">{card.label}</span>
                    </div>
                    <div className="text-xl font-bold text-claude-text">{card.value}</div>
                  </div>
                ))}
              </div>

              {summary && (
                <div className="grid grid-cols-2 gap-3">
                  <div className="bg-claude-hover rounded-lg p-4">
                    <div className="text-xs text-claude-textSecondary mb-1">{t('analytics.requests')}</div>
                    <div className="text-lg font-semibold text-claude-text">{summary.avg_daily_messages.toFixed(1)}</div>
                  </div>
                  <div className="bg-claude-hover rounded-lg p-4">
                    <div className="text-xs text-claude-textSecondary mb-1">{t('analytics.tokenCount')}</div>
                    <div className="text-lg font-semibold text-claude-text">{formatNumber(Math.round(summary.avg_daily_tokens))}</div>
                  </div>
                  <div className="bg-claude-hover rounded-lg p-4">
                    <div className="text-xs text-claude-textSecondary mb-1">{t('analytics.dailyActivity')}</div>
                    <div className="text-lg font-semibold text-claude-text">{summary.most_active_day || '-'}</div>
                  </div>
                  <div className="bg-claude-hover rounded-lg p-4">
                    <div className="text-xs text-claude-textSecondary mb-1">{t('analytics.eventName')}</div>
                    <div className="text-lg font-semibold text-claude-text">{eventCounts.length}</div>
                  </div>
                </div>
              )}

              {eventCounts.length > 0 && (
                <div>
                  <h3 className="text-sm font-medium text-claude-text mb-3">{t('analytics.tokenDistribution')}</h3>
                  <div className="space-y-2">
                    {eventCounts.map(ec => {
                      const maxCount = eventCounts[0]?.count || 1;
                      const pct = (ec.count / maxCount) * 100;
                      return (
                        <div key={ec.event_type} className="flex items-center gap-3">
                          <span className="text-xs text-claude-textSecondary w-20 text-right truncate">
                            {labelMap[ec.event_type] || ec.event_type}
                          </span>
                          <div className="flex-1 h-5 bg-claude-btn-hover rounded-full overflow-hidden">
                            <div
                              className="h-full bg-[#C6613F] rounded-full transition-all"
                              style={{ width: `${pct}%` }}
                            />
                          </div>
                          <span className="text-xs text-claude-textSecondary w-12 text-right">{ec.count}</span>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>
          ) : tab === 'trends' ? (
            <div className="space-y-6">
              <div>
                <h3 className="text-sm font-medium text-claude-text mb-2">{t('chat.thread')}</h3>
                <MiniBarChart data={dailyStats} dataKey="messages_sent" color="#3b82f6" />
              </div>
              <div>
                <h3 className="text-sm font-medium text-claude-text mb-2">{t('analytics.totalTokens')}</h3>
                <MiniBarChart data={dailyStats} dataKey="tokens_output" color="#C6613F" />
              </div>
              <div>
                <h3 className="text-sm font-medium text-claude-text mb-2">{t('analytics.requests')}</h3>
                <MiniBarChart data={dailyStats} dataKey="tools_executed" color="#10b981" />
              </div>
              <div>
                <h3 className="text-sm font-medium text-claude-text mb-2">{t('analytics.failed')}</h3>
                <MiniBarChart data={dailyStats} dataKey="errors" color="#ef4444" />
              </div>
            </div>
          ) : (
            <div className="space-y-1">
              {recentEvents.length === 0 ? (
                <div className="text-center py-8 text-claude-textSecondary text-sm">{t('analytics.noEvents')}</div>
              ) : (
                recentEvents.map(ev => (
                  <div key={ev.id} className="flex items-center gap-3 py-2 px-3 rounded-lg hover:bg-claude-hover">
                    <span className={`text-xs px-2 py-0.5 rounded-full ${eventColorMap[ev.event_type] || 'bg-claude-hover text-claude-textSecondary'}`}>
                      {labelMap[ev.event_type] || ev.event_type}
                    </span>
                    <span className="text-xs text-claude-textSecondary flex-1 truncate">
                      {ev.properties && Object.keys(ev.properties).length > 0
                        ? Object.entries(ev.properties).map(([k, v]) => `${k}=${v}`).join(', ')
                        : '-'}
                    </span>
                    <span className="text-xs text-claude-textSecondary/60 whitespace-nowrap">
                      {ev.timestamp.replace('T', ' ').slice(0, 19)}
                    </span>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default AnalyticsPanel;
