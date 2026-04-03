import React, { useState, useEffect, useRef } from 'react'
import { Send, Paperclip, Mic, Copy, RefreshCw, Trash2 } from 'lucide-react'
import { useBridge } from '../context/BridgeProvider'
import { useApp } from '../context/AppContext'

interface Message {
  id: string
  content: string
  role: 'user' | 'assistant'
  timestamp: string
}

const ChatPage: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [chatId, setChatId] = useState<string | null>(null)
  const { sendChatMessage, createChat, getChatHistory } = useBridge()
  const { activeModel } = useApp()
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const initChat = async () => {
      try {
        const newChatId = await createChat()
        setChatId(newChatId)
        const history = await getChatHistory(newChatId)
        setMessages(history)
      } catch (error) {
        console.error('Error initializing chat:', error)
      }
    }

    initChat()
  }, [createChat, getChatHistory])

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSend = async () => {
    if (!input.trim() || !chatId || isLoading) return

    const userMessage: Message = {
      id: `msg_${Date.now()}_user`,
      content: input.trim(),
      role: 'user',
      timestamp: new Date().toISOString(),
    }

    setMessages(prev => [...prev, userMessage])
    setInput('')
    setIsLoading(true)

    try {
      const response = await sendChatMessage(chatId, userMessage.content)
      const assistantMessage: Message = {
        id: response.id,
        content: response.content,
        role: 'assistant',
        timestamp: response.timestamp,
      }
      setMessages(prev => [...prev, assistantMessage])
    } catch (error) {
      console.error('Error sending message:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const handleCopy = (content: string) => {
    navigator.clipboard.writeText(content)
  }

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Chat Header */}
      <div className="p-4 border-b border-border flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold">Chat</h1>
          <div className="text-xs text-muted-foreground">{activeModel}</div>
        </div>
        <div className="flex items-center gap-2">
          <button className="p-2 rounded-md hover:bg-secondary transition-colors">
            <RefreshCw className="h-5 w-5" />
          </button>
          <button className="p-2 rounded-md hover:bg-secondary transition-colors">
            <Trash2 className="h-5 w-5" />
          </button>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {messages.map((message) => (
          <div key={message.id} className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}>
            <div className={`max-w-3/4 ${message.role === 'user' ? 'mr-4' : 'ml-4'}`}>
              <div className={`p-4 rounded-lg ${message.role === 'user' ? 'bg-primary text-primary-foreground' : 'bg-card border border-border'}`}>
                <div className="whitespace-pre-wrap">{message.content}</div>
              </div>
              <div className="flex items-center justify-between mt-1 px-1">
                <span className="text-xs text-muted-foreground">
                  {new Date(message.timestamp).toLocaleTimeString()}
                </span>
                <button 
                  onClick={() => handleCopy(message.content)}
                  className="text-xs text-muted-foreground hover:text-foreground transition-colors"
                >
                  <Copy className="h-3 w-3 inline" />
                </button>
              </div>
            </div>
          </div>
        ))}
        {isLoading && (
          <div className="flex justify-start ml-4">
            <div className="p-4 rounded-lg bg-card border border-border animate-pulse">
              <div className="space-y-2">
                <div className="h-4 bg-muted rounded w-3/4"></div>
                <div className="h-4 bg-muted rounded w-1/2"></div>
                <div className="h-4 bg-muted rounded w-5/6"></div>
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="p-4 border-t border-border">
        <div className="bg-card border border-border rounded-lg p-2 flex items-center">
          <button className="p-2 rounded-md hover:bg-secondary transition-colors">
            <Paperclip className="h-5 w-5" />
          </button>
          <button className="p-2 rounded-md hover:bg-secondary transition-colors">
            <Mic className="h-5 w-5" />
          </button>
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Type your message..."
            className="flex-1 bg-transparent border-none outline-none resize-none p-2"
            rows={1}
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            className={`p-2 rounded-md transition-colors ${input.trim() && !isLoading ? 'bg-primary hover:bg-primary/90 text-primary-foreground' : 'text-muted-foreground cursor-not-allowed'}`}
          >
            <Send className="h-5 w-5" />
          </button>
        </div>
      </div>
    </div>
  )
}

export default ChatPage
