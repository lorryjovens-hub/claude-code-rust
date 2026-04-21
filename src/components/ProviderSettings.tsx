import React, { useState, useEffect } from 'react';
import { Plus, Trash2, Check, Eye, EyeOff, RefreshCw, ChevronDown, ChevronRight, X, Globe } from 'lucide-react';
import { getProviders, createProvider, updateProvider, deleteProvider, testProviderWebSearch, Provider, ProviderModel } from '../api';

// Auto-detect provider info from URL.
// `webSearch: 'native'` means the bridge has a dedicated native search handler for this provider.
// Anthropic-format providers implicitly support web search via the upstream API's server tool.
const KNOWN_PROVIDERS: Array<{
  match: (url: string) => boolean;
  name: string;
  format: 'anthropic' | 'openai';
  color: string;
  letter: string;
  defaultModels?: ProviderModel[];
  webSearch?: 'native';
}> = [
    {
      match: u => /anthropic\.com/i.test(u), name: 'Anthropic', format: 'anthropic', color: '#D97757', letter: 'A',
      webSearch: 'native',
      defaultModels: [{ id: 'claude-opus-4-6', name: 'Claude Opus 4.6' }, { id: 'claude-sonnet-4-6', name: 'Claude Sonnet 4.6' }, { id: 'claude-haiku-4-5-20251001', name: 'Claude Haiku 4.5' }]
    },
    {
      match: u => /openai\.com/i.test(u), name: 'OpenAI', format: 'openai', color: '#10A37F', letter: 'O',
      defaultModels: [{ id: 'gpt-4o', name: 'GPT-4o' }, { id: 'gpt-4o-mini', name: 'GPT-4o Mini' }, { id: 'o3-mini', name: 'o3-mini' }]
    },
    {
      match: u => /deepseek\.com/i.test(u), name: 'DeepSeek', format: 'openai', color: '#4D6BFE', letter: 'D',
      defaultModels: [{ id: 'deepseek-chat', name: 'DeepSeek V3' }, { id: 'deepseek-reasoner', name: 'DeepSeek R1' }]
    },
    {
      match: u => /bigmodel\.cn/i.test(u), name: 'GLM (Zhipu)', format: 'openai', color: '#3B68FF', letter: 'G',
      webSearch: 'native',
      defaultModels: [{ id: 'glm-5-plus', name: 'GLM-5 Plus' }, { id: 'glm-4-plus', name: 'GLM-4 Plus' }]
    },
    { match: u => /siliconflow/i.test(u), name: 'SiliconFlow', format: 'openai', color: '#7C3AED', letter: 'S' },
    {
      match: u => /minimax/i.test(u), name: 'MiniMax', format: 'openai', color: '#FF6B35', letter: 'M',
      defaultModels: [{ id: 'MiniMax-M1', name: 'MiniMax M1' }]
    },
    {
      match: u => /generativelanguage\.googleapis|gemini/i.test(u), name: 'Google Gemini', format: 'openai', color: '#4285F4', letter: 'G',
      defaultModels: [{ id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro' }, { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash' }]
    },
    {
      match: u => /dashscope\.aliyuncs/i.test(u), name: 'Qwen (Aliyun)', format: 'openai', color: '#FF6A00', letter: 'Q',
      webSearch: 'native',
    },
    {
      match: u => /api-cn\.jiazhuang/i.test(u), name: 'Clawparrot', format: 'anthropic', color: '#C6613F', letter: 'C',
      webSearch: 'native',
      defaultModels: [{ id: 'claude-opus-4-6', name: 'Claude Opus 4.6' }, { id: 'claude-sonnet-4-6', name: 'Claude Sonnet 4.6' }, { id: 'claude-haiku-4-5-20251001', name: 'Claude Haiku 4.5' }]
    },
  ];


function detectProvider(url: string) {
  for (const kp of KNOWN_PROVIDERS) {
    if (kp.match(url)) return kp;
  }
  return null;
}

// Real provider SVG logos
const PROVIDER_LOGOS: Record<string, (size: number) => React.ReactNode> = {
  'Anthropic': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><path d="M13.827 3.52h3.603L24 20.48h-3.603l-6.57-16.96zm-7.258 0h3.767L16.906 20.48h-3.767l-1.932-5.147H4.836L2.904 20.48H-.863L6.57 3.52zm.846 8.832h4.47L9.65 6.36l-2.236 5.992z" fill="#D97757" /></svg>,
  'OpenAI': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><path d="M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.998 5.998 0 0 0-3.998 2.9 6.042 6.042 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073zM13.26 22.43a4.476 4.476 0 0 1-2.876-1.04l.141-.081 4.779-2.758a.795.795 0 0 0 .392-.681v-6.737l2.02 1.168a.071.071 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494zM3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085 4.783 2.759a.771.771 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646zM2.34 7.896a4.485 4.485 0 0 1 2.366-1.973V11.6a.766.766 0 0 0 .388.676l5.815 3.355-2.02 1.168a.076.076 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.872zm16.597 3.855l-5.833-3.387L15.119 7.2a.076.076 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667zm2.01-3.023l-.141-.085-4.774-2.782a.776.776 0 0 0-.785 0L9.409 9.23V6.897a.066.066 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08L8.704 5.46a.795.795 0 0 0-.393.681zm1.097-2.365l2.602-1.5 2.607 1.5v2.999l-2.597 1.5-2.607-1.5z" fill="#10A37F" /></svg>,
  'DeepSeek': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#4D6BFE" /><text x="12" y="16" textAnchor="middle" fill="white" fontSize="12" fontWeight="700" fontFamily="sans-serif">D</text></svg>,
  'GLM (Zhipu)': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#3B68FF" /><text x="12" y="16.5" textAnchor="middle" fill="white" fontSize="11" fontWeight="700" fontFamily="sans-serif">GLM</text></svg>,
  'SiliconFlow': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#7C3AED" /><text x="12" y="16" textAnchor="middle" fill="white" fontSize="12" fontWeight="700" fontFamily="sans-serif">Si</text></svg>,
  'MiniMax': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#FF6B35" /><text x="12" y="16.5" textAnchor="middle" fill="white" fontSize="10" fontWeight="700" fontFamily="sans-serif">MM</text></svg>,
  'Google Gemini': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><path d="M12 24C12 24 24 17.5 24 12S12 0 12 0 0 6.5 0 12s12 12 12 12z" fill="url(#gem)" /><defs><linearGradient id="gem" x1="0" y1="0" x2="24" y2="24"><stop offset="0%" stopColor="#4285F4" /><stop offset="50%" stopColor="#9B72CB" /><stop offset="100%" stopColor="#D96570" /></linearGradient></defs></svg>,
  'Qwen (Aliyun)': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#FF6A00" /><text x="12" y="16.5" textAnchor="middle" fill="white" fontSize="10" fontWeight="700" fontFamily="sans-serif">Qw</text></svg>,
  'Clawparrot': (s) => <svg width={s} height={s} viewBox="0 0 24 24"><circle cx="12" cy="12" r="11" fill="#C6613F" /><text x="12" y="16" textAnchor="middle" fill="white" fontSize="12" fontWeight="700" fontFamily="sans-serif">C</text></svg>,
};

