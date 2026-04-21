import React, { useState, useEffect, useRef } from 'react';
import { ChevronDown, Check, ChevronRight } from 'lucide-react';

export interface SelectableModel {
  id: string;
  name: string;
  enabled: number;
  description?: string;
  tier?: 'opus' | 'sonnet' | 'haiku' | 'extra';
}

// Chat model thinking mapping from localStorage
function getChatModelMap(): Map<string, { thinkingId?: string }> {
  try {
    const models = JSON.parse(localStorage.getItem('chat_models') || '[]');
    const map = new Map<string, { thinkingId?: string }>();
    for (const m of models) {
      map.set(m.id, { thinkingId: m.thinkingId });
      if (m.thinkingId) map.set(m.thinkingId, { thinkingId: m.thinkingId });
    }
    return map;
  } catch { return new Map(); }
}

function stripThinking(modelStr: string) {
  const map = getChatModelMap();
  // Check if this is a known thinking variant — find the base model
  for (const [baseId, cfg] of map) {
    if (cfg.thinkingId === modelStr) return baseId;
  }
  return (modelStr || '').replace(/-thinking$/, '');
}

function withThinking(base: string, thinking: boolean) {
  if (!thinking) return base;
  const map = getChatModelMap();
  const cfg = map.get(base);
  if (cfg?.thinkingId) return cfg.thinkingId;
  return `${base}-thinking`;
}

function isThinking(modelStr: string) {
  const map = getChatModelMap();
  // Check if it's a known thinking variant
  for (const [, cfg] of map) {
    if (cfg.thinkingId === modelStr) return true;
  }
  return typeof modelStr === 'string' && modelStr.endsWith('-thinking');
}

function hasThinkingVariant(_modelId: string): boolean {
  // All models can toggle extended thinking.
  // Models that don't support it will simply ignore the parameter — no harm done.
  return true;
}

// Turn raw model ids / names into friendly labels.
// - claude-opus-4-6 → "Opus 4.6", claude-haiku-4-5-20251001 → "Haiku 4.5"
// - GLM-5 → "GLM 5", Deepseek-V3.2 → "Deepseek V3.2" (hyphens become spaces)
// - Strips provider/org prefix (e.g. "Pro/zai-org/GLM-5" → "GLM 5")
function prettifyModelName(name?: string, id?: string): string {
  for (const candidate of [id, name]) {
    if (!candidate) continue;
    const m = candidate.match(/(opus|sonnet|haiku)-(\d+)-(\d+)/i);
    if (m) {
      const tier = m[1][0].toUpperCase() + m[1].slice(1).toLowerCase();
      return `${tier} ${m[2]}.${m[3]}`;
    }
  }
  const raw = name || id || '';
  if (!raw) return 'Model';
  const lastSlash = raw.lastIndexOf('/');
  const trimmed = lastSlash >= 0 ? raw.slice(lastSlash + 1) : raw;
  return trimmed.replace(/-/g, ' ');
}

interface ModelSelectorProps {
  currentModelString: string;
  models: SelectableModel[];
  onModelChange: (newModelString: string) => void;
  isNewChat?: boolean;
  dropdownPosition?: 'top' | 'bottom';
}

