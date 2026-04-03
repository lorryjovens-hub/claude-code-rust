import { useState, useCallback, useEffect } from "react";
import { useConversations, useCreateConversation, useSendMessage, useDeleteConversation } from "@/hooks/useApi";
import { useStore } from "@/hooks/useStore";
import { MessageList } from "./MessageList";
import { ChatInput } from "./ChatInput";
import { Button } from "@/components/ui/button";
import { Plus, MessageSquare, Loader2 } from "lucide-react";
import { toast } from "sonner";
import type { Message, StreamChunk } from "@/lib/api";
import { chatApi } from "@/lib/api";

export function ChatPage() {
  const { currentConversation, setCurrentConversation, settings } = useStore();
  const [isGenerating, setIsGenerating] = useState(false);
  const [streamingContent, setStreamingContent] = useState("");
  const [streamId, setStreamId] = useState<string | null>(null);

  const { data: conversations = [], isLoading } = useConversations();
  const createConversation = useCreateConversation();
  const sendMessage = useSendMessage();
  const deleteConversation = useDeleteConversation();

  // Sync current conversation with store
  useEffect(() => {
    if (conversations.length > 0 && !currentConversation) {
      setCurrentConversation(conversations[0]);
    }
  }, [conversations, currentConversation, setCurrentConversation]);

  const handleSendMessage = useCallback(
    async (content: string) => {
      let convId = currentConversation?.id;

      // Create new conversation if none exists
      if (!convId) {
        try {
          const newConv = await createConversation.mutateAsync({
            title: content.slice(0, 50),
            model: settings.default_model,
          });
          convId = newConv.id;
          setCurrentConversation(newConv);
        } catch (error) {
          return;
        }
      }

      // Add user message optimistically
      const userMessage: Message = {
        id: `temp-${Date.now()}`,
        role: "user",
        content,
        timestamp: Date.now(),
      };

      // Start generating
      setIsGenerating(true);
      setStreamingContent("");
      setStreamId(null);

      try {
        if (settings.stream_response) {
          // Use streaming response
          await chatApi.streamMessage(
            content,
            (chunk: StreamChunk) => {
              setStreamId(chunk.id);
              setStreamingContent(chunk.content);
              if (chunk.is_complete) {
                setIsGenerating(false);
                // Refresh conversations to get the saved message
                // This would normally be handled by a query invalidation
              }
            },
            convId,
            settings.default_model
          );
        } else {
          // Use regular response
          const response = await sendMessage.mutateAsync({
            message: content,
            conversationId: convId,
            model: settings.default_model,
          });

          // Update with real message
          if (currentConversation) {
            setCurrentConversation({
              ...currentConversation,
              messages: [...currentConversation.messages, userMessage, response],
              updated_at: Date.now(),
            });
          }
          setIsGenerating(false);
        }
      } catch (error: any) {
        toast.error(`发送消息失败: ${error.message}`);
        setIsGenerating(false);
        setStreamingContent("");
      }
    },
    [currentConversation, settings.default_model, settings.stream_response, createConversation, sendMessage, setCurrentConversation]
  );

  const handleNewChat = useCallback(async () => {
    try {
      const newConv = await createConversation.mutateAsync({
        title: "新对话",
        model: settings.default_model,
      });
      setCurrentConversation(newConv);
      toast.success("新对话已创建");
    } catch (error) {
      // Error already handled
    }
  }, [createConversation, settings.default_model, setCurrentConversation]);

  const handleDeleteConversation = useCallback(
    async (id: string) => {
      try {
        await deleteConversation.mutateAsync(id);
        if (currentConversation?.id === id) {
          setCurrentConversation(conversations[0] || null);
        }
      } catch (error) {
        // Error already handled
      }
    },
    [deleteConversation, currentConversation, conversations, setCurrentConversation]
  );

  const handleStop = useCallback(() => {
    setIsGenerating(false);
    setStreamingContent("");
    setStreamId(null);
    toast.info("已停止生成");
  }, []);

  // Get display messages
  const displayMessages = currentConversation?.messages || [];

  // Add streaming content if generating
  if (isGenerating && streamingContent) {
    displayMessages.push({
      id: streamId || "streaming",
      role: "assistant",
      content: streamingContent,
      timestamp: Date.now(),
      model: settings.default_model,
    });
  }

  return (
    <div className="flex h-full">
      {/* Sidebar - Conversation List */}
      <div className="w-64 border-r bg-card flex flex-col">
        <div className="p-3 border-b">
          <Button
            className="w-full bg-claude-orange hover:bg-claude-orange-dark"
            onClick={handleNewChat}
            disabled={createConversation.isPending}
          >
            {createConversation.isPending ? (
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <Plus className="w-4 h-4 mr-2" />
            )}
            新对话
          </Button>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
            </div>
          ) : conversations.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground text-sm">
              暂无对话
            </div>
          ) : (
            conversations.map((conv) => (
              <div
                key={conv.id}
                className={`group flex items-center gap-2 p-2 rounded-lg cursor-pointer transition-colors ${
                  currentConversation?.id === conv.id
                    ? "bg-secondary"
                    : "hover:bg-muted"
                }`}
                onClick={() => setCurrentConversation(conv)}
              >
                <MessageSquare className="w-4 h-4 shrink-0" />
                <span className="flex-1 truncate text-sm">{conv.title}</span>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 opacity-0 group-hover:opacity-100"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDeleteConversation(conv.id);
                  }}
                  disabled={deleteConversation.isPending}
                >
                  <span className="text-xs text-destructive">×</span>
                </Button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        {/* Empty State */}
        {!currentConversation && displayMessages.length === 0 && (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center space-y-4">
              <div className="w-16 h-16 rounded-2xl bg-claude-orange flex items-center justify-center mx-auto">
                <span className="text-white font-bold text-2xl">CC</span>
              </div>
              <h2 className="text-2xl font-semibold">Claude Code</h2>
              <p className="text-muted-foreground max-w-md">
                开始一个新的对话，或者从左侧选择一个历史对话。
                你可以问我任何问题，我会尽力帮助你。
              </p>
              <Button
                className="bg-claude-orange hover:bg-claude-orange-dark"
                onClick={handleNewChat}
              >
                <Plus className="w-4 h-4 mr-2" />
                开始新对话
              </Button>
            </div>
          </div>
        )}

        {/* Message List */}
        {(currentConversation || displayMessages.length > 0) && (
          <MessageList messages={displayMessages} isGenerating={isGenerating} />
        )}

        {/* Input Area */}
        <ChatInput
          onSend={handleSendMessage}
          onStop={handleStop}
          isGenerating={isGenerating}
          isSending={sendMessage.isPending}
          placeholder="输入消息开始对话..."
        />
      </div>
    </div>
  );
}
