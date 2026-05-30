import React, { useState, useRef, useCallback, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import { ChevronDown, ChevronUp, Copy, Check, Loader2 } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneLight, vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';
import 'katex/dist/katex.min.css';

export interface CitationSource {
  url: string;
  title: string;
  cited_text?: string;
}

interface MarkdownRendererProps {
  content: string;
  citations?: CitationSource[];
  showSourcesList?: boolean;
  isStreaming?: boolean;
}

/**
 * 对 citations 按 url 去重，返回去重后的来源列表（保持首次出现顺序）
 */
function deduplicateSources(citations: CitationSource[]): CitationSource[] {
  const seen = new Map<string, CitationSource>();
  for (const c of citations) {
    if (!seen.has(c.url)) {
      seen.set(c.url, c);
    }
  }
  return Array.from(seen.values());
}

/**
 * 获取 url 对应的引用编号（1-based）
 */
function getSourceIndex(url: string, sources: CitationSource[]): number {
  const idx = sources.findIndex((s) => s.url === url);
  return idx >= 0 ? idx + 1 : 0;
}

/**
 * 移除 <cite index="...">...</cite> 标签，保留内部文本
 */
function stripCiteTags(text: string): string {
  return text.replace(/<cite\s+index="[^"]*"\s*>([\s\S]*?)<\/cite>/g, '$1');
}

/**
 * 规范化 $$...$$ 数学块：
 * - LLM 经常输出 `$$...`(同一行紧跟内容) 且中间包含换行，这会导致 remark-math 对齐失败/截断。
 * - 这里将“包含换行的 $$...$$”统一改写成标准块格式：
 *   \n\n$$\n...\n$$\n\n
 *
 * 注意：跳过 ```fenced code```，避免改写代码块里的 $$
 */
function normalizeMathBlocks(text: string): string {
  // Split on fenced code blocks and only normalize non-code segments.
  const parts = text.split(/(```[\s\S]*?```)/g);
  return parts
    .map((part) => {
      if (part.startsWith('```')) return part;
      return part.replace(/\$\$([\s\S]+?)\$\$/g, (_m, inner: string) => {
        if (!inner.includes('\n')) {
          // Keep inline-style $$...$$ untouched to avoid changing layout unexpectedly.
          return `$$${inner}$$`;
        }
        const body = inner.trim();
        return `\n\n$$\n${body}\n$$\n\n`;
      });
    })
    .join('');
}

/** 引用角标组件 */
const CitationBadge: React.FC<{ index: number; source: CitationSource }> = ({ index, source }) => {
  const [showTooltip, setShowTooltip] = useState(false);
  const badgeRef = useRef<HTMLSpanElement>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const handleMouseEnter = () => {
    clearTimeout(timeoutRef.current);
    setShowTooltip(true);
  };

  const handleMouseLeave = () => {
    timeoutRef.current = setTimeout(() => setShowTooltip(false), 200);
  };

  return (
    <span className="relative inline-block" ref={badgeRef}>
      <a
        href={source.url}
        target="_blank"
        rel="noopener noreferrer"
        className="inline-flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[11px] font-medium text-[#2563EB] bg-[#EFF6FF] hover:bg-[#DBEAFE] rounded cursor-pointer no-underline align-super leading-none ml-0.5 transition-colors"
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        {index}
      </a>
      {showTooltip && (
        <div
          className="absolute z-50 bottom-full left-1/2 -translate-x-1/2 mb-2 w-[360px] max-w-[90vw] bg-white border border-[#E5E5E5] rounded-lg shadow-lg p-3 text-left"
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
        >
          <div className="text-[13px] font-medium text-[#111] mb-1 line-clamp-2">{source.title}</div>
          <a
            href={source.url}
            target="_blank"
            rel="noopener noreferrer"
            className="text-[12px] text-[#2563EB] hover:underline break-all line-clamp-1 mb-2 block"
          >
            {source.url}
          </a>
          {source.cited_text && (
            <div className="text-[12px] text-[#6B7280] leading-relaxed border-l-2 border-[#E5E5E5] pl-2 line-clamp-3">
              {source.cited_text}
            </div>
          )}
          <div className="absolute left-1/2 -translate-x-1/2 top-full w-2 h-2 bg-white border-r border-b border-[#E5E5E5] transform rotate-45 -mt-1"></div>
        </div>
      )}
    </span>
  );
};

/** 来源列表折叠组件 */
const SourcesList: React.FC<{ sources: CitationSource[] }> = ({ sources }) => {
  const [expanded, setExpanded] = useState(false);

  if (sources.length === 0) return null;

  return (
    <div className="mt-3 border border-[#E5E5E5] dark:border-[#444341] rounded-lg overflow-hidden code-block-wrapper">
      <div
        className="flex items-center gap-2 px-3 py-2 bg-[#F9F9F7] cursor-pointer hover:bg-[#F2F0EB] transition-colors select-none"
        onClick={() => setExpanded(!expanded)}
      >
        <ChevronDown
          size={14}
          className={`text-[#9CA3AF] transform transition-transform ${expanded ? 'rotate-0' : '-rotate-90'}`}
        />
        <span className="text-[13px] font-medium text-[#6B7280]">
          来源 ({sources.length})
        </span>
      </div>
      {expanded && (
        <div className="border-t border-[#E5E5E5] bg-[#FAFAF8]">
          {sources.map((source, i) => (
            <div key={source.url} className="flex items-start gap-2 px-3 py-2 border-b border-[#F0F0EE] last:border-b-0">
              <span className="inline-flex items-center justify-center min-w-[20px] h-[20px] px-1 text-[11px] font-medium text-[#2563EB] bg-[#EFF6FF] rounded flex-shrink-0 mt-0.5">
                {i + 1}
              </span>
              <div className="min-w-0 flex-1">
                <a
                  href={source.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[13px] font-medium text-[#111] hover:text-[#2563EB] hover:underline line-clamp-1 block"
                >
                  {source.title || source.url}
                </a>
                <span className="text-[11px] text-[#9CA3AF] break-all line-clamp-1 block">
                  {new URL(source.url).hostname}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

const CODE_FOLD_THRESHOLD = 20;
const CODE_FOLD_VISIBLE = 15;

/** 代码块组件 — 折叠 + 常驻工具栏 + 流式指示 */
export const CodeBlock: React.FC<{ language: string; code: string; className?: string; isStreaming?: boolean }> = ({ language, code, className, isStreaming }) => {
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [isDark, setIsDark] = useState(() => {
    if (typeof document === 'undefined') return false;
    return document.documentElement.classList.contains('dark');
  });

  useEffect(() => {
    const checkDark = () => setIsDark(document.documentElement.classList.contains('dark'));
    checkDark();
    const observer = new MutationObserver(checkDark);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => observer.disconnect();
  }, []);

  // Reset expand state when code changes (new streaming content arrives)
  useEffect(() => {
    setExpanded(false);
  }, [code]);

  const handleCopy = useCallback(() => {
    import('../utils/clipboard').then(({ copyToClipboard }) => {
      copyToClipboard(code).then((success) => {
        if (success) {
          setCopied(true);
          setTimeout(() => setCopied(false), 2000);
        }
      });
    });
  }, [code]);

  const lines = code.split('\n');
  const lineCount = lines.length;
  const canFold = lineCount > CODE_FOLD_THRESHOLD;
  const isFolded = canFold && !expanded;
  const displayCode = isFolded ? lines.slice(0, CODE_FOLD_VISIBLE).join('\n') : code;

  const toolbarBg = isDark ? 'bg-[#3A3A38]' : 'bg-[#F3F3F0]';
  const toolbarBorder = isDark ? 'border-[#383836]' : 'border-[#E5E5E5]';

  return (
    <div className={`relative rounded-lg overflow-hidden my-3 text-sm border code-block-wrapper ${isDark ? 'border-[#383836] bg-[#30302E]' : 'border-[#E5E5E5] bg-[#FCFCFA]'} ${className || ''}`}>
      {/* Persistent toolbar */}
      <div className={`flex items-center justify-between px-3 py-1.5 border-b ${toolbarBorder} ${toolbarBg} select-none`}>
        <div className="flex items-center gap-2 min-w-0">
          <span className={`text-[11.5px] font-mono font-medium truncate ${isDark ? 'text-[#B0B0B0]' : 'text-[#555]'}`}>
            {language || 'text'}
          </span>
          <span className={`text-[11px] tabular-nums flex-shrink-0 ${isDark ? 'text-[#888]' : 'text-[#999]'}`}>
            {lineCount} {lineCount === 1 ? 'line' : 'lines'}
          </span>
          {isStreaming && (
            <span className="flex items-center gap-1.5 text-[11px] text-[#D97706] flex-shrink-0">
              <span className="loading-ring !w-3 !h-3 !border-[1.5px]" />
              generating
            </span>
          )}
        </div>
        <div className="flex items-center gap-0.5 flex-shrink-0">
          {canFold && (
            <button
              onClick={() => setExpanded(!expanded)}
              className={`flex items-center gap-0.5 px-1.5 py-1 rounded text-[11px] transition-colors ${isDark ? 'text-[#B0B0B0] hover:text-white hover:bg-[#505050]' : 'text-[#777] hover:text-[#333] hover:bg-[#E8E8E4]'}`}
            >
              {isFolded ? (
                <><ChevronDown size={12} /> Show all</>
              ) : (
                <><ChevronUp size={12} /> Collapse</>
              )}
            </button>
          )}
          <button
            onClick={handleCopy}
            className={`p-1 rounded transition-colors ${isDark ? 'text-[#B0B0B0] hover:text-white hover:bg-[#505050]' : 'text-[#777] hover:text-[#333] hover:bg-[#E8E8E4]'}`}
            title={copied ? '已复制' : '复制代码'}
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
          </button>
        </div>
      </div>

      {/* Code area */}
      <div className="relative">
        {isStreaming && <div className="code-scan-line" />}
        <div className={isFolded ? `overflow-hidden` : ''} style={isFolded ? { maxHeight: `${CODE_FOLD_VISIBLE * 1.6 * 15}px` } : undefined}>
          <SyntaxHighlighter
            language={language || 'text'}
            style={isDark ? vscDarkPlus : oneLight}
            customStyle={{
              margin: 0,
              padding: '12px',
              background: 'transparent',
              fontSize: '15px',
              border: 'none',
              boxShadow: 'none',
            }}
            codeTagProps={{
              style: { fontFamily: "Menlo, Monaco, SF Mono, Cascadia Code, Fira Code, Consolas, Courier New, monospace" }
            }}
          >
            {displayCode}
          </SyntaxHighlighter>
        </div>
        {/* Fade-out overlay when folded */}
        {isFolded && (
          <>
            <div className={`absolute bottom-0 left-0 right-0 h-16 pointer-events-none ${isDark ? 'bg-gradient-to-t from-[#30302E] to-transparent' : 'bg-gradient-to-t from-[#FCFCFA] to-transparent'}`} />
            <button
              onClick={() => setExpanded(true)}
              className={`absolute bottom-2 left-1/2 -translate-x-1/2 px-3 py-1 rounded-full text-[11.5px] font-medium shadow transition-colors ${isDark ? 'bg-[#505050] text-[#DDD] hover:bg-[#606060]' : 'bg-white text-[#555] hover:bg-[#F5F5F5] border border-[#E5E5E5]'}`}
            >
              Show all {lineCount} lines
            </button>
          </>
        )}
      </div>
    </div>
  );
};

const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({ content, citations, showSourcesList = false, isStreaming = false }) => {
  const processed = normalizeMathBlocks(stripCiteTags(content));
  const sources = citations ? deduplicateSources(citations) : [];
  const hasCitations = sources.length > 0;

  // Detect unclosed code fence during streaming (odd number of ``` markers)
  const fenceMatches = content.match(/(?:^|\n)```/gm);
  const hasUnclosedFence = isStreaming && fenceMatches && fenceMatches.length % 2 !== 0;
  let inProgressCode: string | null = null;
  let inProgressLang = '';
  if (hasUnclosedFence && fenceMatches) {
    const parts = content.split(/(?:^|\n)```/gm);
    inProgressCode = parts[parts.length - 1] || '';
    const langMatch = inProgressCode.match(/^(\w+)\s*\n/);
    if (langMatch) {
      inProgressLang = langMatch[1];
      inProgressCode = inProgressCode.slice(langMatch[0].length);
    }
  }

  return (
    <div
      className="markdown-body assistant-markdown text-[16.5px] leading-normal overflow-x-hidden"
      style={{ color: 'var(--text-claude-model-body)' }}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[[rehypeKatex, { throwOnError: false, strict: 'ignore' }]]}
        components={{
          a({ children, href, ...props }: any) {
            return (
              <a
                href={href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-500 dark:text-blue-400 hover:underline"
                {...props}
              >
                {children}
              </a>
            );
          },
          h1({ children, ...props }: any) {
            return (
              <h1
                className="mt-7 mb-3 text-[25px] leading-[1.2] font-bold tracking-[-0.02em]"
                style={{ color: 'var(--text-claude-model-body)' }}
                {...props}
              >
                {children}
              </h1>
            );
          },
          h2({ children, ...props }: any) {
            return (
              <h2
                className="mt-6 mb-3 text-[21px] leading-[1.25] font-bold tracking-[-0.015em]"
                style={{ color: 'var(--text-claude-model-body)' }}
                {...props}
              >
                {children}
              </h2>
            );
          },
          h3({ children, ...props }: any) {
            return (
              <h3
                className="mt-5 mb-2.5 text-[18px] leading-[1.3] font-semibold"
                style={{ color: 'var(--text-claude-model-body)' }}
                {...props}
              >
                {children}
              </h3>
            );
          },
          h4({ children, ...props }: any) {
            return (
              <h4
                className="mt-4 mb-2 text-[16.8px] leading-[1.35] font-semibold"
                style={{ color: 'var(--text-claude-model-body)' }}
                {...props}
              >
                {children}
              </h4>
            );
          },
          h5({ children, ...props }: any) {
            return (
              <h5
                className="mt-3.5 mb-2 text-[15.8px] leading-[1.4] font-semibold"
                style={{ color: 'var(--text-claude-model-body)' }}
                {...props}
              >
                {children}
              </h5>
            );
          },
          h6({ children, ...props }: any) {
            return (
              <h6
                className="mt-3 mb-2 text-[15px] leading-[1.4] font-semibold uppercase tracking-[0.02em] opacity-90"
                style={{ color: 'var(--text-claude-model-body)' }}
                {...props}
              >
                {children}
              </h6>
            );
          },
          p({ children, ...props }: any) {
            return (
              <p
                className="mb-2.5 text-[16.5px] leading-[1.7]"
                style={{ color: 'var(--text-claude-model-body)' }}
                {...props}
              >
                {children}
              </p>
            );
          },
          pre({ children, ...props }: any) {
            return <>{children}</>;
          },
          hr({ children, ...props }: any) {
            return <hr className="my-6 border-t border-claude-border dark:border-[rgb(66,65,62)]" {...props} />;
          },
          table({ children, ...props }: any) {
            return (
              <div className="overflow-x-auto my-4">
                <table className="w-full text-[14.5px]" {...props}>{children}</table>
              </div>
            );
          },
          thead({ children, ...props }: any) {
            return <thead className="border-b border-black dark:border-white" {...props}>{children}</thead>;
          },
          tbody({ children, ...props }: any) {
            return <tbody {...props}>{children}</tbody>;
          },
          tr({ children, ...props }: any) {
            return <tr className="border-b border-black dark:border-white last:border-b-0" {...props}>{children}</tr>;
          },
          th({ children, ...props }: any) {
            return <th className="text-left py-2 pr-4 font-semibold" style={{ color: 'var(--text-claude-model-body)' }} {...props}>{children}</th>;
          },
          td({ children, ...props }: any) {
            return <td className="py-2 pr-4" style={{ color: 'var(--text-claude-model-body)' }} {...props}>{children}</td>;
          },
          code({ node, className, children, ...props }: any) {
            const isBlock = className?.startsWith('language-') || (node?.position?.start?.line !== node?.position?.end?.line);
            const language = className?.replace('language-', '') || '';
            if (isBlock) {
              const codeText = String(children).replace(/\n$/, '');
              return <CodeBlock language={language} code={codeText} className={className} isStreaming={isStreaming} {...props} />;
            }
            return (
              <code className="inline-code px-1.5 py-0 rounded-md text-[14.5px] font-mono border border-transparent leading-none" {...props}>
                {children}
              </code>
            );
          }
        }}
      >
        {processed}
      </ReactMarkdown>

      {/* Render unclosed code fence as in-progress block during streaming */}
      {hasUnclosedFence && inProgressCode && (
        <CodeBlock
          language={inProgressLang}
          code={inProgressCode}
          isStreaming={true}
          className="border-[#D97706] border-opacity-40"
        />
      )}

      {hasCitations && showSourcesList && <SourcesList sources={sources} />}
    </div>
  );
};

export default React.memo(MarkdownRenderer);
