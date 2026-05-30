const DEFAULT_PORT = 30085;

let _bridgePort: number | null = null;

export async function ensureBridgePort(): Promise<number> {
  if (_bridgePort) return _bridgePort;
  try {
    const { detectBridgePort } = await import('../api');
    _bridgePort = await detectBridgePort();
    console.log('[config] Bridge port detected:', _bridgePort);
  } catch {
    console.warn('[config] Failed to detect bridge port, using default:', DEFAULT_PORT);
    _bridgePort = DEFAULT_PORT;
  }
  return _bridgePort;
}

function getBridgeBaseUrl(): string {
  const port = _bridgePort || DEFAULT_PORT;
  return `http://127.0.0.1:${port}`;
}

export const config = {
  api: {
    get baseUrl() { return getBridgeBaseUrl(); },
    preview: {
      get list() { return `${getBridgeBaseUrl()}/api/preview`; },
      get: (id: string) => `${getBridgeBaseUrl()}/api/preview/${id}`,
      set: (id: string) => `${getBridgeBaseUrl()}/api/preview/${id}`,
      delete: (id: string) => `${getBridgeBaseUrl()}/api/preview/${id}`,
      events: (id: string) => `${getBridgeBaseUrl()}/api/preview/${id}/events`,
    },
    skills: {
      get list() { return `${getBridgeBaseUrl()}/api/skills`; },
      get: (id: string) => `${getBridgeBaseUrl()}/api/skills/${id}`,
      get listDesign() { return `${getBridgeBaseUrl()}/api/skills/design`; },
    },
  },
  env: {
    isProduction: import.meta.env.PROD,
    isDevelopment: import.meta.env.DEV,
  },
};

export const getApiUrl = (path: string): string => {
  return `${getBridgeBaseUrl()}${path}`;
};

export const getPreviewEventsUrl = (id: string): string => {
  return config.api.preview.events(id);
};
