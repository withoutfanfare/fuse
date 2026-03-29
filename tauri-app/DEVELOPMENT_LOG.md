# Fuse Development Log

## Cycle: 2026-03-29

- App: Fuse
- Items completed:
  - [Feature] PR label-based quick filters (P2/S) — `label_colours TEXT` column on `pull_requests`, `get_all_labels` command aggregating labels with counts and colours, `useLabelFilter` composable with multi-select toggle, colour-coded label pills on PRTable rows, label chip filter bar in PullRequests view.
  - [Quality] Repository sync health monitoring (P2/S) — `get_sync_health` command querying `sync_log` for consecutive failures per repo, `useSyncHealth` composable with `unhealthyRepos`/`hasIssues` computeds, `SyncHealthBanner.vue` warning component.
  - [Feature] GitHub Actions CI check status display (P2/S) — `statusCheckRollup` fetched during sync, `compute_ci_status` helper (passing/failing/pending), `ci_status TEXT` column, CI badge in PRTable status column, CI filter buttons in filter bar.
- Items attempted but failed: none
- Branch: feature/labels-sync-health-ci-status
- Tests passing: yes (vue-tsc clean, cargo check clean, cargo clippy clean excluding 5 pre-existing warnings)
- Build status: pending
- Notes: Three P2/S features implemented across the full stack. Label colours are stored as a JSON map alongside the existing labels JSON array to avoid breaking existing label filtering logic. CI status is computed from GitHub's `statusCheckRollup` field which provides individual check conclusions — the helper reduces these to a single rollup status. Sync health leverages the existing `sync_log` table to detect failure streaks without adding new tracking infrastructure.

### Files Created

**Rust:**
- `src-tauri/src/commands/labels.rs` — Label summary aggregation command
- `src-tauri/src/commands/sync_health.rs` — Sync health status per repository

**Vue/TypeScript:**
- `src/composables/useLabelFilter.ts` — Label fetching and multi-select state
- `src/composables/useSyncHealth.ts` — Sync health fetching and unhealthy repo computation
- `src/components/SyncHealthBanner.vue` — Warning banner for failing syncs

### Files Modified

**Rust:**
- `src-tauri/src/models/mod.rs` — Added `LabelSummary`, `SyncHealthStatus`, `GhStatusCheck`, `CiCheck` structs; `label_colours` and `ci_status` fields on `PullRequest`; `color` field on `GhLabel`; `status_check_rollup` on `GhPrJson`
- `src-tauri/src/github/mod.rs` — Added `statusCheckRollup` to `GH_PR_FIELDS`
- `src-tauri/src/db/migrations.rs` — Added `label_colours` and `ci_status` column migrations
- `src-tauri/src/commands/sync.rs` — Added `build_label_colours_json` and `compute_ci_status` helpers; updated upsert SQL to include new columns
- `src-tauri/src/commands/pull_requests.rs` — Updated `PR_SELECT`, `PR_SELECT_WITH_BODY`, `parse_pr_row`, `parse_pr_row_with_body` for new column indices
- `src-tauri/src/commands/mod.rs` — Registered `labels` and `sync_health` modules
- `src-tauri/src/lib.rs` — Registered `get_all_labels` and `get_sync_health` commands

**Vue/TypeScript:**
- `src/types/index.ts` — Added `label_colours`, `ci_status` to `PullRequest`; new `LabelSummary` and `SyncHealthStatus` interfaces
- `src/components/PRTable.vue` — Label pills with colour styling, CI status badge, `labelPillStyle` helper
- `src/views/PullRequests.vue` — Label chip filter bar, CI status filter buttons, sync health banner, label/CI filtering in `filteredPrs` computed

---

## Cycle: 2026-03-25 14:00
- App: Fuse
- Items completed:
  - [Foundation] Integrate @stuntrocket/ui shared component library and design tokens (P1/M) — completed the dark mode migration from data-theme attribute to class-based .dark on html, aligning Fuse with the @stuntrocket/ui convention used across the portfolio. The @stuntrocket/ui package (v0.9.1) was already installed with .npmrc configured and style.css imported in base.css; Poppins font was already loaded via Google Fonts; several @stuntrocket/ui components (SAmbientBlobs, STopbar, SCommandPalette, SButton, SCard, SBadge, etc.) were already in use throughout the app. This cycle completed the remaining work: tokens.css light mode selector changed from [data-theme="light"] to html:not(.dark), useTheme composable switched from setAttribute('data-theme') to classList.toggle('dark'), and index.html inline script updated to remove .dark class for light mode instead of setting data-theme attribute.
