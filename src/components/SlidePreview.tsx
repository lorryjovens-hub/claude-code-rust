import React, { useState } from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { SlideInfo } from './DocumentCard';

const COLOR_SCHEMES: Record<string, { primary: string; secondary: string; accent: string; text: string; textDark: string }> = {
  ocean: { primary: '#1A5276', secondary: '#2E86C1', accent: '#3498DB', text: '#FFFFFF', textDark: '#2C3E50' },
  forest: { primary: '#1E8449', secondary: '#27AE60', accent: '#2ECC71', text: '#FFFFFF', textDark: '#1B4332' },
  sunset: { primary: '#E74C3C', secondary: '#E67E22', accent: '#F39C12', text: '#FFFFFF', textDark: '#641E16' },
  lavender: { primary: '#6C3483', secondary: '#8E44AD', accent: '#BB8FCE', text: '#FFFFFF', textDark: '#4A235A' },
  slate: { primary: '#2C3E50', secondary: '#34495E', accent: '#D97757', text: '#FFFFFF', textDark: '#1C2833' },
  coral: { primary: '#C0392B', secondary: '#E74C3C', accent: '#F1948A', text: '#FFFFFF', textDark: '#641E16' },
  teal: { primary: '#008080', secondary: '#20B2AA', accent: '#48D1CC', text: '#FFFFFF', textDark: '#004D4D' },
  midnight: { primary: '#1B2631', secondary: '#2C3E50', accent: '#5DADE2', text: '#FFFFFF', textDark: '#0D1117' },
  rose: { primary: '#C2185B', secondary: '#E91E63', accent: '#F48FB1', text: '#FFFFFF', textDark: '#880E4F' },
  emerald: { primary: '#00695C', secondary: '#00897B', accent: '#4DB6AC', text: '#FFFFFF', textDark: '#004D40' },
  amber: { primary: '#FF8F00', secondary: '#FFA000', accent: '#FFD54F', text: '#FFFFFF', textDark: '#E65100' },
  indigo: { primary: '#283593', secondary: '#3949AB', accent: '#7986CB', text: '#FFFFFF', textDark: '#1A237E' },
  charcoal: { primary: '#37474F', secondary: '#546E7A', accent: '#D97757', text: '#FFFFFF', textDark: '#263238' },
  burgundy: { primary: '#7B1FA2', secondary: '#9C27B0', accent: '#CE93D8', text: '#FFFFFF', textDark: '#4A148C' },
  steel: { primary: '#455A64', secondary: '#607D8B', accent: '#90A4AE', text: '#FFFFFF', textDark: '#263238' },
  professional: { primary: '#1565C0', secondary: '#1976D2', accent: '#42A5F5', text: '#FFFFFF', textDark: '#0D47A1' },
  warm: { primary: '#D97757', secondary: '#E8956A', accent: '#F5C6A8', text: '#FFFFFF', textDark: '#5D3A2A' },
  minimal: { primary: '#424242', secondary: '#616161', accent: '#BDBDBD', text: '#FFFFFF', textDark: '#212121' },
};

interface SlidePreviewProps {
  slides: SlideInfo[];
  title: string;
  colorScheme?: string;
}

