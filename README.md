<img src="docs/figs/banner.png" alt="XMCL" />

[![Test Build](https://img.shields.io/github/actions/workflow/status/Origin173/XMCL/test.yml?label=test%20build&logo=github&style=for-the-badge)](https://github.com/Origin173/XMCL/blob/main/.github/workflows/test.yml)
![Downloads](https://img.shields.io/github/downloads/Origin173/XMCL/total?style=for-the-badge)
![Stars](https://img.shields.io/github/stars/Origin173/XMCL?style=for-the-badge)
[![Ask DeepWiki](https://img.shields.io/badge/Ask-DeepWiki-20B2AA?logo=&style=for-the-badge)](https://deepwiki.com/Origin173/XMCL)

**English** · [简体中文](docs/README.zh-Hans.md) · [繁體中文](docs/README.zh-Hant.md)

> **Note:** XMCL is based on [SJMCL](https://github.com/UNIkeEN/SJMCL), developed by the SJMC (Shanghai Jiao Tong Minecraft Club) team. This project follows the GNU General Public License v3.0 with additional terms as specified in the original project.

## Features

* **Cross Platform**: Supports Windows 10/11, macOS and Linux.
* **Efficient Instance Management**: Supports multiple game directories and instances, allowing the management of all instance resources (such as saves, mods, resource packs, shaders, screenshots, etc.) and settings in one place.
* **Convenient Resource Download**: Supports downloading game clients, mod loaders, various game resources and modpacks from CurseForge and Modrinth.
* **Multi-Account System Support**: Built-in Microsoft login and third-party authentication server support, compatible with the OAuth login process proposed by the Yggdrasil Connect proposal.
* **Deeplink Integration**: Integrates with external websites and tool collections, providing convenient features such as desktop shortcuts for launching instances through system deeplinks.

> Note: some features may be limited by region, platform, or bundle type.

### Built with

[![Tauri](https://img.shields.io/badge/Tauri-v2-FFC131?style=for-the-badge&logo=tauri&logoColor=white&labelColor=24C8DB)](https://tauri.app/)
[![Next JS](https://img.shields.io/badge/next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white)](https://nextjs.org/)
[![Chakra UI](https://img.shields.io/badge/chakra_ui-v2-38B2AC?style=for-the-badge&logo=chakraui&logoColor=white&labelColor=319795)](https://v2.chakra-ui.com/)

## Getting Started

You can download the latest release from [GitHub Releases](https://github.com/Origin173/XMCL/releases).

XMCL currently supports the following platforms:

| Platform  | Versions            | Architectures              | Provided Bundles                        |
|-----------|---------------------|----------------------------|-----------------------------------------|
| Windows   | 7 and above         | `aarch64`, `i686`, `x86_64`| `.msi`, portable `.exe`                 |
| Linux     | webkit2gtk 4.1 (e.g., Ubuntu 22.04) | `x86_64`   | `.AppImage`, `.deb`, `.rpm`, portable binary |

> **Note on macOS Support:** macOS builds are currently not provided due to the requirement of an Apple Developer Program membership ($99/year) for code signing and notarization. Without proper code signing, macOS applications cannot run on modern macOS systems (macOS 10.15+) due to Gatekeeper security requirements. Distributing unsigned applications would result in security warnings and potential legal/security risks. If you are interested in supporting macOS development, please consider contributing or sponsoring the project.

### Windows 7

If you need to run XMCL on Windows 7, please first [download the Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download) and install it. We recommend choosing the 'Evergreen Bootstrapper'.

## Development and Contributing

To get started, clone the repository and install the required dependencies:

```bash
git clone git@github.com:Origin173/XMCL.git
npm install
```

To run the project in development mode:

```bash
npm run tauri dev
```

We warmly invite contributions from everyone.

* Before you get started, please take a moment to review our [Contributing Guide](https://github.com/Origin173/XMCL/blob/main/CONTRIBUTING.md).
* Feel free to share your ideas through [Pull Requests](https://github.com/Origin173/XMCL/pulls) or [GitHub Issues](https://github.com/Origin173/XMCL/issues).

## Copyright

Copyright © 2024-2025 CAUCraft Team.

> NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.

The software is distributed under [GNU General Public License v3.0](/LICENSE).

By GPLv3 License term 7, we require that when you distribute a modified version of the software, you must obey GPLv3 License as well as the following [additional terms](/LICENSE.EXTRA):

1. Use a different software name than XMCL or XMCL Launcher;
2. Mark clearly in your repository README file, your distribution website or thread, Support documents, About Page in the software that your program is based on XMCL and give out the url of the origin repository.

## Acknowledgments

XMCL is based on [SJMCL](https://github.com/UNIkeEN/SJMCL), developed by the SJMCL Team and SJMC (Shanghai Jiao Tong Minecraft Club). We are grateful for their excellent work and open-source contribution that made this project possible.

## Contact Us

You can contact us through [GitHub Issues](https://github.com/Origin173/XMCL/issues) if you want to report bugs, suggest features, or contribute to the project.
