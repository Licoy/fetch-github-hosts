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

### 命令行终端

到 [Releases](https://github.com/Licoy/fetch-github-hosts/releases) 中下载您的系统版本，可以直接通过命令行使用。

#### 参数

| 参数名 | 缩写 | 默认值 | 描述 |
|--------|------|--------|------|
| `--mode` | `-m` | 无（启动 GUI） | 启动模式：`client`（客户端）/ `server`（服务端） |
| `--interval` | `-i` | `60` | 获取 hosts 的间隔时间（分钟） |
| `--port` | `-p` | `9898` | 服务端模式监听端口 |
| `--url` | `-u` | `https://hosts.gitcdn.top/hosts.txt` | 客户端模式远程 hosts 获取链接 |
| `--template` | `-t` | 无（使用内置模板） | 服务端模式自定义 HTML 模板文件路径 |
| `--lang` | `-l` | 自动检测 | 界面语言（`zh-CN`、`en-US`、`ja-JP`） |

#### 启动客户端

```bash
# Linux/macOS
sudo ./fetch-github-hosts -m client

# Windows
fetch-github-hosts.exe -m client

# 自定义获取间隔（10分钟）
sudo ./fetch-github-hosts -m client -i 10

# 自定义获取链接
sudo ./fetch-github-hosts -m client -u http://127.0.0.1:9898/hosts.json
```

#### 启动服务端

```bash
# Linux/macOS
./fetch-github-hosts -m server

# Windows
fetch-github-hosts.exe -m server

# 自定义端口
./fetch-github-hosts -m server -p 6666

# 自定义 HTML 模板文件
./fetch-github-hosts -m server -t /path/to/template.html
```

> 💡 自定义模板支持 `{{FGH_VERSION}}`（版本号）和 `{{FGH_UPDATE_TIME}}`（最近更新时间）两个模板变量

> 💡 不指定 `-m` 参数时将启动图形化界面

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

## 📸 截图

#### 客户端模式
![client](assets/public/docs/client.png)

#### 客户端启动
![client-start](assets/public/docs/client-start.png)

#### 客户端 Hosts 源选择
![client-select](assets/public/docs/client-select.png)

#### 客户端 Hosts 源自定义
![client-custom](assets/public/docs/client-custom.png)

#### 服务端模式
![server](assets/public/docs/server.png)

## 🌟 Star 趋势

[![Stargazers over time](https://starchart.cc/Licoy/fetch-github-hosts.svg)](https://starchart.cc/Licoy/fetch-github-hosts)

## 📄 开源协议

[GPL-3.0](./LICENSE)
