<template>
  <div class="h-screen flex flex-col bg-[var(--fgh-bg)] text-[var(--fgh-text)]">
    <TitleBar />

    <!-- Main Content -->
    <div class="flex-1 overflow-hidden p-4 flex flex-col">
      <UTabs
        :items="tabItems"
        v-model="activeTab"
        :ui="{
          list: 'bg-[var(--fgh-card-bg)] border border-[var(--fgh-border)] rounded-lg',
          trigger: 'data-[state=active]:bg-[#009966] data-[state=active]:text-white text-[var(--fgh-text-muted)] transition-all',
        }"
      >
        <template #content />
      </UTabs>
      <!-- Render all panels with v-show to keep state alive across tab switches -->
      <div class="mt-4 flex-1 overflow-hidden">
        <div v-show="activeTab === 'client'" class="h-full">
          <ClientMode />
        </div>
        <div v-show="activeTab === 'server'" class="h-full">
          <ServerMode />
        </div>
        <div v-show="activeTab === 'about'" class="h-full">
          <AboutPanel />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { t } = useI18n()

const activeTab = ref('client')

const tabItems = computed(() => [
  { label: t('tabs.client'), value: 'client' },
  { label: t('tabs.server'), value: 'server' },
  { label: t('tabs.about'), value: 'about' },
])
</script>
