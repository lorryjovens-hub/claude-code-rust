import { useEffect, useMemo, useCallback } from 'react';
import { useChatStore } from '../../stores/useChatStore';
import { getUserModels, getProviderModels } from '../../api';
import { stripThinking, withThinking, isThinkingModel } from '../../utils/modelHelpers';
import { SelectableModel } from '../../components/ModelSelector';

export function useModelCatalog(viewingIdRef: React.MutableRefObject<string | null>) {
  const {
    modelCatalog, setModelCatalog,
    currentModel, setCurrentModel,
    providersCache, setProvidersCache,
  } = useChatStore();

  // Model state
  const isSelfHostedMode = localStorage.getItem('user_mode') === 'selfhosted';

  const selfHostedModels = useMemo<SelectableModel[]>(() => {
    if (!isSelfHostedMode) return [];
    try {
      const chatModels = JSON.parse(localStorage.getItem('chat_models') || '[]');
      if (chatModels.length === 0) return [];
      const tierDescMap: Record<string, string> = {
        'opus': 'Most capable for ambitious work',
        'sonnet': 'Most efficient for everyday tasks',
        'haiku': 'Fastest for quick answers',
      };
      return chatModels.map((m: any) => ({
        id: m.id,
        name: m.name || m.id,
        enabled: 1,
        tier: m.tier || 'extra',
        description: m.tier && tierDescMap[m.tier] ? tierDescMap[m.tier] : undefined,
      }));
    } catch { return []; }
  }, [isSelfHostedMode]);

  const fallbackCommonModels = useMemo<SelectableModel[]>(() => {
    if (isSelfHostedMode && selfHostedModels.length > 0) {
      const tierOrder = ['opus', 'sonnet', 'haiku'];
      const common = tierOrder.map(t => selfHostedModels.find(m => m.tier === t)).filter(Boolean) as SelectableModel[];
      return common.length > 0 ? common : selfHostedModels;
    }
    return [
      { id: 'claude-opus-4-8', name: 'Opus 4.8', enabled: 1, description: 'Most capable for ambitious work' },
      { id: 'claude-sonnet-4-6', name: 'Sonnet 4.6', enabled: 1, description: 'Most efficient for everyday tasks' },
      { id: 'claude-haiku-4-5-20251001', name: 'Haiku 4.5', enabled: 1, description: 'Fastest for quick answers' },
    ];
  }, [isSelfHostedMode, selfHostedModels]);

  const displayCommonModels = modelCatalog?.common?.length ? modelCatalog.common : fallbackCommonModels;

  const selectorModels = useMemo<SelectableModel[]>(() => {
    const visible: SelectableModel[] = displayCommonModels.map(m => ({
      ...m,
      enabled: typeof m.enabled === 'number' ? m.enabled : 1,
      tier: (m.tier ?? undefined) as SelectableModel['tier'],
    }));
    if (isSelfHostedMode) {
      const seen = new Set(visible.map(m => m.id));
      const extraModels = (modelCatalog?.all || []).filter(m => !seen.has(m.id));
      for (const model of extraModels) {
        visible.push({ ...model, enabled: typeof model.enabled === 'number' ? model.enabled : 1, tier: (model.tier || 'extra') as SelectableModel['tier'] });
        seen.add(model.id);
      }
    }
    return visible;
  }, [displayCommonModels, modelCatalog, isSelfHostedMode]);

  // Provider web-search capability
  const currentProviderSupportsWebSearch = useMemo(() => {
    if (!providersCache.length) return false;
    const bareModel = (currentModel || '').replace(/-thinking$/, '');
    for (const p of providersCache) {
      if ((p.models || []).some((m: any) => m.id === bareModel)) {
        return p.supportsWebSearch === true;
      }
    }
    return false;
  }, [providersCache, currentModel]);

  const isModelSelectable = useCallback((modelString: string) => {
    const base = stripThinking(modelString);
    const pool = modelCatalog?.all || displayCommonModels;
    const found = pool.find(m => m.id === base);
    return !!found && Number(found.enabled) === 1;
  }, [modelCatalog, displayCommonModels]);

  const resolveModelForNewChat = useCallback((preferredModel?: string | null) => {
    const saved = preferredModel || localStorage.getItem('default_model') || 'claude-sonnet-4-6';
    const thinking = isThinkingModel(saved);
    const base = stripThinking(saved);
    const all = modelCatalog?.all || displayCommonModels;
    const preferred = all.find(m => m.id === base);
    if (preferred && Number(preferred.enabled) === 1) {
      return withThinking(base, thinking);
    }
    const fallbackBase = modelCatalog?.fallback_model
      || all.find(m => /sonnet/i.test(m.id) && Number(m.enabled) === 1)?.id
      || all.find(m => Number(m.enabled) === 1)?.id
      || base
      || 'claude-sonnet-4-6';
    return withThinking(fallbackBase, thinking);
  }, [displayCommonModels, modelCatalog]);

  // Initialize currentModel from localStorage on mount
  useEffect(() => {
    if (!currentModel) {
      const saved = localStorage.getItem('default_model');
      if (saved) {
        setCurrentModel(saved);
      } else if (isSelfHostedMode && selfHostedModels.length > 0) {
        setCurrentModel(selfHostedModels[0].id);
      } else {
        try {
          const providers = JSON.parse(localStorage.getItem('app_providers') || '[]');
          for (const p of providers) {
            if (!p.enabled) continue;
            const firstEnabled = (p.models || []).find((m: any) => m.enabled !== false);
            if (firstEnabled) {
              setCurrentModel(firstEnabled.id);
              return;
            }
          }
        } catch {}
        setCurrentModel('claude-sonnet-4-6');
      }
    }
  }, []);

  // Load models from server, refresh every 60s
  useEffect(() => {
    let cancelled = false;
    const isSelfHosted = localStorage.getItem('user_mode') === 'selfhosted';
    const loadModels = async () => {
      try {
        let data: any;
        if (isSelfHosted) {
          let chatModels: any[] = [];
          try { chatModels = JSON.parse(localStorage.getItem('chat_models') || '[]'); } catch {}
          if (chatModels.length > 0) {
            const tierDescMap: Record<string, string> = {
              'opus': 'Most capable for ambitious work',
              'sonnet': 'Most efficient for everyday tasks',
              'haiku': 'Fastest for quick answers',
            };
            const all = chatModels.map((m: any) => ({
              id: m.id, name: m.name || m.id, enabled: 1,
              tier: m.tier || 'extra',
              description: m.tier && tierDescMap[m.tier] ? tierDescMap[m.tier] : undefined,
            }));
            const tierOrder = ['opus', 'sonnet', 'haiku'];
            const common = tierOrder.map(t => all.find((m: any) => m.tier === t)).filter(Boolean);
            data = { all, common: common.length > 0 ? common : all, fallback_model: localStorage.getItem('default_model') || all[0]?.id || 'claude-sonnet-4-6' };
          } else {
            const pModels = await getProviderModels();
            const all = pModels.map(m => ({ id: m.id, name: m.name || m.id, enabled: 1 }));
            data = { all, common: all, fallback_model: all[0]?.id || 'claude-sonnet-4-6' };
          }
        } else {
          data = await getUserModels();
          const descMap: Record<string, string> = {
            'claude-opus-4-6': 'Most capable for ambitious work',
            'claude-sonnet-4-6': 'Most efficient for everyday tasks',
            'claude-haiku-4-5-20251001': 'Fastest for quick answers',
          };
          for (const list of [data?.common, data?.all]) {
            if (!Array.isArray(list)) continue;
            for (const m of list) {
              if (descMap[m.id] && !m.description) m.description = descMap[m.id];
            }
          }
          try {
            const pModels = await getProviderModels();
            if (Array.isArray(pModels) && pModels.length > 0) {
              const existingIds = new Set((data?.all || []).map((m: any) => m.id));
              const merged = [...(data?.all || [])];
              for (const pm of pModels) {
                if (!existingIds.has(pm.id)) {
                  merged.push({ id: pm.id, name: pm.name || pm.id, enabled: 1, tier: 'extra' });
                  existingIds.add(pm.id);
                }
              }
              data = { ...data, all: merged };
            }
          } catch {}
        }
        if (cancelled) return;
        setModelCatalog(data);
        if (!viewingIdRef.current) {
          setCurrentModel(prev => {
            const current = prev || localStorage.getItem('default_model') || 'claude-sonnet-4-6';
            const thinking = isThinkingModel(current);
            const base = stripThinking(current);
            const all: SelectableModel[] = data?.all?.length ? data.all : fallbackCommonModels;
            const preferred = all.find((m: SelectableModel) => m.id === base && Number(m.enabled) === 1);
            if (preferred) return withThinking(base, thinking);
            const fallbackBase = data?.fallback_model
              || all.find((m: SelectableModel) => /sonnet/i.test(m.id) && Number(m.enabled) === 1)?.id
              || all.find((m: SelectableModel) => Number(m.enabled) === 1)?.id
              || base
              || 'claude-sonnet-4-6';
            return withThinking(fallbackBase, thinking);
          });
        }
      } catch { /* ignore */ }
    };
    loadModels();
    const timer = setInterval(loadModels, 60000);
    return () => { cancelled = true; clearInterval(timer); };
  }, [fallbackCommonModels]);

  return {
    selectorModels,
    isModelSelectable,
    resolveModelForNewChat,
    currentProviderSupportsWebSearch,
    isSelfHostedMode,
    selfHostedModels,
  };
}
