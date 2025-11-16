# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Next.js frontend. UI pieces live in `components/`, `layouts/`, hooks in `hooks/`, context providers in `contexts/`, route entries in `pages/`, and shared helpers in `services/` and `utils/`. Translations are under `locales/`, and theme assets in `styles/`.
- `src-tauri/`: Tauri + Rust backend. Core commands sit in `src/`, shared Rust helpers in `libs/`, with capabilities and packaging config in `capabilities/` and `tauri.conf.json`.
- `public/`: Static assets and icons used by the app and installer. `docs/` holds reference material and figures. `scripts/` contains tooling for versioning and locale maintenance.

## Build, Test, and Development Commands
```bash
npm install                     # install JS/TS deps (preferred manager)
npm run tauri dev               # launch Next + Tauri desktop window (uses .env)
npm run build && npm run start  # Next.js production build/preview without bundling
npm run lint                    # ESLint/Prettier check for JS/TS
npm run tauri build             # bundle the desktop app (uses XMCL_* and NEXT_PUBLIC_* envs)
npm run locale diff en          # example locale parity check (replace locale as needed)
```

## Coding Style & Naming Conventions
- TypeScript with ESLint (`eslint-config-next`) and Prettier; imports are auto-sorted via `@trivago/prettier-plugin-sort-imports`. Lint-staged enforces style on commit.
- Components/pages use PascalCase (`ProfileCard.tsx`), hooks start with `use*`, and utility modules favor kebab- or camel-case filenames (`download-manager.ts`, `stringHelpers.ts`).
- Rust backend is formatted with `rustfmt`; prefer running `cargo fmt` and `clippy` locally (VS Code: set `rust-analyzer.check.command` to `clippy`).

## Testing Guidelines
- CI runs ESLint on `src/**/*.{js,jsx,ts,tsx}`, `rustfmt --check` on `src-tauri/src/**/*.rs`, and a `tauri build` smoke check. Run the same before pushing to mirror CI.
- No dedicated unit-test suite is present; when adding tests, colocate them as `*.test.ts`/`*.test.tsx` near the code. Mock network/file access and keep locale snapshots in sync (`npm run locale diff <locale>`).
- For UI/manual verification, start `npm run tauri dev` and exercise key flows (instance management, login, download) alongside any new features.

## Commit & Pull Request Guidelines
- Follow conventional commits seen in history (`fix(tauri): ...`, `chore: ...`, `feat: ...`). Favor focused commits per logical change.
- PRs should include: brief description, related issue links, screenshots/GIFs for UI changes, and notes on env/config impacts. Keep branches up-to-date with `main`.
- Ensure local checks (`npm run lint`, `cargo fmt`/`rustfmt`, `npm run tauri build` when touching build paths) pass. Update docs when behavior or config changes.

## Security & Configuration Tips
- Copy `.env.template` to `.env` and fill required keys (e.g., `XMCL_CURSEFORGE_API_KEY`, `XMCL_OPENLIST_BASE_URL`, `NEXT_PUBLIC_OPENLIST_BASE_URL`). Never commit secrets.
- Linux developers should install platform deps used in CI (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`) before running `tauri build`. Use `npm` to stay aligned with `package-lock.json`.
