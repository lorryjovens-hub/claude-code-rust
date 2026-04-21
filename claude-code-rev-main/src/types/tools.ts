export type ToolProgressData = {
  kind?: string
  [key: string]: unknown
}

export type ShellProgress = ToolProgressData
export type BashProgress = ToolProgressData
export type PowerShellProgress = ToolProgressData
export type MCPProgress = ToolProgressData
export type SkillToolProgress = ToolProgressData
export type TaskOutputProgress = ToolProgressData
export type WebSearchProgress = ToolProgressData
export type AgentToolProgress = ToolProgressData
export type REPLToolProgress = ToolProgressData
export type SdkWorkflowProgress = ToolProgressData
