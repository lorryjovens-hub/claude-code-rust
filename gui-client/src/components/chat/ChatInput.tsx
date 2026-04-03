import { useState, useRef, useCallback } from "react";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Send, Paperclip, Mic, Square, Loader2 } from "lucide-react";

interface ChatInputProps {
  onSend: (message: string) => void;
  onStop?: () => void;
  isGenerating?: boolean;
  isSending?: boolean;
  disabled?: boolean;
  placeholder?: string;
}

export function ChatInput({
  onSend,
  onStop,
  isGenerating,
  isSending,
  disabled,
  placeholder = "输入消息...",
}: ChatInputProps) {
  const [input, setInput] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSend = useCallback(() => {
    if (!input.trim() || disabled || isSending) return;
    onSend(input.trim());
    setInput("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  }, [input, onSend, disabled, isSending]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    // Auto-resize textarea
    const textarea = e.target;
    textarea.style.height = "auto";
    textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`;
  };

  const isLoading = isGenerating || isSending;

  return (
    <div className="border-t bg-card p-4">
      <div className="max-w-4xl mx-auto">
        <div className="relative flex items-end gap-2 bg-background border rounded-2xl p-2 focus-within:ring-2 focus-within:ring-ring">
          {/* Attachment Button */}
          <Button
            variant="ghost"
            size="icon"
            className="shrink-0 h-8 w-8 rounded-xl"
            disabled={disabled || isLoading}
          >
            <Paperclip className="h-4 w-4" />
          </Button>

          {/* Text Input */}
          <Textarea
            ref={textareaRef}
            value={input}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            disabled={disabled || isLoading}
            className="min-h-[40px] max-h-[200px] border-0 bg-transparent focus-visible:ring-0 resize-none py-2 px-0"
            rows={1}
          />

          {/* Voice / Stop Button */}
          {isGenerating ? (
            <Button
              variant="destructive"
              size="icon"
              className="shrink-0 h-8 w-8 rounded-xl"
              onClick={onStop}
            >
              <Square className="h-3 w-3 fill-current" />
            </Button>
          ) : (
            <Button
              variant="ghost"
              size="icon"
              className="shrink-0 h-8 w-8 rounded-xl"
              disabled={disabled || !input.trim() || isSending}
            >
              <Mic className="h-4 w-4" />
            </Button>
          )}

          {/* Send Button */}
          <Button
            className="shrink-0 h-8 w-8 rounded-xl bg-claude-orange hover:bg-claude-orange-dark"
            size="icon"
            onClick={handleSend}
            disabled={disabled || isLoading || !input.trim()}
          >
            {isSending ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Send className="h-4 w-4" />
            )}
          </Button>
        </div>

        {/* Hint */}
        <p className="text-xs text-muted-foreground text-center mt-2">
          按 Enter 发送，Shift + Enter 换行
        </p>
      </div>
    </div>
  );
}
