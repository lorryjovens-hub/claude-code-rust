import React, { useState } from 'react'
import { Trash2, Search, Clock, MessageSquare } from 'lucide-react'

interface ChatHistory {
  id: string
  title: string
  messages: number
  lastModified: string
  model: string
}

const HistoryPage: React.FC = () => {
  const [history, setHistory] = useState<ChatHistory[]>([
    {
      id: 'chat_1',
      title: 'Code refactoring discussion',
      messages: 12,
      lastModified: '2024-01-15T14:30:00Z',
      model: 'claude-3-opus-20240229',
    },
    {
      id: 'chat_2',
      title: 'API integration help',
      messages: 8,
      lastModified: '2024-01-15T10:15:00Z',
      model: 'claude-3-sonnet-20240229',
    },
    {
      id: 'chat_3',
      title: 'Bug fix suggestions',
      messages: 5,
      lastModified: '2024-01-14T16:45:00Z',
      model: 'claude-3-opus-20240229',
    },
    {
      id: 'chat_4',
      title: 'UI design feedback',
      messages: 15,
      lastModified: '2024-01-14T09:20:00Z',
      model: 'claude-3-haiku-20240307',
    },
  ])

  const [search, setSearch] = useState('')
  const [selectedChats, setSelectedChats] = useState<string[]>([])

  const filteredHistory = history.filter(chat => 
    chat.title.toLowerCase().includes(search.toLowerCase())
  )

  const handleSelectChat = (chatId: string) => {
    setSelectedChats(prev => 
      prev.includes(chatId) 
        ? prev.filter(id => id !== chatId)
        : [...prev, chatId]
    )
  }

  const handleDeleteSelected = () => {
    setHistory(prev => prev.filter(chat => !selectedChats.includes(chat.id)))
    setSelectedChats([])
  }

  const handleDeleteChat = (chatId: string) => {
    setHistory(prev => prev.filter(chat => chat.id !== chatId))
  }

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleString()
  }

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Header */}
      <div className="p-4 border-b border-border flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold">History</h1>
          <div className="text-xs text-muted-foreground">Your chat history</div>
        </div>
        {selectedChats.length > 0 && (
          <button
            onClick={handleDeleteSelected}
            className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors bg-destructive hover:bg-destructive/90 text-destructive-foreground"
          >
            <Trash2 className="h-4 w-4" />
            Delete {selectedChats.length} selected
          </button>
        )}
      </div>

      {/* Search */}
      <div className="p-4 border-b border-border">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search chat history..."
            className="w-full bg-background border border-border rounded-lg pl-10 pr-4 py-2 focus:outline-none focus:ring-2 focus:ring-primary"
          />
        </div>
      </div>

      {/* History List */}
      <div className="flex-1 overflow-y-auto">
        {filteredHistory.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
            <div className="text-4xl mb-2">📚</div>
            <p>No chat history found</p>
          </div>
        ) : (
          <div className="divide-y divide-border">
            {filteredHistory.map((chat) => (
              <div key={chat.id} className="p-4 hover:bg-secondary/50 transition-colors">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    <input
                      type="checkbox"
                      checked={selectedChats.includes(chat.id)}
                      onChange={() => handleSelectChat(chat.id)}
                      className="h-4 w-4 rounded border-border text-primary focus:ring-primary"
                    />
                    <MessageSquare className="h-5 w-5 text-muted-foreground" />
                    <div className="flex-1 min-w-0">
                      <h3 className="font-medium truncate">{chat.title}</h3>
                      <div className="text-xs text-muted-foreground">{chat.model}</div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleDeleteChat(chat.id)}
                    className="p-1 rounded-md hover:bg-destructive/20 text-destructive transition-colors"
                  >
                    <Trash2 className="h-4 w-4" />
                  </button>
                </div>
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <div className="flex items-center gap-1">
                    <Clock className="h-3 w-3" />
                    {formatDate(chat.lastModified)}
                  </div>
                  <div>{chat.messages} messages</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default HistoryPage
