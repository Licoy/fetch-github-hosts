import { coversMonitor } from './usePlatform'

function isTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__
}

export function useTauri() {
  /**
   * Safe invoke that returns undefined when not in Tauri
   */
  async function safeInvoke<T>(cmd: string, args?: Record<string, any>): Promise<T | undefined> {
    if (!isTauri()) {
      console.warn(`[FGH] Not in Tauri, skipping invoke: ${cmd}`)
      return undefined
    }
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke<T>(cmd, args)
  }

  /**
   * Safe listen that returns a noop unlisten when not in Tauri
   */
  async function safeListen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void> {
    if (!isTauri()) {
      console.warn(`[FGH] Not in Tauri, skipping listen: ${event}`)
      return () => {}
    }
    const { listen } = await import('@tauri-apps/api/event')
    return listen<T>(event, handler)
  }

  /**
   * Safe open URL
   */
  async function safeOpenUrl(url: string): Promise<void> {
    if (isTauri()) {
      try {
        const { open } = await import('@tauri-apps/plugin-shell')
        await open(url)
        return
      } catch { }
    }
    window.open(url, '_blank')
  }

  /**
   * Window control: minimize
   */
  async function windowMinimize(): Promise<void> {
    if (!isTauri()) return
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().minimize()
    } catch {}
  }

  /**
   * Window control: toggle maximize
   */
  async function windowToggleMaximize(): Promise<void> {
    if (!isTauri()) return
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().toggleMaximize()
    } catch {}
  }

  /**
   * Window control: close.
   * Backend CloseRequested respects config.close_to_tray:
   * true → prevent close and hide to tray; false → allow quit.
   */
  async function windowClose(): Promise<void> {
    if (!isTauri()) return
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().close()
    } catch {}
  }

  /**
   * Window control: hide to tray (does not destroy the window)
   */
  async function windowHide(): Promise<void> {
    if (!isTauri()) return
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().hide()
    } catch {}
  }

  async function quitApp(): Promise<void> {
    await safeInvoke('quit_app')
  }

  async function windowStartDragging(): Promise<void> {
    if (!isTauri()) return
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().startDragging()
    } catch {}
  }

  async function windowIsMaximized(): Promise<boolean> {
    if (!isTauri()) return false
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      return await getCurrentWindow().isMaximized()
    } catch {
      return false
    }
  }

  async function windowIsFullscreen(): Promise<boolean> {
    if (!isTauri()) return false
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      return await getCurrentWindow().isFullscreen()
    } catch {
      return false
    }
  }

  /**
   * macOS Overlay windows can report isFullscreen()=false for a beat
   * (or entirely) while already in a native fullscreen Space. Fall back
   * to comparing the outer size against the current monitor.
   */
  async function windowLooksFullscreen(): Promise<boolean> {
    if (!isTauri()) return false
    try {
      const { getCurrentWindow, currentMonitor } = await import('@tauri-apps/api/window')
      const win = getCurrentWindow()
      if (await win.isFullscreen()) return true
      const [outer, inner, monitor] = await Promise.all([
        win.outerSize(),
        win.innerSize(),
        currentMonitor(),
      ])
      if (monitor && (coversMonitor(outer, monitor.size) || coversMonitor(inner, monitor.size))) {
        return true
      }
      if (typeof screen !== 'undefined' && screen.width > 0 && screen.height > 0) {
        return (
          Math.abs(window.innerWidth - screen.width) <= 8
          && Math.abs(window.innerHeight - screen.height) <= 8
        ) || (
          Math.abs(window.outerWidth - screen.width) <= 8
          && Math.abs(window.outerHeight - screen.height) <= 8
        )
      }
      return false
    } catch {
      if (typeof screen !== 'undefined' && screen.width > 0 && screen.height > 0) {
        return Math.abs(window.innerWidth - screen.width) <= 8
          && Math.abs(window.innerHeight - screen.height) <= 8
      }
      return false
    }
  }

  async function listenWindowResized(handler: () => void): Promise<() => void> {
    if (!isTauri()) return () => {}
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      return await getCurrentWindow().onResized(() => {
        handler()
      })
    } catch {
      return () => {}
    }
  }

  async function syncWindowsChrome(dark: boolean, bg: string): Promise<void> {
    await safeInvoke('sync_windows_chrome', { dark, bg })
  }

  return {
    isTauri: isTauri(),
    safeInvoke,
    safeListen,
    safeOpenUrl,
    windowMinimize,
    windowToggleMaximize,
    windowClose,
    windowHide,
    windowStartDragging,
    windowIsMaximized,
    windowIsFullscreen,
    windowLooksFullscreen,
    listenWindowResized,
    syncWindowsChrome,
    quitApp,
  }
}
