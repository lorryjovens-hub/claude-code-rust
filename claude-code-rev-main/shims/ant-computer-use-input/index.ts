type MouseButton = 'left' | 'right' | 'middle'
type MouseAction = 'press' | 'release' | 'click'
type ScrollAxis = 'vertical' | 'horizontal'

export type FrontmostAppInfo = {
  bundleId?: string
  appName?: string
}

export type ComputerUseInputAPI = {
  moveMouse(x: number, y: number, smooth?: boolean): Promise<void>
  mouseLocation(): Promise<{ x: number; y: number }>
  key(key: string, action?: 'press' | 'release' | 'click'): Promise<void>
  keys(keys: string[]): Promise<void>
  leftClick(): Promise<void>
  rightClick(): Promise<void>
  doubleClick(): Promise<void>
  middleClick(): Promise<void>
  dragMouse(x: number, y: number): Promise<void>
  scroll(x: number, y: number): Promise<void>
  type(text: string): Promise<void>
  mouseButton(
    button: MouseButton,
    action?: MouseAction,
    count?: number,
  ): Promise<void>
  mouseScroll(amount: number, axis?: ScrollAxis): Promise<void>
  typeText(text: string): Promise<void>
  getFrontmostAppInfo(): FrontmostAppInfo | null
}

export type ComputerUseInput =
  | ({ isSupported: false } & Partial<ComputerUseInputAPI>)
  | ({ isSupported: true } & ComputerUseInputAPI)

let cursor = { x: 0, y: 0 }

async function noOp(): Promise<void> {}

const supported: ComputerUseInput = {
  isSupported: process.platform === 'darwin',
  async moveMouse(x: number, y: number): Promise<void> {
    cursor = { x, y }
  },
  async mouseLocation(): Promise<{ x: number; y: number }> {
    return cursor
  },
  async key(_key: string, _action: 'press' | 'release' | 'click' = 'click') {
    await noOp()
  },
  async keys(_keys: string[]) {
    await noOp()
  },
  async leftClick() {
    await noOp()
  },
  async rightClick() {
    await noOp()
  },
  async doubleClick() {
    await noOp()
  },
  async middleClick() {
    await noOp()
  },
  async dragMouse(x: number, y: number) {
    cursor = { x, y }
  },
  async scroll(_x: number, _y: number) {
    await noOp()
  },
  async type(_text: string) {
    await noOp()
  },
  async mouseButton(
    _button: MouseButton,
    _action: MouseAction = 'click',
    _count = 1,
  ) {
    await noOp()
  },
  async mouseScroll(_amount: number, _axis: ScrollAxis = 'vertical') {
    await noOp()
  },
  async typeText(_text: string) {
    await noOp()
  },
  getFrontmostAppInfo(): FrontmostAppInfo | null {
    return null
  },
}

export default supported
