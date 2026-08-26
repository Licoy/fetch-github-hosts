<template>
  <div class="title-bar-nodrag flex items-center">
    <button
      type="button"
      class="caption-btn"
      :title="$t('window.minimize')"
      :aria-label="$t('window.minimize')"
      @click="windowMinimize"
    >
      <UIcon name="i-heroicons-minus" class="size-4" />
    </button>
    <button
      type="button"
      class="caption-btn"
      :title="isMaximized ? $t('window.restore') : $t('window.maximize')"
      :aria-label="isMaximized ? $t('window.restore') : $t('window.maximize')"
      @click="windowToggleMaximize"
    >
      <svg
        v-if="isMaximized"
        width="14"
        height="14"
        viewBox="0 0 14 14"
        fill="none"
        stroke="currentColor"
        stroke-width="1.2"
        aria-hidden="true"
      >
        <rect x="3" y="5" width="8" height="7" rx="0.5" />
        <path d="M5 5V3.5a.5.5 0 0 1 .5-.5H12a.5.5 0 0 1 .5.5V10a.5.5 0 0 1-.5.5h-1.5" />
      </svg>
      <svg
        v-else
        width="14"
        height="14"
        viewBox="0 0 14 14"
        fill="none"
        stroke="currentColor"
        stroke-width="1.2"
        aria-hidden="true"
      >
        <rect x="2.5" y="2.5" width="9" height="9" rx="0.5" />
      </svg>
    </button>
    <button
      type="button"
      class="caption-btn caption-btn-close"
      :title="closeTip"
      :aria-label="closeTip"
      @click="emit('close')"
    >
      <UIcon name="i-heroicons-x-mark" class="size-4" />
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  closeTip: string
}>()
const emit = defineEmits<{
  close: []
}>()

const { windowMinimize, windowToggleMaximize, windowIsMaximized, listenWindowResized } = useTauri()
const isMaximized = ref(false)
const stopListen = ref<(() => void) | undefined>()

onMounted(async () => {
  isMaximized.value = await windowIsMaximized()
  stopListen.value = await listenWindowResized(async () => {
    isMaximized.value = await windowIsMaximized()
  })
})

onUnmounted(() => {
  stopListen.value?.()
})
</script>

<style scoped>
.caption-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--fgh-text-muted);
  cursor: pointer;
}
.caption-btn:hover {
  background: var(--fgh-border);
  color: var(--fgh-text);
}
.caption-btn-close:hover {
  background: #e81123;
  color: #ffffff;
}
</style>
