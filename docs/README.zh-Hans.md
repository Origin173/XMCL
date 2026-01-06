<img src="../docs/figs/banner.png" alt="XMCL" />

[![测试构建](https://img.shields.io/github/actions/workflow/status/Origin173/XMCL/test.yml?label=test%20build&logo=github&style=for-the-badge)](https://github.com/Origin173/XMCL/blob/main/.github/workflows/test.yml)
![下载量](https://img.shields.io/github/downloads/Origin173/XMCL/total?style=for-the-badge)
![Star](https://img.shields.io/github/stars/Origin173/XMCL?style=for-the-badge)
[![Ask DeepWiki](https://img.shields.io/badge/Ask-DeepWiki-20B2AA?logo=&style=for-the-badge)](https://deepwiki.com/Origin173/XMCL)

[English](../README.md) · **简体中文** · [繁體中文](README.zh-Hant.md)

> **注意：** XMCL 基于 SJMC（上海交通大学 Minecraft 社区）团队开发的 [SJMCL](https://github.com/UNIkeEN/SJMCL) 制作。本项目遵循 GNU 通用公共许可证 v3.0 及原项目指定的额外条款。

## 功能特性

* **跨平台支持**：支持 Windows 10/11、macOS 和 Linux。
* **高效实例管理**：支持多个游戏目录和实例，可以在一个地方管理所有实例资源（如存档、模组、资源包、光影、截图等）和设置。
* **便捷资源下载**：支持从 CurseForge 和 Modrinth 下载游戏客户端、模组加载器、各种游戏资源和整合包。
* **多账户系统支持**：内置 Microsoft 登录和第三方认证服务器支持，兼容 Yggdrasil Connect 提案提出的 OAuth 登录流程。
* **深度链接集成**：与外部网站和工具集成，通过系统深度链接提供便捷功能，如桌面快捷方式启动实例。

> 注意：某些功能可能受到地区、平台或软件包类型的限制。

### 技术栈

[![Tauri](https://img.shields.io/badge/Tauri-v2-FFC131?style=for-the-badge&logo=tauri&logoColor=white&labelColor=24C8DB)](https://tauri.app/)
[![Next JS](https://img.shields.io/badge/next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white)](https://nextjs.org/)
[![Chakra UI](https://img.shields.io/badge/chakra_ui-v2-38B2AC?style=for-the-badge&logo=chakraui&logoColor=white&labelColor=319795)](https://v2.chakra-ui.com/)

## 开始使用

你可以从 [GitHub Releases](https://github.com/Origin173/XMCL/releases) 下载最新版本。

XMCL 目前支持以下平台：

| 平台      | 版本                | 架构                       | 提供的安装包                           |
|-----------|---------------------|----------------------------|----------------------------------------|
| Windows   | 7 及以上            | `aarch64`, `i686`, `x86_64`| `.msi`, 便携版 `.exe`                  |
| macOS     | 10.15 及以上        | `aarch64`, `x86_64`        | `.app`, `.dmg`                         |
| Linux     | webkit2gtk 4.1 (如 Ubuntu 22.04) | `x86_64`   | `.AppImage`, `.deb`, `.rpm`, 便携版二进制文件 |

### Windows 7

如果你需要在 Windows 7 上运行 XMCL，请先[下载 Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/#download) 并安装。我们建议选择"常青引导程序"。

## 开发和贡献

首先，克隆仓库并安装所需的依赖：

```bash
git clone git@github.com:Origin173/XMCL.git
npm install
```

在开发模式下运行项目：

```bash
npm run tauri dev
```

我们热烈欢迎所有人的贡献。

* 在开始之前，请花一点时间查看我们的[贡献指南](https://github.com/Origin173/XMCL/blob/main/CONTRIBUTING.md)。
* 欢迎通过 [Pull Requests](https://github.com/Origin173/XMCL/pulls) 或 [GitHub Issues](https://github.com/Origin173/XMCL/issues) 分享你的想法。

## 版权

版权所有 © 2024-2025 CAUCraft 团队。

> 非官方 MINECRAFT 服务。未经 MOJANG 或 MICROSOFT 批准或关联。

本软件根据 [GNU 通用公共许可证 v3.0](/LICENSE) 分发。

根据 GPLv3 许可证第 7 条，当你分发本软件的修改版本时，你必须遵守 GPLv3 许可证以及以下[额外条款](/LICENSE.EXTRA)：

1. 使用与 XMCL 或 XMCL Launcher 不同的软件名称；
2. 在你的仓库 README 文件、分发网站或帖子、支持文档、软件的"关于"页面中明确标注你的程序基于 XMCL，并提供原始仓库的网址。

## 致谢

XMCL 基于 SJMCL 团队和 SJMC（上海交通大学 Minecraft 社区）开发的 [SJMCL](https://github.com/UNIkeEN/SJMCL)。我们感谢他们的出色工作和开源贡献，使这个项目成为可能。

## 联系我们

如果你想报告错误、建议功能或为项目做出贡献，可以通过 [GitHub Issues](https://github.com/Origin173/XMCL/issues) 联系我们。