const ModelSelector: React.FC<ModelSelectorProps> = ({
  currentModelString,
  models,
  onModelChange,
  dropdownPosition,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [dropUp, setDropUp] = useState(false);
  const [showMore, setShowMore] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const currentBase = stripThinking(currentModelString);
  const thinking = isThinking(currentModelString);
  const currentModel = models.find(m => m && m.id === currentBase);
  const currentLabel = prettifyModelName(currentModel?.name, currentModel?.id || currentBase);

  // Split models into main tiers and extra
  const mainModels = models.filter(m => m && m.tier !== 'extra');
  const extraModels = models.filter(m => m && m.tier === 'extra');
  const hasExtra = extraModels.length > 0;

  // Current model supports thinking?
  const currentHasThinking = hasThinkingVariant(currentBase);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setShowMore(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleToggleOpen = () => {
    if (!isOpen && containerRef.current) {
      const rect = containerRef.current.getBoundingClientRect();
      const spaceBelow = window.innerHeight - rect.bottom;
      setDropUp(dropdownPosition === 'top' ? true : (dropdownPosition === 'bottom' ? false : spaceBelow < 280));
    }
    setIsOpen(!isOpen);
    setShowMore(false);
  };

  const handleSelectModel = (baseId: string, enabled: number) => {
    if (!enabled) return;
    // If switching to a model without thinking variant, auto-disable thinking
    const targetHasThinking = hasThinkingVariant(baseId);
    onModelChange(withThinking(baseId, targetHasThinking ? thinking : false));
    setIsOpen(false);
    setShowMore(false);
  };

  const handleToggleThinking = () => {
    if (!currentHasThinking) return;
    onModelChange(withThinking(currentBase, !thinking));
  };

  const renderModelItem = (m: SelectableModel) => {
    const active = currentBase === m.id;
    const disabled = Number(m.enabled) !== 1;
    return (
      <button
        key={m.id || Math.random()}
        onClick={() => handleSelectModel(m.id, m.enabled)}
        disabled={disabled}
        className={`w-full px-4 ${m.description ? 'py-2.5' : 'py-2'} flex items-center justify-between text-left ${disabled ? 'opacity-45 cursor-not-allowed' : 'hover:bg-claude-hover cursor-pointer'}`}
      >
        <div className="flex-1 min-w-0">
          <div className="text-[14.5px] font-[500] text-claude-text truncate">{prettifyModelName(m.name, m.id)}</div>
          {m.description && <div className="text-[12.5px] text-claude-textSecondary mt-0.5">{m.description}</div>}
        </div>
        {active && <Check size={18} className="text-[#3b82f6] ml-2 shrink-0" />}
      </button>
    );
  };

  return (
    <div className="relative inline-block text-right" ref={containerRef}>
      <button
        onClick={handleToggleOpen}
        className="flex items-center gap-1.5 text-[15px] font-medium text-claude-text hover:bg-claude-hover px-3 py-2 rounded-md transition-colors"
      >
        <span>{currentLabel}</span>
        {thinking && <span className="text-claude-textSecondary font-normal">Extended</span>}
        <ChevronDown size={14} className="text-claude-textSecondary" />
      </button>

      {isOpen && !showMore && (
        <div className={`absolute ${dropUp ? 'bottom-full mb-2' : 'top-full mt-2'} right-0 w-[260px] bg-claude-input rounded-xl shadow-xl border border-claude-border z-50 overflow-hidden py-1 text-left`}>
          {/* Main tier models */}
          {mainModels.map(renderModelItem)}

          {/* Extended thinking toggle */}
          <div className="h-[1px] bg-claude-border my-1 mx-4" />
          <div className={`px-4 py-2 flex items-center justify-between text-left select-none ${currentHasThinking ? 'hover:bg-claude-hover cursor-pointer' : ''}`}>
            <div className="flex-1">
              <div className={`text-[14.5px] font-[500] ${currentHasThinking ? 'text-claude-text' : 'text-claude-textSecondary/50'}`}>Extended thinking</div>
              <div className={`text-[12.5px] mt-0.5 ${currentHasThinking ? 'text-claude-textSecondary' : 'text-claude-textSecondary/30'}`}>Think longer for complex tasks</div>
            </div>
            <button
              onClick={(e) => {
                e.stopPropagation();
                handleToggleThinking();
              }}
              disabled={!currentHasThinking}
              className={`w-10 h-6 rounded-full relative transition-colors duration-200 ${!currentHasThinking ? 'bg-claude-border/50 cursor-not-allowed' : thinking ? 'bg-[#3A6FE0]' : 'bg-claude-border cursor-pointer'}`}
            >
              <div className={`absolute top-1 w-4 h-4 rounded-full bg-white shadow-sm transition-transform duration-200 ${thinking && currentHasThinking ? 'left-5' : 'left-1'}`} />
            </button>
          </div>

          {/* More models button */}
          {hasExtra && (<>
            <div className="h-[1px] bg-claude-border my-1 mx-4" />
            <button
              onClick={() => setShowMore(true)}
              className="w-full px-4 py-2.5 flex items-center justify-between text-left hover:bg-claude-hover cursor-pointer"
            >
              <div className="text-[14.5px] font-[500] text-claude-text">More models</div>
              <ChevronRight size={16} className="text-claude-textSecondary" />
            </button>
          </>)}
        </div>
      )}

      {/* More models sub-panel */}
      {isOpen && showMore && (
        <div className={`absolute ${dropUp ? 'bottom-full mb-2' : 'top-full mt-2'} right-0 w-[260px] bg-claude-input rounded-xl shadow-xl border border-claude-border z-50 overflow-hidden py-1 text-left`}>
          <button
            onClick={() => setShowMore(false)}
            className="w-full px-4 py-2 flex items-center gap-2 text-left hover:bg-claude-hover cursor-pointer text-claude-textSecondary"
          >
            <ChevronRight size={14} className="rotate-180" />
            <span className="text-[13px] font-medium">Back</span>
          </button>
          <div className="h-[1px] bg-claude-border my-1 mx-4" />
          {extraModels.map(renderModelItem)}
        </div>
      )}
    </div>
  );
};

export default ModelSelector;

