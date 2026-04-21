import { execFileSync } from 'child_process'

type DisplayGeometry = {
  id: number
  width: number
  height: number
  scaleFactor: number
  originX: number
  originY: number
}

type InstalledApp = {
  bundleId: string
  displayName: string
  path?: string
}

type RunningApp = {
  bundleId: string
  displayName: string
}

type ScreenshotResult = {
  base64: string
  width: number
  height: number
  displayWidth: number
  displayHeight: number
  displayId: number
  originX: number
  originY: number
}

const BLANK_JPEG_BASE64 =
  '/9j/4AAQSkZJRgABAQAAAQABAAD/2wCEAAkGBxAQEBUQEBAVFRUVFRUVFRUVFRUVFRUVFRUXFhUVFRUYHSggGBolHRUVITEhJSkrLi4uFx8zODMsNygtLisBCgoKDg0OGhAQGi0mHyYtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLf/AABEIAAEAAQMBIgACEQEDEQH/xAAXAAADAQAAAAAAAAAAAAAAAAAAAQID/8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAwDAQACEAMQAAAB6gD/xAAVEAEBAAAAAAAAAAAAAAAAAAABAP/aAAgBAQABBQJf/8QAFBEBAAAAAAAAAAAAAAAAAAAAEP/aAAgBAwEBPwEf/8QAFBEBAAAAAAAAAAAAAAAAAAAAEP/aAAgBAgEBPwEf/8QAFBABAAAAAAAAAAAAAAAAAAAAEP/aAAgBAQAGPwJf/8QAFBABAAAAAAAAAAAAAAAAAAAAEP/aAAgBAQABPyFf/9k='

function safeExec(
  file: string,
  args: string[],
): { ok: true; stdout: string } | { ok: false } {
  try {
    const stdout = execFileSync(file, args, {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    })
    return { ok: true, stdout: stdout.trim() }
  } catch {
    return { ok: false }
  }
}

function getDefaultDisplay(): DisplayGeometry {
  return {
    id: 0,
    width: 1440,
    height: 900,
    scaleFactor: 1,
    originX: 0,
    originY: 0,
  }
}

function getDisplay(displayId?: number): DisplayGeometry {
  const display = getDefaultDisplay()
  if (displayId === undefined || displayId === display.id) {
    return display
  }
  return { ...display, id: displayId }
}

function buildScreenshotResult(
  width: number,
  height: number,
  displayId?: number,
): ScreenshotResult {
  const display = getDisplay(displayId)
  return {
    base64: BLANK_JPEG_BASE64,
    width,
    height,
    displayWidth: display.width,
    displayHeight: display.height,
    displayId: display.id,
    originX: display.originX,
    originY: display.originY,
  }
}

function openBundle(bundleId: string): void {
  if (!bundleId) return
  safeExec('open', ['-b', bundleId])
}

function getRunningApps(): RunningApp[] {
  const result = safeExec('osascript', [
    '-e',
    'tell application "System Events" to get the name of every application process',
  ])
  if (!result.ok || result.stdout.length === 0) return []
  return result.stdout
    .split(/\s*,\s*/u)
    .map(name => name.trim())
    .filter(Boolean)
    .map(name => ({
      bundleId: '',
      displayName: name,
    }))
}

function createInstalledApp(displayName: string): InstalledApp {
  return {
    bundleId: '',
    displayName,
  }
}

