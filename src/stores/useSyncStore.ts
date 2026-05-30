import { create } from 'zustand';
import {
  fullSync,
  pushToCloud,
  pullFromCloud,
  getSyncStatus,
  clearSyncState,
} from '../services/syncService';

export type SyncPhase = 'idle' | 'pushing' | 'pulling' | 'merging' | 'error';

interface SyncState {
  phase: SyncPhase;
  lastSyncAt: string | null;
  lastPushAt: string | null;
  lastPullAt: string | null;
  error: string | null;
  progress: { current: number; total: number } | null;
  autoSyncEnabled: boolean;

  setAutoSyncEnabled: (enabled: boolean) => void;
  refreshStatus: () => void;
  syncNow: () => Promise<boolean>;
  pushNow: () => Promise<boolean>;
  pullNow: () => Promise<boolean>;
  resetSync: () => void;
}

const AUTO_SYNC_KEY = 'sync_autoSyncEnabled';

export const useSyncStore = create<SyncState>()((set, get) => ({
  phase: 'idle',
  lastSyncAt: null,
  lastPushAt: getSyncStatus().lastPushAt,
  lastPullAt: getSyncStatus().lastPullAt,
  error: null,
  progress: null,
  autoSyncEnabled: localStorage.getItem(AUTO_SYNC_KEY) !== 'false',

  setAutoSyncEnabled: (enabled) => {
    localStorage.setItem(AUTO_SYNC_KEY, String(enabled));
    set({ autoSyncEnabled: enabled });
  },

  refreshStatus: () => {
    const status = getSyncStatus();
    set({
      lastPushAt: status.lastPushAt,
      lastPullAt: status.lastPullAt,
      error: status.lastError,
    });
  },

  syncNow: async () => {
    const state = get();
    if (state.phase === 'pushing' || state.phase === 'pulling' || state.phase === 'merging') {
      return false;
    }

    set({ phase: 'pushing', error: null, progress: { current: 0, total: 2 } });

    try {
      set({ phase: 'pushing', progress: { current: 1, total: 2 } });
      const result = await fullSync();
      set({
        phase: 'idle',
        lastSyncAt: result.pullResult.serverTimestamp,
        lastPushAt: result.pushResult.serverTimestamp,
        lastPullAt: result.pullResult.serverTimestamp,
        error: null,
        progress: null,
      });
      return true;
    } catch (err: any) {
      set({
        phase: 'error',
        error: err.message || '同步失败',
        progress: null,
      });
      return false;
    }
  },

  pushNow: async () => {
    const state = get();
    if (state.phase !== 'idle' && state.phase !== 'error') return false;

    set({ phase: 'pushing', error: null });
    try {
      const result = await pushToCloud();
      set({
        phase: 'idle',
        lastPushAt: result.serverTimestamp,
        lastSyncAt: result.serverTimestamp,
        error: null,
      });
      return true;
    } catch (err: any) {
      set({ phase: 'error', error: err.message || '推送失败' });
      return false;
    }
  },

  pullNow: async () => {
    const state = get();
    if (state.phase !== 'idle' && state.phase !== 'error') return false;

    set({ phase: 'pulling', error: null });

    try {
      set({ phase: 'merging' });
      const result = await pullFromCloud();
      set({
        phase: 'idle',
        lastPullAt: result.serverTimestamp,
        lastSyncAt: result.serverTimestamp,
        error: null,
      });
      return true;
    } catch (err: any) {
      set({ phase: 'error', error: err.message || '拉取失败' });
      return false;
    }
  },

  resetSync: () => {
    clearSyncState();
    set({
      phase: 'idle',
      lastSyncAt: null,
      lastPushAt: null,
      lastPullAt: null,
      error: null,
      progress: null,
    });
  },
}));