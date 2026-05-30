import React from 'react';

interface SkillInputOverlayProps {
  text: string;
  className?: string;
  style?: React.CSSProperties;
}

export const SkillInputOverlay: React.FC<SkillInputOverlayProps> = ({ text, className, style }) => {
  const match = text.match(/^(\/[a-zA-Z0-9_-]+)([\s\S]*)$/);
  if (!match) return null;
  return (
    <div className={className} style={{ ...style, pointerEvents: 'none', position: 'absolute', top: 0, left: 0, right: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-word' }} aria-hidden>
      <span className="text-[#4B9EFA]">{match[1]}</span>
      <span className="text-claude-text">{match[2] || ''}</span>
    </div>
  );
};
