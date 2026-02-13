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
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex justify-center gap-3">
      <UButton
        v-if="!isRunning"
        :label="$t('common.start')"
        icon="i-heroicons-play"
        color="primary"
        @click="startServer"
      />
      <UButton
        v-else
        :label="$t('common.stop')"
        icon="i-heroicons-stop"
        color="error"
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
    <LogViewer :logs="logs" class="flex-1 min-h-0" />
  </div>
</template>

<script setup lang="ts">
const { safeInvoke, safeListen } = useTauri()
const { t } = useI18n()
const toast = useToast()
const { config: appConfig, loadConfig, updateServer } = useConfig()

const isRunning = ref(false)
const listeningUrl = ref('')
const logs = ref<string[]>([])

const config = reactive({
  interval: 60,
  port: 9898,
})

function addLog(msg: string) {
  const now = new Date().toLocaleString()
  logs.value.unshift(`[${now}] ${msg}`)
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

  try {
    await safeInvoke('start_server', { port, interval })
    isRunning.value = true
    listeningUrl.value = `http://127.0.0.1:${port}`
    addLog(t('server.startSuccess', { port }))
    addLog(t('server.hostsLink', { port }))
    addLog(t('server.hostsJsonLink', { port }))
    await updateServer({
      interval: Number(config.interval),
      port: Number(config.port),
    })
  } catch (e: any) {
    addLog(t('server.startFail', { error: e.toString() }))
  }
}

async function stopServer() {
  try {
    await safeInvoke('stop_server')
    isRunning.value = false
    listeningUrl.value = ''
    addLog(t('server.stopSuccess'))
  } catch (e: any) {
    addLog(e.toString())
  }
}

async function saveConfig() {
  await updateServer({
    interval: Number(config.interval),
    port: Number(config.port),
  })
}

function syncFromSharedConfig() {
  const s = appConfig.value.server
  config.interval = s.interval ?? 60
  config.port = s.port ?? 9898
}

onMounted(async () => {
  await safeListen<{ message: string }>('server-log', (event) => {
    addLog(event.payload.message)
  })
  await loadConfig()
  syncFromSharedConfig()
})
</script>
