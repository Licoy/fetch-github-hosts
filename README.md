简体中文 | [English](./README_EN.md) | [日本語](./README_JA.md)

<div align="center">
<h2>Fetch GitHub Hosts</h2>

<img src="public/logo.png" width="128" height="128" alt="Logo">

`fetch-github-hosts` 是主要为解决研究及学习人员访问 `Github` 过慢或其他问题而提供的 `Github Hosts` 同步工具

[![Release](https://img.shields.io/github/v/release/Licoy/fetch-github-hosts.svg?logo=git)](https://github.com/Licoy/fetch-github-hosts/releases)
[![GitHub Stars](https://img.shields.io/github/stars/Licoy/fetch-github-hosts?style=flat&logo=github)](https://github.com/Licoy/fetch-github-hosts)
[![License](https://img.shields.io/github/license/Licoy/fetch-github-hosts)](./LICENSE)

</div>

## ✨ 特性

- 🖥️ **跨平台桌面客户端** — 支持 macOS (Intel & Apple Silicon)、Windows、Linux
- 🔄 **客户端模式** — 从远程源自动同步 Hosts 到系统
- 🌐 **服务端模式** — 自建 DNS 解析服务，提供 HTTP API 供其他设备使用
- 🌓 **深色/浅色/跟随系统** 三种主题模式
- 🌍 **多语言支持** — 简体中文、English、日本語
- 🔒 **智能提权** — 首次写入 Hosts 时一次性授权，会话期间无需重复输入密码
- 📡 **系统托盘** — 后台运行，一键启停

## 📦 安装

前往 [Releases](https://github.com/Licoy/fetch-github-hosts/releases) 下载对应平台安装包：

| 平台 | 文件类型 | 架构 |
|------|---------|------|
| macOS | `.dmg` | Universal (Intel + Apple Silicon) |
| Windows | `.msi` / `.exe` | x86_64 |
| Linux | `.deb` / `.AppImage` | x86_64 |

## 🚀 使用方法

### 桌面客户端

下载安装后直接运行即可，提供图形化界面操作。

#### 客户端模式

从远程 Hosts 源获取最新的 GitHub 相关 DNS 记录，自动写入系统 hosts 文件。

- 支持多种 Hosts 源（FetchGithubHosts、Github520）
- 支持自定义远程 URL
- 可设置自动获取间隔（分钟）

#### 服务端模式

在本地启动 HTTP 服务，自动解析 GitHub 域名并提供 hosts 文件下载。

- 默认监听端口 `9898`
- 提供 `hosts.txt`（纯文本）和 `hosts.json`（JSON）两种格式
- 内置美观的 Web 页面，支持深色/浅色主题和多语言

### 手动方式

#### 添加 Hosts

访问 [https://hosts.gitcdn.top/hosts.txt](https://hosts.gitcdn.top/hosts.txt)，将全部内容粘贴到系统 hosts 文件中。

- **Linux / macOS**: `/etc/hosts`
- **Windows**: `C:\Windows\System32\drivers\etc\hosts`

#### 刷新 DNS 缓存

```bash
# macOS
sudo dscacheutil -flushcache && sudo killall -HUP mDNSResponder

# Windows
ipconfig /flushdns

# Linux
sudo systemd-resolve --flush-caches
```

#### Linux/macOS 一键使用

```bash
sed -i "/# fetch-github-hosts begin/Q" /etc/hosts && curl https://hosts.gitcdn.top/hosts.txt >> /etc/hosts
```

> 💡 可配合 crontab 定时任务实现自动更新

## 🏗️ 技术栈

| 组件 | 技术 |
|------|------|
| 桌面框架 | [Tauri 2.0](https://v2.tauri.app/) (Rust) |
| 前端框架 | [Nuxt 3](https://nuxt.com/) + [Vue 3](https://vuejs.org/) |
| UI 组件 | [Nuxt UI](https://ui.nuxt.com/) |
| 样式 | [Tailwind CSS 4](https://tailwindcss.com/) |
| 状态管理 | [Pinia](https://pinia.vuejs.org/) |
| 国际化 | [@nuxtjs/i18n](https://i18n.nuxtjs.org/) |

## 🛠️ 开发

### 环境要求

- Node.js ≥ 20
- Rust ≥ 1.70
- macOS / Windows / Linux

### 本地开发

```bash
# 安装依赖
npm install

# 构建前端静态文件
NUXT_CLI_WRAPPER=false npx nuxt generate

# 启动 Tauri 开发模式
npx tauri dev
```

### 构建发布包

```bash
# 构建前端
NUXT_CLI_WRAPPER=false npx nuxt generate

# 构建 Tauri 应用
npx tauri build
```

## 📁 项目结构

```
fetch-github-hosts/
├── components/          # Vue 组件
│   ├── ClientMode.vue   # 客户端模式面板
│   ├── ServerMode.vue   # 服务端模式面板
│   ├── AboutPanel.vue   # 关于面板
│   └── LogViewer.vue    # 日志查看器
├── composables/         # Vue 组合函数
│   └── useTauri.ts      # Tauri API 封装
├── i18n/locales/        # 国际化翻译文件
├── pages/index.vue      # 主页面
├── public/              # 静态资源
├── src-tauri/           # Tauri (Rust) 后端
│   ├── src/
│   │   ├── lib.rs       # 入口 + 系统托盘
│   │   ├── commands.rs  # Tauri 命令
│   │   ├── services.rs  # 客户端/服务端逻辑
│   │   ├── dns.rs       # DNS 解析
│   │   ├── hosts.rs     # Hosts 文件操作
│   │   ├── config.rs    # 配置读写
│   │   └── models.rs    # 数据模型
│   └── icons/           # 应用图标
└── .github/workflows/   # CI/CD
```

## 🌟 Star 趋势

[![Stargazers over time](https://starchart.cc/Licoy/fetch-github-hosts.svg)](https://starchart.cc/Licoy/fetch-github-hosts)

## 📄 开源协议

[GPL-3.0](./LICENSE)
