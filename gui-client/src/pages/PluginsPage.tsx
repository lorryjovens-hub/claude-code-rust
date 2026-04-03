import React, { useState } from 'react'
import { Download, Trash2, RefreshCw, ExternalLink, Search } from 'lucide-react'

interface Plugin {
  id: string
  name: string
  version: string
  description: string
  author: string
  installed: boolean
  icon: string
  url: string
}

const PluginsPage: React.FC = () => {
  const [plugins, setPlugins] = useState<Plugin[]>([
    {
      id: 'code-executor',
      name: 'Code Executor',
      version: '1.0.0',
      description: 'Execute code in various programming languages',
      author: 'Claude Code Team',
      installed: true,
      icon: '💻',
      url: 'https://github.com/lorryjovens-hub/claude-code-rust',
    },
    {
      id: 'file-uploader',
      name: 'File Uploader',
      version: '1.0.0',
      description: 'Upload and analyze files',
      author: 'Claude Code Team',
      installed: false,
      icon: '📁',
      url: 'https://github.com/lorryjovens-hub/claude-code-rust',
    },
    {
      id: 'web-search',
      name: 'Web Search',
      version: '1.0.0',
      description: 'Search the web for information',
      author: 'Claude Code Team',
      installed: false,
      icon: '🌐',
      url: 'https://github.com/lorryjovens-hub/claude-code-rust',
    },
    {
      id: 'image-generator',
      name: 'Image Generator',
      version: '1.0.0',
      description: 'Generate images from text descriptions',
      author: 'Claude Code Team',
      installed: false,
      icon: '🎨',
      url: 'https://github.com/lorryjovens-hub/claude-code-rust',
    },
  ])

  const [search, setSearch] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  const filteredPlugins = plugins.filter(plugin => 
    plugin.name.toLowerCase().includes(search.toLowerCase()) ||
    plugin.description.toLowerCase().includes(search.toLowerCase())
  )

  const handleToggleInstall = (pluginId: string) => {
    setPlugins(prev => prev.map(plugin => 
      plugin.id === pluginId 
        ? { ...plugin, installed: !plugin.installed }
        : plugin
    ))
  }

  const handleRefresh = () => {
    setIsLoading(true)
    // Simulate refresh operation
    setTimeout(() => {
      setIsLoading(false)
    }, 1000)
  }

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Header */}
      <div className="p-4 border-b border-border flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold">Plugins</h1>
          <div className="text-xs text-muted-foreground">Extend Claude Code functionality</div>
        </div>
        <button
          onClick={handleRefresh}
          disabled={isLoading}
          className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors bg-secondary hover:bg-secondary/80"
        >
          {isLoading ? 'Refreshing...' : 'Refresh'}
          <RefreshCw className="h-4 w-4" />
        </button>
      </div>

      {/* Search */}
      <div className="p-4 border-b border-border">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search plugins..."
            className="w-full bg-background border border-border rounded-lg pl-10 pr-4 py-2 focus:outline-none focus:ring-2 focus:ring-primary"
          />
        </div>
      </div>

      {/* Plugins List */}
      <div className="flex-1 overflow-y-auto p-6">
        {filteredPlugins.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
            <div className="text-4xl mb-2">🔍</div>
            <p>No plugins found</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredPlugins.map((plugin) => (
              <div key={plugin.id} className="bg-card border border-border rounded-lg overflow-hidden">
                <div className="p-6">
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center gap-3">
                      <div className="text-2xl">{plugin.icon}</div>
                      <div>
                        <h3 className="font-semibold">{plugin.name}</h3>
                        <div className="text-xs text-muted-foreground">v{plugin.version}</div>
                      </div>
                    </div>
                    <button
                      onClick={() => handleToggleInstall(plugin.id)}
                      className={`flex items-center gap-1 px-3 py-1 rounded-md text-sm transition-colors ${plugin.installed ? 'bg-destructive hover:bg-destructive/90 text-destructive-foreground' : 'bg-primary hover:bg-primary/90 text-primary-foreground'}`}
                    >
                      {plugin.installed ? <Trash2 className="h-3 w-3" /> : <Download className="h-3 w-3" />}
                      {plugin.installed ? 'Uninstall' : 'Install'}
                    </button>
                  </div>
                  <p className="text-sm text-muted-foreground mb-4">{plugin.description}</p>
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <span>By {plugin.author}</span>
                    <a 
                      href={plugin.url} 
                      target="_blank" 
                      rel="noopener noreferrer"
                      className="text-primary hover:underline flex items-center gap-1"
                    >
                      <ExternalLink className="h-3 w-3" />
                    </a>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default PluginsPage
