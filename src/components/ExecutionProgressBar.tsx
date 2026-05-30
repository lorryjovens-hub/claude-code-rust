import React from 'react';
import { useExecutionProgress } from '../hooks/useExecutionProgress';

interface ExecutionProgressBarProps {
  conversationId: string | null;
  compact?: boolean;
}

export const ExecutionProgressBar: React.FC<ExecutionProgressBarProps> = ({
  conversationId,
  compact = false,
}) => {
  const { progress } = useExecutionProgress(conversationId);

  if (!progress || progress.status === 'idle') return null;

  const percent = progress.toolsTotal > 0
    ? Math.round((progress.toolsCompleted / progress.toolsTotal) * 100)
    : 0;

  const statusColor = progress.status === 'running'
    ? 'bg-blue-500'
    : progress.status === 'completed'
    ? 'bg-green-500'
    : progress.status === 'failed'
    ? 'bg-orange-500'
    : 'bg-gray-500';

  const statusText = progress.status === 'running'
    ? 'Executing'
    : progress.status === 'completed'
    ? 'Completed'
    : progress.status === 'failed'
    ? 'Partial failure'
    : 'Cancelled';

  if (compact) {
    return (
      <div className="flex items-center gap-2 px-2 py-1">
        <div className="flex-1 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div
            className={`h-full ${statusColor} transition-all duration-300 ease-out rounded-full`}
            style={{ width: `${percent}%` }}
          />
        </div>
        <span className="text-xs text-gray-500 whitespace-nowrap">
          {progress.toolsCompleted}/{progress.toolsTotal}
        </span>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-3 mb-3">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {progress.status === 'running' && (
            <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          )}
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            {statusText}
          </span>
          <span className="text-xs text-gray-400">
            Layer {progress.currentLayer + 1}/{progress.layersTotal}
          </span>
        </div>
        <span className="text-xs text-gray-400">
          {progress.toolsCompleted}/{progress.toolsTotal} tools · {percent}%
        </span>
      </div>

      <div className="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden mb-2">
        <div
          className={`h-full ${statusColor} transition-all duration-300 ease-out rounded-full`}
          style={{ width: `${percent}%` }}
        />
      </div>

      {progress.currentTools.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {progress.currentTools.map((tool, i) => (
            <span
              key={i}
              className={`text-xs px-2 py-0.5 rounded-full ${
                tool === 'pending'
                  ? 'bg-gray-100 dark:bg-gray-700 text-gray-400 animate-pulse'
                  : 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
              }`}
            >
              {tool === 'pending' ? '⏳' : `⚡ ${tool}`}
            </span>
          ))}
        </div>
      )}

      {progress.status !== 'running' && progress.totalDurationMs > 0 && (
        <div className="mt-2 flex gap-3 text-xs text-gray-400">
          <span>Duration: {(progress.totalDurationMs / 1000).toFixed(1)}s</span>
          <span className="text-green-500">✓ {progress.successCount}</span>
          {progress.failCount > 0 && (
            <span className="text-red-500">✗ {progress.failCount}</span>
          )}
        </div>
      )}
    </div>
  );
};