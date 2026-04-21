export type KeybindingContextName = string
export type KeybindingAction = string
export type ParsedKeystroke = {
  key?: string
  ctrl?: boolean
  alt?: boolean
  shift?: boolean
  meta?: boolean
}
export type ParsedBinding = {
  action: string
  keys: ParsedKeystroke[]
}
export type KeybindingBlock = {
  context?: KeybindingContextName
  bindings?: ParsedBinding[]
}
