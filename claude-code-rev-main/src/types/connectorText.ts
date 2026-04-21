export type ConnectorTextBlock = {
  type?: string
  text?: string
  [key: string]: unknown
}

export function isConnectorTextBlock(value: unknown): value is ConnectorTextBlock {
  return !!value && typeof value === 'object' && 'text' in (value as Record<string, unknown>)
}
