import React, { useState, useMemo, useEffect, useRef } from 'react';
import { ArrowLeft, Copy, Check, ExternalLink, RefreshCw, Loader2, FileCode, MessageSquare } from 'lucide-react';
import inspirationsData from '../data/inspirations.json';
import { buildArtifactHtml, loadArtifactCode } from '../utils/artifactRenderer';
import { getUserArtifacts, getArtifactContent } from '../api';

interface InspirationItem {
  artifact_id: string;
  chat_id: string;
  category: string;
  name: string;
  description: string;
  starting_prompt: string;
  img_src: string;
  content_uuid?: string;
  code_file?: string;
}

/** Renders artifact HTML in an iframe via Blob URL (avoids srcdoc CSP issues in Electron) */
const ArtifactIframe = React.forwardRef<HTMLIFrameElement, { html: string; title: string }>(
  ({ html, title }, ref) => {
    const [blobUrl, setBlobUrl] = useState<string | null>(null);

    useEffect(() => {
      const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      setBlobUrl(url);
      return () => URL.revokeObjectURL(url);
    }, [html]);

    if (!blobUrl) return null;
    return (
      <iframe
        ref={ref}
        src={blobUrl}
        className="w-full border-0 bg-white"
        style={{ minHeight: '400px', height: '500px' }}
        title={title}
      />
    );
  }
);

const CATEGORY_LABELS: Record<string, string> = {
  all: 'All',
  learn: 'Learn something',
  'life-hacks': 'Life hacks',
  games: 'Play a game',
  creative: 'Be creative',
  'touch-grass': 'Touch grass',
};

const CATEGORY_ORDER = ['all', 'learn', 'life-hacks', 'games', 'creative', 'touch-grass'];

interface ArtifactsPageProps {
  onTryPrompt?: (prompt: string) => void;
}

