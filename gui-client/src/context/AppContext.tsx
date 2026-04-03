import React, { createContext, useContext, useState, ReactNode } from 'react'

interface AppContextType {
  theme: 'dark' | 'light'
  setTheme: (theme: 'dark' | 'light') => void
  sidebarCollapsed: boolean
  setSidebarCollapsed: (collapsed: boolean) => void
  activeModel: string
  setActiveModel: (model: string) => void
  models: string[]
}

const AppContext = createContext<AppContextType | undefined>(undefined)

export const useApp = () => {
  const context = useContext(AppContext)
  if (!context) {
    throw new Error('useApp must be used within AppProvider')
  }
  return context
}

interface AppProviderProps {
  children: ReactNode
}

export const AppProvider: React.FC<AppProviderProps> = ({ children }) => {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark')
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const [activeModel, setActiveModel] = useState('claude-3-opus-20240229')
  const [models] = useState([
    'claude-3-opus-20240229',
    'claude-3-sonnet-20240229',
    'claude-3-haiku-20240307',
  ])

  const value = {
    theme,
    setTheme,
    sidebarCollapsed,
    setSidebarCollapsed,
    activeModel,
    setActiveModel,
    models,
  }

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>
}
