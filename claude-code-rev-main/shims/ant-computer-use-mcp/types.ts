export const DEFAULT_GRANT_FLAGS = {
  accessibility: false,
  screenRecording: false,
}

export type CoordinateMode = 'screen' | 'viewport'
export type CuSubGates = Record<string, boolean>
export type Logger = {
  silly?(message: string, ...args: unknown[]): void
  debug?(message: string, ...args: unknown[]): void
  info(message: string): void
  warn(message: string): void
  error(message: string): void
}
export type ComputerUseHostAdapter = {
  logger?: Logger
  executor?: Record<string, unknown>
  [key: string]: unknown
}
export type CuPermissionRequest = {
  apps?: Array<{ bundleId?: string; displayName?: string }>
  flags?: Record<string, boolean>
  tccState?: { accessibility?: boolean; screenRecording?: boolean }
  [key: string]: unknown
}
export type CuPermissionResponse = {
  granted: Array<{ bundleId?: string; displayName?: string; grantedAt?: string }>
  denied: Array<{ bundleId?: string; displayName?: string }>
  flags: Record<string, boolean>
}
