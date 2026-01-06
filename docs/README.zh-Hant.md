<img src="figs/banner.png" alt="XMCL" />

[![Test Build](https://img.shields.io/github/actions/workflow/status/Origin173/XMCL/test.yml?label=test%20build&logo=github&style=for-the-badge)](https://github.com/Origin173/XMCL/blob/main/.github/workflows/test.yml)
![Downloads](https://img.shields.io/github/downloads/Origin173/XMCL/total?style=for-the-badge)
![Stars](https://img.shields.io/github/stars/Origin173/XMCL?style=for-the-badge)
[![Ask DeepWiki](https://img.shields.io/badge/Ask-DeepWiki-20B2AA?logo=&style=for-the-badge)](https://deepwiki.com/Origin173/XMCL)

[English](../README.md) · [简体中文](README.zh-Hans.md) · **繁體中文**

## 功能特性

* **跨平臺支援**：相容 Windows 10/11、macOS 與 Linux。
* **高效的例項管理**：支援多個遊戲目錄與例項，集中管理所有例項資源（如存檔、模組、資源包、光影包、截圖等）與設定。
* **便捷的資源下載**：支援從 CurseForge 與 Modrinth 等源下載遊戲客戶端、Mod 載入器、各類遊戲資源與整合包。
* **多賬戶系統支援**：內建 Microsoft 登入與第三方認證伺服器支援，相容 Yggdrasil Connect 的 OAuth 登入流程規範提案。
* **深度連結整合**：可與外部網站與工具集聯動，支援透過系統深度連結、桌面快捷方式一鍵啟動例項等便捷功能。

> 注意：部分功能可能受地區、執行平臺或程式分發型別限制。

### 技術棧

[![Tauri](https://img.shields.io/badge/Tauri-v2-FFC131?style=for-the-badge&logo=tauri&logoColor=white&labelColor=24C8DB)](https://tauri.app/)
[![Next JS](https://img.shields.io/badge/next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white)](https://nextjs.org/)
[![Chakra UI](https://img.shields.io/badge/chakra_ui-v2-38B2AC?style=for-the-badge&logo=chakraui&logoColor=white&labelColor=319795)](https://v2.chakra-ui.com/)

## 開始使用

開始使用 XMCL，只需前往 [GitHub Releases](https://github.com/Origin173/XMCL/releases) 下載最新版即可。

XMCL 目前支援以下平臺：

| 平臺    | 系統版本            | 架構               | 提供的的分發型別                              |
|---------|---------------------|--------------------|--------------------------------------------|
| Windows | 7 及以上           | `aarch64`, `i686`, `x86_64`   | `.msi`，便攜版 `.exe`                |
| macOS   | 10.15 及以上        | `aarch64`, `x86_64`| `.app`，`.dmg`                   |
| Linux   | webkit2gtk 4.1 (如 Ubuntu 22.04) | `x86_64` | `.AppImage`, `.deb`, `.rpm`, 便攜版二進位制檔案 |

### Windows 7

若您需要在 Windows 7 上執行 XMCL，請先[下載 Microsoft Edge WebView2 運行時](https://developer.microsoft.com/zh-tw/microsoft-edge/webview2#download)並安裝，建議選擇「常青引導程式」。

## 開發與貢獻

首先克隆本專案並安裝前端依賴：

```bash
git clone git@github.com:Origin173/XMCL.git
npm install
```

使用開發模式執行：

```bash
npm run tauri dev
```

我們熱烈歡迎每一位開發者的貢獻。

* 在開始前，請先閱讀我們的 [貢獻指南](https://github.com/Origin173/XMCL/blob/main/CONTRIBUTING.md)（內含開發流程詳細說明）。
* 歡迎透過 [Pull Request](https://github.com/Origin173/XMCL/pulls) 或 [GitHub Issues](https://github.com/Origin173/XMCL/issues) 分享您的想法。

## 版權宣告

版權所有 © 2025 XMCL 團隊。

> 本軟體並非官方 Minecraft 服務。未獲得 Mojang 或 Microsoft 批准或關聯許可。

本專案基於 [GNU 通用公共許可證 v3.0](../LICENSE) 釋出。

XMCL 是 SJMCL 0.5.0 的分支版本。

## 聯絡我們

您可以透過 [GitHub Issues](https://github.com/Origin173/XMCL/issues) 來聯絡我們。
