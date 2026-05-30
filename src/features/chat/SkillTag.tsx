import React, { useState } from 'react';

const SkillTag: React.FC<{ slug: string; description?: string }> = ({ slug, description }) => {
  const [hover, setHover] = useState(false);
  return (
    <span className="relative inline" onMouseEnter={() => setHover(true)} onMouseLeave={() => setHover(false)}>
      <span className={`text-[#4B9EFA] font-medium cursor-default transition-colors ${hover ? 'bg-[#4B9EFA]/10 rounded px-0.5 -mx-0.5' : ''}`}>
        /{slug}
      </span>
      {hover && description && (
        <div className="absolute left-0 top-full mt-2 w-[240px] p-3 bg-claude-input border border-claude-border rounded-xl shadow-lg z-[100] pointer-events-none">
          <div className="text-[12px] text-claude-textSecondary leading-snug mb-1.5">{description.length > 150 ? description.slice(0, 150) + '...' : description}</div>
          <div className="text-[11px] text-claude-textSecondary/60">Skill</div>
        </div>
      )}
    </span>
  );
};

export default SkillTag;