const SlidePreview: React.FC<SlidePreviewProps> = ({ slides, title, colorScheme }) => {
  const [expandedNotes, setExpandedNotes] = useState<Set<number>>(new Set());
  const c = COLOR_SCHEMES[colorScheme || ''] || COLOR_SCHEMES.warm;

  const toggleNotes = (index: number) => {
    setExpandedNotes(prev => {
      const next = new Set(prev);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return next;
    });
  };

  const parseBullets = (content: string) =>
    content.split('\n').filter(l => l.trim()).map(l => l.replace(/^[-*•]\s*/, ''));

  const renderSlide = (slide: SlideInfo, i: number) => {
    const layout = slide.layout || 'content';

    if (layout === 'cover') {
      return (
        <div className="absolute inset-0 flex flex-col items-center justify-center" style={{ backgroundColor: c.primary }}>
          <div className="w-full h-1 absolute top-0" style={{ backgroundColor: c.accent }} />
          <h2 className="text-[20px] font-bold text-center px-6" style={{ color: c.text }}>{slide.title}</h2>
          {slide.content && (
            <p className="text-[13px] mt-2 px-6 text-center" style={{ color: c.accent }}>
              {slide.content.split('\n')[0]}
            </p>
          )}
          <div className="w-full h-1 absolute bottom-0" style={{ backgroundColor: c.accent }} />
        </div>
      );
    }

    if (layout === 'section') {
      return (
        <div className="absolute inset-0 flex flex-col justify-center p-6" style={{ backgroundColor: c.secondary }}>
          <h2 className="text-[18px] font-bold mb-2" style={{ color: c.text }}>{slide.title}</h2>
          <div className="w-12 h-0.5 mb-3" style={{ backgroundColor: c.accent }} />
          {slide.content && (
            <p className="text-[12px]" style={{ color: c.accent }}>{slide.content.split('\n')[0]}</p>
          )}
          <div className="absolute bottom-3 right-4 text-[10px]" style={{ color: 'rgba(255,255,255,0.5)' }}>
            {i + 1} / {slides.length}
          </div>
        </div>
      );
    }

    if (layout === 'two_column') {
      const leftBullets = parseBullets(slide.left_content || '');
      const rightBullets = parseBullets(slide.right_content || '');
      return (
        <div className="absolute inset-0 p-6 flex flex-col bg-white">
          <div className="w-full h-1 rounded mb-3 flex-shrink-0" style={{ backgroundColor: c.primary }} />
          <h3 className="text-[15px] font-bold mb-1 flex-shrink-0" style={{ color: c.textDark }}>{slide.title}</h3>
          <div className="w-16 h-0.5 mb-3 flex-shrink-0" style={{ backgroundColor: c.primary }} />
          <div className="flex-1 flex gap-3 overflow-hidden">
            <div className="flex-1">
              {leftBullets.map((b, j) => (
                <div key={j} className="flex items-start gap-1.5 mb-1">
                  <span className="mt-0.5 flex-shrink-0 text-[10px]" style={{ color: c.primary }}>●</span>
                  <span className="text-[11px] text-[#444] leading-relaxed">{b}</span>
                </div>
              ))}
            </div>
            <div className="w-px flex-shrink-0" style={{ backgroundColor: '#DDD' }} />
            <div className="flex-1">
              {rightBullets.map((b, j) => (
                <div key={j} className="flex items-start gap-1.5 mb-1">
                  <span className="mt-0.5 flex-shrink-0 text-[10px]" style={{ color: c.accent }}>●</span>
                  <span className="text-[11px] text-[#444] leading-relaxed">{b}</span>
                </div>
              ))}
            </div>
          </div>
          <div className="text-[10px] text-[#999] text-right flex-shrink-0">{i + 1} / {slides.length}</div>
        </div>
      );
    }

    if (layout === 'summary') {
      const bullets = parseBullets(slide.content || '');
      return (
        <div className="absolute inset-0 p-6 flex flex-col" style={{ backgroundColor: c.primary }}>
          <div className="w-full h-1 rounded mb-4 flex-shrink-0" style={{ backgroundColor: c.accent }} />
          <h3 className="text-[16px] font-bold text-center mb-4 flex-shrink-0" style={{ color: c.text }}>{slide.title}</h3>
          <div className="flex-1 overflow-hidden flex flex-col items-center">
            {bullets.map((b, j) => (
              <div key={j} className="flex items-start gap-2 mb-2 max-w-[80%]">
                <span className="mt-0.5 flex-shrink-0 text-[11px]" style={{ color: c.accent }}>✓</span>
                <span className="text-[12px] leading-relaxed" style={{ color: c.accent }}>{b}</span>
              </div>
            ))}
          </div>
          <div className="text-[10px] text-right flex-shrink-0" style={{ color: 'rgba(255,255,255,0.5)' }}>
            {i + 1} / {slides.length}
          </div>
        </div>
      );
    }

    // Default: content layout
    const bullets = parseBullets(slide.content || '');
    return (
      <div className="absolute inset-0 p-6 flex flex-col bg-white">
        <div className="w-full h-1 rounded mb-4 flex-shrink-0" style={{ backgroundColor: c.primary }} />
        <h3 className="text-[16px] font-bold mb-1 flex-shrink-0" style={{ color: c.textDark }}>{slide.title}</h3>
        <div className="w-16 h-0.5 mb-3 flex-shrink-0" style={{ backgroundColor: c.primary }} />
        <div className="flex-1 overflow-hidden">
          {bullets.map((line, j) => (
            <div key={j} className="flex items-start gap-2 mb-1.5">
              <span className="mt-1 flex-shrink-0 text-[10px]" style={{ color: c.primary }}>●</span>
              <span className="text-[13px] text-[#444] leading-relaxed">{line}</span>
            </div>
          ))}
        </div>
        <div className="text-[11px] text-[#999] text-right flex-shrink-0">{i + 1} / {slides.length}</div>
      </div>
    );
  };

  return (
    <div className="space-y-4">
      <div className="text-[13px] text-[#999] mb-2">{slides.length} slides</div>
      {slides.map((slide, i) => (
        <div key={i} className="border border-[#E5E5E5] rounded-lg overflow-hidden">
          <div className="relative" style={{ paddingBottom: '56.25%' }}>
            {renderSlide(slide, i)}
          </div>
          {slide.notes && (
            <div className="border-t border-[#E5E5E5]">
              <button
                onClick={() => toggleNotes(i)}
                className="w-full flex items-center gap-1.5 px-4 py-2 text-[12px] text-[#777] hover:bg-[#FAFAFA] transition-colors"
              >
                {expandedNotes.has(i) ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                演讲者备注
              </button>
              {expandedNotes.has(i) && (
                <div className="px-4 pb-3 text-[13px] text-[#555] leading-relaxed whitespace-pre-wrap">
                  {slide.notes}
                </div>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default SlidePreview;
