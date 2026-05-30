import React, { useState, useCallback } from 'react';
import { ChevronLeft, ChevronRight, Lightbulb, BookOpen, Gamepad2, Palette, Trees } from 'lucide-react';
import inspirationsData from '../data/inspirations.json';

interface InspirationItem {
  artifact_id: string;
  chat_id: string;
  category: string;
  name: string;
  description: string;
  starting_prompt: string;
  img_src: string;
  content_uuid: string;
  code_file: string;
}

interface PromptSuggestionsPanelProps {
  onSelectPrompt: (prompt: string) => void;
}

const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  learn: <BookOpen size={16} />,
  'life-hacks': <Lightbulb size={16} />,
  games: <Gamepad2 size={16} />,
  creative: <Palette size={16} />,
  'touch-grass': <Trees size={16} />,
};

const CATEGORY_LABELS: Record<string, string> = {
  learn: 'Learn something',
  'life-hacks': 'Life hacks',
  games: 'Play a game',
  creative: 'Be creative',
  'touch-grass': 'Touch grass',
};

const CATEGORY_ORDER = ['learn', 'life-hacks', 'games', 'creative', 'touch-grass'];

const PromptSuggestionsPanel: React.FC<PromptSuggestionsPanelProps> = ({ onSelectPrompt }) => {
  const [activeCategory, setActiveCategory] = useState<string>('life-hacks');
  const [scrollPositions, setScrollPositions] = useState<Record<string, number>>({});

  const items = (inspirationsData as { items: InspirationItem[] }).items;
  const categoryItems = items.filter(item => item.category === activeCategory);

  const handleScroll = useCallback((direction: 'left' | 'right', container: HTMLDivElement | null) => {
    if (!container) return;
    const scrollAmount = 280;
    const newPos = direction === 'left'
      ? container.scrollLeft - scrollAmount
      : container.scrollLeft + scrollAmount;
    container.scrollTo({ left: newPos, behavior: 'smooth' });
    setScrollPositions(prev => ({ ...prev, [activeCategory]: newPos }));
  }, [activeCategory]);

  const containerRefs = React.useRef<Record<string, HTMLDivElement | null>>({});

  const setContainerRef = useCallback((category: string, el: HTMLDivElement | null) => {
    containerRefs.current[category] = el;
  }, []);

  return (
    <div className="w-full mt-8">
      {/* Category tabs */}
      <div className="flex items-center justify-center gap-2 mb-4">
        {CATEGORY_ORDER.map(cat => {
          const isActive = cat === activeCategory;
          return (
            <button
              key={cat}
              onClick={() => setActiveCategory(cat)}
              className={`flex items-center gap-2 px-4 py-2 rounded-full text-[14px] font-medium transition-all duration-200 ${
                isActive
                  ? 'bg-claude-text text-claude-bg shadow-sm'
                  : 'bg-transparent text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover'
              }`}
            >
              {CATEGORY_ICONS[cat]}
              <span>{CATEGORY_LABELS[cat]}</span>
            </button>
          );
        })}
      </div>

      {/* Suggestion cards */}
      <div className="relative">
        {/* Left scroll arrow */}
        <button
          onClick={() => handleScroll('left', containerRefs.current[activeCategory])}
          className="absolute left-0 top-1/2 -translate-y-1/2 z-10 w-8 h-8 flex items-center justify-center bg-claude-bg/80 backdrop-blur-sm rounded-full shadow-md text-claude-textSecondary hover:text-claude-text hover:bg-claude-bg transition-colors"
        >
          <ChevronLeft size={18} />
        </button>

        {/* Cards container */}
        <div
          ref={(el) => setContainerRef(activeCategory, el)}
          className="flex gap-4 overflow-x-auto scrollbar-hide px-10 py-2 scroll-smooth"
          style={{ scrollbarWidth: 'none', msOverflowStyle: 'none' }}
        >
          {categoryItems.map(item => (
            <button
              key={item.artifact_id}
              onClick={() => onSelectPrompt(item.starting_prompt)}
              className="flex-shrink-0 w-[260px] bg-claude-input border border-claude-border rounded-xl p-4 text-left hover:border-[#CCC] dark:hover:border-[#5a5a58] hover:shadow-md transition-all duration-200 group"
            >
              <div className="flex items-start gap-3 mb-2">
                <div className="w-8 h-8 rounded-lg bg-claude-bg flex items-center justify-center text-claude-textSecondary group-hover:text-claude-text transition-colors flex-shrink-0">
                  {CATEGORY_ICONS[item.category]}
                </div>
                <div className="min-w-0">
                  <h3 className="text-[14px] font-semibold text-claude-text truncate">{item.name}</h3>
                </div>
              </div>
              <p className="text-[13px] text-claude-textSecondary leading-snug line-clamp-2">
                {item.description}
              </p>
            </button>
          ))}
        </div>

        {/* Right scroll arrow */}
        <button
          onClick={() => handleScroll('right', containerRefs.current[activeCategory])}
          className="absolute right-0 top-1/2 -translate-y-1/2 z-10 w-8 h-8 flex items-center justify-center bg-claude-bg/80 backdrop-blur-sm rounded-full shadow-md text-claude-textSecondary hover:text-claude-text hover:bg-claude-bg transition-colors"
        >
          <ChevronRight size={18} />
        </button>
      </div>
    </div>
  );
};

export default PromptSuggestionsPanel;
