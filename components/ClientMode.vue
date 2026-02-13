<template>
  <div class="flex flex-col gap-4 h-full">
    <!-- Config Form -->
    <div class="bg-[var(--fgh-card-bg)] rounded-lg p-4 border border-[var(--fgh-border)]">
      <div class="grid grid-cols-1 gap-4">
        <!-- Interval -->
        <div class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('client.interval') }}</label>
          <UInput
            v-model="config.interval"
            type="number"
            :min="1"
            :disabled="isRunning"
            class="flex-1"
          />
        </div>

        <!-- Hosts Origin Method -->
        <div class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('client.hostsOrigin') }}</label>
          <URadioGroup
            v-model="config.originMethod"
            :items="originMethodOptions"
            :disabled="isRunning"
            orientation="horizontal"
            :ui="{ fieldset: 'flex gap-4' }"
          />
        </div>

        <!-- Official Source Select -->
        <div v-if="config.originMethod === 'official'" class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('client.hostsOrigin') }}</label>
          <USelect
            v-model="config.selectOrigin"
            :items="hostsOriginOptions"
            :disabled="isRunning"
            class="flex-1"
          />
        </div>

        <!-- Custom URL -->
        <div v-else class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('client.remoteUrl') }}</label>
          <UInput
            v-model="config.customUrl"
            :disabled="isRunning"
            class="flex-1"
            placeholder="https://hosts.gitcdn.top/hosts.txt"
          />
        </div>

        <!-- Auto Fetch -->
        <div class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('client.autoFetch') }}</label>
          <USwitch
            v-model="config.autoFetch"
            @update:model-value="onAutoFetchChange"
          />
        </div>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex justify-center gap-3">
      <UButton
        v-if="!isRunning"
        :label="$t('common.start')"
        icon="i-heroicons-play"
        color="primary"
        @click="startFetch"
      />
      <UButton
        v-else
        :label="$t('common.stop')"
        icon="i-heroicons-stop"
        color="error"
        @click="stopFetch"
      />
      <UButton
        :label="$t('client.clearHosts')"
        icon="i-heroicons-trash"
        color="error"
        variant="soft"
        @click="clearHosts"
      />
      <UButton
        :label="$t('client.flushDns')"
        icon="i-heroicons-arrow-path"
        color="neutral"
        variant="soft"
        @click="flushDns"
      />
    </div>

    <!-- Log Viewer -->
    <LogViewer :logs="logs" class="flex-1 min-h-0" />
  </div>
</template>

<script setup lang="ts">
const { safeInvoke, safeListen } = useTauri()
const { t } = useI18n()
const toast = useToast()

const isRunning = ref(false)
const logs = ref<string[]>([])

const hostsOrigins: Record<string, string> = {
  FetchGithubHosts: 'https://hosts.gitcdn.top/hosts.txt',
  Github520: 'https://raw.hellogithub.com/hosts',
}

const config = reactive({
  interval: 60,
  originMethod: 'official',
  selectOrigin: 'FetchGithubHosts',
  customUrl: '',
  autoFetch: false,
})

const originMethodOptions = computed(() => [
  { label: t('client.officialOrigin'), value: 'official' },
  { label: t('client.customOrigin'), value: 'custom' },
])

const hostsOriginOptions = Object.keys(hostsOrigins).map(k => ({
  label: k,
  value: k,
}))

function addLog(msg: string) {
  const now = new Date().toLocaleString()
  logs.value.unshift(`[${now}] ${msg}`)
}

async function startFetch() {
  const interval = Number(config.interval)
  if (!Number.isInteger(interval) || interval < 1) {
    toast.add({ title: t('common.intervalNeedInt'), color: 'warning' })
    return
  }

  const url = config.originMethod === 'official'
    ? hostsOrigins[config.selectOrigin]
    : config.customUrl

  if (!url) {
    toast.add({ title: t('client.remoteUrl'), color: 'warning' })
    return
  }

  try {
    await safeInvoke('start_client', { url, interval })
    isRunning.value = true
    addLog(t('client.remoteUrlLog', { url }))
    await saveConfig()
  } catch (e: any) {
    addLog(t('client.fetchFail', { error: e.toString() }))
  }
}

async function stopFetch() {
  try {
    await safeInvoke('stop_client')
    isRunning.value = false
    addLog(t('client.fetchStop'))
  } catch (e: any) {
    addLog(e.toString())
  }
}

async function clearHosts() {
  try {
    await safeInvoke('clean_hosts')
    toast.add({ title: t('client.clearSuccess'), color: 'success' })
  } catch (e: any) {
    toast.add({ title: t('client.clearFail') + ': ' + e.toString(), color: 'error' })
  }
}

async function flushDns() {
  try {
    const result = await safeInvoke<string>('flush_dns')
    toast.add({ title: t('client.flushDnsSuccess'), color: 'success' })
    addLog(result || t('client.flushDnsSuccess'))
  } catch (e: any) {
    const msg = e?.toString() || ''
    if (msg.includes('USER_CANCELLED')) {
      toast.add({ title: t('common.cancelled'), color: 'warning' })
    } else {
      toast.add({ title: t('client.flushDnsFail') + ': ' + msg, color: 'error' })
      addLog(t('client.flushDnsFail') + ': ' + msg)
    }
  }
}

function onAutoFetchChange(val: boolean) {
  config.autoFetch = val
  saveConfig()
  toast.add({ title: t('client.autoFetchChanged'), color: 'info' })
}

async function saveConfig() {
  try {
    await safeInvoke('save_config', {
      config: {
        lang: 'zh-CN',
        client: {
          interval: Number(config.interval),
          method: config.originMethod,
          select_origin: config.selectOrigin,
          custom_url: config.customUrl,
          auto_fetch: config.autoFetch,
        },
        server: { interval: 60, port: 9898 },
      },
    })
  } catch (_) { }
}

async function loadConfig() {
  try {
    const cfg: any = await safeInvoke('load_config')
    if (cfg) {
      config.interval = cfg.client?.interval ?? 60
      config.originMethod = cfg.client?.method ?? 'official'
      config.selectOrigin = cfg.client?.select_origin ?? 'FetchGithubHosts'
      config.customUrl = cfg.client?.custom_url ?? ''
      config.autoFetch = cfg.client?.auto_fetch ?? false
    }
  } catch (_) { }
}

onMounted(async () => {
  // Listen for log events from backend
  await safeListen<{ message: string }>('client-log', (event) => {
    addLog(event.payload.message)
  })
  await loadConfig()
  // Auto fetch on startup if enabled
  if (config.autoFetch) {
    startFetch()
  }
})
</script>
