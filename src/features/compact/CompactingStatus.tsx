import { useState, useEffect } from 'react';

export default function CompactingStatus() {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setProgress((prev) => {
        if (prev >= 95) return prev;
        const remaining = 95 - prev;
        const inc = Math.max(0.2, remaining * 0.05);
        return Math.min(95, prev + inc);
      });
    }, 100);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex flex-col justify-center ml-2">
      <div className="text-[#404040] dark:text-[#d1d5db] font-serif italic text-[17px] leading-relaxed mb-1">
        Compacting our conversation so we can keep chatting...
      </div>
      <div className="flex items-center gap-3">
        <div className="w-48 h-1.5 bg-[#EAE8E1] dark:bg-white/10 rounded-full overflow-hidden">
          <div
            className="h-full bg-[#404040] dark:bg-[#d1d5db] rounded-full transition-all duration-100 ease-out"
            style={{ width: `${progress}%` }}
          />
        </div>
        <span className="text-[13px] text-[#707070] dark:text-[#9ca3af] font-medium font-mono">
          {Math.round(progress)}%
        </span>
      </div>
    </div>
  );
}
