export const DEFAULT_GRANT_FLAGS = {
  accessibility: false,
  screenRecording: false,
}

export const API_RESIZE_PARAMS = {}

export type ToolDef = {
  name: string
  description: string
  inputSchema?: Record<string, unknown>
}

export type DisplayGeometry = Record<string, unknown>
export type FrontmostApp = Record<string, unknown>
export type InstalledApp = { name?: string; bundleId?: string }
export type ResolvePrepareCaptureResult = Record<string, unknown>
export type RunningApp = Record<string, unknown>
export type ScreenshotResult = Record<string, unknown>
export type ScreenshotDims = {
  width: number
  height: number
  displayWidth?: number
  displayHeight?: number
  displayId?: number
  originX?: number
  originY?: number
}
export type CuPermissionRequest = {
  apps?: Array<{ bundleId?: string; displayName?: string }>
  flags?: Record<string, boolean>
  tccState?: { accessibility?: boolean; screenRecording?: boolean }
}
export type CuPermissionResponse = {
  granted: Array<{ bundleId?: string; displayName?: string; grantedAt?: string }>
  denied: Array<{ bundleId?: string; displayName?: string }>
  flags: Record<string, boolean>
}
export type CuCallToolResult = {
  is_error?: boolean
  content?: Array<{
    type: string
    text?: string
    mimeType?: string
    data?: string
  }>
  telemetry?: Record<string, unknown>
}
export type ComputerUseSessionContext = {
  onPermissionRequest?: (
    req: CuPermissionRequest,
    signal?: AbortSignal,
  ) => Promise<CuPermissionResponse> | CuPermissionResponse
  getAllowedApps?: () => Array<{ bundleId?: string; displayName?: string }>
  getGrantFlags?: () => Record<string, boolean>
  [key: string]: unknown
}
export type ComputerExecutor = Record<string, unknown>

function successText(text: string): CuCallToolResult {
  return {
    content: [{ type: 'text', text }],
  }
}

function errorText(text: string): CuCallToolResult {
  return {
    is_error: true,
    content: [{ type: 'text', text }],
  }
}

export function targetImageSize(width: number, height: number) {
  return [width, height] as const
}

const TOOL_DEFS: ToolDef[] = [
  {
    name: 'request_access',
    description:
      'Request access to applications and computer-use permissions for this session.',
  },
  {
    name: 'list_granted_applications',
    description: 'List applications currently granted for computer use.',
  },
  { name: 'screenshot', description: 'Capture a screenshot.' },
  { name: 'zoom', description: 'Capture a zoomed screenshot region.' },
  { name: 'cursor_position', description: 'Read the current cursor position.' },
  { name: 'mouse_move', description: 'Move the mouse cursor.' },
  { name: 'left_click', description: 'Left click at a coordinate.' },
  { name: 'right_click', description: 'Right click at a coordinate.' },
  { name: 'middle_click', description: 'Middle click at a coordinate.' },
  { name: 'double_click', description: 'Double click at a coordinate.' },
  { name: 'triple_click', description: 'Triple click at a coordinate.' },
  { name: 'left_mouse_down', description: 'Press the left mouse button.' },
  { name: 'left_mouse_up', description: 'Release the left mouse button.' },
  { name: 'left_click_drag', description: 'Drag with the left mouse button.' },
  { name: 'scroll', description: 'Scroll at a coordinate or direction.' },
  { name: 'type', description: 'Type text through the active application.' },
  { name: 'key', description: 'Press a key or key chord.' },
  { name: 'hold_key', description: 'Hold one or more keys for a duration.' },
  { name: 'read_clipboard', description: 'Read clipboard text.' },
  { name: 'write_clipboard', description: 'Write clipboard text.' },
  {
    name: 'open_application',
    description: 'Open an application by bundle identifier.',
  },
  { name: 'wait', description: 'Wait for a short duration.' },
  {
    name: 'computer_batch',
    description: 'Execute a sequence of computer-use actions.',
  },
]

export function buildComputerUseTools() {
  return TOOL_DEFS
}

export function createComputerUseMcpServer(
  adapter?: { logger?: { warn(message: string): void; info?(message: string): void } },
) {
  let closed = false
  const handlers = new Map<unknown, unknown>()

  return {
    async connect() {
      adapter?.logger?.warn(
        'Computer Use MCP is running with a restored compatibility shim; request_access works, but native desktop actions remain unavailable in this workspace.',
      )
    },
    setRequestHandler(schema: unknown, handler: unknown) {
      handlers.set(schema, handler)
    },
    async close() {
      closed = true
      handlers.clear()
      adapter?.logger?.info?.('Computer Use MCP shim closed.')
    },
    get isClosed() {
      return closed
    },
  }
}

export function bindSessionContext(
  _adapter?: unknown,
  _coordinateMode?: unknown,
  ctx?: ComputerUseSessionContext,
) {
  return async (
    name: string,
    args: CuPermissionRequest | Record<string, unknown>,
  ): Promise<CuCallToolResult> => {
    switch (name) {
      case 'request_access': {
        if (ctx?.onPermissionRequest) {
          const response = await ctx.onPermissionRequest(args as CuPermissionRequest)
          const grantedCount = Array.isArray(response.granted)
            ? response.granted.length
            : 0
          return successText(
            grantedCount > 0
              ? `Computer-use access updated for ${grantedCount} application(s).`
              : 'Computer-use access request completed.',
          )
        }
        return errorText(
          'Computer-use access approval is not configured in this restored workspace.',
        )
      }

      case 'list_granted_applications': {
        const apps = ctx?.getAllowedApps?.() ?? []
        if (apps.length === 0) {
          return successText('No computer-use applications are currently granted.')
        }
        const names = apps
          .map(app => app.displayName || app.bundleId || 'unknown')
          .join(', ')
        return successText(`Granted computer-use applications: ${names}`)
      }

      case 'read_clipboard':
        return errorText(
          'Clipboard access is unavailable in the restored computer-use shim.',
        )

      default:
        return errorText(
          `Computer-use tool "${name}" is not available in this restored workspace. The shim currently supports session approval flows, but not native desktop execution.`,
        )
    }
  }
}
