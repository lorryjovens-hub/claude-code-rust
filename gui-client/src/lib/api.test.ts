import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { chatApi, taskApi, modelApi, settingsApi, systemApi } from '@/lib/api';

// Mock the invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => {
    // Return a function to unlisten
    return vi.fn();
  }),
}));

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

describe('API Integration Tests', () => {
  beforeEach(() => {
    mockInvoke.mockClear();
    mockListen.mockClear();
  });

  describe('Chat API', () => {
    it('should send a message', async () => {
      const mockResponse = {
        success: true,
        data: {
          id: '1',
          role: 'assistant',
          content: 'Hello, how can I help you?',
          timestamp: Date.now(),
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await chatApi.sendMessage('Hello', 'conv-1', 'claude-3-5-sonnet');

      expect(mockInvoke).toHaveBeenCalledWith('send_chat_message', {
        message: 'Hello',
        conversationId: 'conv-1',
        model: 'claude-3-5-sonnet',
      });
      expect(result).toEqual(mockResponse.data);
    });

    it('should stream a message', async () => {
      const mockResponse = { success: true, data: null };
      mockInvoke.mockResolvedValue(mockResponse);

      const onChunk = vi.fn();
      await chatApi.streamMessage('Hello', onChunk, 'conv-1', 'claude-3-5-sonnet');

      expect(mockInvoke).toHaveBeenCalledWith('stream_chat_message', {
        message: 'Hello',
        conversationId: 'conv-1',
        model: 'claude-3-5-sonnet',
      });
      expect(mockListen).toHaveBeenCalledWith('stream_chunk', expect.any(Function));
    });

    it('should get conversations', async () => {
      const mockResponse = {
        success: true,
        data: [
          {
            id: 'conv-1',
            title: 'Test Conversation',
            messages: [],
            model: 'claude-3-5-sonnet',
            created_at: Date.now(),
            updated_at: Date.now(),
          },
        ],
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await chatApi.getConversations();

      expect(mockInvoke).toHaveBeenCalledWith('get_conversations', undefined);
      expect(result).toEqual(mockResponse.data);
    });

    it('should get a conversation', async () => {
      const mockResponse = {
        success: true,
        data: {
          id: 'conv-1',
          title: 'Test Conversation',
          messages: [],
          model: 'claude-3-5-sonnet',
          created_at: Date.now(),
          updated_at: Date.now(),
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await chatApi.getConversation('conv-1');

      expect(mockInvoke).toHaveBeenCalledWith('get_conversation', { id: 'conv-1' });
      expect(result).toEqual(mockResponse.data);
    });

    it('should create a conversation', async () => {
      const mockResponse = {
        success: true,
        data: {
          id: 'conv-1',
          title: 'New Conversation',
          messages: [],
          model: 'claude-3-5-sonnet',
          created_at: Date.now(),
          updated_at: Date.now(),
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await chatApi.createConversation('New Conversation', 'claude-3-5-sonnet');

      expect(mockInvoke).toHaveBeenCalledWith('create_conversation', {
        title: 'New Conversation',
        model: 'claude-3-5-sonnet',
      });
      expect(result).toEqual(mockResponse.data);
    });

    it('should delete a conversation', async () => {
      const mockResponse = { success: true, data: null };
      mockInvoke.mockResolvedValue(mockResponse);

      await chatApi.deleteConversation('conv-1');

      expect(mockInvoke).toHaveBeenCalledWith('delete_conversation', { id: 'conv-1' });
    });

    it('should clear a conversation', async () => {
      const mockResponse = { success: true, data: null };
      mockInvoke.mockResolvedValue(mockResponse);

      await chatApi.clearConversation('conv-1');

      expect(mockInvoke).toHaveBeenCalledWith('clear_conversation', { id: 'conv-1' });
    });
  });

  describe('Task API', () => {
    it('should get tasks', async () => {
      const mockResponse = {
        success: true,
        data: [
          {
            id: 'task-1',
            title: 'Test Task',
            description: 'Test description',
            status: 'pending',
            priority: 'high',
            created_at: Date.now(),
            updated_at: Date.now(),
            progress: 0,
            subtasks: [],
          },
        ],
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await taskApi.getTasks();

      expect(mockInvoke).toHaveBeenCalledWith('get_tasks', undefined);
      expect(result).toEqual(mockResponse.data);
    });

    it('should create a task', async () => {
      const mockResponse = {
        success: true,
        data: {
          id: 'task-1',
          title: 'New Task',
          description: 'New task description',
          status: 'pending',
          priority: 'high',
          created_at: Date.now(),
          updated_at: Date.now(),
          progress: 0,
          subtasks: [],
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await taskApi.createTask('New Task', 'New task description', 'high');

      expect(mockInvoke).toHaveBeenCalledWith('create_task', {
        title: 'New Task',
        description: 'New task description',
        priority: 'high',
      });
      expect(result).toEqual(mockResponse.data);
    });

    it('should update a task', async () => {
      const mockResponse = {
        success: true,
        data: {
          id: 'task-1',
          title: 'Updated Task',
          description: 'Updated description',
          status: 'in_progress',
          priority: 'medium',
          created_at: Date.now() - 1000,
          updated_at: Date.now(),
          progress: 50,
          subtasks: [],
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await taskApi.updateTask('task-1', {
        title: 'Updated Task',
        status: 'in_progress',
        progress: 50,
      });

      expect(mockInvoke).toHaveBeenCalledWith('update_task', {
        id: 'task-1',
        title: 'Updated Task',
        status: 'in_progress',
        progress: 50,
      });
      expect(result).toEqual(mockResponse.data);
    });

    it('should delete a task', async () => {
      const mockResponse = { success: true, data: null };
      mockInvoke.mockResolvedValue(mockResponse);

      await taskApi.deleteTask('task-1');

      expect(mockInvoke).toHaveBeenCalledWith('delete_task', { id: 'task-1' });
    });

    it('should generate subtasks', async () => {
      const mockResponse = {
        success: true,
        data: ['Subtask 1', 'Subtask 2', 'Subtask 3'],
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await taskApi.generateSubtasks('task-1', 'Create a website');

      expect(mockInvoke).toHaveBeenCalledWith('generate_subtasks', {
        taskId: 'task-1',
        description: 'Create a website',
      });
      expect(result).toEqual(mockResponse.data);
    });
  });

  describe('Model API', () => {
    it('should get model providers', async () => {
      const mockResponse = {
        success: true,
        data: [
          {
            id: 'anthropic',
            name: 'Anthropic',
            models: [],
            is_enabled: true,
          },
        ],
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await modelApi.getProviders();

      expect(mockInvoke).toHaveBeenCalledWith('get_model_providers', undefined);
      expect(result).toEqual(mockResponse.data);
    });

    it('should get models', async () => {
      const mockResponse = {
        success: true,
        data: [
          {
            id: 'claude-3-5-sonnet',
            name: 'Claude 3.5 Sonnet',
            provider: 'anthropic',
            description: 'Most intelligent model',
            capabilities: ['coding', 'analysis'],
            max_tokens: 8192,
            context_window: 200000,
            pricing: { input: 3.0, output: 15.0 },
            is_available: true,
            is_default: true,
          },
        ],
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await modelApi.getModels('anthropic');

      expect(mockInvoke).toHaveBeenCalledWith('get_models', { providerId: 'anthropic' });
      expect(result).toEqual(mockResponse.data);
    });

    it('should set default model', async () => {
      const mockResponse = { success: true, data: null };
      mockInvoke.mockResolvedValue(mockResponse);

      await modelApi.setDefaultModel('claude-3-5-sonnet');

      expect(mockInvoke).toHaveBeenCalledWith('set_default_model', { modelId: 'claude-3-5-sonnet' });
    });

    it('should test model', async () => {
      const mockResponse = {
        success: true,
        data: { latency: 120, success: true },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await modelApi.testModel('claude-3-5-sonnet');

      expect(mockInvoke).toHaveBeenCalledWith('test_model', { modelId: 'claude-3-5-sonnet' });
      expect(result).toEqual(mockResponse.data);
    });

    it('should update provider config', async () => {
      const mockResponse = { success: true, data: null };
      mockInvoke.mockResolvedValue(mockResponse);

      await modelApi.updateProviderConfig('anthropic', {
        apiKey: 'test-api-key',
        baseUrl: 'https://api.anthropic.com',
      });

      expect(mockInvoke).toHaveBeenCalledWith('update_provider_config', {
        providerId: 'anthropic',
        api_key: 'test-api-key',
        base_url: 'https://api.anthropic.com',
      });
    });
  });

  describe('Settings API', () => {
    it('should get settings', async () => {
      const mockResponse = {
        success: true,
        data: {
          theme: 'system',
          language: 'zh-CN',
          default_model: 'claude-3-5-sonnet',
          api_keys: {},
          shortcuts: {},
          auto_save: true,
          stream_response: true,
          show_thinking: false,
          max_context_messages: 20,
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await settingsApi.get();

      expect(mockInvoke).toHaveBeenCalledWith('get_settings', undefined);
      expect(result).toEqual(mockResponse.data);
    });

    it('should update settings', async () => {
      const mockResponse = {
        success: true,
        data: {
          theme: 'dark',
          language: 'en-US',
          default_model: 'claude-3-5-sonnet',
          api_keys: {},
          shortcuts: {},
          auto_save: true,
          stream_response: true,
          show_thinking: false,
          max_context_messages: 20,
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await settingsApi.update({
        theme: 'dark',
        language: 'en-US',
      });

      expect(mockInvoke).toHaveBeenCalledWith('update_settings', {
        theme: 'dark',
        language: 'en-US',
        default_model: undefined,
        auto_save: undefined,
        stream_response: undefined,
        show_thinking: undefined,
        max_context_messages: undefined,
      });
      expect(result).toEqual(mockResponse.data);
    });

    it('should reset settings', async () => {
      const mockResponse = {
        success: true,
        data: {
          theme: 'system',
          language: 'zh-CN',
          default_model: 'claude-3-5-sonnet',
          api_keys: {},
          shortcuts: {},
          auto_save: true,
          stream_response: true,
          show_thinking: false,
          max_context_messages: 20,
        },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await settingsApi.reset();

      expect(mockInvoke).toHaveBeenCalledWith('reset_settings', undefined);
      expect(result).toEqual(mockResponse.data);
    });
  });

  describe('System API', () => {
    it('should get health status', async () => {
      const mockResponse = {
        success: true,
        data: { status: 'ok', version: '1.0.0' },
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await systemApi.getHealth();

      expect(mockInvoke).toHaveBeenCalledWith('get_health', undefined);
      expect(result).toEqual(mockResponse.data);
    });

    it('should open external URL', async () => {
      const mockResponse = { success: true, data: null };
      mockInvoke.mockResolvedValue(mockResponse);

      await systemApi.openExternal('https://example.com');

      expect(mockInvoke).toHaveBeenCalledWith('open_external', { url: 'https://example.com' });
    });
  });

  describe('Error Handling', () => {
    it('should handle API errors', async () => {
      const mockResponse = {
        success: false,
        error: 'API Error',
      };

      mockInvoke.mockResolvedValue(mockResponse);

      await expect(chatApi.sendMessage('Hello')).rejects.toThrow('API Error');
    });
  });
});
