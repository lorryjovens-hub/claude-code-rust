import type { SyncPayload, SyncResponse } from './syncService';

export function mockPushResponse(payload: SyncPayload): SyncResponse {
  const serverTimestamp = new Date().toISOString();

  return {
    providers: payload.providers.map(p => ({
      ...p,
      enabled: p.enabled !== false,
    })),
    conversations: payload.conversations,
    messagesPerConversation: payload.messagesPerConversation,
    deletedProviderIds: [],
    deletedConversationIds: [],
    serverTimestamp,
  };
}

export function mockPullResponse(
  _lastPullAt: string | null
): SyncResponse {
  const now = new Date().toISOString();
  const oneHourAgo = new Date(Date.now() - 3600000).toISOString();
  const twoHoursAgo = new Date(Date.now() - 7200000).toISOString();

  const convId = `mock-conv-${Date.now()}`;
  const msgId1 = crypto.randomUUID();
  const msgId2 = crypto.randomUUID();
  const msgId3 = crypto.randomUUID();

  return {
    providers: [
      {
        id: 'mock-provider-claude',
        name: 'Anthropic Claude',
        apiKey: 'sk-ant-mock-key-1234567890abcdef',
        baseUrl: 'https://api.anthropic.com/v1',
        format: 'anthropic',
        models: [
          { id: 'claude-sonnet-4-6', name: 'Claude Sonnet 4.6', enabled: true },
          { id: 'claude-opus-4', name: 'Claude Opus 4', enabled: true },
        ],
        enabled: true,
      },
      {
        id: 'mock-provider-openai',
        name: 'OpenAI',
        apiKey: 'sk-mock-openai-key-abcdef123456',
        baseUrl: 'https://api.openai.com/v1',
        format: 'openai',
        models: [
          { id: 'gpt-5', name: 'GPT-5', enabled: true },
          { id: 'gpt-5-mini', name: 'GPT-5 Mini', enabled: true },
        ],
        enabled: true,
      },
    ],
    conversations: [
      {
        id: convId,
        title: '[测试] 云端同步的历史对话',
        model: 'claude-sonnet-4-6',
        provider: 'mock-provider-claude',
        workspace_path: null,
        project_id: null,
        research_mode: false,
        pinned: false,
        archived: false,
        created_at: twoHoursAgo,
        updated_at: oneHourAgo,
        message_count: 3,
      },
    ],
    messagesPerConversation: {
      [convId]: [
        {
          id: msgId1,
          conversation_id: convId,
          role: 'user',
          content: '你好，这是一个来自云端同步的测试对话。',
          thinking: null,
          created_at: twoHoursAgo,
          is_compact_boundary: undefined,
          sort_order: 0,
          toolCalls: [],
        },
        {
          id: msgId2,
          conversation_id: convId,
          role: 'assistant',
          content:
            '你好！我收到了你的测试消息。这表明云端数据同步功能已经可以正常工作了。',
          thinking: null,
          created_at: new Date(Date.now() - 6500000).toISOString(),
          is_compact_boundary: undefined,
          sort_order: 1,
          toolCalls: [],
        },
        {
          id: msgId3,
          conversation_id: convId,
          role: 'user',
          content: '太好了，让我验证一下数据是否完整。',
          thinking: null,
          created_at: oneHourAgo,
          is_compact_boundary: undefined,
          sort_order: 2,
          toolCalls: [],
        },
      ],
    },
    deletedProviderIds: [],
    deletedConversationIds: [],
    serverTimestamp: now,
  };
}

export function mockErrorResponse(status: number, message: string): Response {
  return new Response(
    JSON.stringify({ error: message, status }),
    {
      status,
      statusText: message,
      headers: { 'Content-Type': 'application/json' },
    }
  );
}

export function mockAuthExpiredResponse(): Response {
  return mockErrorResponse(401, '认证已过期，请重新登录');
}

export function mockServerErrorResponse(): Response {
  return mockErrorResponse(500, '服务器内部错误，请稍后重试');
}

export function mockNetworkError(): never {
  throw new Error('Failed to fetch / NetworkError: 网络连接失败');
}

export type MockScenario =
  | 'success'
  | 'empty'
  | 'auth-expired'
  | 'server-error'
  | 'network-error'
  | 'conflict-newer-local'
  | 'conflict-newer-remote';

export function getMockPullResponse(scenario: MockScenario): SyncResponse {
  switch (scenario) {
    case 'empty':
      return {
        providers: [],
        conversations: [],
        messagesPerConversation: {},
        deletedProviderIds: [],
        deletedConversationIds: [],
        serverTimestamp: new Date().toISOString(),
      };

    case 'conflict-newer-local': {
      const older = new Date(Date.now() - 86400000).toISOString();
      const oldConvId = crypto.randomUUID();
      return {
        providers: [],
        conversations: [
          {
            id: oldConvId,
            title: '[冲突测试] 远端旧版本',
            model: 'claude-sonnet-4-6',
            provider: 'mock-provider-claude',
            workspace_path: null,
            project_id: null,
            research_mode: false,
            pinned: false,
            archived: false,
            created_at: older,
            updated_at: older,
            message_count: 1,
          },
        ],
        messagesPerConversation: {
          [oldConvId]: [
            {
              id: crypto.randomUUID(),
              conversation_id: oldConvId,
              role: 'user',
              content: '这是远端旧版本的消息，应该被本地覆盖。',
              thinking: null,
              created_at: older,
              sort_order: 0,
            },
          ],
        },
        deletedProviderIds: [],
        deletedConversationIds: [],
        serverTimestamp: new Date().toISOString(),
      };
    }

    case 'conflict-newer-remote': {
      const newer = new Date(Date.now() + 3600000).toISOString();
      const newConvId = crypto.randomUUID();
      return {
        providers: [],
        conversations: [
          {
            id: newConvId,
            title: '[冲突测试] 远端新版本',
            model: 'gpt-5',
            provider: 'mock-provider-openai',
            workspace_path: null,
            project_id: null,
            research_mode: false,
            pinned: false,
            archived: false,
            created_at: new Date(Date.now() - 3600000).toISOString(),
            updated_at: newer,
            message_count: 2,
          },
        ],
        messagesPerConversation: {
          [newConvId]: [
            {
              id: crypto.randomUUID(),
              conversation_id: newConvId,
              role: 'user',
              content: '这是远端新版本的消息，应该覆盖本地。',
              thinking: null,
              created_at: newer,
              sort_order: 0,
            },
          ],
        },
        deletedProviderIds: [],
        deletedConversationIds: [],
        serverTimestamp: new Date().toISOString(),
      };
    }

    default:
      return mockPullResponse(null);
  }
}