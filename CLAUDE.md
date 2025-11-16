# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

XMCL is a cross-platform Minecraft launcher built with **Tauri v2** architecture, combining a **Rust backend** with a **Next.js frontend**. It's forked from SJMCL 0.5.0 and supports Windows, macOS, and Linux with features like instance management, resource downloading, multi-account authentication, and game launching.

## Development Setup and Commands

### Prerequisites
- **Node.js**: Version 22 or higher (required by Tauri v2)
- **Rust**: Latest stable version with Cargo
- **npm**: For frontend package management
- **Platform-specific dependencies**:
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`

### Environment Setup
**CRITICAL**: Always copy `.env.template` to `.env` before building:
```bash
cp .env.template .env
```

The `.env` file contains required environment variables embedded into the Rust backend at compile time:
- `XMCL_CURSEFORGE_API_KEY`: CurseForge API key for mod downloads
- `NEXT_PUBLIC_DEV_TOOLBAR`: Development toolbar toggle

### Build Commands

#### Initial Setup
```bash
# Clone and install dependencies
git clone git@github.com:Origin173/XMCL.git
cd XMCL
cp .env.template .env
npm install

# Install system dependencies on Linux
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

#### Development
```bash
# Start development server (frontend + backend)
npm run tauri dev
```
This starts the Next.js dev server at `http://localhost:3000` and launches the Tauri application.

#### Production Build
```bash
# Build for production
npx tauri build
```

**Build Times**: Initial Rust compilation: 5-10 minutes, Subsequent builds: 2-5 minutes, Frontend build: 1-2 minutes

### Linting and Code Quality

#### Frontend Linting
```bash
# Lint TypeScript/JavaScript files
npx eslint 'src/**/*.{js,jsx,ts,tsx}' --no-fix --max-warnings=0
```

**Known Issue**: The project uses ESLint v8 config format but modern npm may install ESLint v9. Use `npx --package=eslint@8 eslint ...` if you encounter config errors.

#### Rust Formatting
```bash
# Check Rust code formatting
find src-tauri/src -name '*.rs' | xargs rustfmt --check
```

#### Pre-commit Hooks
```bash
# Runs automatically on commit
npm run lint-staged
```

This validates frontend code with ESLint, Rust code with rustfmt, and locale files consistency.

## High-Level Architecture

### Backend (Rust) - `/src-tauri/src/`
The Rust backend is organized into modular domains:

- **account/** - Player authentication and management (Microsoft OAuth, offline, third-party auth servers)
- **instance/** - Minecraft instance management, configuration, and resource handling
- **launch/** - Game launching logic, process management, and validation
- **resource/** - Game versions, mod loaders, and resource downloading (CurseForge/Modrinth integration)
- **launcher_config/** - Application configuration, Java management, and settings
- **tasks/** - Asynchronous task management system for downloads and background operations
- **discover/** - News and content discovery features
- **storage/** - Data persistence and state management
- **utils/** - Cross-platform utilities and helpers

**Entry Point**: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs` with Tauri commands exposed via `tauri::generate_handler![]`

### Frontend (Next.js) - `/src/`
The TypeScript frontend follows a service-oriented architecture:

- **pages/** - Route-based page components (accounts, downloads, instances, settings)
- **components/** - Reusable UI components organized by feature
- **services/** - API layer that communicates with Rust backend via Tauri commands
- **models/** - TypeScript type definitions and data structures
- **contexts/** - React state management and global providers
- **hooks/** - Custom React hooks
- **locales/** - Internationalization files (react-i18next)

**Framework**: Next.js with static export (`output: "export"` in next.config.ts), Chakra UI v2 for components

### Inter-Process Communication
- Frontend calls Rust via `@tauri-apps/api` npm package
- All API endpoints defined as Tauri commands in respective Rust modules
- State managed via Rust `Mutex<T>` for thread-safe access
- React Context for frontend state management

## Key Architectural Patterns

### Command-Response Architecture
- Rust commands are defined in `commands.rs` modules within each domain
- Frontend services wrap these commands for easy consumption
- All async operations are properly handled with proper error management

### Modular Domain Design
- Each major feature (account, instance, launch) has its own module
- Consistent structure: `models/`, `commands/`, `helpers/` within each module
- Clear separation of concerns between data models, business logic, and API surface

### State Management
- Rust backend manages global state with Mutex-wrapped state objects
- Frontend uses React Context for global state
- State synchronization occurs through Tauri events and commands

### Resource Management
- Comprehensive download and task management system in `tasks/` module
- Background task monitoring with progress reporting
- Smart caching and validation of game resources
- Integration with CurseForge and Modrinth for mod/modpack downloads

### Cross-Platform Design
- Platform-specific code in `utils/` with conditional compilation
- Portable mode support for different deployment scenarios
- Asset packaging for multiple platforms (Windows, macOS, Linux)

## Commit Message and PR Title Conventions

**Format**: `category(domain): content`

**Categories**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`

**Domains**: `frontend`, `backend`, `launch`, `config`, `ui`, `api`, `deps`

**Examples**:
- `feat(frontend): support instance searching`
- `fix(launch): resolve game startup crash on Linux`
- `docs(api): update authentication flow documentation`
- `chore(deps): update tauri to v2.1.0`

## Common Issues and Workarounds

1. **Environment Variables**: Always ensure `.env` exists and contains required values
2. **ESLint Version Conflicts**: Use `npx --package=eslint@8` if modern ESLint causes config issues
3. **Rust Compilation**: First build can take 5-10 minutes - this is normal
4. **Platform Dependencies**: Linux requires webkit2gtk and other system libraries
5. **Node.js Version**: Tauri v2 requires Node.js >=22, older versions will fail
6. **npm install issues**: If npm install fails or hangs, try `npm ci` instead

## Quick Start Checklist

```bash
# 1. Setup environment
cp .env.template .env
npm install

# 2. Verify setup
cargo --version  # Should show Rust toolchain
node --version   # Should be 22+
npm run version check  # Should show matching versions

# 3. Test build components
cd src-tauri && cargo check  # Test Rust compilation
cd .. && npx eslint 'src/**/*.{js,jsx,ts,tsx}' --no-fix --max-warnings=0  # Test linting

# 4. Development
npm run tauri dev  # Start development server
```