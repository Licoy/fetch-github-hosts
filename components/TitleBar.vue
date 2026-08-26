<template>
  <div
    class="title-bar-drag flex h-[36px] flex-shrink-0 items-center justify-between border-b border-[var(--fgh-border)] bg-[var(--fgh-card-bg)] select-none transition-[padding] duration-150"
    :class="barPaddingClass"
    :data-tauri-drag-region="isMacos ? true : undefined"
    @mousedown="onTitleBarMouseDown"
    @dblclick="onTitleBarDoubleClick"
  >
    <div class="flex min-w-0 items-center gap-2">
      <img src="/logo.png" alt="" class="h-5 w-5 flex-shrink-0" />
      <span class="truncate text-[13px] font-semibold opacity-90">
        {{ $t('app.title') }} - {{ versionLabel }}
      </span>
    </div>

    <div class="flex items-center">
      <div class="title-bar-nodrag flex items-center gap-0.5">
        <UDropdownMenu :items="langMenuItems">
          <UButton
            icon="i-heroicons-language"
            color="neutral"
            variant="ghost"
            size="xs"
          >
            <span class="text-xs font-medium">{{ currentLangLabel }}</span>
          </UButton>
        </UDropdownMenu>

        <UDropdownMenu :items="colorModeMenuItems">
          <UButton
            :icon="colorModeIcon"
            color="neutral"
            variant="ghost"
            size="xs"
          />
        </UDropdownMenu>

        <UButton
          color="neutral"
          variant="ghost"
          size="xs"
          title="GitHub"
          @click="openGithub"
        >
          <div class="flex items-center gap-1">
            <UIcon name="i-simple-icons-github" class="text-base" />
            <span class="inline-flex items-center gap-0.5 rounded-full bg-[var(--fgh-primary)] px-1.5 py-0 text-[10px] font-medium leading-4 text-white">
              <UIcon name="i-heroicons-star-solid" class="text-[10px]" />
              Star
            </span>
          </div>
        </UButton>
      </div>

      <WindowCaptionButtons
        v-if="usesCustomCaption"
        :close-tip="closeButtonTip"
        @close="handleClose"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { titleBarPaddingClass } from '~/composables/usePlatform'

const CLOSE_TIP_KEY = 'fgh_close_tip'

const { t, locale, setLocale } = useI18n()
const colorMode = useColorMode()
const toast = useToast()
const { isMacos, isWindows, usesCustomCaption } = usePlatform()
const { safeOpenUrl, windowHide, windowClose, safeListen, syncWindowsChrome } = useTauri()
const { versionLabel, loadVersion } = useAppVersion()
const { config, loadConfig } = useConfig()
const { onTitleBarMouseDown, onTitleBarDoubleClick } = useWindowDrag()
const { isFullscreen } = useWindowFullscreen()

const barPaddingClass = computed(() => titleBarPaddingClass({
  isMacos,
  usesCustomCaption,
  isFullscreen: isFullscreen.value,
}))

const closeButtonTip = computed(() =>
  config.value.close_to_tray ? t('window.closeToTrayTip') : t('window.closeQuitTip'),
)

const isDark = computed(() => colorMode.value === 'dark')

const langMap: Record<string, string> = {
  'zh-CN': '中文',
  'en-US': 'EN',
  'ja-JP': '日本語',
}

const currentLangLabel = computed(() => langMap[locale.value] || locale.value)

const colorModeIcon = computed(() => {
  if (colorMode.preference === 'system') return 'i-heroicons-computer-desktop'
  return isDark.value ? 'i-heroicons-moon' : 'i-heroicons-sun'
})

const langMenuItems = computed(() => [
  [
    {
      label: '简体中文',
      icon: locale.value === 'zh-CN' ? 'i-heroicons-check' : undefined,
      onSelect: () => setLocale('zh-CN'),
    },
    {
      label: 'English',
      icon: locale.value === 'en-US' ? 'i-heroicons-check' : undefined,
      onSelect: () => setLocale('en-US'),
    },
    {
      label: '日本語',
      icon: locale.value === 'ja-JP' ? 'i-heroicons-check' : undefined,
      onSelect: () => setLocale('ja-JP'),
    },
  ],
])

const colorModeMenuItems = computed(() => [
  [
    {
      label: t('colorMode.system'),
      icon: colorMode.preference === 'system' ? 'i-heroicons-check' : 'i-heroicons-computer-desktop',
      onSelect: () => { colorMode.preference = 'system' },
    },
    {
      label: t('colorMode.light'),
      icon: colorMode.preference === 'light' ? 'i-heroicons-check' : 'i-heroicons-sun',
      onSelect: () => { colorMode.preference = 'light' },
    },
    {
      label: t('colorMode.dark'),
      icon: colorMode.preference === 'dark' ? 'i-heroicons-check' : 'i-heroicons-moon',
      onSelect: () => { colorMode.preference = 'dark' },
    },
  ],
])

function cardBg(): string {
  if (typeof getComputedStyle === 'undefined') {
    return isDark.value ? '#1f2437' : '#ffffff'
  }
  const value = getComputedStyle(document.documentElement).getPropertyValue('--fgh-card-bg').trim()
  return value || (isDark.value ? '#1f2437' : '#ffffff')
}

function showClosedToTrayTip(): boolean {
  if (typeof sessionStorage === 'undefined') return false
  if (sessionStorage.getItem(CLOSE_TIP_KEY)) return false
  toast.add({ title: t('window.closedToTray'), color: 'info' })
  sessionStorage.setItem(CLOSE_TIP_KEY, '1')
  return true
}

async function openGithub() {
  await safeOpenUrl('https://github.com/Licoy/fetch-github-hosts')
}

async function handleClose() {
  if (config.value.close_to_tray) {
    if (showClosedToTrayTip()) {
      await new Promise((r) => setTimeout(r, 600))
    }
    await windowHide()
    return
  }
  await windowClose()
}

let unlistenClosedToTray: (() => void) | undefined

watch(
  [isDark, () => colorMode.value],
  () => {
    if (!isWindows) return
    void syncWindowsChrome(isDark.value, cardBg())
  },
  { immediate: true },
)

onMounted(() => {
  loadVersion()
  loadConfig()
  void (async () => {
    unlistenClosedToTray = await safeListen('window-closed-to-tray', () => {
      showClosedToTrayTip()
    })
  })()
})

onUnmounted(() => {
  unlistenClosedToTray?.()
})
</script>
