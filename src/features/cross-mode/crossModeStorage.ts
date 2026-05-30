const STORAGE_KEY = 'cross_mode_overrides';

export function getCrossModeOverride(
  convId: string,
): 'clawparrot' | 'selfhosted' | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const map = JSON.parse(raw);
    return map[convId] || null;
  } catch {
    return null;
  }
}

export function setCrossModeOverride(
  convId: string,
  mode: 'clawparrot' | 'selfhosted',
): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const map = raw ? JSON.parse(raw) : {};
    map[convId] = mode;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
  } catch {
    /* noop */
  }
}

export function clearCrossModeOverride(convId: string): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return;
    const map = JSON.parse(raw);
    delete map[convId];
    localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
  } catch {
    /* noop */
  }
}
