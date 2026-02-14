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
        :loading="isLoading"
        :disabled="isLoading"
        @click="startFetch"
      />
      <UButton
        v-else
        :label="$t('common.stop')"
        icon="i-heroicons-stop"
        color="error"
        :loading="isLoading"
        :disabled="isLoading"
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
    <LogViewer :logs="logs" class="flex-1 min-h-0" @clear="clearLogs" />
  </div>
</template>

<script setup lang="ts">
import type { LogEntry } from './LogViewer.vue'

const { safeInvoke, safeListen } = useTauri()
const { t } = useI18n()
const toast = useToast()
const { config: appConfig, loadConfig, updateClient } = useConfig()

const isRunning = ref(false)
const isLoading = ref(false)
const logs = ref<LogEntry[]>([])

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

function addLog(message: string, level: 'info' | 'success' | 'error' = 'info') {
  const now = new Date().toLocaleString()
  const entry: LogEntry = { time: now, message, level }
  logs.value.unshift(entry)
  // Persist log
  safeInvoke('append_log', {
    source: 'client',
    entry: JSON.stringify(entry),
  })
}

/** Translate backend i18n log payload and add to logs */
function addBackendLog(key: string, params: Record<string, any> | null, level: string) {
  const message = t(key, params || {})
  addLog(message, (level as 'info' | 'success' | 'error') || 'info')
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

  isLoading.value = true
  try {
    await safeInvoke('start_client', { url, interval })
    isRunning.value = true
    isLoading.value = false
    addLog(t('client.remoteUrlLog', { url }), 'info')
    await syncToSharedConfig()
  } catch (e: any) {
    // Privilege escalation failed or other error: keep isRunning = false (Task 7)
    isRunning.value = false
    isLoading.value = false
    const msg = e?.toString() || ''
    if (msg.includes('USER_CANCELLED')) {
      toast.add({ title: t('common.cancelled'), color: 'warning' })
      addLog(t('common.cancelled'), 'error')
    } else {
      addLog(t('client.fetchFail', { error: msg }), 'error')
    }
  }
}

async function stopFetch() {
  isLoading.value = true
  try {
    await safeInvoke('stop_client')
    isRunning.value = false
    isLoading.value = false
    addLog(t('client.fetchStop'), 'info')
  } catch (e: any) {
    isLoading.value = false
    addLog(e.toString(), 'error')
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
    addLog(result || t('client.flushDnsSuccess'), 'success')
  } catch (e: any) {
    const msg = e?.toString() || ''
    if (msg.includes('USER_CANCELLED')) {
      toast.add({ title: t('common.cancelled'), color: 'warning' })
    } else {
      toast.add({ title: t('client.flushDnsFail') + ': ' + msg, color: 'error' })
      addLog(t('client.flushDnsFail') + ': ' + msg, 'error')
    }
  }
}

async function clearLogs() {
  logs.value = []
  await safeInvoke('clear_logs', { source: 'client' })
}

function onAutoFetchChange(val: boolean) {
  config.autoFetch = val
  updateClient({
    interval: Number(config.interval),
    method: config.originMethod,
    select_origin: config.selectOrigin,
    custom_url: config.customUrl,
    auto_fetch: config.autoFetch,
  })
  toast.add({ title: t('client.autoFetchChanged'), color: 'info' })
}

async function syncToSharedConfig() {
  await updateClient({
    interval: Number(config.interval),
    method: config.originMethod,
    select_origin: config.selectOrigin,
    custom_url: config.customUrl,
    auto_fetch: config.autoFetch,
  })
}

function syncFromSharedConfig() {
  const c = appConfig.value.client
  config.interval = c.interval ?? 60
  config.originMethod = c.method ?? 'official'
  config.selectOrigin = c.select_origin ?? 'FetchGithubHosts'
  config.customUrl = c.custom_url ?? ''
  config.autoFetch = c.auto_fetch ?? false
}

onMounted(async () => {
  // Listen for log events from backend (now with i18n key + params + level)
  await safeListen<{ key: string; params?: Record<string, any>; level: string }>('client-log', (event) => {
    addBackendLog(event.payload.key, event.payload.params || null, event.payload.level)
  })
  await loadConfig()
  syncFromSharedConfig()

  // Load persisted logs
  try {
    const persisted = await safeInvoke<string[]>('load_logs', { source: 'client' })
    if (persisted && persisted.length > 0) {
      // Load in reverse order (newest first, matching display order)
      for (const line of [...persisted].reverse()) {
        try {
          const entry = JSON.parse(line)
          if (entry.time && entry.message) {
            logs.value.push(entry)
          }
        } catch {
          // Legacy plain text log line — display as info
          logs.value.push({ time: '', message: line, level: 'info' })
        }
      }
    }
  } catch {}

  // Auto fetch on startup if enabled
  if (config.autoFetch) {
    startFetch()
  }
})
</script>
