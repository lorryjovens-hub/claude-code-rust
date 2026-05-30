import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export interface ActiveTask {
  id?: string;
  name?: string;
  status?: string;
  description?: string;
  last_tool_name?: string;
  started_at?: string;
  [key: string]: unknown;
}

export interface PendingFile {
  id: string;
  name?: string;
  file: File;
  fileId?: string;
  fileName: string;
  fileType?: 'text' | 'image' | 'audio' | 'video' | 'document';
  mimeType: string;
  path?: string;
  content?: string;
  size: number;
  type?: string;
  progress: number;
  status: 'uploading' | 'done' | 'error';
  error?: string;
  source?: 'github';
  ghRepo?: string;
  ghRef?: string;
  lineCount?: number;
  previewUrl?: string;
}

interface AskUserQuestion {
  question: string;
  header?: string;
  options?: Array<{ label: string; description?: string }>;
  multiSelect?: boolean;
}

interface AskUserDialogData {
  request_id: string;
  tool_use_id: string;
  questions: AskUserQuestion[];
  answers: Record<string, string>;
}

interface ToolPermissionDialogData {
  request_id: string;
  tool_use_id: string;
  tool_name: string;
  input: Record<string, unknown>;
}

interface ToolState {
  activeTasks: Map<string, ActiveTask>;
  toolPermissionDialog: ToolPermissionDialogData | null;
  askUserDialog: AskUserDialogData | null;
  expandedMessages: Set<number>;
  copiedMessageIdx: number | null;
  editingMessageIdx: number | null;
  editingContent: string;
  pendingFiles: PendingFile[];

  addTask: (id: string, task: ActiveTask) => void;
  removeTask: (id: string) => void;
  setActiveTasks: (tasks: Map<string, ActiveTask> | ((prev: Map<string, ActiveTask>) => Map<string, ActiveTask>)) => void;
  setToolPermissionDialog: (dialog: ToolPermissionDialogData | null) => void;
  setAskUserDialog: (dialog: AskUserDialogData | null | ((prev: AskUserDialogData | null) => AskUserDialogData | null)) => void;
  toggleExpandedMessage: (idx: number) => void;
  setCopiedMessageIdx: (idx: number | null) => void;
  setEditingMessageIdx: (idx: number | null) => void;
  setEditingContent: (content: string) => void;
  setPendingFiles: (files: PendingFile[] | ((prev: PendingFile[]) => PendingFile[])) => void;
  resetTool: () => void;
}

const getInitialState = () => ({
  activeTasks: new Map<string, ActiveTask>(),
  toolPermissionDialog: null as ToolPermissionDialogData | null,
  askUserDialog: null as AskUserDialogData | null,
  expandedMessages: new Set<number>(),
  copiedMessageIdx: null as number | null,
  editingMessageIdx: null as number | null,
  editingContent: '',
  pendingFiles: [] as PendingFile[],
});

export const useToolStore = create<ToolState>()(
  subscribeWithSelector((set) => ({
    ...getInitialState(),

    addTask: (id, task) =>
      set((state) => {
        if (state.activeTasks.has(id) && state.activeTasks.get(id) === task) return state;
        const next = new Map(state.activeTasks);
        next.set(id, task);
        return { activeTasks: next };
      }),

    removeTask: (id) =>
      set((state) => {
        if (!state.activeTasks.has(id)) return state;
        const next = new Map(state.activeTasks);
        next.delete(id);
        return { activeTasks: next };
      }),

    setActiveTasks: (activeTasks) =>
      set((state) => ({
        activeTasks: typeof activeTasks === 'function' ? activeTasks(state.activeTasks) : activeTasks,
      })),

    setToolPermissionDialog: (toolPermissionDialog) => set({ toolPermissionDialog }),

    setAskUserDialog: (askUserDialog) =>
      set((state) => ({
        askUserDialog: typeof askUserDialog === 'function' ? askUserDialog(state.askUserDialog) : askUserDialog,
      })),

    toggleExpandedMessage: (idx) =>
      set((state) => {
        const next = new Set(state.expandedMessages);
        if (next.has(idx)) {
          next.delete(idx);
        } else {
          next.add(idx);
        }
        return { expandedMessages: next };
      }),

    setCopiedMessageIdx: (copiedMessageIdx) => set({ copiedMessageIdx }),
    setEditingMessageIdx: (editingMessageIdx) => set({ editingMessageIdx }),
    setEditingContent: (editingContent) => set({ editingContent }),

    setPendingFiles: (pendingFiles) =>
      set((state) => ({
        pendingFiles: typeof pendingFiles === 'function' ? pendingFiles(state.pendingFiles) : pendingFiles,
      })),

    resetTool: () => set(getInitialState()),
  }))
);
