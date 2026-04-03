import { useRef, useEffect } from "react";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Bot, User, Loader2 } from "lucide-react";
import type { Message } from "@/types";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface MessageListProps {
  messages: Message[];
  isGenerating?: boolean;
}

export function MessageList({ messages, isGenerating }: MessageListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages, isGenerating]);

  return (
    <ScrollArea className="flex-1 px-4">
      <div className="space-y-6 py-4 max-w-4xl mx-auto">
        {messages.map((message, index) => (
          <MessageItem key={message.id || index} message={message} />
        ))}
        {isGenerating && (
          <div className="flex items-start gap-4">
            <Avatar className="w-8 h-8 bg-claude-orange">
              <AvatarFallback className="bg-claude-orange text-white">
                <Bot className="w-4 h-4" />
              </AvatarFallback>
            </Avatar>
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span className="text-sm">思考中...</span>
            </div>
          </div>
        )}
        <div ref={scrollRef} />
      </div>
    </ScrollArea>
  );
}

interface MessageItemProps {
  message: Message;
}

function MessageItem({ message }: MessageItemProps) {
  const isUser = message.role === "user";

  return (
    <div
      className={cn(
        "flex items-start gap-4 animate-in",
        isUser ? "flex-row-reverse" : "flex-row"
      )}
    >
      <Avatar className={cn("w-8 h-8", isUser ? "bg-primary" : "bg-claude-orange")}>
        <AvatarFallback
          className={cn(
            "text-white",
            isUser ? "bg-primary" : "bg-claude-orange"
          )}
        >
          {isUser ? <User className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
        </AvatarFallback>
      </Avatar>

      <div
        className={cn(
          "flex-1 max-w-[85%]",
          isUser ? "items-end" : "items-start"
        )}
      >
        <div
          className={cn(
            "rounded-2xl px-4 py-3",
            isUser
              ? "bg-claude-orange text-white rounded-br-md"
              : "bg-muted rounded-bl-md"
          )}
        >
          {isUser ? (
            <p className="text-sm whitespace-pre-wrap">{message.content}</p>
          ) : (
            <div className="prose prose-sm dark:prose-invert max-w-none">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {message.content}
              </ReactMarkdown>
            </div>
          )}
        </div>

        {/* Message metadata */}
        <div className="flex items-center gap-2 mt-1 px-1">
          {message.model && (
            <Badge variant="secondary" className="text-xs">
              {message.model}
            </Badge>
          )}
          {message.tokens && (
            <span className="text-xs text-muted-foreground">
              {message.tokens.total} tokens
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
