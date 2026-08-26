<template>
  <UApp>
    <NuxtPage />
  </UApp>
</template>

<script setup lang="ts">
const { os } = usePlatform()

if (import.meta.client) {
  document.documentElement.dataset.os = os
}
</script>
