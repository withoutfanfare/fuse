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

### [Quality] Add stale review detection with reminder notifications for unacted reviews
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-22
- **Status:** completed (2026-03-22)
- **Description:** The existing notification system triggers alerts on PR events (status changes, risk thresholds), but the most common review bottleneck is inaction — a PR sits in "review requested" status for days without anyone starting. Fuse tracks when reviews are requested but has no mechanism to detect and remind the user when they haven't acted within a configurable window. A stale review detector that monitors time-since-review-requested and surfaces progressively urgent reminders would reduce review queue latency and prevent PRs from going stale, which is the single biggest complaint in most code review workflows.
- **Acceptance criteria:**
  - Stale review threshold configurable per repository (default: 24 hours since review requested)
  - Reminder notification triggered when a review-requested PR exceeds the threshold without status change
  - Reminder escalation: first reminder at threshold, second at 2x threshold, then daily
  - Stale review badge visible on PR cards in the list view (amber at 1x threshold, red at 2x)
  - Stale PRs surfaced in a "Needs your attention" section on the aggregate dashboard
  - Reminders suppressed for PRs the user has already started reviewing (has local checklist progress or time tracked)

### [Feature] Add PR dependency awareness showing blocking and blocked-by relationships
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-22
- **Status:** completed (2026-03-22)
- **Description:** In active codebases, PRs often have implicit dependencies — a feature PR blocked by an infrastructure PR, or a migration PR that must merge before the feature consuming it. The PR list shows individual items without relationship context, so reviewers cannot tell which reviews would unblock other work. Detecting dependencies from PR descriptions (common patterns: "depends on #123", "blocked by #456", "after #789 merges") and displaying them as relationship badges on PR cards would help reviewers prioritise reviews that unblock others, reducing overall queue latency. This complements the stale review detection item by adding a second dimension to review prioritisation: urgency (staleness) and impact (blocking others).
- **Acceptance criteria:**
  - PR descriptions parsed for dependency patterns: "depends on", "blocked by", "after #N merges", "requires #N"
  - Dependency relationships displayed as badges on PR cards (e.g. "Blocks #456", "Blocked by #123")
  - Blocked PRs visually distinguished in the list view (muted or tagged)
  - Dependency chain visible in PR detail view showing the full blocking/blocked-by graph
  - Dependencies resolved against the local PR database (cross-repo dependencies noted but not resolved)
  - Aggregate dashboard shows count of blocking PRs as a priority metric

### [Feature] Add PR label-based quick filters in aggregate and list views
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-22
- **Status:** pending
- **Description:** GitHub labels encode priority, area, and type information (bug, feature, breaking-change, needs-review, WIP) that reviewers use to prioritise their review queue. Fuse's existing filter presets (completed) cover workflow states (My Reviews, High Risk, Stale) but ignore label metadata entirely. Adding label-based filtering — as standalone filters or composable with existing presets — would let reviewers focus on specific PR categories (e.g. "show only breaking-change PRs" or "hide WIP PRs"), aligning Fuse's filtering with the GitHub workflow conventions that teams already use to organise their PRs.
- **Acceptance criteria:**
  - Labels synced from GitHub during PR fetch and stored in the database alongside PR metadata
  - Label filter available in the filter bar: dropdown showing all labels across synced PRs with occurrence counts
  - Multiple labels selectable (OR logic: show PRs with any selected label)
  - Label filters composable with existing presets and status/risk filters
  - Label pills displayed on PR cards in the list view (colour-coded matching GitHub label colours)
  - Aggregate dashboard respects label filters when computing summary statistics

### [Quality] Add review session auto-save preventing progress loss on unexpected quit
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-22
- **Status:** completed (2026-03-22)
- **Description:** During an active review session, reviewers accumulate state — checklist items checked, annotations drafted, time tracked, files marked as reviewed. If Fuse crashes, is force-quit, or the system restarts unexpectedly, all unsaved review progress is lost. The existing time tracking (completed) persists elapsed time to the database, but checklist progress and annotation drafts exist only in component state. Periodically auto-saving the full review session state (checklist progress, draft annotations, reviewed files, current position) to the database would prevent the most frustrating review workflow failure — losing 20 minutes of triage progress to an unexpected quit.
- **Acceptance criteria:**
  - Review session state auto-saved to the database every 30 seconds while a PR detail view is active
  - Saved state includes: checklist item statuses, draft annotation text, files marked as reviewed, scroll position
  - On reopening a PR, saved session state restored with a "Resume previous session?" prompt
  - Auto-save does not interfere with active editing (save runs on a background timer, not on every keystroke)
  - Stale session data cleaned up after 7 days or when the PR is closed/merged
  - Session restore graceful if PR data has changed since the save (new commits, updated files)

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
