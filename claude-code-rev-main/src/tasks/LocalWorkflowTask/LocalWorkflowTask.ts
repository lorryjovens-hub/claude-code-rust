export type LocalWorkflowTaskState = Record<string, unknown>

export function isLocalWorkflowTask(_value: unknown): boolean {
  return false
}
