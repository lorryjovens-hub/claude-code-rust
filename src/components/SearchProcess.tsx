import React, { useState, useEffect, useRef } from 'react';
import { ChevronDown, Globe, Check } from 'lucide-react';
import { CitationSource } from './MarkdownRenderer';

interface SearchLog {
  query: string;
  results: CitationSource[];
  tokens?: number;
}

interface SearchProcessProps {
  logs: SearchLog[];
  isThinking?: boolean;
  isDone?: boolean;
}

const Favicon: React.FC<{ url: string }> = ({ url }) => {
  const [error, setError] = useState(false);
  let hostname = '';
  try {
    hostname = new URL(url).hostname;
  } catch (e) {
    hostname = 'unknown';
  }
  const faviconUrl = `https://www.google.com/s2/favicons?domain=${hostname}&sz=64`;

  if (error || !hostname) {
    return (
      <div className="w-4 h-4 rounded-sm bg-[#E5E7EB] flex items-center justify-center flex-shrink-0 text-[9px] text-[#6B7280] font-bold uppercase select-none">
        {hostname.slice(0, 1)}
      </div>
    );
  }

  return (
    <img 
      src={faviconUrl} 
      alt="" 
      className="w-4 h-4 rounded-sm flex-shrink-0 select-none"
      onError={() => setError(true)}
    />
  );
};

const SearchLogItem: React.FC<{ log: SearchLog; defaultOpen?: boolean }> = ({ log, defaultOpen }) => {
  const [isOpen, setIsOpen] = useState(!!defaultOpen);

  useEffect(() => {
    setIsOpen(!!defaultOpen);
  }, [defaultOpen]);

  return (
    <div className="relative pl-8 pb-2">
      {/* Node Icon */}
      <div className="absolute left-0 top-0.5 z-10 bg-claude-bg text-claude-textSecondary">
        <Globe size={16} />
      </div>
      
      <div 
        className="flex items-center justify-between mb-1 cursor-pointer group select-none py-0.5"
        onClick={() => setIsOpen(!isOpen)}
      >
        <div className="flex items-center gap-2 text-claude-textSecondary group-hover:text-claude-text transition-colors">
          <span className="text-[13px] font-medium text-claude-text">{log.query}</span>
        </div>
        <div className="flex items-center gap-2 text-claude-textSecondary">
            <span className="text-[11px]">{log.results.length} results</span>
            <ChevronDown 
              size={14} 
              className={`transform transition-transform duration-200 ${isOpen ? 'rotate-180' : 'rotate-0'}`}
            />
        </div>
      </div>

      {isOpen && (
        <div className="bg-claude-bgSecondary border border-claude-border rounded-xl overflow-hidden shadow-sm mt-1">
          <div className="max-h-[180px] overflow-y-auto overflow-x-hidden custom-scrollbar">
            {log.results.map((result, rIndex) => (
              <a
                key={rIndex}
                href={result.url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2.5 px-3 py-1.5 hover:bg-claude-hover transition-colors no-underline group border-b border-black/5 dark:border-white/5 last:border-b-0"
              >
                <Favicon url={result.url} />
                <div className="flex-1 min-w-0 flex items-center gap-2">
                  <span className="text-[12px] text-claude-text font-medium truncate max-w-[70%] transition-colors">
                    {result.title || result.url}
                  </span>
                  <span className="text-[11px] text-claude-textSecondary truncate flex-shrink-0">
                    {new URL(result.url).hostname}
                  </span>
                </div>
              </a>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

const SearchProcess: React.FC<SearchProcessProps> = ({ logs, isThinking, isDone }) => {
  const [isExpanded, setIsExpanded] = useState(!isDone);
  const containerRef = useRef<HTMLDivElement>(null);

  // 计算总 token 数
  const totalTokens = logs.reduce((sum, log) => sum + (log.tokens || 0), 0);

  useEffect(() => {
    if (isDone) {
      setIsExpanded(false);
    } else {
      setIsExpanded(true);
    }
  }, [isDone]);

  useEffect(() => {
    // Rely on parent MainContent to handle scrolling to bottom when content changes
  }, [logs, isExpanded]);

  if (!logs || logs.length === 0) return null;

  return (
    <div className="mb-2 font-sans">
      <div 
        className="flex items-center gap-2 text-claude-textSecondary text-[14px] cursor-pointer hover:text-claude-text transition-colors select-none mb-1"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <span>Searched the web</span>
        <ChevronDown 
          size={14} 
          className={`transform transition-transform duration-200 ${isExpanded ? 'rotate-180' : ''}`}
        />
      </div>

      <div 
        className={`transition-all duration-500 ease-in-out overflow-hidden ${isExpanded ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'}`}
        ref={containerRef}
      >
        <div className="relative pb-2">
          <div className="absolute left-[7.5px] top-2 bottom-2 w-px bg-claude-border" />
          
          {logs.map((log, index) => (
            <SearchLogItem key={index} log={log} defaultOpen={!isDone} />
          ))}

          {!isThinking && (
            <div className="relative pl-8 pt-1 pb-1">
              <div className="absolute left-0 top-1 z-10 bg-claude-bg text-claude-textSecondary">
                <Check size={16} />
              </div>
              <div className="flex items-center gap-2 text-claude-textSecondary">
                <span className="text-[13px]">Done</span>
                {totalTokens > 0 && (
                  <span className="text-[11px] text-claude-textSecondary">· {totalTokens.toLocaleString()} tokens</span>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default SearchProcess;
