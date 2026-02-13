/**
 * Check if we're running inside a Tauri webview
 */
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
   * Window control: close
   */
  async function windowClose(): Promise<void> {
    if (!isTauri()) return
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().close()
    } catch {}
  }

  return { isTauri: isTauri(), safeInvoke, safeListen, safeOpenUrl, windowMinimize, windowToggleMaximize, windowClose }
}
