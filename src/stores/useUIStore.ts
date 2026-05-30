import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

interface UIState {
  sidebarCollapsed: boolean;
  showSettings: boolean;
  showUpgrade: boolean;
  showPlusMenu: boolean;
  showAgentPanel: boolean;
  showAnalyticsPanel: boolean;
  showOnboarding: boolean;
  showDirectoryModal: boolean;
  showPromptSuggestions: boolean;
  showArtifacts: boolean;
  showMcpPanel: boolean;
  showSkillsSubmenu: boolean;
  showProjectsSubmenu: boolean;
  showGithubModal: boolean;
  showCompactDialog: boolean;
  showSlashPalette: boolean;
  showTerminalPanel: boolean;
  slashPaletteInput: string;
  inputHeight: number;
  isDragging: boolean;
  documentPanelWidth: number;
  terminalPanelHeight: number;
  zoomLevel: number;
  language: string;

  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setShowSettings: (show: boolean) => void;
  setShowUpgrade: (show: boolean) => void;
  setShowPlusMenu: (show: boolean | ((prev: boolean) => boolean)) => void;
  setShowAgentPanel: (show: boolean) => void;
  setShowAnalyticsPanel: (show: boolean) => void;
  setShowOnboarding: (show: boolean) => void;
  setShowDirectoryModal: (show: boolean) => void;
  setShowPromptSuggestions: (show: boolean) => void;
  setShowArtifacts: (show: boolean) => void;
  setShowMcpPanel: (show: boolean) => void;
  setShowSkillsSubmenu: (show: boolean | ((prev: boolean) => boolean)) => void;
  setShowProjectsSubmenu: (show: boolean | ((prev: boolean) => boolean)) => void;
  setShowGithubModal: (show: boolean) => void;
  setShowCompactDialog: (show: boolean) => void;
  setShowSlashPalette: (show: boolean) => void;
  setShowTerminalPanel: (show: boolean) => void;
  setSlashPaletteInput: (input: string) => void;
  setInputHeight: (height: number) => void;
  setIsDragging: (dragging: boolean) => void;
  setDocumentPanelWidth: (width: number) => void;
  setTerminalPanelHeight: (height: number) => void;
  setZoomLevel: (level: number) => void;
  setLanguage: (lang: string) => void;
}

const getInitialLanguage = (): string => {
  const stored = localStorage.getItem('app_language');
  if (stored) return stored;
  const browserLang = navigator.language || (navigator as any).userLanguage || '';
  if (browserLang.toLowerCase().startsWith('zh')) return 'zh';
  return 'en';
};

export const useUIStore = create<UIState>()(
  subscribeWithSelector((set) => ({
    sidebarCollapsed: false,
    showSettings: false,
    showUpgrade: false,
    showPlusMenu: false,
    showAgentPanel: false,
    showAnalyticsPanel: false,
    showOnboarding: typeof window !== 'undefined' ? !localStorage.getItem('onboarding_done') : false,
    showDirectoryModal: false,
    showPromptSuggestions: true,
    showArtifacts: false,
    showMcpPanel: false,
    showSkillsSubmenu: false,
    showProjectsSubmenu: false,
    showGithubModal: false,
    showCompactDialog: false,
    showSlashPalette: false,
    showTerminalPanel: false,
    slashPaletteInput: '',
    inputHeight: 0,
    isDragging: false,
    documentPanelWidth: 400,
    terminalPanelHeight: 300,
    zoomLevel: 1,
    language: getInitialLanguage(),

    toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
    setSidebarCollapsed: (sidebarCollapsed) => set({ sidebarCollapsed }),
    setShowSettings: (showSettings) => set({ showSettings }),
    setShowUpgrade: (showUpgrade) => set({ showUpgrade }),
    setShowPlusMenu: (showPlusMenu) =>
      set((state) => ({
        showPlusMenu: typeof showPlusMenu === 'function' ? showPlusMenu(state.showPlusMenu) : showPlusMenu,
      })),
    setShowAgentPanel: (showAgentPanel) => set({ showAgentPanel }),
    setShowAnalyticsPanel: (showAnalyticsPanel) => set({ showAnalyticsPanel }),
    setShowOnboarding: (showOnboarding) => set({ showOnboarding }),
    setShowDirectoryModal: (showDirectoryModal) => set({ showDirectoryModal }),
    setShowPromptSuggestions: (showPromptSuggestions) => set({ showPromptSuggestions }),
    setShowArtifacts: (showArtifacts) => set({ showArtifacts }),
    setShowMcpPanel: (showMcpPanel) => set({ showMcpPanel }),
    setShowSkillsSubmenu: (showSkillsSubmenu) =>
      set((state) => ({
        showSkillsSubmenu: typeof showSkillsSubmenu === 'function' ? showSkillsSubmenu(state.showSkillsSubmenu) : showSkillsSubmenu,
      })),
    setShowProjectsSubmenu: (showProjectsSubmenu) =>
      set((state) => ({
        showProjectsSubmenu: typeof showProjectsSubmenu === 'function' ? showProjectsSubmenu(state.showProjectsSubmenu) : showProjectsSubmenu,
      })),
    setShowGithubModal: (showGithubModal) => set({ showGithubModal }),
    setShowCompactDialog: (showCompactDialog) => set({ showCompactDialog }),
    setShowSlashPalette: (showSlashPalette) => set({ showSlashPalette }),
    setShowTerminalPanel: (showTerminalPanel) => set({ showTerminalPanel }),
    setSlashPaletteInput: (slashPaletteInput) => set({ slashPaletteInput }),
    setInputHeight: (inputHeight) => set({ inputHeight }),
    setIsDragging: (isDragging) => set({ isDragging }),
    setDocumentPanelWidth: (documentPanelWidth) => set({ documentPanelWidth }),
    setTerminalPanelHeight: (terminalPanelHeight) => set({ terminalPanelHeight }),
    setZoomLevel: (zoomLevel) => set({ zoomLevel }),
    setLanguage: (language) => {
      set({ language });
      localStorage.setItem('app_language', language);
      document.documentElement.lang = language;
    },
  }))
);

// ── Selector hooks — subscribe to individual fields ──

export function useSidebarCollapsed() { return useUIStore(s => s.sidebarCollapsed); }
export function useShowSettings() { return useUIStore(s => s.showSettings); }
export function useShowAgentPanel() { return useUIStore(s => s.showAgentPanel); }
export function useShowMcpPanel() { return useUIStore(s => s.showMcpPanel); }
export function useShowTerminalPanel() { return useUIStore(s => s.showTerminalPanel); }
export function useShowSlashPalette() { return useUIStore(s => s.showSlashPalette); }
export function useSlashPaletteInput() { return useUIStore(s => s.slashPaletteInput); }
export function useZoomLevel() { return useUIStore(s => s.zoomLevel); }
export function useDocumentPanelWidth() { return useUIStore(s => s.documentPanelWidth); }
export function useTerminalPanelHeight() { return useUIStore(s => s.terminalPanelHeight); }
