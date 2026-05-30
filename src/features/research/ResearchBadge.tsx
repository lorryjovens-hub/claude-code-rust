import { X } from 'lucide-react';
import { IconResearch } from '../../components/Icons';

interface ResearchBadgeProps {
  onToggle: () => void;
}

export default function ResearchBadge({ onToggle }: ResearchBadgeProps) {
  return (
    <div className="group/research relative ml-1 flex items-center bg-[#DBEAFE] dark:bg-[#1E3A5F] rounded-lg p-1.5">
      <IconResearch size={16} className="text-[#2E7CF6] flex-shrink-0" />
      <span className="inline-flex items-center overflow-hidden w-0 group-hover/research:w-[18px] transition-[width] duration-150 ease-out">
        <button
          onClick={onToggle}
          className="ml-1 flex-shrink-0 flex items-center justify-center hover:opacity-70 transition-opacity"
          aria-label="Disable research mode"
        >
          <X size={14} className="text-[#2E7CF6]" />
        </button>
      </span>
      <div className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-1 bg-claude-tooltipBg text-claude-tooltipText rounded-md text-[11px] whitespace-nowrap opacity-0 group-hover/research:opacity-100 pointer-events-none transition-opacity">
        Research mode
      </div>
    </div>
  );
}
