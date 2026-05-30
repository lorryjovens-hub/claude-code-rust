import React, { useEffect, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { X } from 'lucide-react';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
  className?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closeOnOverlayClick?: boolean;
  showCloseButton?: boolean;
}

const sizeStyles: Record<string, string> = {
  sm: 'w-[400px]',
  md: 'w-[560px]',
  lg: 'w-[720px]',
  xl: 'w-[900px]',
};

const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  className = '',
  size = 'md',
  closeOnOverlayClick = true,
  showCloseButton = true,
}) => {
  const handleEscape = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  }, [onClose]);

  useEffect(() => {
    if (!isOpen) return;

    document.addEventListener('keydown', handleEscape);
    document.body.style.overflow = 'hidden';

    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = '';
    };
  }, [isOpen, handleEscape]);

  if (!isOpen) return null;

  return createPortal(
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/40 animate-fade-in"
      onClick={closeOnOverlayClick ? onClose : undefined}
    >
      <div
        className={`bg-claude-input rounded-2xl shadow-xl ${sizeStyles[size]} max-h-[85vh] flex flex-col animate-fade-in ${className}`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        {(title || showCloseButton) && (
          <div className="flex items-center justify-between px-6 py-4 border-b border-claude-border">
            {title && (
              <h2 className="text-[18px] font-semibold text-claude-text">{title}</h2>
            )}
            {showCloseButton && (
              <button
                onClick={onClose}
                className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover rounded-lg transition-colors"
              >
                <X size={18} />
              </button>
            )}
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4">
          {children}
        </div>
      </div>
    </div>,
    document.body
  );
};

export default React.memo(Modal);
