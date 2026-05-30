import React, { useState, useRef, useEffect } from 'react';
import { FileText, Download, X, Code, Eye, RefreshCw, Copy, ChevronDown } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneLight, vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import MarkdownRenderer from './MarkdownRenderer';
import SlidePreview from './SlidePreview';
import DocxPreview from './DocxPreview';
import PdfPreview from './PdfPreview';
import { DocumentInfo } from './DocumentCard';
import { copyToClipboard } from '../utils/clipboard';
import { buildArtifactHtml } from '../utils/artifactRenderer';

/** Renders HTML/React artifact content in a sandboxed iframe */
const ArtifactPreview: React.FC<{ content: string; type: string }> = ({ content, type }) => {
  const [blobUrl, setBlobUrl] = React.useState<string | null>(null);

  React.useEffect(() => {
    const html = buildArtifactHtml(content, type);
    const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    setBlobUrl(url);
    return () => URL.revokeObjectURL(url);
  }, [content, type]);

  if (!blobUrl) return null;
  return (
    <iframe
      src={blobUrl}
      className="w-full h-full border-0 bg-white"
      title="Artifact Preview"
    />
  );
};

interface DocumentPanelProps {
  document: DocumentInfo;
  onClose: () => void;
}

const DocumentPanel: React.FC<DocumentPanelProps> = ({ document: doc, onClose }) => {
  const fmt = (doc.format || 'markdown').toLowerCase();
  // Detect actual format from file extension if fmt is generic (e.g. "text")
  const extMatch = (doc.title || '').match(/\.(\w+)$/);
  const ext = extMatch ? extMatch[1].toLowerCase() : '';

  const isBinary = ['pptx', 'docx', 'xlsx', 'pdf'].includes(fmt);
  const isMarkdown = ['markdown', 'md'].includes(fmt) || ext === 'md';
  const contentStart = (doc.content || '').trimStart().slice(0, 50).toLowerCase();
  const isHtml = ['html', 'htm'].includes(fmt) || ['html', 'htm'].includes(ext) || contentStart.startsWith('<!doctype html') || contentStart.startsWith('<html');
  const isReact = ['jsx', 'tsx'].includes(fmt) || ['jsx', 'tsx'].includes(ext);
  const isRenderable = isHtml || isReact;
  const isCode = !isBinary && !isMarkdown && !isRenderable;

  const [viewMode, setViewMode] = useState<'preview' | 'code'>(isCode ? 'code' : 'preview');
  const [showCopyMenu, setShowCopyMenu] = useState(false);
  const copyMenuRef = useRef<HTMLDivElement>(null);
  const copyBtnRef = useRef<HTMLButtonElement>(null);
  const [copied, setCopied] = useState(false);
  const [isDark, setIsDark] = useState(() => {
    if (typeof window === 'undefined') return false;
    return window.document.documentElement.classList.contains('dark');
  });

  useEffect(() => {
    const checkDark = () => setIsDark(window.document.documentElement.classList.contains('dark'));
    checkDark();

    const observer = new MutationObserver(checkDark);
    observer.observe(window.document.documentElement, { attributes: true, attributeFilter: ['class'] });

    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (copyMenuRef.current && !copyMenuRef.current.contains(event.target as Node) &&
        copyBtnRef.current && !copyBtnRef.current.contains(event.target as Node)) {
        setShowCopyMenu(false);
      }
    };
    if (showCopyMenu) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showCopyMenu]);

  // Synchronize scroll between line numbers and content
  const lineNumbersRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);

  const handleScroll = () => {
      if (lineNumbersRef.current && contentRef.current) {
          lineNumbersRef.current.scrollTop = contentRef.current.scrollTop;
      }
  };

  const BINARY_FORMATS = ['pptx', 'docx', 'xlsx', 'pdf'];
  const LANG_TO_EXT: Record<string, string> = {
    markdown: 'md', python: 'py', javascript: 'js', typescript: 'ts',
    java: 'java', c: 'c', cpp: 'cpp', csharp: 'cs',
    go: 'go', rust: 'rs', ruby: 'rb', php: 'php',
    swift: 'swift', kotlin: 'kt', scala: 'scala',
    html: 'html', css: 'css', scss: 'scss',
    sql: 'sql', shell: 'sh', bash: 'sh', powershell: 'ps1',
    yaml: 'yml', json: 'json', xml: 'xml', toml: 'toml',
    ini: 'ini', dockerfile: 'Dockerfile',
    r: 'r', matlab: 'm', lua: 'lua', perl: 'pl',
    dart: 'dart', vue: 'vue', svelte: 'svelte',
  };

  const handleDownload = async (format?: string) => {
    setShowCopyMenu(false);

    if (format === 'pdf' && fmt === 'markdown') {
        const blob = new Blob([doc.content || ''], { type: 'text/markdown;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = window.document.createElement('a');
        a.href = url;
        a.download = `${doc.title}.md`;
        a.click();
        URL.revokeObjectURL(url);
        return;
    }

    if (!BINARY_FORMATS.includes(fmt)) {
      // Text-based: markdown or code files
      const ext = LANG_TO_EXT[fmt] || fmt;
      const blob = new Blob([doc.content || ''], { type: 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const a = window.document.createElement('a');
      a.href = url;
      a.download = doc.title.includes('.') ? doc.title : `${doc.title}.${ext}`;
      a.click();
      URL.revokeObjectURL(url);
    } else {
      const extMap: Record<string, string> = { docx: '.docx', pptx: '.pptx', xlsx: '.xlsx', pdf: '.pdf' };
      const ext = extMap[fmt] || '.bin';
      try {
        const token = localStorage.getItem('auth_token');
        const res = await fetch(`/api/documents/${doc.id}/raw`, {
          headers: token ? { Authorization: `Bearer ${token}` } : {},
        });
        if (!res.ok) return;
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        const a = window.document.createElement('a');
        a.href = url;
        a.download = `${doc.title}${ext}`;
        a.click();
        URL.revokeObjectURL(url);
      } catch {
        // silent fail
      }
    }
  };

  const handleCopyContent = () => {
    if (doc.content) {
        copyToClipboard(doc.content).then(success => {
            if (success) {
                setCopied(true);
                setTimeout(() => setCopied(false), 2000);
            }
        });
    }
  };

  return (
    <div className="flex-1 h-full flex flex-col bg-claude-input border-l border-claude-border min-w-0">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-claude-border flex-shrink-0">
        <div className="flex items-center min-w-0 gap-3 flex-1">
          {(isMarkdown || isRenderable) && (
            <div className="flex bg-claude-btnHover rounded-lg p-0.5 flex-shrink-0">
                <button
                    onClick={() => setViewMode('preview')}
                    className={`p-1.5 rounded-md transition-all ${viewMode === 'preview' ? 'bg-white dark:bg-[#555] shadow-sm text-claude-text' : 'text-claude-textSecondary hover:text-claude-text'}`}
                    title="Preview"
                >
                    <Eye size={16} />
                </button>
                <button
                    onClick={() => setViewMode('code')}
                    className={`p-1.5 rounded-md transition-all ${viewMode === 'code' ? 'bg-white dark:bg-[#555] shadow-sm text-claude-text' : 'text-claude-textSecondary hover:text-claude-text'}`}
                    title="Code"
                >
                    <Code size={16} />
                </button>
            </div>
          )}
          <div className="flex items-center min-w-0 text-[14px] truncate">
            <span className="text-claude-text font-normal truncate">{doc.title}</span>
            {!doc.title.toLowerCase().endsWith(`.${fmt}`) && (
              <span className="text-claude-textSecondary font-normal ml-1 flex-shrink-0">· {fmt.toUpperCase()}</span>
            )}
          </div>
        </div>
        
        <div className="flex items-center gap-2 flex-shrink-0 ml-4">
          <div className="relative flex items-center">
            <button
                onClick={handleCopyContent}
                className="h-7 flex items-center px-3 text-[13px] font-medium text-claude-text border border-claude-border border-r-0 rounded-l-lg hover:bg-claude-btnHover transition-colors"
            >
                {copied ? 'Copied' : 'Copy'}
            </button>
            <button
                ref={copyBtnRef}
                onClick={() => setShowCopyMenu(!showCopyMenu)}
                className="h-7 px-2 flex items-center justify-center border border-claude-border rounded-r-lg hover:bg-claude-btnHover transition-colors text-claude-text"
            >
                <ChevronDown size={14} />
            </button>

            {showCopyMenu && (
                <div 
                    ref={copyMenuRef}
                    className="absolute top-full right-0 mt-1 w-48 bg-white dark:bg-claude-input border border-claude-border rounded-lg shadow-lg py-1 z-50"
                >
                    <button 
                        onClick={() => handleDownload()}
                        className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-btnHover transition-colors"
                    >
                        Download
                    </button>
                    {fmt === 'markdown' && (
                        <button 
                            onClick={() => handleDownload('pdf')}
                            className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-btnHover transition-colors"
                        >
                            Download as PDF
                        </button>
                    )}
                    <button 
                        className="w-full text-left px-4 py-2 text-[13px] text-claude-text hover:bg-claude-btnHover transition-colors"
                        onClick={() => setShowCopyMenu(false)}
                    >
                        Publish artifact
                    </button>
                </div>
            )}
          </div>

          <button
            onClick={() => {}} // Retry/Refresh logic placeholder
            className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-btnHover rounded-lg transition-colors"
            title="Reload"
          >
            <RefreshCw size={16} />
          </button>
          
          <button
            onClick={onClose}
            className="p-1.5 text-claude-textSecondary hover:text-claude-text hover:bg-claude-btnHover rounded-lg transition-colors"
            title="Close"
          >
            <X size={20} />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className={`flex-1 overflow-y-auto chat-font-scope ${fmt === 'docx' || fmt === 'pdf' ? 'px-6 py-6 bg-claude-hover' : 'px-8 py-6'} ${viewMode === 'code' || isCode ? '!p-0 overflow-hidden bg-[#FAFAFA] dark:bg-[#1E1E1E]' : ''} ${viewMode === 'preview' && isRenderable ? '!p-0 !overflow-hidden' : ''}`}>
        {/* Live HTML/React preview */}
        {viewMode === 'preview' && isRenderable && doc.content ? (
          <ArtifactPreview content={doc.content} type={isReact ? 'application/vnd.ant.react' : 'text/html'} />
        ) : viewMode === 'code' || isCode ? (
             <div className="flex h-full font-mono text-[13px] leading-relaxed relative bg-[#FAFAFA] dark:bg-[#1E1E1E] overflow-hidden">
                 {/* Unified scrollable area with line numbers + code side by side */}
                <div ref={contentRef} onScroll={handleScroll} className="flex-1 overflow-auto bg-[#FAFAFA] dark:bg-[#1E1E1E]">
                    <div className="flex min-h-full">
                        {/* Line Numbers — inside the scroll container so they scroll together */}
                        <div className="flex-none w-[40px] bg-[#FAFAFA] dark:bg-[#1E1E1E] text-right pt-4 pr-2 select-none text-claude-textSecondary opacity-50 sticky left-0">
                            {doc.content?.split('\n').map((_: string, i: number) => (
                                <div key={i} style={{ lineHeight: '1.625' }}>{i + 1}</div>
                            ))}
                        </div>
                        {/* Code Content */}
                        <div className="flex-1 min-w-0">
                            <SyntaxHighlighter
                                language={isCode ? fmt : 'markdown'}
                                style={isDark ? vscDarkPlus : oneLight}
                                customStyle={{
                                    margin: 0,
                                    padding: '16px 16px 16px 8px',
                                    background: 'transparent',
                                    fontSize: '14px',
                                    fontFamily: 'Menlo, Monaco, SF Mono, Cascadia Code, Fira Code, Consolas, Courier New, monospace',
                                    lineHeight: '1.625',
                                    border: 'none',
                                    boxShadow: 'none',
                                    minHeight: '100%',
                                }}
                                codeTagProps={{
                                    style: { fontFamily: "inherit" }
                                }}
                            >
                                {doc.content || ''}
                            </SyntaxHighlighter>
                        </div>
                    </div>
                </div>
            </div>
        ) : (
            <>
                {fmt === 'pptx' && doc.slides ? (
                  <SlidePreview slides={doc.slides} title={doc.title} colorScheme={doc.colorScheme} />
                ) : fmt === 'docx' && doc.content ? (
                  <DocxPreview content={doc.content} title={doc.title} />
                ) : fmt === 'pdf' && doc.sections ? (
                  <PdfPreview sections={doc.sections} title={doc.title} />
                ) : fmt === 'xlsx' && doc.sheets ? (
                  <div className="space-y-6">
                    {doc.sheets.map((sheet, si) => (
                      <div key={si}>
                        <div className="text-[14px] font-medium text-claude-text mb-2">{sheet.name}</div>
                        <div className="overflow-x-auto border border-claude-border rounded-lg">
                          <table className="w-full text-[13px]">
                            <thead>
                              <tr className="bg-[#4472C4] text-white">
                                {sheet.headers.map((h, hi) => (
                                  <th key={hi} className="px-3 py-2 text-left font-medium whitespace-nowrap">{h}</th>
                                ))}
                              </tr>
                            </thead>
                            <tbody>
                              {sheet.rows.map((row, ri) => (
                                <tr key={ri} className={ri % 2 === 0 ? 'bg-claude-bg' : 'bg-transparent'}>
                                  {row.map((cell, ci) => (
                                    <td key={ci} className="px-3 py-1.5 border-t border-claude-border whitespace-nowrap text-claude-text">{cell ?? ''}</td>
                                  ))}
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <MarkdownRenderer content={doc.content || ''} />
                )}
            </>
        )}
      </div>
    </div>
  );
};

export default DocumentPanel;
