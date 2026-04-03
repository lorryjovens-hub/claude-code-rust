// Chat Types
export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  model?: string;
  tokens?: {
    prompt: number;
    completion: number;
    total: number;
  };
  metadata?: {
    thinking?: string;
    toolCalls?: ToolCall[];
  };
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  result?: unknown;
}

export interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  model: string;
  created_at: number;
  updated_at: number;
  system_prompt?: string;
}

// Task Types
export interface Task {
  id: string;
  title: string;
  description: string;
  status: "pending" | "in_progress" | "completed" | "failed";
  priority: "low" | "medium" | "high";
  created_at: number;
  updated_at: number;
  completed_at?: number;
  progress: number;
  subtasks: SubTask[];
}

export interface SubTask {
  id: string;
  title: string;
  completed: boolean;
}

// Model Types
export interface Model {
  id: string;
  name: string;
  provider: string;
  description: string;
  capabilities: string[];
  max_tokens: number;
  context_window: number;
  pricing: {
    input: number;
    output: number;
  };
  is_available: boolean;
  is_default: boolean;
}

export interface ModelProvider {
  id: string;
  name: string;
  icon?: string;
  api_key?: string;
  base_url?: string;
  models: Model[];
  is_enabled: boolean;
}

// Settings Types
export interface AppSettings {
  theme: string;
  language: string;
  default_model: string;
  api_keys: Record<string, string>;
  shortcuts: Record<string, string>;
  auto_save: boolean;
  stream_response: boolean;
  show_thinking: boolean;
  max_context_messages: number;
}

// API Types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

export interface StreamChunk {
  id: string;
  content: string;
  isThinking?: boolean;
  isComplete?: boolean;
}

// UI Types
export type ViewType = "chat" | "tasks" | "models" | "settings" | "history";

export interface SidebarItem {
  id: ViewType;
  label: string;
  icon: string;
  badge?: number;
}
