import React from 'react';
import { Clock, Edit, Trash2, Play, ToggleLeft, ToggleRight } from 'lucide-react';
import { TaskInfo } from '../utils/tauriAPI';

interface TaskRowProps {
  task: TaskInfo;
  onToggle: (id: string, enabled: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
  onExecute: () => void;
}

function formatLastRun(lastRunAt: string | null, lastRunStatus: string | null): string {
  if (!lastRunAt) return 'Not yet run';
  const date = new Date(lastRunAt);
  const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  const mon = date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  const status = lastRunStatus === 'success' ? '✓' : lastRunStatus === 'failed' ? '✗' : '';
  return `${status} ${mon} ${time}`;
}

export default function TaskRow({ task, onToggle, onEdit, onDelete, onExecute }: TaskRowProps) {
  const statusColor = task.is_enabled
    ? 'text-green-600 dark:text-green-400'
    : 'text-claude-textSecondary';

  return (
    <div className="flex items-center gap-3 px-4 py-3 border-b border-claude-border hover:bg-claude-hover/50 transition-colors group">
      <div className="flex items-center gap-2 flex-1 min-w-0">
        <div className={`w-2 h-2 rounded-full shrink-0 ${task.is_enabled ? 'bg-green-500' : 'bg-gray-400'}`} />
        <div className="flex flex-col min-w-0">
          <span className="text-[13px] font-medium text-claude-text truncate">{task.name}</span>
          <span className="text-[11px] text-claude-textSecondary truncate font-mono">{task.cron_expression}</span>
        </div>
      </div>

      <div className="flex items-center gap-1 text-[11px] text-claude-textSecondary shrink-0">
        <Clock size={12} />
        <span>{formatLastRun(task.last_run_at, task.last_run_status)}</span>
      </div>

      <div className="flex items-center gap-0.5 shrink-0">
        <span className={`text-[11px] px-1.5 py-0.5 rounded ${statusColor} bg-claude-hover`}>
          {task.is_enabled ? 'Enabled' : 'Disabled'}
        </span>
      </div>

      <div className="flex items-center gap-0.5 shrink-0">
        <button
          onClick={() => onToggle(task.id, !task.is_enabled)}
          className="p-1.5 rounded-md hover:bg-claude-btn-hover text-claude-textSecondary hover:text-claude-text transition-colors"
          title={task.is_enabled ? 'Disable' : 'Enable'}
        >
          {task.is_enabled ? <ToggleRight size={16} /> : <ToggleLeft size={16} />}
        </button>
        <button
          onClick={onExecute}
          className="p-1.5 rounded-md hover:bg-claude-btn-hover text-claude-textSecondary hover:text-green-600 transition-colors"
          title="Execute now"
        >
          <Play size={14} />
        </button>
        <button
          onClick={onEdit}
          className="p-1.5 rounded-md hover:bg-claude-btn-hover text-claude-textSecondary hover:text-claude-text transition-colors"
          title="Edit"
        >
          <Edit size={14} />
        </button>
        <button
          onClick={onDelete}
          className="p-1.5 rounded-md hover:bg-claude-btn-hover text-claude-textSecondary hover:text-red-500 transition-colors"
          title="Delete"
        >
          <Trash2 size={14} />
        </button>
      </div>
    </div>
  );
}