- Items attempted but failed: none
- Branch: feature/scooda-design-tokens
- Tests passing: yes (vue-tsc clean, cargo check clean, cargo clippy clean excluding 4 pre-existing warnings)
- Build status: pending
- Notes: The Foundation integration was substantially complete from prior work — @stuntrocket/ui v0.9.1 installed, CSS imported, Poppins loaded, and many shared components already in use. The only remaining gap was the dark mode mechanism: Fuse used data-theme attributes while @stuntrocket/ui expects .dark class on html. This cycle closed that gap with a minimal, focused change across 3 files. The teal accent override (#14b8a6) and all Fuse-specific tokens (risk colours, pipeline track, density mode, focus mode, high contrast mode) are preserved as app-specific additions layered on top of the shared token system.

## 2026-03-22 — Stale Review Detection, Dependency Badges, and Session Auto-Save

### Summary

Implemented three P2/S roadmap items focused on review prioritisation and session resilience. Full-stack changes: Rust backend commands, SQLite schema updates, TypeScript types, Vue composables, and component integration.

### Items Implemented

1. **Stale review detection with reminder notifications** — `get_stale_review_requests` command in `stale.rs` with configurable threshold (`stale_review_hours` setting, default 24h), escalation levels (1/2/3), and local progress detection (checklist + time log). `useStaleReviews` composable with `stalePrMap` and `attentionItems` computeds. "Needs your attention" section on `AggregateDashboard.vue` with escalation badges. Stale review badges on `PRTable.vue` rows (amber at level 1, red at level 2+).

2. **PR dependency awareness with blocking/blocked-by badges** — Existing `pr_dependencies` table and `compute_dependencies`/`get_pr_dependencies` commands leveraged. `PRTable.vue` now accepts optional `dependencies` prop and renders "Blocks N" (red) and "Needs N" (teal) inline badges beside PR titles using computed `blocksCountMap`/`dependsOnCountMap`.

3. **Review session auto-save** — `save_session_snapshot` and `cleanup_stale_sessions` commands in `stale.rs`. `useSessionAutoSave` composable with 30-second interval persistence of reviewed files. Integrated into `PullRequestDetail.vue` with checklist watcher and "Saved" indicator badge. Auto-save starts after secondary data loads and stops/flushes on unmount.

### Files Created

- `src/composables/useStaleReviews.ts` — Stale review fetching and attention item computation
- `src/composables/useSessionAutoSave.ts` — Periodic session snapshot persistence

### Files Modified

**Rust:**
- `src-tauri/src/commands/stale.rs` — Added `StaleReviewItem` struct, `parse_iso_timestamp`, `get_stale_review_requests`, `save_session_snapshot`, `cleanup_stale_sessions`
- `src-tauri/src/db/migrations.rs` — Added `stale_review_hours` default setting
- `src-tauri/src/lib.rs` — Registered 3 new commands

**Frontend:**
- `src/types/index.ts` — Added `StaleReviewItem` interface
- `src/components/PRTable.vue` — Added stale review badges, dependency badges, new props
- `src/views/AggregateDashboard.vue` — Added "Needs your attention" section with stale review items
- `src/views/PullRequestDetail.vue` — Integrated auto-save composable with checklist watcher

### Branch

`feature/stale-detection-dependencies-autosave`

---

## 2026-03-20 — Batch Implementation of 10 Roadmap Items

### Summary

Implemented all 10 pending functional roadmap items in a single development session. The implementation spans the full stack: Rust backend (SQLite schema, command modules, GitHub CLI integration), TypeScript types, Vue composables and stores, and Vue components/views.

### Items Implemented

1. **PR notification system with configurable alerts** — `notification_rules` table, `commands/notifications.rs`, enhanced `useNotifications` composable with quiet hours and risk threshold settings.

2. **Keyboard-driven PR review workflow shortcuts** — Extended `useKeyboardShortcuts` with `ReviewContext` for file navigation (]/[), checklist toggle (c), annotation jump (a), and complete-and-advance (Shift+Enter). Updated `ShortcutOverlay` with review shortcuts column.

3. **PR review time tracking and personal velocity insights** — `review_time_log` table, `commands/time_tracking.rs` with velocity stats (avg time, by risk tier, weekly trend), `ReviewTimeDashboard.vue` component on the main dashboard.

4. **Offline mode with cached PR display** — `useOfflineMode` composable detecting navigator.onLine + events, `OfflineBanner.vue` with stale indicator and retry button, per-PR staleness via `last_synced_at`.

5. **Incremental PR sync with delta fetching** — `last_delta_sync_at` column on repositories, `github::fetch_prs_since()` using `gh pr list --search "updated:>TIMESTAMP"`, `sync_pull_requests_incremental` command, exposed as `syncIncremental()` in store and command palette.

6. **Configurable review checklist templates per repository** — `checklist_templates` + `checklist_template_items` tables, `commands/checklist_templates.rs` (CRUD), `useChecklistTemplates` composable, `ChecklistTemplateEditor.vue`.

7. **Multi-repository aggregate dashboard** — `commands/aggregate.rs` with cross-repo stats (total open, review-requested, high-risk, stale), top 10 risk PRs, per-repo summaries. `AggregateDashboard.vue` view at `/aggregate` with sidebar nav entry.

8. **PR filter presets** — `filter_presets` table with built-in seed data (My Reviews, High Risk, Stale), `commands/filter_presets.rs`, `useFilterPresets` composable, `FilterPresetsBar.vue` in the PR list view.

9. **PR diff syntax highlighting** — `highlight.js` dependency with 13 language grammars loaded lazily via dynamic import. `useSyntaxHighlight` composable, updated `DiffViewer.vue` with `v-html` highlighted content and custom dark mode colour scheme.

10. **Review summary generation for GitHub posting** — `useReviewSummary` composable generating GitHub-flavoured Markdown from checklist state, bookmarks, risk assessment, and time spent. `commands/review_summary.rs` for `gh pr comment` posting. `ReviewSummaryPanel.vue` in PR detail overview tab.

### Files Created

**Rust (src-tauri/src/):**
- `commands/notifications.rs` — Notification rule CRUD
- `commands/time_tracking.rs` — Review time logging and velocity stats
- `commands/checklist_templates.rs` — Per-repo checklist template CRUD
- `commands/filter_presets.rs` — Filter preset CRUD
- `commands/aggregate.rs` — Cross-repo aggregate dashboard stats
- `commands/review_summary.rs` — Post review summary to GitHub via gh CLI

**Vue/TypeScript (src/):**
- `composables/useOfflineMode.ts` — Offline detection and stale indicators
- `composables/useReviewSummary.ts` — Review summary Markdown generation
- `composables/useFilterPresets.ts` — Filter preset management
- `composables/useChecklistTemplates.ts` — Checklist template management
- `composables/useSyntaxHighlight.ts` — Lazy highlight.js integration
- `components/OfflineBanner.vue` — Offline warning banner
- `components/FilterPresetsBar.vue` — Preset chip bar
- `components/ChecklistTemplateEditor.vue` — Template CRUD UI
- `components/ReviewTimeDashboard.vue` — Personal velocity dashboard
- `components/ReviewSummaryPanel.vue` — Summary generation and posting
- `views/AggregateDashboard.vue` — Multi-repo overview

### Files Modified

**Rust:**
- `db/migrations.rs` — 5 new tables, 1 new column, 4 new settings
- `commands/mod.rs` — 6 new module declarations
- `commands/sync.rs` — Incremental sync command and delta fetch logic
- `github/mod.rs` — `fetch_prs_since()` for delta sync
- `lib.rs` — 17 new command registrations

**Vue/TypeScript:**
- `types/index.ts` — 15+ new interfaces
- `stores/pullRequests.ts` — `syncIncremental()` method
- `composables/useKeyboardShortcuts.ts` — `ReviewContext` and review shortcuts
- `composables/useNotifications.ts` — Configurable rules, quiet hours, change notifications
- `composables/useCommandPalette.ts` — Aggregate nav and incremental sync entries
- `components/DiffViewer.vue` — Syntax highlighting integration
- `components/ShortcutOverlay.vue` — Review shortcuts column
- `components/layout/AppSidebar.vue` — Aggregate nav entry
- `components/layout/AppHeader.vue` — Aggregate page title
- `views/Dashboard.vue` — ReviewTimeDashboard section
- `views/PullRequests.vue` — FilterPresetsBar, OfflineBanner
- `views/PullRequestDetail.vue` — ReviewSummaryPanel
- `router/index.ts` — `/aggregate` route

### Dependencies Added

- `highlight.js` — Syntax highlighting library (lazily loaded, core + 13 language grammars)

### Items Skipped

- **Tauri auto-updater** — Requires update endpoint infrastructure (server-side)
- **Inline file-level review comments with GitHub sync** — L-sized item requiring complex GitHub review API integration
- **Design System Adoption** — Separate initiative (Foundation, Migration, Polish)
