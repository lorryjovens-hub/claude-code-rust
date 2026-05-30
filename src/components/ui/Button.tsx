import React from 'react';
import { Loader2 } from 'lucide-react';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  isLoading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}

const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-claude-text text-claude-bg hover:opacity-90 active:opacity-80 disabled:opacity-50',
  secondary: 'bg-white dark:bg-claude-input text-claude-text border border-black/10 dark:border-white/10 hover:bg-gray-50 dark:hover:bg-claude-hover active:bg-gray-100 dark:active:bg-claude-hover/80 disabled:opacity-50',
  ghost: 'bg-transparent text-claude-text hover:bg-claude-hover active:bg-claude-hover/80 disabled:opacity-50',
  danger: 'bg-red-500 text-white hover:bg-red-600 active:bg-red-700 disabled:opacity-50',
};

const sizeStyles: Record<ButtonSize, string> = {
  sm: 'px-2.5 py-1.5 text-[12px] gap-1',
  md: 'px-3.5 py-2 text-[14px] gap-1.5',
  lg: 'px-5 py-2.5 text-[15px] gap-2',
};

const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  size = 'md',
  isLoading = false,
  leftIcon,
  rightIcon,
  children,
  className = '',
  disabled,
  ...props
}) => {
  const baseClasses = 'inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200 cursor-pointer select-none focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:ring-offset-2 dark:focus:ring-offset-claude-bg';

  return (
    <button
      className={`${baseClasses} ${variantStyles[variant]} ${sizeStyles[size]} ${className}`}
      disabled={disabled || isLoading}
      {...props}
    >
      {isLoading && <Loader2 size={size === 'sm' ? 12 : 14} className="animate-spin" />}
      {!isLoading && leftIcon && <span className="flex-shrink-0">{leftIcon}</span>}
      {children}
      {!isLoading && rightIcon && <span className="flex-shrink-0">{rightIcon}</span>}
    </button>
  );
};

export default React.memo(Button);