const ProviderIcon: React.FC<{ name: string; color: string; letter: string; size?: number }> = ({ name, color, letter, size = 32 }) => {
  const logo = PROVIDER_LOGOS[name];
  if (logo) return <div className="flex-shrink-0">{logo(size)}</div>;
  return (
    <div className="rounded-lg flex items-center justify-center font-bold text-white flex-shrink-0"
      style={{ width: size, height: size, backgroundColor: color, fontSize: size * 0.38 }}>
      {letter}
    </div>
  );
};

const API_BASE = 'http://127.0.0.1:30080/api';

// Chat models: the subset of models shown in the conversation model selector
interface ChatModel { id: string; name: string; providerId: string; providerName: string; thinkingId?: string; tier?: 'opus' | 'sonnet' | 'haiku' | 'extra'; }

// Composite key that uniquely identifies a model across providers.
// Two providers can expose the same model id (e.g. claude-opus-4-6);
// uid = "providerId:modelId" prevents them from colliding in the UI.
const modelUid = (m: { id: string; providerId: string }) => `${m.providerId}:${m.id}`;

const TIER_DEFS: { key: 'opus' | 'sonnet' | 'haiku'; label: string; description: string }[] = [
  { key: 'opus', label: 'Opus 档', description: 'Most capable for ambitious work' },
  { key: 'sonnet', label: 'Sonnet 档', description: 'Most efficient for everyday tasks' },
  { key: 'haiku', label: 'Haiku 档', description: 'Fastest for quick answers' },
];

function loadChatModels(): ChatModel[] {
  try {
    const raw: ChatModel[] = JSON.parse(localStorage.getItem('chat_models') || '[]');
    // Migrate legacy entries without a tier to 'extra' so they are visible and don't ghost-block tier dropdowns
    let migrated = false;
    for (const m of raw) { if (!m.tier) { m.tier = 'extra'; migrated = true; } }
    if (migrated) localStorage.setItem('chat_models', JSON.stringify(raw));
    return raw;
  } catch { return []; }
}
function saveChatModels(models: ChatModel[]) {
  localStorage.setItem('chat_models', JSON.stringify(models));
}

