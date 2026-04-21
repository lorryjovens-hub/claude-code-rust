import React from 'react';

interface CodeExecutionProps {
  code: string;
  status: 'running' | 'done' | 'error';
  stdout?: string;
  stderr?: string;
  images?: string[];
  error?: string | null;
}

export default function CodeExecution({ status, images }: CodeExecutionProps) {
  const hasImages = images && images.length > 0;

  // 执行中：显示加载动画
  if (status === 'running') {
    return (
      <div className="my-3 flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
        <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
        正在执行代码...
      </div>
    );
  }

  // 执行完成：只显示图片（如果有）
  if (!hasImages) return null;

  return (
    <div className="my-3 space-y-2">
      {images!.map((img, i) => (
        <div key={i} className="rounded-lg overflow-hidden">
          <img src={`data:image/png;base64,${img}`} alt={`图表 ${i + 1}`} className="max-w-full" />
        </div>
      ))}
    </div>
  );
}
