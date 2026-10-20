import type { ProviderModel } from '../api/providers';

export function isDirectMiniMax(url: string): boolean {
  try {
    return ['api.minimax.io', 'api.minimaxi.com'].includes(new URL(url).hostname);
  } catch {
    return false;
  }
}

export const MINIMAX_MODELS: ProviderModel[] = [
  {
    id: 'MiniMax-M3.1-Flash',
    name: 'MiniMax M3.1 Flash',
    contextWindow: 1_000_000,
    maxTokens: 524_288,
    pricingUsdPerMillionTokens: { input: 0.15, output: 0.6, cacheRead: 0.015, cacheWrite: null },
    inputModalities: ['text', 'image', 'video'],
    thinking: ['adaptive'],
  },
  {
    id: 'MiniMax-M3',
    name: 'MiniMax M3',
    contextWindow: 1_000_000,
    // M3 pricing varies by context length; the flat pricing schema cannot represent it.
    inputModalities: ['text', 'image', 'video'],
    thinking: ['adaptive', 'disabled'],
  },
];

// Discovery is not a migration: retain every explicitly saved/custom model.
export function mergeDiscoveredModels(
  url: string,
  discovered: ProviderModel[],
  saved: ProviderModel[] = [],
): ProviderModel[] {
  if (!isDirectMiniMax(url)) return discovered;
  const currentIds = new Set(MINIMAX_MODELS.map(model => model.id));
  const result: ProviderModel[] = MINIMAX_MODELS.map(model => ({
    ...model,
    enabled: true,
    ...saved.find(existing => existing.id === model.id),
  }));
  for (const model of [...saved, ...discovered]) {
    if (result.some(existing => existing.id === model.id)) continue;
    const legacyText = /^(?:MiniMax-(?:M\d|Text-)|abab)/i.test(model.id) && !currentIds.has(model.id);
    if (saved.some(existing => existing.id === model.id) || !legacyText) result.push(model);
  }
  return result;
}
