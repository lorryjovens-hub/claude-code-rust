type Stats = {
  collapsedSpans: number
  stagedSpans: number
  health: {
    totalErrors: number
    totalEmptySpawns: number
    emptySpawnWarningEmitted: boolean
  }
}

const stats: Stats = {
  collapsedSpans: 0,
  stagedSpans: 0,
  health: {
    totalErrors: 0,
    totalEmptySpawns: 0,
    emptySpawnWarningEmitted: false,
  },
}

const listeners = new Set<() => void>()

export function subscribe(listener: () => void): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

export function getStats(): Stats {
  return stats
}

export function isContextCollapseEnabled(): boolean {
  return false
}

export function resetContextCollapse(): void {}

export async function applyCollapsesIfNeeded<T>(messages: T): Promise<{
  messages: T
  changed: boolean
}> {
  return { messages, changed: false }
}

export function isWithheldPromptTooLong(): boolean {
  return false
}

export function recoverFromOverflow<T>(messages: T): T {
  return messages
}
