# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Fuse** is a Tauri v2 desktop application for PR monitoring, triage, and AI-assisted code review. It syncs pull requests via the GitHub CLI (`gh`), scores them by risk, and provides AI review assistance through Claude CLI. macOS is the primary target platform.

## Build & Development Commands

All commands run from the `tauri-app/` directory:

```sh
npm install                 # Install frontend dependencies
npm run tauri dev           # Dev mode (Vite + Tauri window)
npm run tauri build         # Production build → src-tauri/target/release/bundle/
npm run build               # Frontend-only build (tsc + vite)
./dev.sh                    # Convenience dev script
```

Vite dev server runs on port 1420. Rust backend recompiles automatically with `tauri dev`.

## Architecture

**Tauri v2 (Rust) backend** communicates with **Vue 3 frontend** over IPC (`invoke`). All GitHub interaction goes through `gh` CLI via `tauri-plugin-shell` — no API tokens managed directly.

- **Rust backend** (`tauri-app/src-tauri/src/`): Command modules in `commands/`, each exposing `#[tauri::command]` functions registered in `lib.rs`. SQLite database (WAL mode) via `rusqlite` with idempotent migrations in `db/migrations.rs`. Background polling loop in `polling.rs`.
- **Vue frontend** (`tauri-app/src/`): Composition API throughout. Pinia stores in `stores/`, 38 composables in `composables/`, components flat under `components/` with `layout/` and `skeletons/` subdirectories.
- **Database state**: `DbState(Mutex<Connection>)` managed as Tauri state. All command handlers receive it via `tauri::State<'_, DbState>`.
- **External CLIs**: `gh` (GitHub), `grove` (worktrees, optional), `claude` (AI reviews, optional) — all invoked via `tauri-plugin-shell`.

### Adding a Feature

1. Schema in `db/migrations.rs` → 2. Rust commands in `commands/*.rs`, register in `lib.rs` → 3. TypeScript interfaces in `src/types/index.ts` → 4. Pinia store or composable → 5. Vue components/views

### Key Architectural Decisions

- Frameless transparent window with custom `TitleBar` component (`decorations: false`, `transparent: true`, `macOSPrivateApi: true`)
- CSS custom properties design system in `src/styles/tokens.css` — glassmorphic panels, theme tokens, density modes
- Rust lib crate named `pr_review_companion_lib` (differs from package name `fuse`)
- CSP restricts images to `self`, `github.com`, and `avatars.githubusercontent.com`

## Conventions

- **British English** for all documentation, comments, and user-facing text
- Vue SFCs in PascalCase, composables prefixed with `use`, Rust commands grouped by concern
- All TypeScript interfaces centralised in `src/types/index.ts`
- Skeleton loading components in `components/skeletons/` for all data-loading states
- Icons from `lucide-vue-next`; markdown via `marked` + `dompurify`

## Prerequisites

- Rust (stable), Node.js 18+, Tauri CLI v2 (`cargo install tauri-cli --version "^2"`)
- GitHub CLI (`gh`) authenticated
- Optional: Grove CLI, Claude CLI
