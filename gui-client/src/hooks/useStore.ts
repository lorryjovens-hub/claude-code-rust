import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Conversation, Task, Model, AppSettings, ViewType } from "@/types";

interface AppState {
  // View State
  currentView: ViewType;
  setCurrentView: (view: ViewType) => void;

  // Chat State
  conversations: Conversation[];
  currentConversation: Conversation | null;
  isGenerating: boolean;
  setConversations: (conversations: Conversation[]) => void;
  setCurrentConversation: (conversation: Conversation | null) => void;
  setIsGenerating: (isGenerating: boolean) => void;

  // Task State
  tasks: Task[];
  setTasks: (tasks: Task[]) => void;

  // Model State
  models: Model[];
  currentModel: Model | null;
  setModels: (models: Model[]) => void;
  setCurrentModel: (model: Model | null) => void;

  // Settings State
  settings: AppSettings;
  setSettings: (settings: Partial<AppSettings>) => void;

  // UI State
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
  theme: "light" | "dark" | "system";
  setTheme: (theme: "light" | "dark" | "system") => void;
}

const defaultSettings: AppSettings = {
  theme: "system",
  language: "zh-CN",
  default_model: "claude-3-5-sonnet",
  api_keys: {},
  shortcuts: {
    "new-chat": "Ctrl+N",
    "send-message": "Enter",
    "new-line": "Shift+Enter",
    "search": "Ctrl+K",
  },
  auto_save: true,
  stream_response: true,
  show_thinking: false,
  max_context_messages: 20,
};

export const useStore = create<AppState>()(
  persist(
    (set) => ({
      // View State
      currentView: "chat",
      setCurrentView: (view) => set({ currentView: view }),

      // Chat State
      conversations: [],
      currentConversation: null,
      isGenerating: false,
      setConversations: (conversations) => set({ conversations }),
      setCurrentConversation: (conversation) => set({ currentConversation: conversation }),
      setIsGenerating: (isGenerating) => set({ isGenerating }),

      // Task State
      tasks: [],
      setTasks: (tasks) => set({ tasks }),

      // Model State
      models: [],
      currentModel: null,
      setModels: (models) => set({ models }),
      setCurrentModel: (model) => set({ currentModel: model }),

      // Settings State
      settings: defaultSettings,
      setSettings: (newSettings) =>
        set((state) => ({
          settings: { ...state.settings, ...newSettings },
        })),

      // UI State
      sidebarOpen: true,
      setSidebarOpen: (open) => set({ sidebarOpen: open }),
      theme: "system",
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: "claude-code-gui-storage",
      partialize: (state) => ({
        settings: state.settings,
        theme: state.theme,
        sidebarOpen: state.sidebarOpen,
      }),
    }
  )
);
