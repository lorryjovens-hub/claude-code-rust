import { useChatStore } from '../../stores/useChatStore';
import { useStreamingStore } from '../../stores/useStreamingStore';
import { useProjectStore } from '../../stores/useProjectStore';
import {
  compactConversation,
  deleteMessagesFrom,
  branchConversation,
  getUserUsage,
  getSkills,
} from '../../api';

type SlashCommand = (activeId: string | null, args: string) => boolean | Promise<boolean>;

function sysMsg(content: string) {
  const { setMessages } = useChatStore.getState();
  setMessages((prev) => [
    ...prev,
    {
      id: `sys-${Date.now()}`,
      role: 'assistant' as const,
      content,
      created_at: new Date().toISOString(),
      type: 'system' as const,
    },
  ]);
}

function clearInput() {
  useChatStore.getState().setInputText('');
}

const cmdHelp: SlashCommand = () => {
  sysMsg(
    `**Available Commands:**\n\n` +
      `/help — Show this help message\n` +
      `/model [name] — Show or switch the current model\n` +
      `/cost — Show token usage for this session\n` +
      `/compact — Compact the conversation to free up context\n` +
      `/clear — Clear the conversation history\n` +
      `/rewind — Rewind to a previous message\n` +
      `/branch — Branch the conversation from a message\n` +
      `/search <query> — Search the web for information\n` +
      `/export — Export current conversation as Markdown`,
  );
  clearInput();
  return true;
};

const cmdModel: SlashCommand = (_activeId, args) => {
  const { setCurrentModel, currentModel } = useChatStore.getState();
  if (args) {
    setCurrentModel(args);
    sysMsg(`Model switched to **${args}**`);
  } else {
    sysMsg(`Current model: **${currentModel || 'default'}**`);
  }
  clearInput();
  return true;
};

const cmdCost: SlashCommand = () => {
  const { tokenUsage } = useProjectStore.getState();
  if (tokenUsage) {
    const total = tokenUsage.input_tokens! + tokenUsage.output_tokens!;
    sysMsg(
      `**Token Usage (this session):**\n\n` +
        `Input: ${tokenUsage.input_tokens!.toLocaleString()} tokens\n` +
        `Output: ${tokenUsage.output_tokens!.toLocaleString()} tokens\n` +
        `Total: ${total.toLocaleString()} tokens`,
    );
  } else {
    sysMsg('No token usage data available for this session yet.');
  }
  clearInput();
  return true;
};

const cmdCompact: SlashCommand = (activeId) => {
  if (activeId) {
    compactConversation(activeId)
      .then((result) => {
        sysMsg(
          `**Conversation compacted.**\n\nSaved ${result.tokensSaved?.toLocaleString() || 0} tokens (${result.messagesCompacted || 0} messages compacted).`,
        );
      })
      .catch((err) => {
        sysMsg(`Compact failed: ${err}`);
      });
  }
  clearInput();
  return true;
};

const cmdClear: SlashCommand = (activeId) => {
  const { setMessages } = useChatStore.getState();
  const { setTokenUsage, setContextInfo } = useProjectStore.getState();
  if (activeId) {
    setMessages([]);
    setTokenUsage(null);
    setContextInfo(null);
  }
  clearInput();
  return true;
};

const cmdRewind: SlashCommand = (activeId) => {
  const { messages, setMessages } = useChatStore.getState();
  if (messages.length > 0 && activeId) {
    const lastUserMsgIdx = messages
      .map((m, i) => (m.role === 'user' ? i : -1))
      .filter((i) => i >= 0)
      .pop();
    if (lastUserMsgIdx !== undefined && lastUserMsgIdx >= 0) {
      const msg = messages[lastUserMsgIdx];
      deleteMessagesFrom(activeId, msg.id!)
        .then(() => {
          setMessages((prev) => prev.slice(0, lastUserMsgIdx));
        })
        .catch((err) => console.error('Rewind failed:', err));
    }
  }
  clearInput();
  return true;
};

const cmdBranch: SlashCommand = (activeId) => {
  const { messages } = useChatStore.getState();
  if (messages.length > 0 && activeId) {
    const lastUserMsgIdx = messages
      .map((m, i) => (m.role === 'user' ? i : -1))
      .filter((i) => i >= 0)
      .pop();
    if (lastUserMsgIdx !== undefined && lastUserMsgIdx >= 0) {
      const msg = messages[lastUserMsgIdx];
      branchConversation(activeId, msg.id)
        .then((result) => {
          if (result.success && result.new_conversation_id) {
            window.dispatchEvent(
              new CustomEvent('conversationCreated', {
                detail: { id: result.new_conversation_id },
              }),
            );
            sysMsg(
              `Branched conversation. New conversation ID: ${result.new_conversation_id}`,
            );
          }
        })
        .catch((err) => console.error('Branch failed:', err));
    }
  }
  clearInput();
  return true;
};

