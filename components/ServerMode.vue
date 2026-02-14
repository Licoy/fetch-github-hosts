<template>
  <div class="flex flex-col gap-4 h-full">
    <!-- Config Form -->
    <div class="bg-[var(--fgh-card-bg)] rounded-lg p-4 border border-[var(--fgh-border)]">
      <div class="grid grid-cols-1 gap-4">
        <!-- Interval -->
        <div class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('server.interval') }}</label>
          <UInput
            v-model="config.interval"
            type="number"
            :min="1"
            :disabled="isRunning"
            class="flex-1"
          />
        </div>

        <!-- Port -->
        <div class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('server.port') }}</label>
          <UInput
            v-model="config.port"
            type="number"
            :min="1"
            :max="65535"
            :disabled="isRunning"
            class="flex-1"
          />
        </div>

        <!-- Template Path -->
        <div class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('server.templatePath') }}</label>
          <UInput
            v-model="config.templatePath"
            :disabled="isRunning"
            class="flex-1"
            :placeholder="$t('server.templatePathPlaceholder')"
          />
        </div>

        <!-- Template Variables Info -->
        <div class="flex items-start gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0">{{ $t('server.templateVars') }}</label>
          <div class="flex-1 text-xs opacity-60 font-mono space-y-1">
            <div class="flex items-center gap-2">
              <code v-pre>{{FGH_VERSION}}</code>
              <UButton icon="i-heroicons-clipboard-document" size="2xs" color="neutral" variant="ghost" @click="copyText('{{FGH_VERSION}}')" />
              <span>— {{ $t('server.varVersion') }}</span>
            </div>
            <div class="flex items-center gap-2">
              <code v-pre>{{FGH_UPDATE_TIME}}</code>
              <UButton icon="i-heroicons-clipboard-document" size="2xs" color="neutral" variant="ghost" @click="copyText('{{FGH_UPDATE_TIME}}')" />
              <span>— {{ $t('server.varUpdateTime') }}</span>
            </div>
          </div>
        </div>

        <!-- Export Default Template -->
        <div class="flex items-center gap-4">
          <label class="text-sm opacity-70 w-36 shrink-0"></label>
          <UButton
            :label="$t('server.exportTemplate')"
            icon="i-heroicons-document-arrow-down"
            color="neutral"
            variant="soft"
            size="xs"
            @click="exportDefaultTemplate"
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
        @click="startServer"
      />
      <UButton
        v-else
        :label="$t('common.stop')"
        icon="i-heroicons-stop"
        color="error"
        :loading="isLoading"
        :disabled="isLoading"
        @click="stopServer"
      />
    </div>

    <!-- Status -->
    <div class="text-center">
      <a
        v-if="isRunning && listeningUrl"
        :href="listeningUrl"
        target="_blank"
        class="text-sm text-[#009966] hover:underline"
      >
        {{ $t('server.listeningAddr', { addr: listeningUrl }) }}
      </a>
      <span v-else class="text-sm opacity-40">
        {{ $t('server.listeningWait') }}
      </span>
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
const { config: appConfig, loadConfig, updateServer } = useConfig()

const isRunning = ref(false)
const isLoading = ref(false)
const listeningUrl = ref('')
const logs = ref<LogEntry[]>([])

const config = reactive({
  interval: 60,
  port: 9898,
  templatePath: '',
})

function addLog(message: string, level: 'info' | 'success' | 'error' = 'info') {
  const now = new Date().toLocaleString()
  const entry: LogEntry = { time: now, message, level }
  logs.value.unshift(entry)
  // Persist log
  safeInvoke('append_log', {
    source: 'server',
    entry: JSON.stringify(entry),
  })
}

/** Translate backend i18n log payload and add to logs */
function addBackendLog(key: string, params: Record<string, any> | null, level: string) {
  const message = t(key, params || {})
  addLog(message, (level as 'info' | 'success' | 'error') || 'info')
}

async function startServer() {
  const interval = Number(config.interval)
  const port = Number(config.port)

  if (!Number.isInteger(interval) || interval < 1) {
    toast.add({ title: t('common.intervalNeedInt'), color: 'warning' })
    return
  }
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    toast.add({ title: t('common.portNeedInt'), color: 'warning' })
    return
  }

  isLoading.value = true
  try {
    await safeInvoke('start_server', { port, interval })
    // Don't set isRunning here — wait for server.httpStarted event in the listener
    // The listener will set isRunning = true and isLoading = false
    listeningUrl.value = `http://127.0.0.1:${port}`
    addLog(t('server.startSuccess', { port }), 'success')
    addLog(t('server.hostsLink', { port }), 'info')
    addLog(t('server.hostsJsonLink', { port }), 'info')
    await updateServer({
      interval: Number(config.interval),
      port: Number(config.port),
      template_path: config.templatePath,
    })
  } catch (e: any) {
    isLoading.value = false
    addLog(t('server.startFail', { error: e.toString() }), 'error')
  }
}

async function stopServer() {
  isLoading.value = true
  try {
    await safeInvoke('stop_server')
    isRunning.value = false
    isLoading.value = false
    listeningUrl.value = ''
    addLog(t('server.stopSuccess'), 'info')
  } catch (e: any) {
    isLoading.value = false
    addLog(e.toString(), 'error')
  }
}

async function clearLogs() {
  logs.value = []
  await safeInvoke('clear_logs', { source: 'server' })
}

async function copyText(text: string) {
  try {
    await safeInvoke('copy_to_clipboard', { text })
    toast.add({ title: t('common.copied'), color: 'success' })
  } catch (e: any) {
    toast.add({ title: e.toString(), color: 'error' })
  }
}

async function exportDefaultTemplate() {
  try {
    const path = await safeInvoke<string>('export_default_template')
    if (path) {
      config.templatePath = path
      toast.add({ title: t('server.templateExported', { path }), color: 'success' })
    }
  } catch (e: any) {
    toast.add({ title: t('server.templateExportFail') + ': ' + e.toString(), color: 'error' })
  }
}

async function saveConfig() {
  await updateServer({
    interval: Number(config.interval),
    port: Number(config.port),
    template_path: config.templatePath,
  })
}

function syncFromSharedConfig() {
  const s = appConfig.value.server
  config.interval = s.interval ?? 60
  config.port = s.port ?? 9898
  config.templatePath = s.template_path ?? ''
}

onMounted(async () => {
  // Listen for log events from backend (now with i18n key + params + level)
  await safeListen<{ key: string; params?: Record<string, any>; level: string }>('server-log', (event) => {
    const { key, params, level } = event.payload
    addBackendLog(key, params || null, level)
    // When HTTP server is ready, switch to running state
    if (key === 'server.httpStarted') {
      isRunning.value = true
      isLoading.value = false
    }
  })
  await loadConfig()
  syncFromSharedConfig()

  // Load persisted logs
  try {
    const persisted = await safeInvoke<string[]>('load_logs', { source: 'server' })
    if (persisted && persisted.length > 0) {
      // Load in reverse order (newest first, matching display order)
      for (const line of [...persisted].reverse()) {
        try {
          const entry = JSON.parse(line)
          if (entry.time && entry.message) {
            logs.value.push(entry)
          }
        } catch {
          logs.value.push({ time: '', message: line, level: 'info' })
        }
      }
    }
  } catch {}
})
</script>
