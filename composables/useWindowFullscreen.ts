const FULLSCREEN_RECHECK_MS = [0, 50, 160, 400]

function applyFullscreenAttr(active: boolean) {
  if (typeof document === 'undefined') return
  if (active) document.documentElement.dataset.windowFullscreen = ''
  else delete document.documentElement.dataset.windowFullscreen
}

export function useWindowFullscreen() {
  const { isMacos } = usePlatform()
  const { windowLooksFullscreen, listenWindowResized, safeListen } = useTauri()
  const isFullscreen = ref(false)
  const timers: number[] = []
  let unlistenResized: (() => void) | undefined
  let unlistenEvent: (() => void) | undefined

  function setFullscreen(active: boolean) {
    if (isFullscreen.value === active) {
      applyFullscreenAttr(active)
      return
    }
    isFullscreen.value = active
    applyFullscreenAttr(active)
  }

  async function syncFullscreen() {
    if (!isMacos) {
      setFullscreen(false)
      return
    }
    setFullscreen(await windowLooksFullscreen())
  }

  function clearTimers() {
    for (const id of timers) clearTimeout(id)
    timers.length = 0
  }

  function syncFullscreenSoon() {
    clearTimers()
    for (const ms of FULLSCREEN_RECHECK_MS) {
      const id = window.setTimeout(() => {
        void syncFullscreen()
      }, ms)
      timers.push(id)
    }
  }

  onMounted(() => {
    if (!isMacos) return
    void (async () => {
      await syncFullscreen()
      unlistenResized = await listenWindowResized(() => {
        syncFullscreenSoon()
      })
      unlistenEvent = await safeListen<boolean>('window-fullscreen-changed', (event) => {
        if (event.payload) {
          setFullscreen(true)
          return
        }
        void syncFullscreen()
      })
      window.addEventListener('resize', syncFullscreenSoon)
    })()
  })

  onUnmounted(() => {
    clearTimers()
    unlistenResized?.()
    unlistenEvent?.()
    if (typeof window !== 'undefined') {
      window.removeEventListener('resize', syncFullscreenSoon)
    }
    applyFullscreenAttr(false)
  })

  return { isFullscreen }
}