const cmdStats: SlashCommand = () => {
  getUserUsage()
    .then((usage) => {
      sysMsg(
        `**Usage Statistics:**\n\n` +
          `Total Messages: ${usage.total_messages || 0}\n` +
          `Total Conversations: ${usage.total_conversations || 0}\n` +
          `Total Tokens: ${usage.total_tokens?.toLocaleString() || 0}\n` +
          `Total API Calls: ${usage.total_api_calls || 0}`,
      );
    })
    .catch(() => {
      sysMsg('Unable to fetch usage statistics.');
    });
  clearInput();
  return true;
};

const cmdUsage: SlashCommand = () => {
  const { tokenUsage } = useProjectStore.getState();
  const { messages, currentModel } = useChatStore.getState();
  if (tokenUsage) {
    const total = tokenUsage.input_tokens! + tokenUsage.output_tokens!;
    const ratio = tokenUsage.output_tokens! / Math.max(1, tokenUsage.input_tokens!);
    sysMsg(
      `**Detailed Token Usage:**\n\n` +
        `Input Tokens: ${tokenUsage.input_tokens!.toLocaleString()}\n` +
        `Output Tokens: ${tokenUsage.output_tokens!.toLocaleString()}\n` +
        `Total Tokens: ${total.toLocaleString()}\n` +
        `Output/Input Ratio: ${ratio.toFixed(2)}\n` +
        `Messages Sent: ${messages.filter((m) => m.role === 'user').length}\n` +
        `Model: ${currentModel || 'default'}`,
    );
  } else {
    sysMsg('No detailed usage data available for this session.');
  }
  clearInput();
  return true;
};

const cmdTheme: SlashCommand = () => {
  const currentTheme = localStorage.getItem('theme') || 'system';
  const themes = ['light', 'dark', 'system'];
  const nextTheme = themes[(themes.indexOf(currentTheme) + 1) % themes.length];
  localStorage.setItem('theme', nextTheme);
  if (nextTheme === 'dark') {
    document.documentElement.classList.add('dark');
  } else if (nextTheme === 'light') {
    document.documentElement.classList.remove('dark');
  } else {
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }
  sysMsg(`Theme switched to **${nextTheme}**`);
  clearInput();
  return true;
};

const cmdConfig: SlashCommand = () => {
  const { currentModel, researchMode, planMode } = useChatStore.getState();
  sysMsg(
    `**Current Configuration:**\n\n` +
      `Model: ${currentModel || 'default'}\n` +
      `Theme: ${localStorage.getItem('theme') || 'system'}\n` +
      `User Mode: ${localStorage.getItem('user_mode') || 'selfhosted'}\n` +
      `Research Mode: ${researchMode ? 'On' : 'Off'}\n` +
      `Plan Mode: ${planMode ? 'On' : 'Off'}`,
  );
  clearInput();
  return true;
};

const cmdSkills: SlashCommand = () => {
  getSkills()
    .then((skills) => {
      const skillList = skills
        .map((s: any) => `**${s.name}** — ${s.description || 'No description'}`)
        .join('\n');
      sysMsg(`**Available Skills:**\n\n${skillList || 'No skills available'}`);
    })
    .catch(() => {
      sysMsg('Unable to fetch skills list.');
    });
  clearInput();
  return true;
};

const cmdTasks: SlashCommand = (activeId) => {
  const { messages, currentModel, researchMode } = useChatStore.getState();
  const { isStreaming } = useStreamingStore.getState();
  sysMsg(
    `**Active Tasks & Agents:**\n\n` +
      `Messages in conversation: ${messages.length}\n` +
      `Current model: ${currentModel || 'default'}\n` +
      `Streaming active: ${isStreaming(activeId || '') ? 'Yes' : 'No'}\n` +
      `Research mode: ${researchMode ? 'Active' : 'Inactive'}`,
  );
  clearInput();
  return true;
};

const cmdDoctor: SlashCommand = (activeId) => {
  const { isStreaming } = useStreamingStore.getState();
  sysMsg(
    `**System Diagnostics:**\n\n` +
      `Platform: ${navigator.platform}\n` +
      `User Agent: ${navigator.userAgent}\n` +
      `API Key: ${localStorage.getItem('ANTHROPIC_API_KEY') ? 'Set' : 'Not set'}\n` +
      `User Mode: ${localStorage.getItem('user_mode') || 'selfhosted'}\n` +
      `Theme: ${localStorage.getItem('theme') || 'system'}\n` +
      `Stream Status: ${isStreaming(activeId || '') ? 'Connected' : 'Disconnected'}`,
  );
  clearInput();
  return true;
};

const cmdResume: SlashCommand = (activeId) => {
  const lastConv = localStorage.getItem('last_conversation_id');
  if (lastConv && lastConv !== activeId) {
    window.dispatchEvent(
      new CustomEvent('navigateToConversation', { detail: { id: lastConv } }),
    );
  } else {
    sysMsg('No previous conversation to resume.');
  }
  clearInput();
  return true;
};

const cmdPlan: SlashCommand = () => {
  const { planMode, setPlanMode } = useChatStore.getState();
  setPlanMode(!planMode);
  sysMsg(`Plan mode ${!planMode ? 'enabled' : 'disabled'}`);
  clearInput();
  return true;
};

