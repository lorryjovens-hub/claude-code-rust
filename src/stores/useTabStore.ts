import { create } from 'zustand';

export interface TabItem {
  id: string;
  conversationId: string | null;
  title: string;
  model?: string;
  unreadCount?: number;
  pinned?: boolean;
  firstMessage?: string;
  createdAt: number;
  lastActiveAt: number;
}

const MAX_TABS = 10;

interface TabState {
  openTabs: TabItem[];
  activeTabId: string | null;

  openTab: (tab: Omit<TabItem, 'id' | 'createdAt' | 'lastActiveAt'> & { id?: string }) => string;
  closeTab: (tabId: string) => void;
  switchTab: (tabId: string) => void;
  renameTab: (tabId: string, title: string) => void;
  pinTab: (tabId: string, pinned: boolean) => void;
  markTabUnread: (tabId: string, count?: number) => void;
  clearTabUnread: (tabId: string) => void;
  setActiveTabConversation: (tabId: string, conversationId: string | null) => void;
  canOpenTab: () => boolean;
}

function generateTabId(): string {
  return `tab-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export const useTabStore = create<TabState>((set, get) => ({
  openTabs: [],
  activeTabId: null,

  openTab: (tab) => {
    const { openTabs } = get();

    if (tab.conversationId) {
      const existing = openTabs.find(t => t.conversationId === tab.conversationId);
      if (existing) {
        get().switchTab(existing.id);
        return existing.id;
      }
    }

    if (openTabs.length >= MAX_TABS) {
      const oldestNonPinned = openTabs
        .filter(t => !t.pinned)
        .sort((a, b) => a.lastActiveAt - b.lastActiveAt)[0];

      if (oldestNonPinned) {
        get().closeTab(oldestNonPinned.id);
      } else {
        const oldest = openTabs.sort((a, b) => a.lastActiveAt - b.lastActiveAt)[0];
        if (oldest) get().closeTab(oldest.id);
      }
    }

    const id = tab.id || generateTabId();
    const now = Date.now();
    const newTab: TabItem = {
      id,
      conversationId: tab.conversationId ?? null,
      title: tab.title,
      model: tab.model,
      unreadCount: tab.unreadCount ?? 0,
      pinned: tab.pinned ?? false,
      firstMessage: tab.firstMessage,
      createdAt: now,
      lastActiveAt: now,
    };

    set((state) => ({
      openTabs: [...state.openTabs, newTab],
      activeTabId: id,
    }));

    return id;
  },

  closeTab: (tabId: string) => {
    set((state) => {
      const idx = state.openTabs.findIndex(t => t.id === tabId);
      if (idx === -1) return state;

      const tabs = state.openTabs.filter(t => t.id !== tabId);

      let newActiveId = state.activeTabId;
      if (state.activeTabId === tabId) {
        if (tabs.length === 0) {
          newActiveId = null;
        } else if (idx >= tabs.length) {
          newActiveId = tabs[tabs.length - 1].id;
        } else {
          newActiveId = tabs[idx].id;
        }
      }

      return { openTabs: tabs, activeTabId: newActiveId };
    });
  },

  switchTab: (tabId: string) => {
    set((state) => {
      const exists = state.openTabs.some(t => t.id === tabId);
      if (!exists) return state;
      return {
        activeTabId: tabId,
        openTabs: state.openTabs.map(t =>
          t.id === tabId ? { ...t, lastActiveAt: Date.now() } : t
        ),
      };
    });
  },

  renameTab: (tabId: string, title: string) => {
    set((state) => ({
      openTabs: state.openTabs.map(t =>
        t.id === tabId ? { ...t, title } : t
      ),
    }));
  },

  pinTab: (tabId: string, pinned: boolean) => {
    set((state) => ({
      openTabs: state.openTabs.map(t =>
        t.id === tabId ? { ...t, pinned } : t
      ),
    }));
  },

  markTabUnread: (tabId: string, count = 1) => {
    set((state) => ({
      openTabs: state.openTabs.map(t =>
        t.id === tabId ? { ...t, unreadCount: (t.unreadCount ?? 0) + count } : t
      ),
    }));
  },

  clearTabUnread: (tabId: string) => {
    set((state) => ({
      openTabs: state.openTabs.map(t =>
        t.id === tabId ? { ...t, unreadCount: 0 } : t
      ),
    }));
  },

  setActiveTabConversation: (tabId: string, conversationId: string | null) => {
    set((state) => ({
      openTabs: state.openTabs.map(t =>
        t.id === tabId ? { ...t, conversationId } : t
      ),
    }));
  },

  canOpenTab: () => {
    return get().openTabs.length < MAX_TABS;
  },
}));
