export type EffortLevel = 'low' | 'medium' | 'high' | string
export type SDKSession = Record<string, unknown>
export type SDKSessionOptions = Record<string, unknown>
export type SDKSessionInfo = Record<string, unknown>
export type SessionMessage = Record<string, unknown>
export type ListSessionsOptions = Record<string, unknown>
export type GetSessionInfoOptions = Record<string, unknown>
export type GetSessionMessagesOptions = Record<string, unknown>
export type SessionMutationOptions = Record<string, unknown>
export type ForkSessionOptions = Record<string, unknown>
export type ForkSessionResult = Record<string, unknown>
export type Options = Record<string, unknown>
export type InternalOptions = Record<string, unknown>
export type Query = Record<string, unknown>
export type InternalQuery = Record<string, unknown>
export type McpSdkServerConfigWithInstance = Record<string, unknown>
export type AnyZodRawShape = Record<string, unknown>
export type InferShape<T> = T
export type SdkMcpToolDefinition<Schema> = {
  schema?: Schema
  [key: string]: unknown
}
