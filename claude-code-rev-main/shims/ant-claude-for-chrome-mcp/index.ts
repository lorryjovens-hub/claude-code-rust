export type PermissionMode =
  | 'ask'
  | 'skip_all_permission_checks'
  | 'follow_a_plan'

export type Logger = {
  info(message: string): void
  warn(message: string): void
  error(message: string): void
}

export type ClaudeForChromeContext = {
  serverName?: string
  logger?: Logger
  [key: string]: unknown
}

export const BROWSER_TOOLS: Array<{ name: string; description: string }> = [
  {
    name: 'navigate',
    description: 'Navigate a browser tab to a URL.',
  },
  {
    name: 'read_page',
    description: 'Capture high-level page state from the active tab.',
  },
  {
    name: 'get_page_text',
    description: 'Read visible page text from the active tab.',
  },
  {
    name: 'find',
    description: 'Find a pattern within page content.',
  },
  {
    name: 'form_input',
    description: 'Fill or update form inputs in the page.',
  },
  {
    name: 'computer',
    description: 'Perform browser-scoped mouse and keyboard actions.',
  },
  {
    name: 'javascript_tool',
    description: 'Run page-scoped JavaScript in the browser tab.',
  },
  {
    name: 'tabs_context_mcp',
    description: 'List or inspect browser tabs.',
  },
  {
    name: 'tabs_create_mcp',
    description: 'Create a new browser tab.',
  },
  {
    name: 'resize_window',
    description: 'Resize the browser window.',
  },
  {
    name: 'upload_image',
    description: 'Upload an image into the current page flow.',
  },
  {
    name: 'read_console_messages',
    description: 'Read browser console messages.',
  },
  {
    name: 'read_network_requests',
    description: 'Read captured network requests.',
  },
  {
    name: 'shortcuts_list',
    description: 'List extension/browser shortcuts.',
  },
  {
    name: 'shortcuts_execute',
    description: 'Execute a configured extension/browser shortcut.',
  },
  {
    name: 'gif_creator',
    description: 'Create or manage simple browser recordings.',
  },
  {
    name: 'update_plan',
    description: 'Update an in-browser action plan.',
  },
]

export function createClaudeForChromeMcpServer(context: ClaudeForChromeContext) {
  let closed = false
  const handlers = new Map<unknown, unknown>()

  return {
    async connect() {
      context.logger?.warn(
        'Claude in Chrome MCP is running with a restored compatibility shim; browser actions are not available in this workspace.',
      )
    },
    setRequestHandler(schema: unknown, handler: unknown) {
      handlers.set(schema, handler)
    },
    async close() {
      closed = true
      handlers.clear()
      context.logger?.info(
        'Claude in Chrome MCP shim closed.',
      )
    },
    get isClosed() {
      return closed
    },
  }
}
