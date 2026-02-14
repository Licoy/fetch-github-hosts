<template>
  <div class="flex flex-col items-center gap-6 py-4">
    <!-- Logo -->
    <img src="/logo.png" alt="Fetch Github Hosts" class="w-32 h-32 rounded-2xl" />

    <!-- About Content -->
    <div class="text-center max-w-md space-y-2">
      <h2 class="text-xl font-bold">{{ $t('about.intro') }}</h2>
      <p class="text-sm opacity-60 leading-relaxed">
        {{ $t('about.introContent') }}
      </p>
      <USeparator class="my-4" />
      <div class="text-sm opacity-50">
        <p>{{ $t('about.license') }}: {{ $t('about.licenseContent') }}</p>
        <p class="mt-1">{{ $t('about.version') }}: {{ versionLabel }}</p>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-3 flex-wrap justify-center">
      <UButton
        label="Github"
        icon="i-simple-icons-github"
        color="neutral"
        variant="outline"
        @click="openUrl('https://github.com/Licoy/fetch-github-hosts')"
      />

      <UButton
        :label="$t('about.feedback')"
        icon="i-heroicons-chat-bubble-left-right"
        color="neutral"
        variant="outline"
        @click="openUrl('https://github.com/Licoy/fetch-github-hosts/issues')"
      />

      <UButton
        :label="$t('about.checkUpdate')"
        icon="i-heroicons-arrow-path"
        color="primary"
        variant="outline"
        :loading="checking"
        @click="checkUpdate"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
const { safeInvoke, safeOpenUrl } = useTauri()
const { t } = useI18n()
const toast = useToast()
const { versionLabel } = useAppVersion()

const checking = ref(false)

async function openUrl(url: string) {
  await safeOpenUrl(url)
}

async function checkUpdate() {
  checking.value = true
  try {
    const result: any = await safeInvoke('check_update')
    if (result.has_update) {
      const confirmed = confirm(t('about.updateContent'))
      if (confirmed) {
        await openUrl(result.url)
      }
    } else {
      toast.add({ title: t('about.currentNewest'), color: 'success' })
    }
  } catch (e: any) {
    toast.add({ title: t('about.updateCheckFail') + ': ' + e.toString(), color: 'error' })
  } finally {
    checking.value = false
  }
}
</script>
