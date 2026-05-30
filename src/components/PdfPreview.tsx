import React from 'react';
import { PdfSection } from './DocumentCard';

interface PdfPreviewProps {
  sections: PdfSection[];
  title: string;
}

const FF = "Georgia, 'Noto Serif SC', 'Source Han Serif SC', serif";
const FS = "'Helvetica Neue', Arial, 'PingFang SC', sans-serif";

const page: React.CSSProperties = {
  width: '100%', minHeight: 560, background: '#fff', borderRadius: 4,
  boxShadow: '0 1px 4px rgba(0,0,0,0.10), 0 0 0 1px rgba(0,0,0,0.04)',
  padding: '40px 48px', display: 'flex', flexDirection: 'column', fontFamily: FS,
};

const cover: React.CSSProperties = {
  ...page, minHeight: 420, justifyContent: 'center', alignItems: 'center',
};

const ACCENT = '#4472C4';

const renderSection = (sec: PdfSection, idx: number) => {
  switch (sec.type) {
    case 'heading': {
      const level = sec.level || 1;
      const content = typeof sec.content === 'string' ? sec.content : '';
      if (level === 1) return (
        <h1 key={idx} className="text-[20px] font-bold mt-7 mb-3" style={{ fontFamily: FS, color: ACCENT, lineHeight: 1.3 }}>{content}</h1>
      );
      if (level === 2) return (
        <h2 key={idx} className="text-[16px] font-bold text-[#2D2D2D] mt-6 mb-2" style={{ fontFamily: FS, lineHeight: 1.3 }}>{content}</h2>
      );
      return (
        <h3 key={idx} className="text-[13.5px] font-bold text-[#444] mt-4 mb-1.5" style={{ fontFamily: FS }}>{content}</h3>
      );
    }
    case 'paragraph': {
      const text = typeof sec.content === 'string' ? sec.content : '';
      return (
        <div key={idx}>
          {text.split('\n\n').map((para, pi) => {
            const trimmed = para.trim();
            return trimmed ? (
              <p key={pi} className="text-[12px] text-[#2D2D2D] mb-2"
                style={{ fontFamily: FS, lineHeight: 1.7, textAlign: 'justify' }}>{trimmed}</p>
            ) : null;
          })}
        </div>
      );
    }
    case 'table': {
      const headers = sec.headers || [];
      const rows = sec.rows || [];
      return (
        <div key={idx} className="overflow-x-auto my-3">
          <table className="w-full border-collapse text-[11px]" style={{ fontFamily: FS }}>
            {headers.length > 0 && (
              <thead>
                <tr style={{ background: ACCENT }}>
                  {headers.map((h, hi) => (
                    <th key={hi} className="px-2.5 py-1.5 text-left font-medium text-white text-[11px]"
                      style={{ border: '0.5px solid #D9D9D9' }}>{h}</th>
                  ))}
                </tr>
              </thead>
            )}
            <tbody>
              {rows.map((row, ri) => (
                <tr key={ri} style={{ background: ri % 2 === 1 ? '#F5F7FA' : '#fff' }}>
                  {row.map((cell, ci) => (
                    <td key={ci} className="px-2.5 py-1.5 text-[11px] text-[#2D2D2D]"
                      style={{ border: '0.5px solid #D9D9D9' }}>{cell ?? ''}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    }
    case 'list': {
      const items = Array.isArray(sec.content) ? sec.content
        : typeof sec.content === 'string' ? sec.content.split('\n').filter(l => l.trim()) : [];
      const Tag = sec.ordered ? 'ol' : 'ul';
      return (
        <Tag key={idx} className={`${sec.ordered ? 'list-decimal' : 'list-disc'} pl-5 mb-2 space-y-0.5`}>
          {items.map((item, li) => (
            <li key={li} className="text-[12px] text-[#2D2D2D]"
              style={{ fontFamily: FS, lineHeight: 1.6 }}>{item}</li>
          ))}
        </Tag>
      );
    }
    case 'pagebreak':
      return <div key={idx} className="my-6 border-t-2 border-dashed border-[#D9D9D9]" />;
    default:
      return null;
  }
};

const PdfPreview: React.FC<PdfPreviewProps> = ({ sections, title }) => {
  return (
    <div className="flex flex-col items-center gap-5 pb-8">
      {/* Cover page */}
      <div style={cover}>
        <div className="w-16 h-[2px] mb-8" style={{ background: ACCENT }} />
        <h1 className="text-[26px] font-bold text-[#2D2D2D] text-center leading-tight mb-3"
          style={{ fontFamily: FS }}>{title}</h1>
        <div className="w-16 h-[2px] mt-8" style={{ background: ACCENT }} />
      </div>
      {/* Content pages */}
      <div style={page}>
        <div className="flex justify-between pb-1.5 mb-3 border-b" style={{ borderColor: ACCENT }}>
          <span className="text-[9px] text-[#999]" style={{ fontFamily: FS }}>{title}</span>
        </div>
        <div className="flex-1">
          {sections.map((sec, idx) => renderSection(sec, idx))}
        </div>
        <div className="flex justify-center pt-2 mt-4 border-t border-[#D9D9D9]">
          <span className="text-[9px] text-[#999]" style={{ fontFamily: FS }}>Page 1</span>
        </div>
      </div>
    </div>
  );
};

export default PdfPreview;
