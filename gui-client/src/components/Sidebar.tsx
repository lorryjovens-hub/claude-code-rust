import React from 'react'
import { Link, useLocation } from 'react-router-dom'
import { MessageSquare, Settings, Puzzle, History, Plus, Menu, X, Moon, Sun } from 'lucide-react'
import { useApp } from '../context/AppContext'
import { useBridge } from '../context/BridgeProvider'

const Sidebar: React.FC = () => {
  const location = useLocation()
  const { theme, setTheme, sidebarCollapsed, setSidebarCollapsed, models, activeModel, setActiveModel } = useApp()
  const { createChat } = useBridge()

  const handleNewChat = async () => {
    try {
      await createChat()
    } catch (error) {
      console.error('Error creating new chat:', error)
    }
  }

  const handleModelChange = (model: string) => {
    setActiveModel(model)
  }

  const navigationItems = [
    { icon: MessageSquare, label: 'Chat', path: '/chat' },
    { icon: Settings, label: 'Settings', path: '/settings' },
    { icon: Puzzle, label: 'Plugins', path: '/plugins' },
    { icon: History, label: 'History', path: '/history' },
  ]

  return (
    <div className={`bg-card border-r border-border flex flex-col transition-all duration-300 ${sidebarCollapsed ? 'w-20' : 'w-64'}`}>
      {/* Header */}
      <div className="p-4 border-b border-border flex items-center justify-between">
        <div className={`flex items-center ${sidebarCollapsed ? 'justify-center w-full' : ''}`}>
          <div className="h-8 w-8 bg-primary rounded-lg flex items-center justify-center mr-2">
            <MessageSquare className="h-5 w-5 text-primary-foreground" />
          </div>
          {!sidebarCollapsed && (
            <span className="text-xl font-bold">Claude Code</span>
          )}
        </div>
        <button
          onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
          className="p-2 rounded-md hover:bg-secondary transition-colors"
        >
          {sidebarCollapsed ? <Menu className="h-5 w-5" /> : <X className="h-5 w-5" />}
        </button>
      </div>

      {/* New Chat Button */}
      <div className={`p-4 ${sidebarCollapsed ? 'flex justify-center' : ''}`}>
        <button
          onClick={handleNewChat}
          className="w-full flex items-center justify-center gap-2 bg-primary hover:bg-primary/90 text-primary-foreground py-2 px-4 rounded-lg font-medium transition-colors"
        >
          <Plus className="h-4 w-4" />
          {!sidebarCollapsed && <span>New Chat</span>}
        </button>
      </div>

      {/* Model Selection */}
      {!sidebarCollapsed && (
        <div className="px-4 mb-4">
          <div className="text-xs text-muted-foreground mb-2 uppercase tracking-wider">Model</div>
          <div className="flex flex-col space-y-1">
            {models.map((model) => (
              <button
                key={model}
                onClick={() => handleModelChange(model)}
                className={`py-1.5 px-3 rounded-md text-sm transition-colors ${activeModel === model ? 'bg-primary text-primary-foreground' : 'hover:bg-secondary'}`}
              >
                {model.replace('-', ' ').split(' ').slice(0, 2).join(' ')}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Navigation */}
      <nav className="flex-1 p-2">
        <ul className="space-y-1">
          {navigationItems.map((item) => {
            const Icon = item.icon
            const isActive = location.pathname === item.path
            return (
              <li key={item.path}>
                <Link
                  to={item.path}
                  className={`flex items-center gap-3 px-3 py-2 rounded-md transition-colors ${isActive ? 'bg-secondary' : 'hover:bg-secondary/50'}`}
                >
                  <Icon className="h-5 w-5" />
                  {!sidebarCollapsed && <span>{item.label}</span>}
                </Link>
              </li>
            )
          })}
        </ul>
      </nav>

      {/* Footer */}
      <div className={`p-4 border-t border-border ${sidebarCollapsed ? 'flex justify-center' : 'flex items-center justify-between'}`}>
        <button
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
          className="p-2 rounded-md hover:bg-secondary transition-colors"
        >
          {theme === 'dark' ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
        </button>
        {!sidebarCollapsed && (
          <div className="text-xs text-muted-foreground">v0.1.0</div>
        )}
      </div>
    </div>
  )
}

export default Sidebar
