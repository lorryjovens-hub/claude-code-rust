export type MessageOrigin = {
  kind?: string
  [key: string]: unknown
}

export type MessageBase = {
  uuid?: string
  parentUuid?: string
  timestamp?: string
  createdAt?: string
  isMeta?: boolean
  isVirtual?: boolean
  isCompactSummary?: boolean
  toolUseResult?: unknown
  origin?: MessageOrigin
  [key: string]: unknown
}

export type AttachmentMessage = MessageBase & {
  type: 'attachment'
  path?: string
}

export type UserMessage = MessageBase & {
  type: 'user'
  message: {
    content: string | Array<{ type: string; text?: string; [key: string]: unknown }>
    [key: string]: unknown
  }
}

export type AssistantMessage = MessageBase & {
  type: 'assistant'
  message?: {
    content?: unknown
    [key: string]: unknown
  }
}

export type ProgressMessage = MessageBase & {
  type: 'progress'
  progress?: unknown
}

export type SystemMessageLevel = 'info' | 'warning' | 'error' | string

export type SystemMessage = MessageBase & {
  type: 'system'
  subtype?: string
  level?: SystemMessageLevel
  message?: string
}

export type SystemLocalCommandMessage = SystemMessage & {
  subtype: 'local_command'
}

export type SystemBridgeStatusMessage = SystemMessage
export type SystemTurnDurationMessage = SystemMessage
export type SystemThinkingMessage = SystemMessage
export type SystemMemorySavedMessage = SystemMessage
export type SystemStopHookSummaryMessage = SystemMessage
export type SystemInformationalMessage = SystemMessage
export type SystemCompactBoundaryMessage = SystemMessage
export type SystemMicrocompactBoundaryMessage = SystemMessage
export type SystemPermissionRetryMessage = SystemMessage
export type SystemScheduledTaskFireMessage = SystemMessage
export type SystemAwaySummaryMessage = SystemMessage
export type SystemAgentsKilledMessage = SystemMessage
export type SystemApiMetricsMessage = SystemMessage
export type SystemAPIErrorMessage = SystemMessage & { error?: string }
export type SystemFileSnapshotMessage = SystemMessage

export type HookResultMessage = MessageBase & {
  type: 'hook_result'
}

export type ToolUseSummaryMessage = MessageBase & {
  type: 'tool_use_summary'
}

export type TombstoneMessage = MessageBase & {
  type: 'tombstone'
}

export type StreamEvent = {
  type?: string
  [key: string]: unknown
}

export type RequestStartEvent = StreamEvent

export type StopHookInfo = {
  [key: string]: unknown
}

export type CompactMetadata = {
  [key: string]: unknown
}

export type PartialCompactDirection = 'older' | 'newer' | 'both' | string

export type CollapsedReadSearchGroup = {
  [key: string]: unknown
}

export type GroupedToolUseMessage = MessageBase & {
  type: 'grouped_tool_use'
}

export type CollapsibleMessage = MessageBase

export type NormalizedAssistantMessage = AssistantMessage
export type NormalizedUserMessage = UserMessage
export type NormalizedMessage =
  | NormalizedAssistantMessage
  | NormalizedUserMessage
  | ProgressMessage
  | SystemMessage
  | AttachmentMessage

export type RenderableMessage = Message

export type Message =
  | UserMessage
  | AssistantMessage
  | ProgressMessage
  | SystemMessage
  | AttachmentMessage
  | HookResultMessage
  | ToolUseSummaryMessage
  | TombstoneMessage
  | GroupedToolUseMessage

