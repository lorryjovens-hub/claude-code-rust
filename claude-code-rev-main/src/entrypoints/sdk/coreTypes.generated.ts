export type SDKMessage = {
  type: string
  uuid?: string
  [key: string]: unknown
}

export type SDKUserMessage = SDKMessage
export type SDKResultMessage = SDKMessage
export type SDKResultSuccess = SDKMessage
export type SDKSessionInfo = Record<string, unknown>
