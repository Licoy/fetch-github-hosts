[简体中文](./README.md) | [English](./README_EN.md) | 日本語

<div align="center">
<h2>Fetch GitHub Hosts</h2>

<img src="public/logo.png" width="128" height="128" alt="Logo">

研究者や学習者が GitHub へのアクセスを高速化するための GitHub Hosts 同期ツール

[![Release](https://img.shields.io/github/v/release/Licoy/fetch-github-hosts.svg?logo=git)](https://github.com/Licoy/fetch-github-hosts/releases)
[![GitHub Stars](https://img.shields.io/github/stars/Licoy/fetch-github-hosts?style=flat&logo=github)](https://github.com/Licoy/fetch-github-hosts)
[![License](https://img.shields.io/github/license/Licoy/fetch-github-hosts)](./LICENSE)

</div>

## ✨ 機能

- 🖥️ **クロスプラットフォーム対応** — macOS (Intel & Apple Silicon)、Windows、Linux
- 🔄 **クライアントモード** — リモートソースからシステムへ自動的に Hosts を同期
- 🌐 **サーバーモード** — DNS 解決サービスを自前で構築、HTTP API を提供
- 🌓 **ダーク / ライト / システム** テーマモード
- 🌍 **多言語対応** — 简体中文、English、日本語
- 🔒 **スマート権限昇格** — 初回のみパスワード入力、セッション中は再入力不要
- 📡 **システムトレイ** — バックグラウンド実行、ワンクリックで起動/停止

## 📦 インストール

[Releases](https://github.com/Licoy/fetch-github-hosts/releases) からお使いのプラットフォーム用のインストーラーをダウンロードしてください：

| プラットフォーム | ファイル形式 | アーキテクチャ |
|----------------|------------|--------------|
| macOS | `.dmg` | Universal (Intel + Apple Silicon) |
| Windows | `.msi` / `.exe` | x86_64 |
| Linux | `.deb` / `.AppImage` | x86_64 |

## 🚀 使い方

### デスクトップクライアント

ダウンロード・インストール後、起動するだけです。グラフィカルなインターフェースで操作できます。

#### クライアントモード

リモートの Hosts ソースから最新の GitHub DNS レコードを取得し、システムの hosts ファイルに書き込みます。

- 複数の Hosts ソースに対応（FetchGithubHosts、Github520）
- カスタム URL の設定が可能
- 自動取得間隔の設定（分単位）

#### サーバーモード

ローカルで HTTP サーバーを起動し、GitHub ドメインを自動解決して hosts ファイルを提供します。

- デフォルトポート: `9898`
- `hosts.txt`（プレーンテキスト）と `hosts.json`（JSON）の2形式を提供
- ダーク/ライトテーマ・多言語対応の Web ページを内蔵

### 手動設定

#### Hosts の追加

[https://hosts.gitcdn.top/hosts.txt](https://hosts.gitcdn.top/hosts.txt) にアクセスし、内容をシステムの hosts ファイルに貼り付けてください。

- **Linux / macOS**: `/etc/hosts`
- **Windows**: `C:\Windows\System32\drivers\etc\hosts`

#### DNS キャッシュのフラッシュ

```bash
# macOS
sudo dscacheutil -flushcache && sudo killall -HUP mDNSResponder

# Windows
ipconfig /flushdns

# Linux
sudo systemd-resolve --flush-caches
```

#### Linux/macOS ワンライナー

```bash
sed -i "/# fetch-github-hosts begin/Q" /etc/hosts && curl https://hosts.gitcdn.top/hosts.txt >> /etc/hosts
```

> 💡 crontab で定期実行すれば自動更新できます

## 🏗️ 技術スタック

| コンポーネント | 技術 |
|-------------|------|
| デスクトップフレームワーク | [Tauri 2.0](https://v2.tauri.app/) (Rust) |
| フロントエンド | [Nuxt 3](https://nuxt.com/) + [Vue 3](https://vuejs.org/) |
| UI コンポーネント | [Nuxt UI](https://ui.nuxt.com/) |
| スタイリング | [Tailwind CSS 4](https://tailwindcss.com/) |
| 状態管理 | [Pinia](https://pinia.vuejs.org/) |
| 国際化 | [@nuxtjs/i18n](https://i18n.nuxtjs.org/) |

## 🛠️ 開発

### 必要環境

- Node.js ≥ 20
- Rust ≥ 1.70
- macOS / Windows / Linux

### ローカル開発

```bash
# 依存関係のインストール
npm install

# フロントエンドの静的ビルド
NUXT_CLI_WRAPPER=false npx nuxt generate

# Tauri 開発モードの起動
npx tauri dev
```

### プロダクションビルド

```bash
# フロントエンドのビルド
NUXT_CLI_WRAPPER=false npx nuxt generate

# Tauri アプリのビルド
npx tauri build
```

## 📁 プロジェクト構造

```
fetch-github-hosts/
├── components/          # Vue コンポーネント
│   ├── ClientMode.vue   # クライアントモードパネル
│   ├── ServerMode.vue   # サーバーモードパネル
│   ├── AboutPanel.vue   # 概要パネル
│   └── LogViewer.vue    # ログビューア
├── composables/         # Vue コンポーザブル
│   └── useTauri.ts      # Tauri API ラッパー
├── i18n/locales/        # 翻訳ファイル
├── pages/index.vue      # メインページ
├── public/              # 静的アセット
├── src-tauri/           # Tauri (Rust) バックエンド
│   ├── src/
│   │   ├── lib.rs       # エントリ + システムトレイ
│   │   ├── commands.rs  # Tauri コマンド
│   │   ├── services.rs  # クライアント/サーバーロジック
│   │   ├── dns.rs       # DNS 解決
│   │   ├── hosts.rs     # Hosts ファイル操作
│   │   ├── config.rs    # 設定の読み書き
│   │   └── models.rs    # データモデル
│   └── icons/           # アプリアイコン
└── .github/workflows/   # CI/CD
```

## 🌟 スター推移

[![Stargazers over time](https://starchart.cc/Licoy/fetch-github-hosts.svg)](https://starchart.cc/Licoy/fetch-github-hosts)

## 📄 ライセンス

[GPL-3.0](./LICENSE)
