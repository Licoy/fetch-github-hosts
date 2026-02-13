/**
 * Shared config composable — single source of truth for AppConfig.
 * Both ClientMode and ServerMode read/write through this composable
 * to avoid overwriting each other's config sections.
 */

interface ClientConfig {
  interval: number
  method: string
  select_origin: string
  custom_url: string
  auto_fetch: boolean
}

interface ServerConfig {
  interval: number
  port: number
}

interface AppConfig {
  lang: string
  client: ClientConfig
  server: ServerConfig
}

const _config = ref<AppConfig>({
  lang: 'zh-CN',
  client: {
    interval: 60,
    method: 'official',
    select_origin: 'FetchGithubHosts',
    custom_url: '',
    auto_fetch: false,
  },
  server: {
    interval: 60,
    port: 9898,
  },
})

const _loaded = ref(false)

export function useConfig() {
  const { safeInvoke } = useTauri()
  const { locale } = useI18n()

  async function loadConfig() {
    try {
      const cfg = await safeInvoke<AppConfig>('load_config')
      if (cfg) {
        _config.value = {
          lang: cfg.lang || 'zh-CN',
          client: {
            interval: cfg.client?.interval ?? 60,
            method: cfg.client?.method ?? 'official',
            select_origin: cfg.client?.select_origin ?? 'FetchGithubHosts',
            custom_url: cfg.client?.custom_url ?? '',
            auto_fetch: cfg.client?.auto_fetch ?? false,
          },
          server: {
            interval: cfg.server?.interval ?? 60,
            port: cfg.server?.port ?? 9898,
          },
        }
        _loaded.value = true
      }
    } catch (_) {}
  }

  async function saveConfig() {
    try {
      // Always use current locale as lang
      _config.value.lang = locale.value
      await safeInvoke('save_config', {
        configData: _config.value,
      })
    } catch (_) {}
  }

  /** Update client config fields and save */
  async function updateClient(partial: Partial<ClientConfig>) {
    Object.assign(_config.value.client, partial)
    await saveConfig()
  }

  /** Update server config fields and save */
  async function updateServer(partial: Partial<ServerConfig>) {
    Object.assign(_config.value.server, partial)
    await saveConfig()
  }

  return {
    config: _config,
    loaded: _loaded,
    loadConfig,
    saveConfig,
    updateClient,
    updateServer,
  }
}