const cmdMcp: SlashCommand = () => {
  sysMsg(
    `**MCP Server Management:**\n\n` +
      `Open the MCP management panel to configure Model Context Protocol servers.\n\n` +
      `Use the sidebar MCP button or click the MCP icon in the toolbar.`,
  );
  clearInput();
  return true;
};

const cmdSearch: SlashCommand = (_activeId, args) => {
  const query = args.trim();
  if (!query) {
    sysMsg('Usage: `/search <query>` — Search the web for information.\n\nExample: `/search latest Claude AI news 2026`');
    clearInput();
    return true;
  }
  // Insert a user message that will trigger the WebSearch tool
  const { setMessages, setInputText } = useChatStore.getState();
  setMessages((prev) => [
    ...prev,
    {
      id: `search-${Date.now()}`,
      role: 'user' as const,
      content: `Search the web for: ${query}\n\nPlease use the WebSearch tool to find up-to-date information about this topic, then summarize the findings with citations.`,
      created_at: new Date().toISOString(),
    },
  ]);
  clearInput();
  return true;
};

const cmdExport: SlashCommand = async (activeId) => {
  if (!activeId) { sysMsg('没有活动的对话可以导出。'); clearInput(); return true; }
  try {
    const { getConversation } = await import('../../api');
    const data = await getConversation(activeId);
    const msgs = data?.messages || [];
    let md = `# ${data?.title || '对话导出'}\n\n*导出时间: ${new Date().toLocaleString()}*\n\n`;
    for (const m of msgs) {
      const role = m.role === 'user' ? '👤 用户' : '🤖 Claude';
      const time = m.created_at ? new Date(m.created_at).toLocaleString() : '';
      md += `## ${role} ${time ? `(${time})` : ''}\n\n${m.content || ''}\n\n`;
      if (m.toolCalls?.length) {
        md += `> 🔧 工具调用: ${m.toolCalls.map((t: any) => t.name).join(', ')}\n\n`;
      }
    }
    const blob = new Blob([md], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = `conversation-${activeId.slice(0, 8)}.md`;
    a.click(); URL.revokeObjectURL(url);
    sysMsg(`✅ 对话已导出为 Markdown 文件 (${(blob.size / 1024).toFixed(1)}KB)`);
  } catch (e) { sysMsg(`❌ 导出失败: ${e}`); }
  clearInput();
  return true;
};

const cmdWorktree: SlashCommand = () => {
  sysMsg(
    `**Git Worktree Management:**\n\n` +
      `Create isolated worktrees for parallel development.\n\n` +
      `Use the Agent Worktree panel in the sidebar to manage worktrees.`,
  );
  clearInput();
  return true;
};

export const SLASH_COMMANDS = [
  { name: '/help', description: 'Show available commands' },
  { name: '/model', description: 'Show or switch model' },
  { name: '/cost', description: 'Show token usage' },
  { name: '/compact', description: 'Compact conversation' },
  { name: '/clear', description: 'Clear conversation' },
  { name: '/rewind', description: 'Rewind to previous message' },
  { name: '/branch', description: 'Branch conversation' },
  { name: '/stats', description: 'Show usage statistics' },
  { name: '/usage', description: 'Show detailed token usage' },
  { name: '/theme', description: 'Toggle theme' },
  { name: '/config', description: 'Show configuration' },
  { name: '/skills', description: 'List available skills' },
  { name: '/tasks', description: 'Show active tasks' },
  { name: '/doctor', description: 'System diagnostics' },
  { name: '/resume', description: 'Resume last conversation' },
  { name: '/plan', description: 'Toggle plan mode' },
  { name: '/mcp', description: 'MCP server management' },
  { name: '/worktree', description: 'Git worktree management' },
  { name: '/search', description: 'Search the web' },
  { name: '/export', description: 'Export conversation as Markdown' },
];

const commandMap: Record<string, SlashCommand> = {
  '/help': cmdHelp,
  '/model': cmdModel,
  '/cost': cmdCost,
  '/compact': cmdCompact,
  '/clear': cmdClear,
  '/rewind': cmdRewind,
  '/branch': cmdBranch,
  '/stats': cmdStats,
  '/usage': cmdUsage,
  '/theme': cmdTheme,
  '/config': cmdConfig,
  '/skills': cmdSkills,
  '/tasks': cmdTasks,
  '/doctor': cmdDoctor,
  '/resume': cmdResume,
  '/plan': cmdPlan,
  '/mcp': cmdMcp,
  '/worktree': cmdWorktree,
  '/search': cmdSearch,
  '/export': cmdExport,
};

export async function handleSlashCommand(text: string, activeId: string | null): Promise<boolean> {
  const trimmed = text.trim();
  if (!trimmed.startsWith('/')) return false;
  const parts = trimmed.split(/\s+/);
  const cmd = parts[0].toLowerCase();
  const args = parts.slice(1).join(' ');

  const handler = commandMap[cmd];
  if (handler) {
    return await handler(activeId, args);
  }
  return false;
}
