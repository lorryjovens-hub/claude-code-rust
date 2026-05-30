import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

interface EnabledSkill {
  id: string;
  name: string;
  description?: string;
}

interface SelectedSkill {
  name: string;
  slug: string;
  description?: string;
}

export interface Project {
  id: string;
  name: string;
  path?: string;
  description?: string;
  instructions?: string;
  workspace_path?: string;
  is_archived?: number | boolean;
  file_count?: number;
  chat_count?: number;
  created_at?: string;
  updated_at?: string;
}

export interface ContextInfo {
  files?: string[];
  tokens: number;
  limit?: number;
  summary?: string;
}

export interface TokenUsage {
  total?: number;
  used?: number;
  remaining?: number;
  input_tokens?: number;
  output_tokens?: number;
  reset_at?: string;
}

interface ProjectState {
  projectList: Project[];
  currentProjectId: string | null;
  pendingProjectId: string | null;
  enabledSkills: EnabledSkill[];
  selectedSkill: SelectedSkill | null;
  contextInfo: ContextInfo | null;
  tokenUsage: TokenUsage | null;

  setProjectList: (list: Project[] | ((prev: Project[]) => Project[])) => void;
  setCurrentProjectId: (id: string | null) => void;
  setPendingProjectId: (id: string | null) => void;
  setEnabledSkills: (skills: EnabledSkill[]) => void;
  setSelectedSkill: (skill: SelectedSkill | null) => void;
  setContextInfo: (info: ContextInfo | null) => void;
  setTokenUsage: (usage: TokenUsage | null | ((prev: TokenUsage | null) => TokenUsage | null)) => void;
  resetProject: () => void;
}

const initialState = {
  projectList: [] as Project[],
  currentProjectId: null as string | null,
  pendingProjectId: null as string | null,
  enabledSkills: [] as EnabledSkill[],
  selectedSkill: null as SelectedSkill | null,
  contextInfo: null as ContextInfo | null,
  tokenUsage: null as TokenUsage | null,
};

export const useProjectStore = create<ProjectState>()(
  subscribeWithSelector((set) => ({
    ...initialState,

    setProjectList: (projectList) =>
      set((state) => ({
        projectList: typeof projectList === 'function' ? projectList(state.projectList) : projectList,
      })),
    setCurrentProjectId: (currentProjectId) => set({ currentProjectId }),
    setPendingProjectId: (pendingProjectId) => set({ pendingProjectId }),
    setEnabledSkills: (enabledSkills) => set({ enabledSkills }),
    setSelectedSkill: (selectedSkill) => set({ selectedSkill }),
    setContextInfo: (contextInfo) => set({ contextInfo }),
    setTokenUsage: (tokenUsage) =>
      set((state) => ({
        tokenUsage: typeof tokenUsage === 'function' ? tokenUsage(state.tokenUsage) : tokenUsage,
      })),
    resetProject: () => set(initialState),
  }))
);
