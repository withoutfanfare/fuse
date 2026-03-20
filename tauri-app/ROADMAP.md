# Fuse Roadmap

Desktop PR review companion — intelligent pull request monitoring, triage, and AI-assisted code review.

## Completed

### [Feature] Add PR notification system with configurable alerts
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Native macOS notifications triggered by configurable rules (risk score threshold, author, label, review-requested, status change). Quiet hours setting to suppress notifications outside working hours. Notification history viewable in-app via the notification centre.
- **Implementation:** `notification_rules` DB table, `commands/notifications.rs` (CRUD), enhanced `useNotifications` composable with rule evaluation, quiet hours, and risk-threshold-based alerts. Existing `polling.rs` already sends native macOS notifications for PR changes.

### [UX/UI] Add keyboard-driven PR review workflow shortcuts
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-19
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Review workflow shortcuts for navigating between files (]/[/n/p), toggling checklist items (c), jumping to annotations (a), and completing review with advance (Shift+Enter). All documented in the shortcut overlay (?) and command palette.
- **Implementation:** Extended `useKeyboardShortcuts` with `ReviewContext` interface. Updated `ShortcutOverlay` with third column for review shortcuts.

### [Innovation] Add PR review time tracking and personal velocity insights
- **Priority:** P3 (nice-to-have)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Passive time tracking per PR (start on detail open, pause on blur, stop on status change). Personal velocity dashboard showing average review time, time by risk tier, and weekly trends.
- **Implementation:** `review_time_log` DB table, `commands/time_tracking.rs`, `ReviewTimeDashboard.vue` component added to Dashboard view. Existing `useReviewTimer` composable drives data into the new time log.

### [Quality] Add offline mode with cached PR display
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Offline detection with stale data indicators. When network is unavailable, app displays cached data with "Last synced: X ago" banner and retry button. AI review features noted as unavailable.
- **Implementation:** `useOfflineMode` composable (navigator.onLine + events), `OfflineBanner.vue` component. Per-PR staleness via `last_synced_at` field already in schema.

### [Performance] Add incremental PR sync with delta fetching
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Delta sync fetches only PRs updated since last sync timestamp using `gh pr list --search "updated:>TIMESTAMP"`. Full re-sync remains available as manual action.
- **Implementation:** `last_delta_sync_at` column on repositories, `github::fetch_prs_since()`, `sync_pull_requests_incremental` command. Exposed in store as `syncIncremental()` and command palette.

### [Feature] Add configurable review checklist templates per repository
- **Priority:** P3 (nice-to-have)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-20
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Per-repository checklist templates with ordered items, optional descriptions, and a default template (security, tests, documentation, performance). Templates editable via the repository settings view.
- **Implementation:** `checklist_templates` + `checklist_template_items` DB tables, `commands/checklist_templates.rs`, `useChecklistTemplates` composable, `ChecklistTemplateEditor.vue` component.

### [Feature] Add multi-repository aggregate dashboard
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Cross-repo dashboard with summary cards (total open, review-requested, high-risk, stale), top 10 highest-risk PRs across all repositories with repo badges, and per-repository summary rows with PR count, oldest age, and last sync time.
- **Implementation:** `commands/aggregate.rs`, `AggregateDashboard.vue` view, route at `/aggregate`, sidebar navigation entry.

### [UX/UI] Add PR filter presets for common review workflows
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-20
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Built-in presets (My Reviews, High Risk, Stale) plus user-defined presets. Accessible from a preset bar above the filter controls. Save current filters, rename, and delete user presets.
- **Implementation:** `filter_presets` DB table with built-in seed data, `commands/filter_presets.rs`, `useFilterPresets` composable, `FilterPresetsBar.vue` component integrated into PullRequests view.

### [Quality] Add PR diff syntax highlighting for common languages
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-20
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Language-aware syntax highlighting in the diff view, detecting language from file extension. Supports TypeScript, JavaScript, Rust, Python, Go, PHP, CSS, HTML/XML, JSON, YAML, Markdown, SQL, Bash. Loaded lazily via dynamic import.
- **Implementation:** `highlight.js` dependency (core + 13 language grammars), `useSyntaxHighlight` composable with lazy loading, updated `DiffViewer.vue` with v-html highlighted content and dark mode colour theme.

### [Feature] Add review summary generation for GitHub posting
- **Priority:** P3 (nice-to-have)
- **Size:** S (< 1hr)
- **Added:** 2026-03-20
- **Completed:** 2026-03-20
- **Status:** done
- **Description:** Generate formatted review summary (checklist status, annotation excerpts, risk assessment, files reviewed, time spent) as GitHub-flavoured Markdown. Copy to clipboard or post directly to the PR as a comment via gh CLI.
- **Implementation:** `useReviewSummary` composable, `commands/review_summary.rs` for gh CLI posting, `ReviewSummaryPanel.vue` component integrated into PR detail overview tab.

## Pending

### [Distribution] Add Tauri auto-updater integration
- **Priority:** P3 (nice-to-have)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** pending
- **Description:** As a tool that interacts with GitHub's evolving API surface, Fuse needs a reliable update mechanism. Tauri's built-in updater plugin can check for new versions and install updates seamlessly. Without this, users must manually download and replace the app binary, which creates version fragmentation and delays critical fixes reaching users.
- **Acceptance criteria:**
  - Tauri updater plugin configured with update endpoint
  - App checks for updates on launch and periodically (configurable interval)
  - Update notification shown in the UI with release notes summary
  - User can install update immediately or defer to next launch
  - Current version displayed in settings/about view

### [UX/UI] Add inline file-level review comments with GitHub sync
- **Priority:** P3 (nice-to-have)
- **Size:** L (3-8hrs)
- **Added:** 2026-03-19
- **Status:** pending
- **Description:** Fuse currently provides AI review annotations and a local checklist, but reviewers cannot post review comments back to GitHub from within the app. The review loop is broken — users must switch to the browser to leave feedback. Adding inline file-level comment composition that syncs to GitHub via the gh CLI would close the review loop entirely, making Fuse a complete review environment rather than a triage-only tool.
- **Acceptance criteria:**
  - Comment composition UI on diff view file lines (click line number to add comment)
  - Comments posted to GitHub as PR review comments via gh CLI
  - Existing GitHub comments fetched and displayed inline alongside AI annotations
  - Pending comments collected into a review batch (submit all at once with approve/request-changes/comment status)
  - Comment sync status indicator (local draft, posting, posted, failed)
  - Reply-to-comment support for threaded conversations

## Design System Adoption

These items implement the @stuntrocket/ui design system to achieve premium visual uniformity across all Tauri applications. Items are ordered by dependency — foundation must complete before migration, migration before polish.

### [Foundation] Integrate @stuntrocket/ui shared component library and design tokens
- **Priority:** P1 (critical)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** pending

### [UI Migration] Replace bespoke components with @stuntrocket/ui shared components
- **Priority:** P1 (critical)
- **Size:** XL (8hrs+)
- **Added:** 2026-03-19
- **Status:** pending

### [Polish] Achieve full @stuntrocket/ui styleguide visual conformance
- **Priority:** P2 (important)
- **Size:** L (3-8hrs)
- **Added:** 2026-03-19
- **Status:** pending
