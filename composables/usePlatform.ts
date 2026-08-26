export type AppOs = 'macos' | 'windows' | 'linux' | 'web'

export function detectAppOs(ua?: string): AppOs {
  const source = ua ?? (typeof navigator === 'undefined' ? '' : navigator.userAgent)
  if (/Windows/i.test(source)) return 'windows'
  if (/Mac OS X|Macintosh/i.test(source)) return 'macos'
  if (/Linux|X11/i.test(source)) return 'linux'
  return 'web'
}

export function usesCustomCaption(os: AppOs): boolean {
  return os === 'windows' || os === 'linux'
}

/** macOS overlay traffic lights occupy ~72px until the window is fullscreen. */
export function titleBarPaddingClass(input: {
  isMacos: boolean
  usesCustomCaption: boolean
  isFullscreen: boolean
}): string {
  // Left inset on macOS is applied in CSS so it can follow data-window-fullscreen.
  if (input.isMacos) return 'pr-3'
  if (input.usesCustomCaption) return 'pl-3 pr-0'
  return 'px-3'
}

/** True when traffic lights are gone (native fullscreen), not merely zoomed. */
export function coversMonitor(
  outer: { width: number; height: number },
  monitor: { width: number; height: number },
  slop = 8,
): boolean {
  return outer.width >= monitor.width - slop && outer.height >= monitor.height - slop
}

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__
}

export function usePlatform() {
  const os = detectAppOs()
  const inTauri = isTauriRuntime()
  return {
    os,
    inTauri,
    usesCustomCaption: inTauri && usesCustomCaption(os),
    isMacos: inTauri && os === 'macos',
    isWindows: inTauri && os === 'windows',
    isLinux: inTauri && os === 'linux',
    isWeb: !inTauri || os === 'web',
  }
}
