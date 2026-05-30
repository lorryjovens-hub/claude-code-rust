import React from 'react';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  className?: string;
  wrapperClassName?: string;
}

const Input: React.FC<InputProps> = ({
  label,
  error,
  leftIcon,
  rightIcon,
  className = '',
  wrapperClassName = '',
  id,
  ...props
}) => {
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;

  return (
    <div className={`flex flex-col gap-1.5 ${wrapperClassName}`}>
      {label && (
        <label htmlFor={inputId} className="text-[13px] font-medium text-claude-text">
          {label}
        </label>
      )}
      <div className="relative">
        {leftIcon && (
          <div className="absolute left-3 top-1/2 -translate-y-1/2 text-claude-textSecondary pointer-events-none">
            {leftIcon}
          </div>
        )}
        <input
          id={inputId}
          className={`
            w-full px-3 py-2 bg-claude-input border border-claude-border rounded-lg 
            text-claude-text placeholder-claude-textSecondary
            focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500
            transition-all duration-200 text-[14px]
            ${leftIcon ? 'pl-10' : ''}
            ${rightIcon ? 'pr-10' : ''}
            ${error ? 'border-red-500 focus:border-red-500 focus:ring-red-500/20' : ''}
            ${className}
          `}
          {...props}
        />
        {rightIcon && (
          <div className="absolute right-3 top-1/2 -translate-y-1/2 text-claude-textSecondary">
            {rightIcon}
          </div>
        )}
      </div>
      {error && (
        <span className="text-[12px] text-red-500">{error}</span>
      )}
    </div>
  );
};

export default React.memo(Input);
