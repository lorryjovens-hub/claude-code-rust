function getChatModelMap(): Map<string, { thinkingId?: string }> {
  try {
    const models = JSON.parse(localStorage.getItem('chat_models') || '[]');
    const map = new Map<string, { thinkingId?: string }>();
    for (const m of models) {
      map.set(m.id, { thinkingId: m.thinkingId });
      if (m.thinkingId) map.set(m.thinkingId, { thinkingId: m.thinkingId });
    }
    return map;
  } catch { return new Map(); }
}

export function stripThinking(model: string): string {
  const map = getChatModelMap();
  for (const [baseId, cfg] of map) {
    if (cfg.thinkingId === model) return baseId;
  }
  return (model || '').replace(/-thinking$/, '');
}

export function withThinking(base: string, thinking: boolean): string {
  if (!thinking) return base;
  const map = getChatModelMap();
  const cfg = map.get(base);
  if (cfg?.thinkingId) return cfg.thinkingId;
  return `${base}-thinking`;
}

export function isThinkingModel(model: string): boolean {
  const map = getChatModelMap();
  for (const [, cfg] of map) {
    if (cfg.thinkingId === model) return true;
  }
  return typeof model === 'string' && model.endsWith('-thinking');
}
