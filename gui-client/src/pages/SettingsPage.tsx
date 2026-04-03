import React, { useState } from 'react'
import { Save, X, Check, ExternalLink } from 'lucide-react'

interface SettingSection {
  title: string
  description: string
  settings: Setting[]
}

interface Setting {
  id: string
  label: string
  type: 'text' | 'number' | 'boolean' | 'select'
  value: any
  options?: { label: string; value: any }[]
  placeholder?: string
}

const SettingsPage: React.FC = () => {
  const [settings, setSettings] = useState<SettingSection[]>([
    {
      title: 'API Configuration',
      description: 'Configure your API settings',
      settings: [
        {
          id: 'apiKey',
          label: 'API Key',
          type: 'text',
          value: '',
          placeholder: 'Enter your API key',
        },
        {
          id: 'apiEndpoint',
          label: 'API Endpoint',
          type: 'text',
          value: 'https://api.anthropic.com/v1/messages',
          placeholder: 'API endpoint URL',
        },
      ],
    },
    {
      title: 'Application Settings',
      description: 'Configure application behavior',
      settings: [
        {
          id: 'theme',
          label: 'Theme',
          type: 'select',
          value: 'dark',
          options: [
            { label: 'Dark', value: 'dark' },
            { label: 'Light', value: 'light' },
          ],
        },
        {
          id: 'autoSave',
          label: 'Auto Save',
          type: 'boolean',
          value: true,
        },
        {
          id: 'maxTokens',
          label: 'Max Tokens',
          type: 'number',
          value: 1000,
        },
      ],
    },
    {
      title: 'Advanced Settings',
      description: 'Advanced configuration options',
      settings: [
        {
          id: 'temperature',
          label: 'Temperature',
          type: 'number',
          value: 0.7,
        },
        {
          id: 'topP',
          label: 'Top P',
          type: 'number',
          value: 0.9,
        },
      ],
    },
  ])

  const [isSaving, setIsSaving] = useState(false)
  const [showSuccess, setShowSuccess] = useState(false)

  const handleSettingChange = (sectionIndex: number, settingIndex: number, value: any) => {
    const updatedSettings = [...settings]
    updatedSettings[sectionIndex].settings[settingIndex].value = value
    setSettings(updatedSettings)
  }

  const handleSave = async () => {
    setIsSaving(true)
    // Simulate save operation
    setTimeout(() => {
      setIsSaving(false)
      setShowSuccess(true)
      setTimeout(() => setShowSuccess(false), 2000)
    }, 1000)
  }

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Header */}
      <div className="p-4 border-b border-border flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold">Settings</h1>
          <div className="text-xs text-muted-foreground">Configure your Claude Code settings</div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleSave}
            disabled={isSaving}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${isSaving ? 'bg-secondary text-muted-foreground' : 'bg-primary hover:bg-primary/90 text-primary-foreground'}`}
          >
            {isSaving ? 'Saving...' : 'Save'}
            {isSaving ? <X className="h-4 w-4" /> : <Save className="h-4 w-4" />}
          </button>
          {showSuccess && (
            <div className="flex items-center gap-1 text-green-400 text-sm">
              <Check className="h-4 w-4" />
              Saved
            </div>
          )}
        </div>
      </div>

      {/* Settings Content */}
      <div className="flex-1 overflow-y-auto p-6 space-y-8">
        {settings.map((section, sectionIndex) => (
          <div key={section.title} className="bg-card border border-border rounded-lg p-6">
            <h2 className="text-lg font-semibold mb-2">{section.title}</h2>
            <p className="text-sm text-muted-foreground mb-4">{section.description}</p>
            <div className="space-y-4">
              {section.settings.map((setting, settingIndex) => (
                <div key={setting.id} className="space-y-2">
                  <label className="text-sm font-medium">{setting.label}</label>
                  {setting.type === 'text' && (
                    <input
                      type="text"
                      value={setting.value}
                      onChange={(e) => handleSettingChange(sectionIndex, settingIndex, e.target.value)}
                      placeholder={setting.placeholder}
                      className="w-full bg-background border border-border rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary"
                    />
                  )}
                  {setting.type === 'number' && (
                    <input
                      type="number"
                      value={setting.value}
                      onChange={(e) => handleSettingChange(sectionIndex, settingIndex, parseFloat(e.target.value) || 0)}
                      className="w-full bg-background border border-border rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary"
                    />
                  )}
                  {setting.type === 'boolean' && (
                    <div className="flex items-center">
                      <input
                        type="checkbox"
                        checked={setting.value}
                        onChange={(e) => handleSettingChange(sectionIndex, settingIndex, e.target.checked)}
                        className="h-4 w-4 rounded border-border text-primary focus:ring-primary"
                      />
                    </div>
                  )}
                  {setting.type === 'select' && setting.options && (
                    <select
                      value={setting.value}
                      onChange={(e) => handleSettingChange(sectionIndex, settingIndex, e.target.value)}
                      className="w-full bg-background border border-border rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary"
                    >
                      {setting.options.map((option) => (
                        <option key={option.value} value={option.value}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                  )}
                </div>
              ))}
            </div>
          </div>
        ))}

        {/* About Section */}
        <div className="bg-card border border-border rounded-lg p-6">
          <h2 className="text-lg font-semibold mb-2">About</h2>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">Version</span>
              <span className="text-sm font-medium">0.1.0</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">License</span>
              <span className="text-sm font-medium">MIT</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">GitHub</span>
              <a 
                href="https://github.com/lorryjovens-hub/claude-code-rust" 
                target="_blank" 
                rel="noopener noreferrer"
                className="text-sm text-primary hover:underline flex items-center gap-1"
              >
                Repository <ExternalLink className="h-3 w-3" />
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default SettingsPage
