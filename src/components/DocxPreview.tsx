import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface DocxPreviewProps {
  content: string;
  title: string;
}

const FB = "Calibri, 'Segoe UI', sans-serif";
const FH = "'Calibri Light', 'Segoe UI', sans-serif";
const FM = "Consolas, 'Courier New', monospace";

const page: React.CSSProperties = {
  width: '100%', minHeight: 560, background: '#fff', borderRadius: 4,
  boxShadow: '0 1px 4px rgba(0,0,0,0.10), 0 0 0 1px rgba(0,0,0,0.04)',
  padding: '40px 48px', display: 'flex', flexDirection: 'column', fontFamily: FB,
};

const cover: React.CSSProperties = {
  ...page, minHeight: 420, justifyContent: 'center', alignItems: 'center',
};

const comp = {
  h1: ({ children }: any) => (
    <h1 className="text-[20px] font-bold text-[#1F3864] mt-7 mb-3 pb-1.5 border-b-2 border-[#2E75B6]"
      style={{ fontFamily: FH, lineHeight: 1.3 }}>{children}</h1>
  ),
  h2: ({ children }: any) => (
    <h2 className="text-[16px] font-bold text-[#2E75B6] mt-6 mb-2"
      style={{ fontFamily: FH, lineHeight: 1.3 }}>{children}</h2>
  ),
  h3: ({ children }: any) => (
    <h3 className="text-[13.5px] font-bold text-[#333] mt-4 mb-1.5"
      style={{ fontFamily: FH }}>{children}</h3>
  ),
  h4: ({ children }: any) => (
    <h4 className="text-[12.5px] font-semibold italic text-[#666] mt-3 mb-1"
      style={{ fontFamily: FH }}>{children}</h4>
  ),
  p: ({ children }: any) => (
    <p className="text-[12px] text-[#333] mb-2"
      style={{ fontFamily: FB, lineHeight: 1.65, textAlign: 'justify' }}>{children}</p>
  ),
  strong: ({ children }: any) => <strong className="font-bold text-[#222]">{children}</strong>,
  em: ({ children }: any) => <em className="italic">{children}</em>,
  a: ({ href, children }: any) => (
    <a href={href} className="text-[#4472C4] underline text-[12px]" target="_blank" rel="noopener noreferrer">{children}</a>
  ),
  ul: ({ children }: any) => <ul className="list-disc pl-5 mb-2 space-y-0.5">{children}</ul>,
  ol: ({ children }: any) => <ol className="list-decimal pl-5 mb-2 space-y-0.5">{children}</ol>,
  li: ({ children }: any) => (
    <li className="text-[12px] text-[#333]" style={{ fontFamily: FB, lineHeight: 1.55 }}>{children}</li>
  ),
  blockquote: ({ children }: any) => (
    <div className="border-l-[3px] border-[#4472C4] bg-[#EDF2F9] pl-3 pr-2 py-2 my-2 rounded-r">
      <div className="text-[11.5px] text-[#555] italic" style={{ fontFamily: FB }}>{children}</div>
    </div>
  ),
  code: ({ className, children, ...props }: any) => {
    if (className?.startsWith('language-')) {
      return (
        <div className="border border-[#D6DCE4] bg-[#F6F8FA] rounded my-2 px-3 py-2 overflow-x-auto">
          <pre className="text-[10.5px] text-[#333] whitespace-pre" style={{ fontFamily: FM, lineHeight: 1.5 }}>
            <code>{children}</code>
          </pre>
        </div>
      );
    }
    return (
      <code className="bg-[#F6F8FA] text-[#C7254E] px-1 py-0.5 rounded text-[10.5px] border border-[#E8E8E8]"
        style={{ fontFamily: FM }} {...props}>{children}</code>
    );
  },
  pre: ({ children }: any) => <>{children}</>,
  table: ({ children }: any) => (
    <div className="overflow-x-auto my-3">
      <table className="w-full border-collapse text-[11px]" style={{ fontFamily: FB }}>{children}</table>
    </div>
  ),
  thead: ({ children }: any) => <thead className="bg-[#2E75B6] text-white">{children}</thead>,
  tbody: ({ children }: any) => <tbody>{children}</tbody>,
  tr: ({ children }: any) => <tr className="even:bg-[#F2F7FB] border-b border-[#D6DCE4]">{children}</tr>,
  th: ({ children }: any) => (
    <th className="px-2.5 py-1.5 text-left font-semibold text-[11px] text-white border border-[#2E75B6]">{children}</th>
  ),
  td: ({ children }: any) => (
    <td className="px-2.5 py-1.5 text-[11px] text-[#333] border border-[#D6DCE4]">{children}</td>
  ),
  hr: () => <div className="my-6 border-t-2 border-[#D6DCE4]" />,
};

const DocxPreview: React.FC<DocxPreviewProps> = ({ content, title }) => {
  const d = new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' });
  return (
    <div className="flex flex-col items-center gap-5 pb-8">
      <div style={cover}>
        <div className="w-20 h-[3px] bg-[#2E75B6] mb-8" />
        <h1 className="text-[26px] font-bold text-[#1F3864] text-center leading-tight mb-3"
          style={{ fontFamily: FH }}>{title}</h1>
        <div className="w-20 h-[3px] bg-[#2E75B6] mt-8 mb-6" />
        <p className="text-[13px] text-[#888] mt-2" style={{ fontFamily: FB }}>{d}</p>
      </div>
      <div style={page}>
        <div className="flex justify-end pb-1.5 mb-3 border-b border-[#D6DCE4]">
          <span className="text-[9px] text-[#aaa] italic" style={{ fontFamily: FB }}>{title}</span>
        </div>
        <div className="flex-1">
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={comp}>{content}</ReactMarkdown>
        </div>
        <div className="flex justify-center pt-2 mt-4 border-t border-[#D6DCE4]">
          <span className="text-[9px] text-[#aaa]" style={{ fontFamily: FB }}>— 1 —</span>
        </div>
      </div>
    </div>
  );
};

export default DocxPreview;