const SearchableModelSelect = ({
  value,
  onChange,
  options,
  placeholder,
  emptyLabel,
  dashed
}: {
  value: string;
  onChange: (val: string) => void;
  options: { id: string, providerId: string, name: string, providerName: string }[];
  placeholder: string;
  emptyLabel?: string;
  dashed?: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');

  const ref = React.useRef<HTMLDivElement>(null);
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    if (open) {
      document.addEventListener('mousedown', handleClick);
      setSearch(''); // Reset search when opening
    }
    return () => document.removeEventListener('mousedown', handleClick);
  }, [open]);

  const filteredOptions = options.filter(o =>
    o.id.toLowerCase().includes(search.toLowerCase()) ||
    (o.name || '').toLowerCase().includes(search.toLowerCase()) ||
    o.providerName.toLowerCase().includes(search.toLowerCase())
  );

  const optUid = (o: typeof options[0]) => `${o.providerId}:${o.id}`;
  const selectedOption = options.find(o => optUid(o) === value);

  return (
    <div className="relative w-full" ref={ref}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className={`w-full ${dashed ? 'px-3 py-2 border-dashed rounded-[10px] text-claude-textSecondary' : 'px-3 py-1.5 rounded-lg text-claude-text'} bg-transparent border ${dashed ? 'border-claude-border/40' : 'border-claude-border/60'} text-[13px] text-left outline-none hover:border-[#387ee0]/40 focus:border-[#387ee0]/60 transition-colors flex items-center justify-between`}
      >
        <span className="truncate">{selectedOption ? `${selectedOption.name} (${selectedOption.providerName})` : placeholder}</span>
        <ChevronDown size={12} className="text-claude-textSecondary flex-shrink-0 ml-2" />
      </button>

      {open && (
        <div className="absolute z-[100] mt-1 w-[360px] max-w-[80vw] bg-[#ffffff] dark:bg-[#202020] border border-claude-border rounded-[10px] shadow-[0_8px_30px_rgb(0,0,0,0.12)] dark:shadow-[0_8px_30px_rgb(0,0,0,0.5)] overflow-hidden flex flex-col max-h-[380px]">
          <div className="p-2 border-b border-claude-border/50 bg-black/5 dark:bg-white/5">
            <input
              type="text"
              autoFocus
              value={search}
              onChange={e => setSearch(e.target.value)}
              placeholder="搜索模型名称或供应商..."
              className="w-full px-3 py-1.5 bg-claude-input border border-claude-border rounded-[6px] text-[13px] text-claude-text outline-none focus:border-[#387ee0]/60 transition-colors"
            />
          </div>
          <div className="overflow-y-auto flex-1 p-1 relative">
            {emptyLabel && (
              <button
                onClick={() => { onChange(''); setOpen(false); }}
                className={`w-full text-left px-3 py-2 rounded-[6px] text-[13px] mb-0.5 transition-colors hover:bg-claude-hover ${value === '' ? 'bg-claude-hover text-[#387ee0]' : 'text-claude-textSecondary'}`}
              >
                {emptyLabel}
              </button>
            )}
            {filteredOptions.length === 0 && <div className="px-3 py-4 text-center text-[12px] text-claude-textSecondary">未找到匹配模型</div>}
            {filteredOptions.map(o => {
              const uid = optUid(o);
              const selected = value === uid;
              return (
              <button
                key={uid}
                onClick={() => { onChange(uid); setOpen(false); }}
                className={`w-full text-left px-3 py-2 rounded-[6px] text-[13px] mb-0.5 transition-colors hover:bg-black/[0.04] dark:hover:bg-white/[0.04] flex flex-col gap-0.5 ${selected ? 'bg-[#387ee0]/10 text-[#387ee0]' : 'text-claude-text'}`}
              >
                <div className="flex items-center justify-between w-full">
                  <span className={`font-semibold truncate pr-2 ${selected ? 'text-[#387ee0]' : 'text-claude-text'}`}>{o.name}</span>
                  {selected && <Check size={14} className="flex-shrink-0 text-[#387ee0]" />}
                </div>
                <div className={`text-[11px] truncate ${selected ? 'text-[#387ee0]/70' : 'text-claude-textSecondary/60'}`}>
                  {o.providerName} &bull; <span className="font-mono">{o.id}</span>
                </div>
              </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};

const ProviderSettings: React.FC = () => {
  const [providerList, setProviderList] = useState<Provider[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showKeyMap, setShowKeyMap] = useState<Record<string, boolean>>({});
  const [fetchingModels, setFetchingModels] = useState(false);
  const [modelPage, setModelPage] = useState(0);
  const MODELS_PER_PAGE = 10;
  const [defaultModel, setDefaultModel] = useState(localStorage.getItem('default_model') || '');
  const [chatModels, setChatModels] = useState<ChatModel[]>(loadChatModels());

  // Per-provider web-search probe state. Valid values: 'testing' | 'success' | 'failed'.
  // Absence means "never tested" (show as not supported).
  const [webSearchTestState, setWebSearchTestState] = useState<Record<string, 'testing' | 'success' | 'failed'>>({});

  // New provider form
  const [showAdd, setShowAdd] = useState(false);
  const [newUrl, setNewUrl] = useState('');
  const [newKey, setNewKey] = useState('');

  useEffect(() => { loadProviders(); }, []);

  const loadProviders = async () => {
    try {
      const list = await getProviders();
      setProviderList(list);
      if (list.length > 0 && !selectedId) setSelectedId(list[0].id);
    } catch (_) { }
  };

  // Run the web-search probe for a provider and reflect the result in UI state.
  // Kicked off automatically after import and also from the manual "Retest" button.
  const handleTestWebSearch = async (id: string) => {
    setWebSearchTestState(prev => ({ ...prev, [id]: 'testing' }));
    try {
      const result = await testProviderWebSearch(id);
      setWebSearchTestState(prev => ({ ...prev, [id]: result.ok ? 'success' : 'failed' }));
      // Bridge has already persisted supportsWebSearch/webSearchStrategy; pull the fresh record.
      const list = await getProviders();
      setProviderList(list);
    } catch (_) {
      setWebSearchTestState(prev => ({ ...prev, [id]: 'failed' }));
    }
  };

  const handleQuickAdd = async () => {
    if (!newUrl.trim() && !newKey.trim()) return;
    const url = newUrl.trim();
    const key = newKey.trim();
    const detected = detectProvider(url);
    const format = detected?.format || 'openai';

    // Every provider starts with supportsWebSearch=false. It flips to true only after the
    // probe endpoint returns success.
    const p = await createProvider({
      name: detected?.name || extractDomainName(url),
      baseUrl: url,
      format,
      apiKey: key,
      models: (detected?.defaultModels || []).map(m => ({ ...m, enabled: true })),
      enabled: true,
      supportsWebSearch: false,
    });
    setProviderList(prev => [...prev, p]);
    setSelectedId(p.id);
    setShowAdd(false);
    setNewUrl('');
    setNewKey('');

    // Auto-probe: fetch models from /v1/models endpoint for all providers
    if (key) {
      try {
        let endpoint = url.replace(/\/+$/, '').replace(/\/(chat\/completions|messages)$/, '').replace(/\/+$/, '');
        if (!endpoint.endsWith('/v1')) endpoint += '/v1';
        const res = await fetch(endpoint + '/models', { headers: { 'Authorization': 'Bearer ' + key } });
        if (res.ok) {
          const data = await res.json();
          const models = (data.data || [])
            .filter((m: any) => m.id && typeof m.id === 'string')
            .map((m: any) => ({ id: m.id, name: m.id, enabled: true }));
          if (models.length > 0) {
            await updateProvider(p.id, { models });
            setProviderList(prev => prev.map(x => x.id === p.id ? { ...x, models } : x));
          }
        }
        // Also try Anthropic format if OpenAI fails
      } catch (_) { }
    }

    // Kick off the web-search capability test automatically after import.
    // Wait a tick so the user sees the provider card before the spinner appears.
    setTimeout(() => { handleTestWebSearch(p.id); }, 300);
  };

  // Extract a readable name from domain
  function extractDomainName(url: string): string {
    try {
      const host = new URL(url).hostname;
      const parts = host.split('.');
      // e.g. api.penguinsaichat.dpdns.org → penguinsaichat
      if (parts.length >= 3) return parts[parts.length - 3].charAt(0).toUpperCase() + parts[parts.length - 3].slice(1);
      if (parts.length >= 2) return parts[0].charAt(0).toUpperCase() + parts[0].slice(1);
      return host;
    } catch (_) { return 'Custom'; }
  }

  const handleUpdate = async (id: string, updates: Partial<Provider>) => {
    const updated = await updateProvider(id, updates);
    setProviderList(prev => prev.map(p => p.id === id ? { ...p, ...updated } : p));
  };

  const handleDelete = async (id: string) => {
    await deleteProvider(id);
    setProviderList(prev => prev.filter(p => p.id !== id));
    if (selectedId === id) setSelectedId(providerList.find(p => p.id !== id)?.id || null);
  };

  // Auto-fetch models from /v1/models endpoint
  const handleFetchModels = async (p: Provider) => {
    if (!p.baseUrl || !p.apiKey) return;
    setFetchingModels(true);
    try {
      let endpoint = p.baseUrl.replace(/\/+$/, '').replace(/\/(chat\/completions|messages)$/, '').replace(/\/+$/, '');
      if (!endpoint.endsWith('/v1')) endpoint += '/v1';
      endpoint += '/models';

      const headers: Record<string, string> = {};
      if (p.format === 'openai') headers['Authorization'] = 'Bearer ' + p.apiKey;
      else headers['x-api-key'] = p.apiKey;

      const res = await fetch(endpoint, { headers });
      if (res.ok) {
        const data = await res.json();
        const models: ProviderModel[] = (data.data || [])
          .filter((m: any) => m.id && typeof m.id === 'string')
          .map((m: any) => ({ id: m.id, name: m.id, enabled: true }));
        if (models.length > 0) {
          await handleUpdate(p.id, { models });
          // Re-load full provider list to ensure allAvailableModels is up-to-date
          await loadProviders();
        }
      }
    } catch (_) { }
    setFetchingModels(false);
  };

  const selected = providerList.find(p => p.id === selectedId);

  const getProviderMeta = (p: Provider) => {
    const detected = detectProvider(p.baseUrl || '');
    return {
      color: detected?.color || '#6B7280',
      letter: detected?.letter || p.name.charAt(0).toUpperCase(),
    };
  };

  // All models across all providers (for the "add to chat" dropdown)
  const allAvailableModels: ChatModel[] = [];
  for (const p of providerList) {
    if (!p.enabled) continue;
    for (const m of (p.models || [])) {
      if (m.enabled === false) continue;
      allAvailableModels.push({ id: m.id, name: m.name || m.id, providerId: p.id, providerName: p.name });
    }
  }

  // Detect thinking variant for a model ID across all providers
  const detectThinkingId = (modelId: string): string | undefined => {
    for (const p of providerList) {
      if ((p.models || []).some(pm => pm.id === modelId + '-thinking')) {
        return modelId + '-thinking';
      }
    }
    return undefined;
  };

  const handleSetTierModel = (tier: 'opus' | 'sonnet' | 'haiku', uid: string) => {
    // Remove any existing model in this tier
    let updated = chatModels.filter(cm => cm.tier !== tier);
    if (uid) {
      const src = allAvailableModels.find(m => modelUid(m) === uid);
      if (src) {
        const thinkingId = detectThinkingId(src.id);
        updated = [...updated, { ...src, tier, thinkingId }];
      }
    }
    setChatModels(updated);
    saveChatModels(updated);
    // Auto-set default to first tier model
    if (!updated.some(cm => cm.id === defaultModel)) {
      const first = updated.find(cm => cm.tier === 'opus') || updated[0];
      if (first) { setDefaultModel(first.id); localStorage.setItem('default_model', first.id); }
    }
  };

  const handleAddExtraModel = (m: ChatModel) => {
    if (chatModels.some(cm => modelUid(cm) === modelUid(m))) return;
    const thinkingId = detectThinkingId(m.id);
    const updated = [...chatModels, { ...m, tier: 'extra' as const, thinkingId }];
    setChatModels(updated);
    saveChatModels(updated);
  };

  const handleRemoveChatModel = (uid: string) => {
    const removed = chatModels.find(cm => modelUid(cm) === uid);
    const updated = chatModels.filter(cm => modelUid(cm) !== uid);
    setChatModels(updated);
    saveChatModels(updated);
    if (removed && defaultModel === removed.id) {
      const newDefault = updated[0]?.id || '';
      setDefaultModel(newDefault);
      localStorage.setItem('default_model', newDefault);
    }
  };

  const handleSetDefault = (id: string) => {
    setDefaultModel(id);
    localStorage.setItem('default_model', id);
  };

  return (
    <div>
      {/* ===== Chat Models Section ===== */}
      <div className="relative z-50 mb-10 animate-fade-in">
        <h3 className="text-[16px] font-semibold text-claude-text mb-1">对话模型</h3>
        <p className="text-[12px] text-claude-textSecondary/60 mb-4">为每个档位分配模型，它们将显示在对话下拉框中。没有 Thinking 变体的模型将无法开启 Extended thinking。</p>

        {/* Tier slots: Opus / Sonnet / Haiku */}
        <div className="space-y-3 mb-6">
          {TIER_DEFS.map(tier => {
            const assigned = chatModels.find(cm => cm.tier === tier.key);
            const assignedIsDefault = assigned && defaultModel === assigned.id;
            // Models available for this tier (not already assigned to another tier; ignore tierless entries)
            const usedUids = new Set(chatModels.filter(cm => cm.tier && cm.tier !== tier.key).map(cm => modelUid(cm)));
            const available = allAvailableModels.filter(m => !usedUids.has(modelUid(m)) && !m.id.endsWith('-thinking'));
            return (
              <div key={tier.key} className={`rounded-[12px] border transition-colors ${assigned ? (assignedIsDefault ? 'bg-[#387ee0]/5 border-[#387ee0]/40' : 'bg-black/[0.02] dark:bg-white/[0.02] border-claude-border') : 'border-dashed border-claude-border/40'}`}>
                <div className="flex items-center gap-3 px-4 py-3">
                  {/* Default star */}
                  {assigned && (
                    <button onClick={() => handleSetDefault(assigned.id)} title={assignedIsDefault ? '当前默认' : '设为默认'}
                      className={`flex-shrink-0 transition-colors ${assignedIsDefault ? 'text-[#387ee0]' : 'text-claude-textSecondary/30 hover:text-[#387ee0]/80'}`}>
                      <svg width="16" height="16" viewBox="0 0 24 24" fill={assignedIsDefault ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" /></svg>
                    </button>
                  )}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-[14px] font-semibold text-claude-text">{tier.label}</span>
                      <span className="text-[12px] text-claude-textSecondary">{tier.description}</span>
                    </div>
                    <div className="flex items-center gap-2 mt-1.5">
                      <div className="flex-1 max-w-[320px]">
                        <SearchableModelSelect
                          value={assigned ? modelUid(assigned) : ''}
                          onChange={uid => handleSetTierModel(tier.key, uid)}
                          options={[...(assigned && !available.find(x => modelUid(x) === modelUid(assigned)) ? [assigned] : []), ...available]}
                          placeholder="未分配"
                          emptyLabel="未分配"
                        />
                      </div>
                      {assigned && assigned.thinkingId && (
                        <span className="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400 flex-shrink-0">Thinking</span>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        {/* More models section */}
        {(() => {
          const extraModels = chatModels.filter(cm => cm.tier === 'extra');
          const usedUids = new Set(chatModels.map(cm => modelUid(cm)));
          const availableForExtra = allAvailableModels.filter(m => !usedUids.has(modelUid(m)) && !m.id.endsWith('-thinking'));
          return (
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-[13px] font-medium text-claude-textSecondary">More models</span>
              </div>
              {extraModels.length > 0 && (
                <div className="space-y-1.5 mb-3">
                  {extraModels.map(cm => (
                    <div key={modelUid(cm)} className={`rounded-[10px] border transition-colors ${defaultModel === cm.id ? 'bg-[#387ee0]/5 border-[#387ee0]/40' : 'bg-black/[0.02] dark:bg-white/[0.02] border-claude-border/60 hover:bg-black/[0.04] dark:hover:bg-white/[0.04]'}`}>
                      <div className="flex items-center gap-2.5 px-3 py-2">
                        <button onClick={() => handleSetDefault(cm.id)} title={defaultModel === cm.id ? '当前默认' : '设为默认'}
                          className={`flex-shrink-0 transition-colors ${defaultModel === cm.id ? 'text-[#387ee0]' : 'text-claude-textSecondary/30 hover:text-[#387ee0]/80'}`}>
                          <svg width="14" height="14" viewBox="0 0 24 24" fill={defaultModel === cm.id ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" /></svg>
                        </button>
                        <div className="flex-1 min-w-0 flex items-center gap-1.5">
                          <input type="text" value={cm.name}
                            onChange={e => {
                              const updated = chatModels.map(c => modelUid(c) === modelUid(cm) ? { ...c, name: e.target.value } : c);
                              setChatModels(updated);
                              saveChatModels(updated);
                            }}
                            className="text-[13px] text-claude-text font-medium bg-transparent outline-none w-[140px] border-b border-transparent hover:border-claude-border/40 focus:border-[#387ee0]/60 transition-colors"
                          />
                          <span className="text-[11px] text-claude-textSecondary/50 truncate">{cm.providerName}</span>
                        </div>
                        {cm.thinkingId && (
                          <span className="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400 flex-shrink-0">Thinking</span>
                        )}
                        <button onClick={() => handleRemoveChatModel(modelUid(cm))} className="p-0.5 text-claude-textSecondary/20 hover:text-red-400 transition-colors">
                          <X size={12} />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
              {availableForExtra.length > 0 && (
                <div>
                  <SearchableModelSelect
                    value=""
                    onChange={uid => {
                      const m = allAvailableModels.find(x => modelUid(x) === uid);
                      if (m) handleAddExtraModel(m);
                    }}
                    options={availableForExtra}
                    placeholder="+ 添加更多模型..."
                    dashed={true}
                  />
                </div>
              )}
            </div>
          );
        })()}
      </div>

      <hr className="border-claude-border/40 mb-6" />

      {/* ===== Provider Management ===== */}
      <h3 className="text-[16px] font-semibold text-claude-text mb-4">模型供应商</h3>
      <div className="flex gap-6 min-h-[400px] animate-fade-in">
        {/* Left: Provider list */}
        <div className="w-[240px] flex-shrink-0 flex flex-col gap-2">
          <div className="flex items-center justify-between mb-3">
            <span className="text-[13px] font-medium text-claude-textSecondary">供应商</span>
            <button
              onClick={() => setShowAdd(true)}
              className="p-1 text-claude-textSecondary hover:text-claude-text transition-colors rounded hover:bg-claude-hover"
              title="Add provider"
            >
              <Plus size={14} />
            </button>
          </div>

          <div className="flex-1 space-y-0.5 overflow-y-auto">
            {providerList.map(p => {
              const meta = getProviderMeta(p);
              const isActive = selectedId === p.id;
              return (
                <button
                  key={p.id}
                  onClick={() => { setSelectedId(p.id); setModelPage(0); }}
                  className={`w-full flex items-center gap-3 px-3 py-3 rounded-[12px] transition-colors text-left border ${isActive ? 'bg-claude-input border-claude-border shadow-sm' : 'border-transparent hover:bg-claude-hover/80'
                    }`}
                >
                  <ProviderIcon name={p.name} color={meta.color} letter={meta.letter} size={28} />
                  <div className="flex-1 min-w-0">
                    <div className={`text-[13px] truncate ${isActive ? 'text-claude-text font-medium' : 'text-claude-textSecondary'}`}>
                      {p.name}
                    </div>
                    <div className="text-[10px] text-claude-textSecondary/50 flex items-center gap-1.5">
                      <span>{(p.models || []).length} models</span>
                      {webSearchTestState[p.id] === 'testing' ? (
                        <span className="flex items-center gap-1 text-[#387ee0] font-medium" title="正在测试网页搜索能力">
                          <RefreshCw size={9} className="animate-spin" />
                          <span>测试中</span>
                        </span>
                      ) : p.supportsWebSearch ? (
                        <span className="flex items-center gap-0.5 text-[#387ee0]" title="已验证支持网页搜索">
                          <Globe size={9} />
                        </span>
                      ) : null}
                    </div>
                  </div>
                  {!p.enabled && (
                    <div className="w-1.5 h-1.5 rounded-full bg-claude-textSecondary/30 flex-shrink-0" title="Disabled" />
                  )}
                </button>
              );
            })}
          </div>
        </div>

        {/* Right: Provider detail */}
        <div className="flex-1 overflow-y-auto">
          {/* Quick add dialog */}
          {showAdd && (
            <div className="mb-6 p-5 rounded-[16px] border border-claude-border bg-claude-input shadow-sm">
              <div className="text-[15px] font-medium text-claude-text mb-4">添加模型供应商</div>
              <div className="space-y-3">
                <input
                  type="text"
                  value={newUrl}
                  onChange={e => setNewUrl(e.target.value)}
                  placeholder="API 地址（如 https://api.openai.com）"
                  className="w-full bg-transparent border border-claude-border rounded-[8px] px-3 py-2.5 text-[14px] text-claude-text outline-none focus:border-[#387ee0]/60 transition-colors placeholder:text-claude-textSecondary/40"
                  autoFocus
                />
                <input
                  type="password"
                  value={newKey}
                  onChange={e => setNewKey(e.target.value)}
                  placeholder="API Key"
                  className="w-full bg-transparent border border-claude-border rounded-[8px] px-3 py-2.5 text-[14px] text-claude-text outline-none focus:border-[#387ee0]/60 transition-colors placeholder:text-claude-textSecondary/40"
                  onKeyDown={e => { if (e.key === 'Enter') handleQuickAdd(); }}
                />
                {newUrl.trim() && (() => {
                  const det = detectProvider(newUrl.trim());
                  return det ? (
                    <div className="flex items-center gap-2 text-[12px] text-claude-textSecondary">
                      <ProviderIcon name={det.name} color={det.color} letter={det.letter} size={20} />
                      <span>已识别：<strong className="text-claude-text">{det.name}</strong>（{det.format === 'openai' ? 'OpenAI 兼容格式' : 'Anthropic 格式'}）</span>
                    </div>
                  ) : (
                    <div className="text-[12px] text-claude-textSecondary/60">未识别的供应商，添加后将自动探测格式和可用模型</div>
                  );
                })()}
                <div className="flex gap-2 pt-2">
                  <button onClick={handleQuickAdd} className="px-4 py-2 text-[14px] font-medium text-claude-bg bg-claude-text rounded-lg transition-colors hover:opacity-90">
                    添加
                  </button>
                  <button onClick={() => { setShowAdd(false); setNewUrl(''); setNewKey(''); }} className="px-4 py-2 text-[14px] font-medium text-claude-text border border-claude-border hover:bg-claude-hover rounded-lg transition-colors">
                    取消
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Selected provider detail */}
          {selected ? (() => {
            const meta = getProviderMeta(selected);
            return (
              <div className="flex-1 space-y-6 bg-claude-input border border-claude-border rounded-[16px] p-6 shadow-sm">
                {/* Header */}
                <div className="flex items-center gap-3">
                  <ProviderIcon name={selected.name} color={meta.color} letter={meta.letter} size={36} />
                  <div className="flex-1">
                    <input
                      type="text"
                      value={selected.name}
                      onChange={e => handleUpdate(selected.id, { name: e.target.value })}
                      className="text-[18px] font-semibold text-claude-text bg-transparent outline-none w-full"
                    />
                  </div>
                  <button
                    onClick={() => handleDelete(selected.id)}
                    className="p-1.5 text-claude-textSecondary/30 hover:text-red-400 transition-colors rounded-lg hover:bg-red-400/10"
                    title="Delete provider"
                  >
                    <Trash2 size={15} />
                  </button>
                  <button
                    onClick={() => handleUpdate(selected.id, { enabled: !selected.enabled })}
                    className={`w-10 h-6 rounded-full relative transition-colors ${selected.enabled ? 'bg-[#387ee0]' : 'bg-claude-border'}`}
                  >
                    <div className={`absolute top-1 w-4 h-4 rounded-full bg-white shadow-sm transition-transform ${selected.enabled ? 'left-5' : 'left-1'}`} />
                  </button>
                </div>

                {/* API Key */}
                <div>
                  <label className="text-[12px] text-claude-textSecondary mb-1.5 block font-medium">API 密钥</label>
                  <div className="flex items-center gap-2">
                    <input
                      type={showKeyMap[selected.id] ? 'text' : 'password'}
                      value={selected.apiKey || ''}
                      onChange={e => handleUpdate(selected.id, { apiKey: e.target.value })}
                      placeholder="sk-..."
                      className="flex-1 bg-transparent border border-claude-border rounded-[8px] px-3 py-2 text-[14px] text-claude-text outline-none focus:border-[#387ee0]/60 transition-colors placeholder:text-claude-textSecondary/40 font-mono"
                    />
                    <button
                      onClick={() => setShowKeyMap(prev => ({ ...prev, [selected.id]: !prev[selected.id] }))}
                      className="p-2 text-claude-textSecondary hover:text-claude-text transition-colors rounded-lg hover:bg-claude-hover"
                    >
                      {showKeyMap[selected.id] ? <EyeOff size={14} /> : <Eye size={14} />}
                    </button>
                  </div>
                </div>

                {/* Base URL */}
                <div>
                  <label className="text-[12px] text-claude-textSecondary mb-1.5 block font-medium">API 地址</label>
                  <input
                    type="text"
                    value={selected.baseUrl || ''}
                    onChange={e => {
                      const newUrl = e.target.value;
                      const det = detectProvider(newUrl);
                      const patch: Partial<Provider> = { baseUrl: newUrl };
                      if (det && det.format !== selected.format) patch.format = det.format;
                      // URL change invalidates any previous test result — user must retest
                      patch.supportsWebSearch = false;
                      patch.webSearchStrategy = null;
                      patch.webSearchTestedAt = undefined;
                      handleUpdate(selected.id, patch);
                    }}
                    className="w-full bg-transparent border border-claude-border rounded-[8px] px-3 py-2 text-[14px] text-claude-text outline-none focus:border-[#387ee0]/60 transition-colors placeholder:text-claude-textSecondary/40 font-mono"
                  />
                </div>

                {/* Format */}
                <div>
                  <label className="text-[12px] text-claude-textSecondary mb-1.5 block font-medium">API 格式</label>
                  <div className="flex gap-2">
                    {(['openai', 'anthropic'] as const).map(f => (
                      <button
                        key={f}
                        onClick={() => handleUpdate(selected.id, { format: f })}
                        className={`px-3.5 py-1.5 rounded-lg text-[12px] font-medium transition-all ${selected.format === f
                          ? 'bg-black/[0.05] dark:bg-white/[0.1] text-claude-text border border-claude-textSecondary/50'
                          : 'border border-claude-border/40 text-claude-textSecondary hover:text-claude-text hover:border-claude-textSecondary/30'
                          }`}
                      >
                        {f === 'openai' ? 'OpenAI 兼容' : 'Anthropic'}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Web search capability — determined solely by the probe result */}
                {(() => {
                  const state = webSearchTestState[selected.id];
                  const isTesting = state === 'testing';
                  const hasTested = !!selected.webSearchTestedAt;
                  const supported = selected.supportsWebSearch === true;
                  const strategy = selected.webSearchStrategy;
                  const testedAt = selected.webSearchTestedAt ? new Date(selected.webSearchTestedAt).toLocaleString() : null;
                  return (
                    <div>
                      <label className="text-[12px] text-claude-textSecondary mb-1.5 block font-medium flex items-center gap-1.5">
                        <Globe size={12} /> 网页搜索能力
                      </label>
                      <div className={`rounded-[10px] border p-3 flex items-start gap-3 transition-colors ${
                        isTesting ? 'border-[#387ee0]/40 bg-[#387ee0]/[0.06]' :
                        supported ? 'border-[#387ee0]/40 bg-[#387ee0]/[0.04]' :
                        hasTested ? 'border-claude-border/60 bg-claude-hover/30' :
                        'border-claude-border/60'
                      }`}>
                        <div className="flex-shrink-0 mt-0.5">
                          {isTesting ? (
                            <RefreshCw size={16} className="text-[#387ee0] animate-spin" />
                          ) : supported ? (
                            <Check size={16} className="text-[#387ee0]" strokeWidth={2.5} />
                          ) : hasTested ? (
                            <X size={16} className="text-claude-textSecondary/60" />
                          ) : (
                            <Globe size={16} className="text-claude-textSecondary/50" />
                          )}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className={`text-[12.5px] font-medium mb-0.5 ${isTesting || supported ? 'text-claude-text' : 'text-claude-textSecondary'}`}>
                            {isTesting ? '正在测试网页搜索能力...' :
                             supported ? '已验证支持网页搜索' :
                             hasTested ? '此供应商不支持网页搜索' :
                             '尚未测试'}
                          </div>
                          <div className="text-[11px] text-claude-textSecondary/80 leading-relaxed">
                            {isTesting ? '正在向供应商发送一次带 web_search 工具的探测请求（最长 45 秒）' :
                             supported ? (
                               <>
                                 策略：<span className="font-mono text-claude-text">{strategy || '—'}</span>
                                 {testedAt && <span className="ml-2 opacity-60">· {testedAt}</span>}
                               </>
                             ) :
                             hasTested ? (
                               <>
                                 {selected.webSearchTestReason || '探测未返回有效搜索结果'}
                                 <div className="mt-0.5 opacity-70">对话中模型请求的 web_search 工具会被自动剥除，不会虚假搜索</div>
                               </>
                             ) :
                             '新导入的供应商默认不启用网页搜索。点击右侧"测试"按钮验证。'}
                          </div>
                        </div>
                        <button
                          onClick={() => handleTestWebSearch(selected.id)}
                          disabled={isTesting || !selected.apiKey || !selected.baseUrl}
                          className="flex-shrink-0 px-3 py-1.5 text-[11.5px] font-medium rounded-lg border border-claude-border/60 text-claude-textSecondary hover:text-claude-text hover:bg-claude-hover transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                        >
                          {isTesting ? '测试中...' : hasTested ? '重新测试' : '测试'}
                        </button>
                      </div>
                    </div>
                  );
                })()}

                {/* Models */}
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <label className="text-[12px] text-claude-textSecondary font-medium">模型列表</label>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleFetchModels(selected)}
                        disabled={fetchingModels}
                        className="text-[11px] text-claude-textSecondary hover:text-claude-text transition-colors flex items-center gap-1 px-2 py-1 rounded hover:bg-claude-hover"
                      >
                        <RefreshCw size={11} className={fetchingModels ? 'animate-spin' : ''} />
                        获取模型列表
                      </button>
                      <button
                        onClick={() => {
                          const models = [...(selected.models || []), { id: '', name: '', enabled: true }];
                          handleUpdate(selected.id, { models });
                        }}
                        className="text-[11px] text-claude-textSecondary hover:text-claude-text transition-colors flex items-center gap-1 px-2 py-1 rounded hover:bg-claude-hover"
                      >
                        <Plus size={11} /> 添加
                      </button>
                    </div>
                  </div>
                  <div className="space-y-0.5 pr-2 -mr-2">
                    {(selected.models || []).slice(modelPage * MODELS_PER_PAGE, (modelPage + 1) * MODELS_PER_PAGE).map((m, _pi) => {
                      const mi = modelPage * MODELS_PER_PAGE + _pi; // real index in full array
                      const hasThinking = (selected.models || []).some(x => x.id === m.id + '-thinking') || m.id.endsWith('-thinking');
                      return (
                        <div key={mi} className="flex items-center gap-2 group rounded-lg px-2 py-1.5 -mx-2 transition-colors hover:bg-claude-hover/50">
                          <button
                            onClick={() => {
                              const models = [...(selected.models || [])];
                              models[mi] = { ...models[mi], enabled: models[mi].enabled === false ? true : false };
                              handleUpdate(selected.id, { models });
                            }}
                            className={`w-4 h-4 rounded border flex-shrink-0 flex items-center justify-center transition-colors ${m.enabled !== false ? 'bg-claude-text border-claude-text' : 'border-claude-border'
                              }`}
                          >
                            {m.enabled !== false && <Check size={10} className="text-claude-bg" strokeWidth={3} />}
                          </button>
                          <div className="flex-1 min-w-0 flex items-center gap-2">
                            <input
                              type="text"
                              value={m.name || ''}
                              onChange={e => {
                                const models = [...(selected.models || [])];
                                models[mi] = { ...models[mi], name: e.target.value };
                                handleUpdate(selected.id, { models });
                              }}
                              placeholder={m.id || '显示名称'}
                              className="w-[100px] bg-transparent text-[12.5px] text-claude-text outline-none py-0.5 placeholder:text-claude-textSecondary/30 truncate"
                            />
                            <input
                              type="text"
                              value={m.id}
                              onChange={e => {
                                const models = [...(selected.models || [])];
                                models[mi] = { ...models[mi], id: e.target.value };
                                handleUpdate(selected.id, { models });
                              }}
                              placeholder="model-id"
                              className="flex-1 bg-transparent text-[11px] text-claude-textSecondary/50 font-mono outline-none py-0.5 placeholder:text-claude-textSecondary/25 truncate"
                            />
                          </div>
                          {hasThinking && !m.id.endsWith('-thinking') && (
                            <span className="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400 flex-shrink-0" title="支持扩展思考">Thinking</span>
                          )}
                          <button
                            onClick={() => {
                              const models = (selected.models || []).filter((_, i) => i !== mi);
                              handleUpdate(selected.id, { models });
                            }}
                            className="p-0.5 text-claude-textSecondary/0 group-hover:text-claude-textSecondary/30 hover:!text-red-400 transition-colors flex-shrink-0"
                          >
                            <X size={12} />
                          </button>
                        </div>
                      );
                    })}
                    {(!selected.models || selected.models.length === 0) && (
                      <div className="text-[12px] text-claude-textSecondary/40 py-2">暂无模型 — 点击「获取模型列表」自动拉取，或手动添加。</div>
                    )}
                    {(selected.models || []).length > MODELS_PER_PAGE && (
                      <div className="flex items-center justify-between pt-2 mt-1 border-t border-claude-border/30">
                        <button
                          onClick={() => setModelPage(p => Math.max(0, p - 1))}
                          disabled={modelPage === 0}
                          className="text-[11px] px-2 py-1 rounded text-claude-textSecondary hover:bg-claude-hover disabled:opacity-30 disabled:cursor-default transition-colors"
                        >← 上一页</button>
                        <span className="text-[11px] text-claude-textSecondary/50">
                          {modelPage + 1} / {Math.ceil((selected.models || []).length / MODELS_PER_PAGE)}
                        </span>
                        <button
                          onClick={() => setModelPage(p => Math.min(Math.ceil((selected.models || []).length / MODELS_PER_PAGE) - 1, p + 1))}
                          disabled={modelPage >= Math.ceil((selected.models || []).length / MODELS_PER_PAGE) - 1}
                          className="text-[11px] px-2 py-1 rounded text-claude-textSecondary hover:bg-claude-hover disabled:opacity-30 disabled:cursor-default transition-colors"
                        >下一页 →</button>
                      </div>
                    )}
                  </div>
                </div>

                {/* Default model display */}
                {defaultModel && (
                  <div className="text-[11px] text-claude-textSecondary/50 flex items-center gap-1.5 pt-1">
                    <span>默认对话模型：</span>
                    <span className="text-claude-text font-medium">{
                      (() => {
                        for (const p of providerList) {
                          const m = (p.models || []).find(x => x.id === defaultModel);
                          if (m) return m.name || m.id;
                        }
                        return defaultModel;
                      })()
                    }</span>
                  </div>
                )}

              </div>
            );
          })() : !showAdd && (
            <div className="flex flex-col items-center justify-center h-full text-claude-textSecondary/40">
              <div className="text-[14px] mb-2">还没有配置供应商</div>
              <button
                onClick={() => setShowAdd(true)}
                className="text-[13px] text-claude-textSecondary hover:text-claude-text transition-colors"
              >
                + 添加第一个供应商
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ProviderSettings;
