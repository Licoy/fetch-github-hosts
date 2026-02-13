// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false },
  ssr: false,

  modules: [
    '@nuxt/ui',
    '@nuxtjs/i18n',
    '@pinia/nuxt',
  ],

  // Disable all remote font providers (use local font only via CSS @font-face)
  fonts: {
    providers: {
      google: false,
      googleicons: false,
      bunny: false,
      fontshare: false,
      adobe: false,
      fontsource: false,
    },
  },

  icon: {
    serverBundle: 'local',
    clientBundle: {
      scan: true,
    },
  },

  colorMode: {
    preference: 'system',
    fallback: 'dark',
    classSuffix: '',
  },

  i18n: {
    locales: [
      { code: 'zh-CN', name: '简体中文', file: 'zh-CN.json' },
      { code: 'en-US', name: 'English', file: 'en-US.json' },
      { code: 'ja-JP', name: '日本語', file: 'ja-JP.json' },
    ],
    defaultLocale: 'zh-CN',
    strategy: 'no_prefix',
    lazy: false,
  },

  css: ['~/assets/css/main.css'],

  // Tauri: dev server on fixed port
  devServer: {
    port: 1420,
  },

  // Generate static files for Tauri
  nitro: {
    preset: 'static',
  },

  // Vite config for Tauri
  vite: {
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      strictPort: true,
      hmr: {
        protocol: 'ws',
        host: '0.0.0.0',
        port: 1421,
      },
      watch: {
        ignored: ['**/src-tauri/**'],
      },
    },
  },
})