const ArtifactsPage: React.FC<ArtifactsPageProps> = ({ onTryPrompt }) => {
  const [activeTab, setActiveTab] = useState<'inspiration' | 'your_artifacts'>('inspiration');
  const [activeFilter, setActiveFilter] = useState('all');
  const [selectedItem, setSelectedItem] = useState<InspirationItem | null>(null);
  const [copied, setCopied] = useState(false);
  const [artifactHtml, setArtifactHtml] = useState<string | null>(null);
  const [artifactLoading, setArtifactLoading] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [userArtifacts, setUserArtifacts] = useState<any[]>([]);
  const [userArtifactsLoading, setUserArtifactsLoading] = useState(false);

  // Load user artifacts when tab switches
  useEffect(() => {
    if (activeTab === 'your_artifacts') {
      setUserArtifactsLoading(true);
      getUserArtifacts().then(data => {
        setUserArtifacts(Array.isArray(data) ? data : []);
      }).catch(() => setUserArtifacts([])).finally(() => setUserArtifactsLoading(false));
    }
  }, [activeTab]);

  const handleOpenUserArtifact = async (artifact: any) => {
    try {
      const data = await getArtifactContent(artifact.file_path);
      if (data?.content && onTryPrompt) {
        // Store artifact for DocumentPanel and navigate
        sessionStorage.setItem('artifact_remix', JSON.stringify({
          name: artifact.title,
          description: '',
          code: { content: data.content, type: 'text/html', title: artifact.title },
          prompt: '',
        }));
        onTryPrompt('__remix__');
      }
    } catch {}
  };

  // Load artifact code when detail view opens
  useEffect(() => {
    if (!selectedItem?.code_file) {
      setArtifactHtml(null);
      return;
    }
    setArtifactLoading(true);
    setArtifactHtml(null);
    loadArtifactCode(selectedItem.code_file).then(data => {
      if (data) {
        setArtifactHtml(buildArtifactHtml(data.content, data.type));
      }
      setArtifactLoading(false);
    });
  }, [selectedItem]);

  const handleRefreshArtifact = () => {
    if (iframeRef.current && artifactHtml) {
      iframeRef.current.srcdoc = artifactHtml;
    }
  };

  const items = inspirationsData.items as InspirationItem[];

  const filteredItems = useMemo(() => {
    if (activeFilter === 'all') return items;
    return items.filter(item => item.category === activeFilter);
  }, [items, activeFilter]);

  const handleCopyPrompt = async (prompt: string) => {
    await navigator.clipboard.writeText(prompt);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleTryIt = (prompt: string) => {
    if (onTryPrompt) {
      onTryPrompt(prompt);
    }
  };

  const handleCustomize = async (item: InspirationItem) => {
    // Load the artifact code
    let artifactCode: any = null;
    if (item.code_file) {
      try {
        const res = await fetch(`./artifacts/code/${item.code_file}`);
        if (res.ok) artifactCode = await res.json();
      } catch {}
    }

    // Store artifact data for MainContent to pick up
    sessionStorage.setItem('artifact_remix', JSON.stringify({
      name: item.name,
      description: item.description,
      code: artifactCode,
      prompt: item.starting_prompt,
    }));

    if (onTryPrompt) {
      onTryPrompt('__remix__');
    }
  };

  // Detail view
  if (selectedItem) {
    return (
      <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
        <div className="max-w-[800px] mx-auto px-4 py-6 md:px-8 md:py-10">
          {/* Back button */}
          <button
            onClick={() => setSelectedItem(null)}
            className="flex items-center gap-1.5 text-claude-textSecondary hover:text-claude-text transition-colors mb-5"
          >
            <ArrowLeft size={16} />
            <span className="text-[14px]">Back</span>
          </button>

          {/* Creator badge */}
          <div className="flex items-center gap-2.5 mb-3">
            <div className="w-7 h-7 rounded-full bg-[#2A2928] dark:bg-[#1a1a19] border border-gray-500/30 dark:border-gray-500/40 flex items-center justify-center flex-shrink-0 p-1">
              <svg viewBox="0 0 1024 1024" className="w-full h-full">
                <path d="M715.922286 204.8h-140.946286l256.877714 649.728H972.8L715.922286 204.8z m-407.844572 0L51.2 854.528h143.945143l52.955428-135.936h268.909715l51.931428 135.936h143.981715L456.009143 204.8H308.077714z m-13.970285 392.813714l87.954285-227.876571 87.954286 227.876571h-175.908571z" fill="#c5c5c5" />
              </svg>
            </div>
            <span className="text-[14px] text-claude-text font-medium">Anthropic</span>
          </div>

          {/* Title + description + Customize button row */}
          <div className="flex items-start justify-between gap-6 mb-6">
            <div className="flex-1 min-w-0">
              <h1 className="text-[20px] font-semibold text-claude-text mb-1.5">{selectedItem.name}</h1>
              <p className="text-[14px] text-claude-textSecondary leading-relaxed">{selectedItem.description}</p>
            </div>
            <button
              onClick={() => handleCustomize(selectedItem)}
              className="flex items-center gap-2 px-4 py-2 bg-white text-[#1a1a1a] border border-white/80 rounded-xl text-[13px] font-medium hover:bg-gray-100 transition-colors flex-shrink-0 mt-1"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="6" cy="6" r="3" /><circle cx="6" cy="18" r="3" /><line x1="20" y1="4" x2="8.12" y2="15.88" /><line x1="14.47" y1="14.48" x2="20" y2="20" /><line x1="8.12" y1="8.12" x2="12" y2="12" /></svg>
              Customize
            </button>
          </div>

          {/* Large preview area */}
          <div className="w-full rounded-xl overflow-hidden border border-white/[0.08] mb-8 flex flex-col">
            {/* Dark top toolbar */}
            <div className="w-full h-8 bg-[#1a1a19] dark:bg-[#111110] flex items-center justify-end px-2.5 gap-1 flex-shrink-0">
              <button
                onClick={handleRefreshArtifact}
                className="w-6 h-6 rounded flex items-center justify-center text-gray-500 hover:text-white hover:bg-white/10 transition-colors"
                title="Refresh"
              >
                <RefreshCw size={13} />
              </button>
              <button
                onClick={() => {
                  if (artifactHtml) {
                    const blob = new Blob([artifactHtml], { type: 'text/html;charset=utf-8' });
                    const url = URL.createObjectURL(blob);
                    window.open(url, '_blank');
                  }
                }}
                className="w-6 h-6 rounded flex items-center justify-center text-gray-500 hover:text-white hover:bg-white/10 transition-colors"
                title="Open in new window"
              >
                <ExternalLink size={13} />
              </button>
            </div>
            {/* Live artifact or fallback image */}
            {artifactLoading ? (
              <div className="w-full flex items-center justify-center bg-white dark:bg-[#1a1a19]" style={{ minHeight: '400px' }}>
                <Loader2 size={24} className="animate-spin text-claude-textSecondary" />
              </div>
            ) : artifactHtml ? (
              <ArtifactIframe ref={iframeRef} html={artifactHtml} title={selectedItem.name} />
            ) : (
              <img
                src={`./artifacts/previews/${selectedItem.img_src}`}
                alt={selectedItem.name}
                className="w-full object-cover"
                style={{ minHeight: '360px' }}
                onError={(e) => {
                  const el = e.target as HTMLImageElement;
                  el.style.display = 'none';
                }}
              />
            )}
          </div>

          {/* Bottom: About + Keep learning */}
          <div className="flex gap-10 flex-wrap">
            {/* About - left */}
            <div className="flex-1 min-w-[300px]">
              <h2 className="text-[16px] font-semibold text-claude-text mb-4">About</h2>
              <div className="bg-[#f5f5f4] dark:bg-[#121212] border border-black/[0.06] dark:border-white/[0.06] rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-[14px] font-semibold text-claude-text">Starting prompt</h3>
                  <button
                    onClick={() => handleCopyPrompt(selectedItem.starting_prompt)}
                    className="p-1.5 rounded-md text-claude-textSecondary hover:bg-claude-hover hover:text-claude-text transition-colors"
                    title={copied ? 'Copied!' : 'Copy prompt'}
                  >
                    {copied ? <Check size={15} /> : <Copy size={15} />}
                  </button>
                </div>
                <p className="text-[14px] text-claude-textSecondary leading-relaxed whitespace-pre-wrap italic">
                  {selectedItem.starting_prompt}
                </p>
              </div>
            </div>

            {/* Keep learning - right */}
            <div className="w-[200px] flex-shrink-0">
              <h2 className="text-[16px] font-semibold text-claude-text mb-4">Keep learning</h2>
              <div className="flex flex-col gap-2">
                <button
                  onClick={() => handleTryIt(selectedItem.starting_prompt)}
                  className="flex items-center justify-between w-full px-4 py-2.5 border border-claude-border rounded-xl text-[13px] font-medium text-claude-text hover:bg-claude-hover transition-colors"
                >
                  <span>View full chat</span>
                  <ExternalLink size={13} className="text-claude-textSecondary" />
                </button>
                <a
                  href="https://support.anthropic.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-between w-full px-4 py-2.5 border border-claude-border rounded-xl text-[13px] font-medium text-claude-text hover:bg-claude-hover transition-colors"
                >
                  <span>Artifacts guide</span>
                  <ExternalLink size={13} className="text-claude-textSecondary" />
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Main grid view
  return (
    <div className="flex-1 h-full bg-claude-bg overflow-y-auto">
      <div className="max-w-[800px] mx-auto px-4 py-8 md:px-8 md:py-12">
        <div className="flex items-center justify-between mb-8">
          <h1
            className="font-[Spectral] text-[32px] text-claude-text"
            style={{ fontWeight: 500, WebkitTextStroke: '0.5px currentColor' }}
          >
            Artifacts
          </h1>
          <button
            onClick={() => handleTryIt('Create a new interactive artifact.')}
            className="flex items-center gap-2 px-3.5 py-1.5 bg-claude-text text-claude-bg hover:opacity-90 rounded-lg transition-opacity font-medium"
            style={{ fontSize: '14px' }}
          >
            New artifact
          </button>
        </div>

        {/* Tabs */}
        <div className="flex items-center gap-6 border-b border-[#262624] mb-6">
          <button
            onClick={() => setActiveTab('inspiration')}
            className={`pb-3 text-[14px] font-medium transition-colors border-b-2 ${activeTab === 'inspiration' ? 'text-claude-text border-claude-text' : 'text-claude-textSecondary border-transparent hover:text-claude-text'}`}
          >
            Inspiration
          </button>
          <button
            onClick={() => setActiveTab('your_artifacts')}
            className={`pb-3 text-[14px] font-medium transition-colors border-b-2 ${activeTab === 'your_artifacts' ? 'text-claude-text border-claude-text' : 'text-claude-textSecondary border-transparent hover:text-claude-text'}`}
          >
            Your artifacts
          </button>
        </div>

        {activeTab === 'inspiration' ? (
          <>
            {/* Filters */}
            <div className="flex flex-wrap items-center gap-2 mb-8">
              {CATEGORY_ORDER.map(id => (
                <button
                  key={id}
                  onClick={() => setActiveFilter(id)}
                  className={`px-4 py-1.5 rounded-full text-[13px] transition-colors ${activeFilter === id
                    ? 'bg-claude-text text-claude-bg font-medium'
                    : 'text-claude-textSecondary hover:bg-black/5 dark:hover:bg-white/5'
                    }`}
                >
                  {CATEGORY_LABELS[id]}
                </button>
              ))}
            </div>

            {/* Grid */}
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-6">
              {filteredItems.map(item => (
                <div
                  key={item.artifact_id}
                  className="group cursor-pointer"
                  onClick={() => setSelectedItem(item)}
                >
                  <div className="w-full aspect-[4/3] rounded-[16px] overflow-hidden mb-3 border border-transparent group-hover:border-[#262624] relative bg-claude-input transition-colors duration-200">
                    <img
                      src={`./artifacts/previews/${item.img_src}`}
                      alt={item.name}
                      className="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-300"
                      onError={(e) => {
                        // Fallback: show a gradient placeholder
                        const el = e.target as HTMLImageElement;
                        el.style.display = 'none';
                        el.parentElement!.classList.add('bg-gradient-to-br', 'from-claude-hover', 'to-claude-input');
                      }}
                    />
                  </div>
                  <h3 className="text-[14px] font-medium text-claude-text group-hover:underline decoration-1 underline-offset-2">
                    {item.name}
                  </h3>
                  <p className="text-[12px] text-claude-textSecondary mt-0.5 line-clamp-2">
                    {item.description}
                  </p>
                </div>
              ))}
            </div>
          </>
        ) : (
          /* Your artifacts */
          userArtifactsLoading ? (
            <div className="flex items-center justify-center py-20">
              <Loader2 size={24} className="animate-spin text-claude-textSecondary" />
            </div>
          ) : userArtifacts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-20 text-claude-textSecondary">
              <div className="w-16 h-16 rounded-2xl bg-claude-hover flex items-center justify-center mb-4">
                <FileCode size={24} className="opacity-40" />
              </div>
              <p className="text-[15px] font-medium text-claude-text mb-1">No artifacts yet</p>
              <p className="text-[13px]">HTML files created by Claude will appear here.</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
              {userArtifacts.map(artifact => (
                <div
                  key={artifact.id}
                  onClick={() => handleOpenUserArtifact(artifact)}
                  className="group cursor-pointer border border-claude-border/30 rounded-xl p-4 hover:bg-claude-hover transition-colors"
                >
                  <div className="flex items-start gap-3">
                    <div className="w-10 h-10 rounded-lg bg-claude-input flex items-center justify-center flex-shrink-0 mt-0.5">
                      <FileCode size={20} className="text-claude-textSecondary" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="text-[14px] font-medium text-claude-text truncate group-hover:underline decoration-1 underline-offset-2">
                        {artifact.title}
                      </h3>
                      <div className="flex items-center gap-1.5 mt-1">
                        <MessageSquare size={12} className="text-claude-textSecondary" />
                        <span className="text-[12px] text-claude-textSecondary truncate">{artifact.conversation_title}</span>
                      </div>
                      <span className="text-[11px] text-claude-textSecondary/60 mt-1 block">
                        {new Date(artifact.created_at).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )
        )}
      </div>
    </div>
  );
};

export default ArtifactsPage;
