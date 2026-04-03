import { useState } from "react";
import { useConversations, useDeleteConversation } from "@/hooks/useApi";
import { useStore } from "@/hooks/useStore";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Search,
  MessageSquare,
  Trash2,
  Clock,
  ChevronRight,
  Loader2,
} from "lucide-react";
import type { Conversation } from "@/lib/api";

export function HistoryPage() {
  const { setCurrentConversation, setCurrentView } = useStore();
  const [searchQuery, setSearchQuery] = useState("");
  const { data: conversations = [], isLoading } = useConversations();
  const deleteConversation = useDeleteConversation();

  const handleDeleteConversation = async (id: string) => {
    await deleteConversation.mutateAsync(id);
  };

  const handleOpenConversation = (conversation: Conversation) => {
    setCurrentConversation(conversation);
    setCurrentView("chat");
  };

  const filteredConversations = conversations.filter(
    (conv) =>
      conv.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      conv.messages.some((m) =>
        m.content.toLowerCase().includes(searchQuery.toLowerCase())
      )
  );

  // Group conversations by date
  const groupedConversations = filteredConversations.reduce(
    (groups, conv) => {
      const date = new Date(conv.updated_at * 1000);
      const today = new Date();
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);

      let group = "更早";
      if (date.toDateString() === today.toDateString()) {
        group = "今天";
      } else if (date.toDateString() === yesterday.toDateString()) {
        group = "昨天";
      } else if (today.getTime() - date.getTime() < 7 * 24 * 60 * 60 * 1000) {
        group = "本周";
      }

      if (!groups[group]) {
        groups[group] = [];
      }
      groups[group].push(conv);
      return groups;
    },
    {} as Record<string, Conversation[]>
  );

  const groupOrder = ["今天", "昨天", "本周", "更早"];

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-2xl font-semibold">对话历史</h1>
      </div>

      {/* Search */}
      <div className="p-4 border-b">
        <div className="relative max-w-xl">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <Input
            placeholder="搜索对话..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
      </div>

      {/* Conversation List */}
      <ScrollArea className="flex-1 p-4">
        <div className="max-w-4xl mx-auto">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="w-8 h-8 animate-spin text-claude-orange" />
            </div>
          ) : filteredConversations.length === 0 ? (
            <div className="text-center py-12">
              <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mx-auto mb-4">
                <MessageSquare className="w-8 h-8 text-muted-foreground" />
              </div>
              <h3 className="text-lg font-medium mb-2">
                {searchQuery ? "未找到匹配的对话" : "暂无对话历史"}
              </h3>
              <p className="text-muted-foreground">
                {searchQuery
                  ? "尝试使用其他关键词搜索"
                  : "开始一个新对话，它将显示在这里"}
              </p>
            </div>
          ) : (
            <div className="space-y-6">
              {groupOrder.map(
                (group) =>
                  groupedConversations[group]?.length > 0 && (
                    <div key={group}>
                      <h3 className="text-sm font-medium text-muted-foreground mb-3">
                        {group}
                      </h3>
                      <div className="space-y-2">
                        {groupedConversations[group].map((conversation) => (
                          <ConversationCard
                            key={conversation.id}
                            conversation={conversation}
                            onOpen={() => handleOpenConversation(conversation)}
                            onDelete={() =>
                              handleDeleteConversation(conversation.id)
                            }
                            isDeleting={deleteConversation.isPending}
                          />
                        ))}
                      </div>
                    </div>
                  )
              )}
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}

interface ConversationCardProps {
  conversation: Conversation;
  onOpen: () => void;
  onDelete: () => void;
  isDeleting: boolean;
}

function ConversationCard({
  conversation,
  onOpen,
  onDelete,
  isDeleting,
}: ConversationCardProps) {
  const lastMessage = conversation.messages[conversation.messages.length - 1];

  const formatRelativeTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (seconds < 60) return "刚刚";
    if (minutes < 60) return `${minutes}分钟前`;
    if (hours < 24) return `${hours}小时前`;
    if (days < 7) return `${days}天前`;

    return date.toLocaleDateString("zh-CN");
  };

  return (
    <Card
      className="cursor-pointer hover:shadow-md transition-shadow group"
      onClick={onOpen}
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <MessageSquare className="w-4 h-4 text-claude-orange" />
              <h4 className="font-medium truncate">{conversation.title}</h4>
            </div>
            {lastMessage && (
              <p className="text-sm text-muted-foreground truncate">
                {lastMessage.role === "user" ? "你: " : "AI: "}
                {lastMessage.content.slice(0, 100)}
              </p>
            )}
            <div className="flex items-center gap-4 mt-2">
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <Clock className="w-3 h-3" />
                {formatRelativeTime(conversation.updated_at)}
              </div>
              <Badge variant="secondary" className="text-xs">
                {conversation.messages.length} 条消息
              </Badge>
              {conversation.model && (
                <Badge variant="outline" className="text-xs">
                  {conversation.model}
                </Badge>
              )}
            </div>
          </div>
          <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={(e) => {
                e.stopPropagation();
                onDelete();
              }}
              disabled={isDeleting}
            >
              {isDeleting ? (
                <Loader2 className="w-4 h-4 animate-spin text-destructive" />
              ) : (
                <Trash2 className="w-4 h-4 text-destructive" />
              )}
            </Button>
            <ChevronRight className="w-4 h-4 text-muted-foreground" />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
