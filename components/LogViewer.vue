<template>
  <div class="bg-[var(--fgh-card-bg)] rounded-lg border border-[var(--fgh-border)] flex flex-col overflow-hidden" :style="{ minHeight: '180px' }">
    <div class="flex items-center justify-between px-3 py-2 border-b border-[var(--fgh-border)]">
      <span class="text-xs opacity-50 font-mono">{{ $t('log.title') }}</span>
      <UButton
        icon="i-heroicons-trash"
        size="xs"
        color="neutral"
        variant="ghost"
        @click="$emit('clear')"
        v-if="logs.length > 0"
      />
    </div>
    <div ref="logContainer" class="log-selectable flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed">
      <div v-if="logs.length === 0" class="opacity-20 text-center py-8">
        {{ $t('log.empty') }}
      </div>
      <div
        v-for="(log, idx) in logs"
        :key="idx"
        class="py-0.5 border-b border-[var(--fgh-border)]/50 last:border-0"
        :class="getLogClass(log)"
      >
        {{ getLogText(log) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
export interface LogEntry {
  time: string
  message: string
  level: 'info' | 'success' | 'error'
}

defineProps<{
  logs: (string | LogEntry)[]
}>()

defineEmits(['clear'])

function getLogClass(log: string | LogEntry): string {
  if (typeof log === 'string') return 'opacity-70'
  switch (log.level) {
    case 'error': return 'text-red-500'
    case 'success': return 'text-green-500'
    default: return 'opacity-70'
  }
}

function getLogText(log: string | LogEntry): string {
  if (typeof log === 'string') return log
  return `[${log.time}] ${log.message}`
}
</script>
