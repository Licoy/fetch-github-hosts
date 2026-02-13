[简体中文](./README.md) | English | [日本語](./README_JA.md)

<div align="center">
<h2>Fetch GitHub Hosts</h2>

<img src="public/logo.png" width="128" height="128" alt="Logo">

A GitHub Hosts synchronization tool designed to help researchers and learners access GitHub faster

[![Release](https://img.shields.io/github/v/release/Licoy/fetch-github-hosts.svg?logo=git)](https://github.com/Licoy/fetch-github-hosts/releases)
[![GitHub Stars](https://img.shields.io/github/stars/Licoy/fetch-github-hosts?style=flat&logo=github)](https://github.com/Licoy/fetch-github-hosts)
[![License](https://img.shields.io/github/license/Licoy/fetch-github-hosts)](./LICENSE)

</div>

## ✨ Features

- 🖥️ **Cross-platform Desktop App** — Supports macOS (Intel & Apple Silicon), Windows, Linux
- 🔄 **Client Mode** — Automatically sync hosts from remote sources to your system
- 🌐 **Server Mode** — Self-hosted DNS resolution service with HTTP API
- 🌓 **Dark / Light / System** theme modes
- 🌍 **Multi-language** — 简体中文, English, 日本語
- 🔒 **Smart Elevation** — One-time password prompt, no repeated authorization needed
- 📡 **System Tray** — Run in background with one-click start/stop

## 📦 Installation

Download from [Releases](https://github.com/Licoy/fetch-github-hosts/releases):

| Platform | File Type | Architecture |
|----------|-----------|--------------|
| macOS | `.dmg` | Universal (Intel + Apple Silicon) |
| Windows | `.msi` / `.exe` | x86_64 |
| Linux | `.deb` / `.AppImage` | x86_64 |

## 🚀 Usage

### Desktop Client

Download, install, and run. The app provides a graphical user interface.

#### Client Mode

Fetches the latest GitHub DNS records from remote hosts sources and writes them to the system hosts file.

- Multiple hosts sources (FetchGithubHosts, Github520)
- Custom remote URL support
- Configurable auto-fetch interval (minutes)

#### Server Mode

Starts a local HTTP server that auto-resolves GitHub domains and serves hosts files.

- Default port: `9898`
- Provides `hosts.txt` (plain text) and `hosts.json` (JSON) formats
- Built-in web page with dark/light theme and multi-language support

### Manual Method

#### Add Hosts

Visit [https://hosts.gitcdn.top/hosts.txt](https://hosts.gitcdn.top/hosts.txt) and paste the content into your system hosts file.

- **Linux / macOS**: `/etc/hosts`
- **Windows**: `C:\Windows\System32\drivers\etc\hosts`

#### Flush DNS Cache

```bash
# macOS
sudo dscacheutil -flushcache && sudo killall -HUP mDNSResponder

# Windows
ipconfig /flushdns

# Linux
sudo systemd-resolve --flush-caches
```

#### One-liner for Linux/macOS

```bash
sed -i "/# fetch-github-hosts begin/Q" /etc/hosts && curl https://hosts.gitcdn.top/hosts.txt >> /etc/hosts
```

> 💡 Set up a crontab task for automatic updates

## 🏗️ Tech Stack

| Component | Technology |
|-----------|-----------|
| Desktop Framework | [Tauri 2.0](https://v2.tauri.app/) (Rust) |
| Frontend | [Nuxt 3](https://nuxt.com/) + [Vue 3](https://vuejs.org/) |
| UI Components | [Nuxt UI](https://ui.nuxt.com/) |
| Styling | [Tailwind CSS 4](https://tailwindcss.com/) |
| State Management | [Pinia](https://pinia.vuejs.org/) |
| i18n | [@nuxtjs/i18n](https://i18n.nuxtjs.org/) |

## 🛠️ Development

### Requirements

- Node.js ≥ 20
- Rust ≥ 1.70
- macOS / Windows / Linux

### Local Development

```bash
# Install dependencies
npm install

# Build static frontend
NUXT_CLI_WRAPPER=false npx nuxt generate

# Start Tauri dev mode
npx tauri dev
```

### Build for Production

```bash
# Build frontend
NUXT_CLI_WRAPPER=false npx nuxt generate

# Build Tauri app
npx tauri build
```

## 📁 Project Structure

```
fetch-github-hosts/
├── components/          # Vue components
│   ├── ClientMode.vue   # Client mode panel
│   ├── ServerMode.vue   # Server mode panel
│   ├── AboutPanel.vue   # About panel
│   └── LogViewer.vue    # Log viewer
├── composables/         # Vue composables
│   └── useTauri.ts      # Tauri API wrappers
├── i18n/locales/        # i18n translation files
├── pages/index.vue      # Main page
├── public/              # Static assets
├── src-tauri/           # Tauri (Rust) backend
│   ├── src/
│   │   ├── lib.rs       # Entry + System tray
│   │   ├── commands.rs  # Tauri commands
│   │   ├── services.rs  # Client/Server logic
│   │   ├── dns.rs       # DNS resolution
│   │   ├── hosts.rs     # Hosts file operations
│   │   ├── config.rs    # Config read/write
│   │   └── models.rs    # Data models
│   └── icons/           # App icons
└── .github/workflows/   # CI/CD
```

## 🌟 Star History

[![Stargazers over time](https://starchart.cc/Licoy/fetch-github-hosts.svg)](https://starchart.cc/Licoy/fetch-github-hosts)

## 📄 License

[GPL-3.0](./LICENSE)
