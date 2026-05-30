import React, { useState, useRef, useEffect, useCallback } from 'react';

export interface TooltipProps {
  content: string;
  children: React.ReactNode;
  position?: 'top' | 'bottom' | 'left' | 'right';
  delay?: number;
  className?: string;
}

const positionStyles: Record<string, { container: string; arrow: string }> = {
  top: {
    container: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    arrow: 'left-1/2 -translate-x-1/2 top-full -mt-1',
  },
  bottom: {
    container: 'top-full left-1/2 -translate-x-1/2 mt-2',
    arrow: 'left-1/2 -translate-x-1/2 bottom-full -mb-1',
  },
  left: {
    container: 'right-full top-1/2 -translate-y-1/2 mr-2',
    arrow: 'right-0 top-1/2 -translate-y-1/2 -mr-1',
  },
  right: {
    container: 'left-full top-1/2 -translate-y-1/2 ml-2',
    arrow: 'left-0 top-1/2 -translate-y-1/2 -ml-1',
  },
};

const Tooltip: React.FC<TooltipProps> = ({
  content,
  children,
  position = 'top',
  delay = 200,
  className = '',
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const triggerRef = useRef<HTMLDivElement>(null);

  const showTooltip = useCallback(() => {
    timeoutRef.current = setTimeout(() => {
      setIsVisible(true);
    }, delay);
  }, [delay]);

  const hideTooltip = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
  }, []);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const posStyle = positionStyles[position];

  return (
    <div
      ref={triggerRef}
      className="relative inline-block"
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
    >
      {children}
      {isVisible && content && (
        <div
          className={`absolute z-50 ${posStyle.container} px-2.5 py-1.5 bg-tooltip text-tooltip text-[12px] rounded-md shadow-lg whitespace-nowrap pointer-events-none animate-fade-in ${className}`}
        >
          {content}
          <div
            className={`absolute w-2 h-2 bg-tooltip transform rotate-45 ${posStyle.arrow}`}
          />
        </div>
      )}
    </div>
  );
};

export default React.memo(Tooltip);
