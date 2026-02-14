<template>
  <div class="h-screen flex flex-col bg-[var(--fgh-bg)] text-[var(--fgh-text)]">
    <!-- Title Bar (draggable for Tauri, custom window controls) -->
    <div
      data-tauri-drag-region
      class="flex items-center justify-between px-4 py-2 bg-[var(--fgh-card-bg)] border-b border-[var(--fgh-border)] select-none"
    >
      <!-- Left: macOS-style traffic lights -->
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-[7px] group mr-2">
          <button
            class="window-btn window-btn-close"
            title="Close"
            @click="handleClose"
          >
            <svg class="window-btn-icon" viewBox="0 0 12 12"><path d="M3.5 3.5l5 5M8.5 3.5l-5 5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
          </button>
          <button
            class="window-btn window-btn-minimize"
            title="Minimize"
            @click="handleMinimize"
          >
            <svg class="window-btn-icon" viewBox="0 0 12 12"><path d="M2.5 6h7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
          </button>
          <button
            class="window-btn window-btn-maximize"
            title="Maximize"
            @click="handleMaximize"
          >
            <svg class="window-btn-icon" viewBox="0 0 12 12"><rect x="2.5" y="2.5" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1" fill="none"/></svg>
          </button>
        </div>
        <img src="/logo.png" alt="logo" class="w-5 h-5" />
        <span class="text-sm font-semibold opacity-90">
          {{ $t('app.title') }} - {{ versionLabel }}
        </span>
      </div>
      <!-- Right side controls -->
      <div class="flex items-center gap-1">
        <!-- Language Switcher Dropdown -->
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

        <!-- Color Mode Dropdown -->
        <UDropdownMenu :items="colorModeMenuItems">
          <UButton
            :icon="colorModeIcon"
            color="neutral"
            variant="ghost"
            size="xs"
          />
        </UDropdownMenu>

        <!-- GitHub Link with Star Badge -->
        <UButton
          color="neutral"
          variant="ghost"
          size="xs"
          title="GitHub"
          @click="openGithub"
        >
          <div class="flex items-center gap-1">
            <UIcon name="i-simple-icons-github" class="text-base" />
            <span class="inline-flex items-center gap-0.5 text-[10px] px-1.5 py-0 rounded-full bg-[var(--fgh-primary)] text-white font-medium leading-4">
              <UIcon name="i-heroicons-star-solid" class="text-[10px]" />
              Star
            </span>
          </div>
        </UButton>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 overflow-hidden p-4">
      <UTabs
        :items="tabItems"
        v-model="activeTab"
        class="h-full"
        :ui="{
          list: 'bg-[var(--fgh-card-bg)] border border-[var(--fgh-border)] rounded-lg',
          trigger: 'data-[state=active]:bg-[#009966] data-[state=active]:text-white text-[var(--fgh-text-muted)] transition-all',
        }"
      >
        <template #content="{ item }">
          <div class="mt-4 h-full">
            <ClientMode v-if="item.value === 'client'" />
            <ServerMode v-else-if="item.value === 'server'" />
            <AboutPanel v-else-if="item.value === 'about'" />
          </div>
        </template>
      </UTabs>
    </div>
  </div>
</template>

<script setup lang="ts">
const { t, locale, setLocale } = useI18n()
const colorMode = useColorMode()
const { safeOpenUrl, windowMinimize, windowToggleMaximize, windowClose } = useTauri()
const { versionLabel, loadVersion } = useAppVersion()

const activeTab = ref('client')

onMounted(() => {
  loadVersion()
})

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

const tabItems = computed(() => [
  { label: t('tabs.client'), value: 'client' },
  { label: t('tabs.server'), value: 'server' },
  { label: t('tabs.about'), value: 'about' },
])

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

async function openGithub() {
  await safeOpenUrl('https://github.com/Licoy/fetch-github-hosts')
}

async function handleClose() {
  await windowClose()
}

async function handleMinimize() {
  await windowMinimize()
}

async function handleMaximize() {
  await windowToggleMaximize()
}
</script>

<style scoped>
/* macOS-style traffic light buttons */
.window-btn {
  width: 13px;
  height: 13px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  transition: filter 0.15s;
}
.window-btn-icon {
  width: 8px;
  height: 8px;
  opacity: 0;
  color: rgba(0,0,0,0.5);
  transition: opacity 0.15s;
}
.group:hover .window-btn-icon {
  opacity: 1;
}
.window-btn-close {
  background-color: #ff5f57;
}
.window-btn-close:hover {
  filter: brightness(0.85);
}
.window-btn-minimize {
  background-color: #febc2e;
}
.window-btn-minimize:hover {
  filter: brightness(0.85);
}
.window-btn-maximize {
  background-color: #28c840;
}
.window-btn-maximize:hover {
  filter: brightness(0.85);
}
</style>
