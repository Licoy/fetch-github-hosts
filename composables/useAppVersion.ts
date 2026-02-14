const _version = ref('0.0.0')
const _loaded = ref(false)

export function useAppVersion() {
  const { safeInvoke } = useTauri()

  async function loadVersion() {
    if (_loaded.value) return
    try {
      const v = await safeInvoke<string>('get_version')
      if (v) {
        _version.value = v
      }
    } catch {
      // fallback
    }
    _loaded.value = true
  }

  const versionLabel = computed(() => `V${_version.value}`)

  return {
    version: _version,
    versionLabel,
    loadVersion,
  }
}
