export function useWindowDrag() {
  const { usesCustomCaption } = usePlatform()
  const { windowStartDragging, windowToggleMaximize } = useTauri()
  const dragTimer = ref<ReturnType<typeof setTimeout> | null>(null)

  function clearDragTimer() {
    if (dragTimer.value) {
      clearTimeout(dragTimer.value)
      dragTimer.value = null
    }
  }

  function onTitleBarMouseDown(e: MouseEvent) {
    const target = e.target as HTMLElement | null
    if (!target) return
    if (target.closest('button') || target.closest('a') || target.closest('.title-bar-nodrag')) {
      return
    }
    e.preventDefault()
    if (usesCustomCaption) {
      clearDragTimer()
      dragTimer.value = setTimeout(() => {
        void windowStartDragging()
      }, 200)
      return
    }
    void windowStartDragging()
  }

  function onTitleBarDoubleClick() {
    if (!usesCustomCaption) return
    clearDragTimer()
    void windowToggleMaximize()
  }

  onUnmounted(() => {
    clearDragTimer()
  })

  return {
    onTitleBarMouseDown,
    onTitleBarDoubleClick,
  }
}
