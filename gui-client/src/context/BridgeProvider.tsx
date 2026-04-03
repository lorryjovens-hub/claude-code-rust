import React, { createContext, useContext, ReactNode, useCallback } from 'react'

// Add type declaration for Tauri
declare global {
  interface Window {
    __TAURI__?: boolean
  }
}

interface BridgeContextType {
  sendMessage: (message: any) => Promise<any>
  getModels: () => Promise<string[]>
  createChat: () => Promise<string>
  sendChatMessage: (chatId: string, message: string) => Promise<any>
  getChatHistory: (chatId: string) => Promise<any[]>
}

const BridgeContext = createContext<BridgeContextType | undefined>(undefined)

export const useBridge = () => {
  const context = useContext(BridgeContext)
  if (!context) {
    throw new Error('useBridge must be used within BridgeProvider')
  }
  return context
}

interface BridgeProviderProps {
  children: ReactNode
}

export const BridgeProvider: React.FC<BridgeProviderProps> = ({ children }) => {
  const sendMessage = useCallback(async (message: any): Promise<any> => {
    try {
      // Always use mock implementation for now
      console.log('Mock message sent:', message)
      return { success: true, data: 'Mock response' }
    } catch (error) {
      console.error('Error sending message:', error)
      throw error
    }
  }, [])

  const getModels = useCallback(async (): Promise<string[]> => {
    try {
      // Mock response
      return [
        'claude-3-opus-20240229',
        'claude-3-sonnet-20240229',
        'claude-3-haiku-20240307',
      ]
    } catch (error) {
      console.error('Error getting models:', error)
      return []
    }
  }, [])

  const createChat = useCallback(async (): Promise<string> => {
    try {
      // Mock response
      return `chat_${Date.now()}`
    } catch (error) {
      console.error('Error creating chat:', error)
      throw error
    }
  }, [])

  const sendChatMessage = useCallback(async (chatId: string, message: string): Promise<any> => {
    try {
      // Mock response
      console.log('Mock chat message:', { chatId, message })
      return {
        id: `msg_${Date.now()}`,
        content: `Mock response to: ${message}`,
        role: 'assistant',
        timestamp: new Date().toISOString(),
      }
    } catch (error) {
      console.error('Error sending chat message:', error)
      throw error
    }
  }, [])

  const getChatHistory = useCallback(async (_chatId: string): Promise<any[]> => {
    try {
      // Mock response
      return [
        {
          id: 'msg_1',
          content: 'Hello! How can I help you today?',
          role: 'assistant',
          timestamp: new Date().toISOString(),
        },
      ]
    } catch (error) {
      console.error('Error getting chat history:', error)
      return []
    }
  }, [])

  const value = {
    sendMessage,
    getModels,
    createChat,
    sendChatMessage,
    getChatHistory,
  }

  return <BridgeContext.Provider value={value}>{children}</BridgeContext.Provider>
}
