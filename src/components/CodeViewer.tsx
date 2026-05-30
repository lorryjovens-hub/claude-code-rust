import React, { useState, useEffect } from 'react';
import { Copy, Check, ChevronDown, ChevronRight, FileCode } from 'lucide-react';
import { copyToClipboard } from '../utils/clipboard';

export interface CodeViewerProps {
  code: string;
  language?: string;
  fileName?: string;
  lineNumbers?: boolean;
}

const CodeViewer: React.FC<CodeViewerProps> = ({
  code,
  language,
  fileName,
  lineNumbers = true,
}) => {
  const [isDark, setIsDark] = useState(false);
  const [copied, setCopied] = useState(false);
  const [collapsed, setCollapsed] = useState(false);

  useEffect(() => {
    const check = () => setIsDark(document.documentElement.classList.contains('dark'));
    check();
    const observer = new MutationObserver(check);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => observer.disconnect();
  }, []);

  const lines = code.split('\n');
  const displayName = fileName || (language ? `code.${language}` : 'code');

  const handleCopy = () => {
    copyToClipboard(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={`rounded-md overflow-hidden border text-[12px] font-mono ${isDark ? 'border-[#383836] bg-[#1e1e1e]' : 'border-[#E5E5E5] bg-[#FCFCFA]'}`}>
      {/* Header */}
      <div
        className={`flex items-center justify-between px-3 py-1.5 cursor-pointer ${isDark ? 'bg-[#2d2d2d] border-b border-[#383836]' : 'bg-[#f5f5f0] border-b border-[#E5E5E5]'}`}
        onClick={() => setCollapsed(!collapsed)}
      >
        <div className="flex items-center gap-2 min-w-0">
          <button className="p-0.5">
            {collapsed ? <ChevronRight size={14} className={isDark ? 'text-[#999]' : 'text-[#666]'} /> : <ChevronDown size={14} className={isDark ? 'text-[#999]' : 'text-[#666]'} />}
          </button>
          <FileCode size={14} className={isDark ? 'text-[#e0a370]' : 'text-[#b35c2a]'} />
          <span className={`truncate ${isDark ? 'text-[#e0a370]' : 'text-[#b35c2a]'}`}>{displayName}</span>
          {language && (
            <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${isDark ? 'bg-[#404040] text-[#999]' : 'bg-[#e8e8e4] text-[#666]'}`}>
              {language}
            </span>
          )}
          <span className={isDark ? 'text-[#555]' : 'text-[#aaa]'}>
            {lines.length} {lines.length === 1 ? 'line' : 'lines'}
          </span>
        </div>
        <button
          onClick={(e) => { e.stopPropagation(); handleCopy(); }}
          className={`p-1 rounded transition-colors flex-shrink-0 ml-4 ${isDark ? 'hover:bg-[#404040] text-[#999]' : 'hover:bg-[#e8e8e4] text-[#666]'}`}
          title="Copy code"
        >
          {copied ? <Check size={12} /> : <Copy size={12} />}
        </button>
      </div>

      {/* Code body */}
      {!collapsed && (
        <div className="overflow-x-auto max-h-[400px] overflow-y-auto">
          <table className="w-full border-collapse">
            <tbody>
              {lines.map((line, i) => (
                <tr key={i}>
                  {lineNumbers && (
                    <td className={`select-none text-right px-2 w-[1%] whitespace-nowrap ${isDark ? 'text-[#555] border-r border-[#333]' : 'text-[#bbb] border-r border-[#eee]'}`}>
                      {i + 1}
                    </td>
                  )}
                  <td className={`px-3 py-0 whitespace-pre-wrap break-all ${isDark ? 'text-[#ccc]' : 'text-[#333]'}`}>
                    {line || '\u00A0'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default CodeViewer;