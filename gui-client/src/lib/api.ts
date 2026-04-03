import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ============================================================
// Stream Types
// ============================================================

export interface StreamChunk {
  id: string;
  content: string;
  is_thinking: boolean;
  is_complete: boolean;
  model?: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

async function callApi<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const response = await invoke<ApiResponse<T>>(cmd, args);
  if (!response.success) {
    throw new Error(response.error || "Unknown error");
  }
  return response.data as T;
}

// ============================================================
// Types
// ============================================================

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

// ============================================================
// Chat API
// ============================================================

export const chatApi = {
  sendMessage: async (
    message: string,
    conversationId?: string,
    model?: string
  ): Promise<Message> => {
    return callApi("send_chat_message", { message, conversationId, model });
  },

  streamMessage: async (
    message: string,
    onChunk: (chunk: StreamChunk) => void,
    conversationId?: string,
    model?: string
  ): Promise<void> => {
    // Listen for stream chunks
    const unlisten = await listen<StreamChunk>("stream_chunk", (event) => {
      onChunk(event.payload);
    });

    try {
      await callApi("stream_chat_message", { message, conversationId, model });
    } finally {
      unlisten();
    }
  },

  getConversations: async (): Promise<Conversation[]> => {
    return callApi("get_conversations");
  },

  getConversation: async (id: string): Promise<Conversation | null> => {
    return callApi("get_conversation", { id });
  },

  createConversation: async (
    title?: string,
    model?: string
  ): Promise<Conversation> => {
    return callApi("create_conversation", { title, model });
  },

  deleteConversation: async (id: string): Promise<void> => {
    return callApi("delete_conversation", { id });
  },

  clearConversation: async (id: string): Promise<void> => {
    return callApi("clear_conversation", { id });
  },
};

// ============================================================
// Task API
// ============================================================

export const taskApi = {
  getTasks: async (): Promise<Task[]> => {
    return callApi("get_tasks");
  },

  createTask: async (
    title: string,
    description: string,
    priority: string
  ): Promise<Task> => {
    return callApi("create_task", { title, description, priority });
  },

  updateTask: async (
    id: string,
    updates: {
      title?: string;
      description?: string;
      status?: string;
      priority?: string;
      progress?: number;
      subtasks?: SubTask[];
    }
  ): Promise<Task> => {
    return callApi("update_task", { id, ...updates });
  },

  deleteTask: async (id: string): Promise<void> => {
    return callApi("delete_task", { id });
  },

  generateSubtasks: async (taskId: string, description: string): Promise<string[]> => {
    return callApi("generate_subtasks", { taskId, description });
  },
};

// ============================================================
// Model API
// ============================================================

export const modelApi = {
  getProviders: async (): Promise<ModelProvider[]> => {
    return callApi("get_model_providers");
  },

  getModels: async (providerId?: string): Promise<Model[]> => {
    return callApi("get_models", { providerId });
  },

  setDefaultModel: async (modelId: string): Promise<void> => {
    return callApi("set_default_model", { modelId });
  },

  testModel: async (modelId: string): Promise<{ latency: number; success: boolean }> => {
    return callApi("test_model", { modelId });
  },

  updateProviderConfig: async (
    providerId: string,
    config: { apiKey?: string; baseUrl?: string }
  ): Promise<void> => {
    return callApi("update_provider_config", {
      providerId,
      api_key: config.apiKey,
      base_url: config.baseUrl,
    });
  },
};

// ============================================================
// Settings API
// ============================================================

export const settingsApi = {
  get: async (): Promise<AppSettings> => {
    return callApi("get_settings");
  },

  update: async (settings: Partial<AppSettings>): Promise<AppSettings> => {
    return callApi("update_settings", {
      theme: settings.theme,
      language: settings.language,
      default_model: settings.default_model,
      auto_save: settings.auto_save,
      stream_response: settings.stream_response,
      show_thinking: settings.show_thinking,
      max_context_messages: settings.max_context_messages,
    });
  },

  reset: async (): Promise<AppSettings> => {
    return callApi("reset_settings");
  },
};

// ============================================================
// System API
// ============================================================

export const systemApi = {
  getHealth: async (): Promise<{ status: string; version: string }> => {
    return callApi("get_health");
  },

  openExternal: async (url: string): Promise<void> => {
    return callApi("open_external", { url });
  },
};
