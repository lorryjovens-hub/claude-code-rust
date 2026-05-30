import React from 'react';
import { Loader2 } from 'lucide-react';

export interface SpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  color?: string;
  className?: string;
  label?: string;
}

const sizeMap: Record<string, number> = {
  sm: 14,
  md: 24,
  lg: 36,
};

const containerSizeMap: Record<string, string> = {
  sm: 'w-5 h-5',
  md: 'w-8 h-8',
  lg: 'w-12 h-12',
};

const Spinner: React.FC<SpinnerProps> = ({
  size = 'md',
  className = '',
  label,
}) => {
  return (
    <div className={`flex flex-col items-center justify-center gap-2 ${className}`}>
      <div className={containerSizeMap[size]}>
        <Loader2
          size={sizeMap[size]}
          className="animate-spin text-claude-textSecondary"
        />
      </div>
      {label && (
        <span className="text-[13px] text-claude-textSecondary animate-shimmer-text">
          {label}
        </span>
      )}
    </div>
  );
};

export default React.memo(Spinner);