export type ComputerUseAPI = {
  _drainMainRunLoop(): void
  tcc: {
    checkAccessibility(): boolean
    checkScreenRecording(): boolean
  }
  hotkey: {
    registerEscape(onEscape: () => void): boolean
    unregister(): void
    notifyExpectedEscape(): void
  }
  display: {
    getSize(displayId?: number): DisplayGeometry
    listAll(): DisplayGeometry[]
  }
  apps: {
    prepareDisplay(
      allowlistBundleIds: string[],
      surrogateHost: string,
      displayId?: number,
    ): Promise<{ hidden: string[]; activated?: string }>
    previewHideSet(
      allowlistBundleIds: string[],
      displayId?: number,
    ): Promise<Array<{ bundleId: string; displayName: string }>>
    findWindowDisplays(
      bundleIds: string[],
    ): Promise<Array<{ bundleId: string; displayIds: number[] }>>
    appUnderPoint(
      x: number,
      y: number,
    ): Promise<{ bundleId: string; displayName: string } | null>
    listInstalled(): Promise<InstalledApp[]>
    iconDataUrl(path: string): string | null
    listRunning(): Promise<RunningApp[]>
    open(bundleId: string): Promise<void>
    unhide(bundleIds: string[]): Promise<void>
  }
  screenshot: {
    captureExcluding(
      allowedBundleIds: string[],
      quality: number,
      width: number,
      height: number,
      displayId?: number,
    ): Promise<ScreenshotResult>
    captureRegion(
      allowedBundleIds: string[],
      x: number,
      y: number,
      width: number,
      height: number,
      outW: number,
      outH: number,
      quality: number,
      displayId?: number,
    ): Promise<ScreenshotResult>
  }
  resolvePrepareCapture(
    allowedBundleIds: string[],
    surrogateHost: string,
    quality: number,
    targetW: number,
    targetH: number,
    preferredDisplayId?: number,
    autoResolve?: boolean,
    doHide?: boolean,
  ): Promise<
    ScreenshotResult & {
      hidden: string[]
      activated?: string
      autoResolved: boolean
    }
  >
}

const stub: ComputerUseAPI = {
  _drainMainRunLoop() {},
  tcc: {
    checkAccessibility() {
      return false
    },
    checkScreenRecording() {
      return false
    },
  },
  hotkey: {
    registerEscape(_onEscape: () => void) {
      return false
    },
    unregister() {},
    notifyExpectedEscape() {},
  },
  display: {
    getSize(displayId?: number) {
      return getDisplay(displayId)
    },
    listAll() {
      return [getDefaultDisplay()]
    },
  },
  apps: {
    async prepareDisplay(
      _allowlistBundleIds: string[],
      _surrogateHost: string,
      _displayId?: number,
    ) {
      return { hidden: [] as string[] }
    },
    async previewHideSet(
      _allowlistBundleIds: string[],
      _displayId?: number,
    ) {
      return []
    },
    async findWindowDisplays(bundleIds: string[]) {
      return bundleIds.map(bundleId => ({
        bundleId,
        displayIds: [],
      }))
    },
    async appUnderPoint(_x: number, _y: number) {
      return null
    },
    async listInstalled() {
      return getRunningApps().map(app => createInstalledApp(app.displayName))
    },
    iconDataUrl(_path: string) {
      return null
    },
    async listRunning() {
      return getRunningApps()
    },
    async open(bundleId: string) {
      openBundle(bundleId)
    },
    async unhide(_bundleIds: string[]) {},
  },
  screenshot: {
    async captureExcluding(
      _allowedBundleIds: string[],
      _quality: number,
      width: number,
      height: number,
      displayId?: number,
    ) {
      return buildScreenshotResult(width, height, displayId)
    },
    async captureRegion(
      _allowedBundleIds: string[],
      _x: number,
      _y: number,
      _width: number,
      _height: number,
      outW: number,
      outH: number,
      _quality: number,
      displayId?: number,
    ) {
      return buildScreenshotResult(outW, outH, displayId)
    },
  },
  async resolvePrepareCapture(
    _allowedBundleIds: string[],
    _surrogateHost: string,
    _quality: number,
    targetW: number,
    targetH: number,
    preferredDisplayId?: number,
    autoResolve = false,
    _doHide = false,
  ) {
    return {
      ...buildScreenshotResult(targetW, targetH, preferredDisplayId),
      hidden: [],
      autoResolved: autoResolve,
    }
  },
}

export default stub
