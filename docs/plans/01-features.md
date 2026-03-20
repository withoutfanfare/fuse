# Feature Implementation Plan

## Architecture Summary (Current State)

The app follows a clear pattern:
- **Backend**: Tauri v2 commands in `src-tauri/src/commands/*.rs`, registered in `lib.rs` via `generate_handler![]`. Database state is `DbState(Mutex<Connection>)`. GitHub interaction via `std::process::Command` calling `gh` CLI. Models in `models/mod.rs`. Schema in `db/migrations.rs`.
- **Frontend**: Vue 3 Composition API + Pinia stores calling `invoke()` from `@tauri-apps/api/core`. Views in `views/`, components in `components/`, composables in `composables/`, types in `types/index.ts`.
- **Dependencies**: rusqlite (bundled), chrono, serde, thiserror on Rust side. Vue 3, Pinia, vue-router, @tauri-apps/api, @tauri-apps/plugin-opener on frontend.
- **Conventions**: `CommandError` enum for all error types. `snake_case` Tauri command names. Pinia stores use setup function syntax. CSS uses design tokens from `tokens.css`. All commands are synchronous Rust functions (no async/tokio yet).

---

## Feature 1: Auto-Sync Polling

**Status**: ✅ Implemented
**Implementation notes**: `polling.rs` backend module with Tokio timer, `start_polling`/`stop_polling`/`update_poll_interval` commands in `lib.rs`, `sync-completed` event listener in `pullRequests.ts` store, auto-start in `App.vue`.

### Summary
Background polling on a configurable interval that automatically calls `sync_pull_requests` using a Tokio timer in the Rust backend, reading `poll_interval_seconds` from the `app_settings` table.

### Files to create
- `tauri-app/src-tauri/src/polling.rs` — Tokio-based polling loop module that reads settings, triggers sync, and emits events to the frontend

### Files to modify
- `tauri-app/src-tauri/Cargo.toml` — Add `tokio` with `time` and `rt` features
- `tauri-app/src-tauri/src/lib.rs` — Add `mod polling;`, spawn polling task in `.setup()`, register new commands `start_polling`, `stop_polling`, `update_poll_interval`
- `tauri-app/src-tauri/src/db/migrations.rs` — No change needed; `app_settings` table already has `poll_interval_seconds` seeded
- `tauri-app/src/stores/pullRequests.ts` — Add `listen()` call for a `sync-completed` event from the backend, auto-refresh store data when received
- `tauri-app/src/App.vue` — Start listening for sync events on mount
- `tauri-app/src-tauri/capabilities/default.json` — Add `"event:default"` permission if needed for event emission

### Implementation steps
1. Add `tokio = { version = "1", features = ["time", "rt", "sync"] }` to Cargo.toml dependencies
2. Create `polling.rs` with a struct `PollState` wrapping an `Arc<AtomicBool>` for enabled/disabled and an `Arc<AtomicU64>` for interval seconds
3. In `polling.rs`, implement a function `start_poll_loop(app_handle: AppHandle, db_state: State<DbState>, poll_state: Arc<PollState>)` that loops with `tokio::time::sleep`, reads `poll_interval_seconds` from `app_settings`, calls the sync logic, and emits a Tauri event `sync-completed` with the `Vec<SyncResult>` payload
4. In `lib.rs`, change the Tauri runtime to use async setup: create a `PollState`, manage it, and spawn the polling task using `tauri::async_runtime::spawn`
5. Add commands `start_polling`, `stop_polling`, `update_poll_interval` that modify the `PollState`
6. In `pullRequests.ts`, import `listen` from `@tauri-apps/api/event` and set up a listener for `sync-completed` that calls `fetchAll()` and `fetchStats()`
7. In `App.vue`, invoke `start_polling` on mount

### Dependencies
- Feature 5 (Settings Backend Connection) should be built first or concurrently, since poll interval needs to be read/written from `app_settings`

### Estimated scope
**Large** (3+ hours) — Requires introducing Tokio async runtime, managing shared state across threads, and event emission pattern

---

## Feature 2: Native Notifications

**Status**: ✅ Implemented
**Implementation notes**: `tauri-plugin-notification` registered in `lib.rs`, notification logic in `sync.rs` detecting new and high-risk PRs, CSP and capability permissions configured.

### Summary
Send macOS native notifications via `tauri-plugin-notification` when new PRs or high-risk PRs are detected during sync.

### Files to create
None — logic integrates into existing sync flow

### Files to modify
- `tauri-app/src-tauri/Cargo.toml` — Add `tauri-plugin-notification = "2"`
- `tauri-app/src-tauri/src/lib.rs` — Register `.plugin(tauri_plugin_notification::init())`
- `tauri-app/src-tauri/capabilities/default.json` — Add `"notification:default"` permission
- `tauri-app/src-tauri/src/commands/sync.rs` — After upsert, compare old vs new PR counts, detect new PRs, compute risk scores, and send notifications via the plugin
- `tauri-app/src-tauri/src/models/mod.rs` — Optionally add a helper `risk_score()` method on the Rust side (or compute inline)
- `tauri-app/package.json` — Add `@tauri-apps/plugin-notification` frontend dependency

### Implementation steps
1. Add the Cargo dependency and npm dependency for the notification plugin
2. Register the plugin in `lib.rs` and add the capability permission
3. In `sync.rs`, before the upsert transaction, query the existing PR IDs/numbers for the repo being synced
4. After upsert, determine which PRs are new (number not in pre-sync set)
5. For new PRs, compute a basic risk score (replicate the logic from `useRiskScore.ts` in Rust: file count, additions+deletions, age)
6. If any new PRs exist or any have risk score >= 7, call `app.notification().builder().title("New PRs").body(...)` to send the notification
7. Pass `AppHandle` through to the sync function (requires refactoring `sync_pull_requests` to accept `app: AppHandle` or extracting it from state)

### Dependencies
- None strictly, but benefits from Feature 1 (Auto-Sync) since notifications are most useful during background polling

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 3: PR Diff Viewer

**Status**: ✅ Implemented
**Implementation notes**: `commands/diff.rs` with `fetch_pr_diff`/`get_cached_diff`, `DiffViewer.vue` and `DiffFileTree.vue` components, `useDiff.ts` composable, `diff_cache` table, `head_sha` column on `pull_requests`, syntax highlighting integration.

### Summary
Fetch the diff for a PR via `gh pr diff`, display it with syntax highlighting and a collapsible file tree, caching by `head_sha` to avoid redundant fetches.

### Files to create
- `tauri-app/src-tauri/src/commands/diff.rs` — Tauri commands `fetch_pr_diff` and `get_cached_diff`
- `tauri-app/src/components/DiffViewer.vue` — Main diff display component with file tree and syntax-highlighted hunks
- `tauri-app/src/components/DiffFileTree.vue` — Collapsible file tree sidebar for the diff
- `tauri-app/src/composables/useDiff.ts` — Composable for diff fetching/parsing logic

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod diff;`
- `tauri-app/src-tauri/src/lib.rs` — Register `commands::diff::fetch_pr_diff`, `commands::diff::get_cached_diff`
- `tauri-app/src-tauri/src/models/mod.rs` — Add `GhPrJson.headRefOid` (head_sha) field, add `DiffCache` struct, add `head_sha` column to `pull_requests` table
- `tauri-app/src-tauri/src/db/migrations.rs` — Add `ALTER TABLE pull_requests ADD COLUMN head_sha TEXT;` and new `CREATE TABLE IF NOT EXISTS diff_cache (pr_id INTEGER, head_sha TEXT, diff_text TEXT, cached_at TEXT, UNIQUE(pr_id))`
- `tauri-app/src-tauri/src/github/mod.rs` — Add `fetch_diff(full_name: &str, pr_number: i64) -> Result<String, CommandError>` function calling `gh pr diff`; update `GH_PR_FIELDS` to include `headRefOid`
- `tauri-app/src/views/PullRequestDetail.vue` — Add a tab/section to show the DiffViewer component
- `tauri-app/src/types/index.ts` — Add `DiffFile`, `DiffHunk` interfaces
- `tauri-app/package.json` — Add `shiki` or `highlight.js` for syntax highlighting

### Implementation steps
1. Add `headRefOid` to `GH_PR_FIELDS` and `GhPrJson` struct, add `head_sha` to `PullRequest` model and the `pull_requests` schema
2. Add migration for `head_sha` column and `diff_cache` table
3. Update sync upsert to store `head_sha`
4. In `github/mod.rs`, add `fetch_diff()` that runs `gh pr diff --repo <name> <number>` and returns raw unified diff text
5. In `commands/diff.rs`, implement `fetch_pr_diff` that checks `diff_cache` by `pr_id` + `head_sha`; if cache miss, calls `fetch_diff()`, stores result, returns it
6. On the frontend, create `useDiff.ts` composable that invokes `fetch_pr_diff` and parses the unified diff into `DiffFile[]` with hunks
7. Build `DiffFileTree.vue` as a collapsible tree organised by directory path
8. Build `DiffViewer.vue` that combines the file tree and renders each file's hunks with syntax highlighting (using `shiki` or `highlight.js`)
9. Integrate into `PullRequestDetail.vue` as a new collapsible section or tab

### Dependencies
- None strictly, but `head_sha` migration needs careful sequencing with other schema changes

### Estimated scope
**Large** (3+ hours) — Diff parsing, syntax highlighting, and the file tree UI are all non-trivial

---

## Feature 4: Review Rules UI

**Status**: ✅ Implemented
**Implementation notes**: `ReviewRulesEditor.vue` with drag-and-drop reordering, integrated into `Repositories.vue` per-repo and `PullRequestDetail.vue` as read-only checklist.

### Summary
A frontend interface for the existing `review_rules` table, allowing users to add, edit, reorder (drag-and-drop), and delete per-repository checklist items.

### Files to create
- `tauri-app/src/components/ReviewRulesEditor.vue` — Full CRUD editor with drag-and-drop reordering

### Files to modify
- `tauri-app/src/views/PullRequestDetail.vue` — Add a section or sidebar panel showing the repo's review rules as a checklist
- `tauri-app/src/views/Repositories.vue` — Add a "Manage Rules" button per repo that opens the editor (inline or modal)
- `tauri-app/src/stores/pullRequests.ts` — Already has `fetchRules()` and `setRules()` — these are sufficient
- `tauri-app/src/types/index.ts` — Already has `ReviewRule` interface — no change needed

### Implementation steps
1. Create `ReviewRulesEditor.vue` that takes a `repoId` prop
2. On mount, call `prStore.fetchRules(repoId)` to load existing rules
3. Display rules as an editable list with text inputs, a drag handle (use HTML5 drag and drop or a lightweight library like `vuedraggable`), and delete buttons per row
4. Add an "Add Rule" button that appends a new empty text input at the bottom
5. On save, collect all rule texts in order and call `prStore.setRules(repoId, rules)`
6. In `Repositories.vue`, add a toggle/button on each `RepositoryCard` that expands the `ReviewRulesEditor` below the card
7. In `PullRequestDetail.vue`, display the rules as a read-only checklist (with checkboxes for the reviewer's reference, stored client-side in session only)

### Dependencies
- None — backend commands `get_review_rules` and `set_review_rules` already exist

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 5: Settings Backend Connection

**Status**: ✅ Implemented
**Implementation notes**: `commands/settings.rs` with `get_settings`/`update_setting`, `stores/settings.ts` Pinia store, `Settings.vue` wired to backend persistence via `app_settings` table.

### Summary
Wire the existing `Settings.vue` form to new Tauri commands that read from and write to the `app_settings` table in SQLite.

### Files to create
- `tauri-app/src-tauri/src/commands/settings.rs` — Commands `get_settings` and `update_setting`
- `tauri-app/src/stores/settings.ts` — Pinia store for app settings

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod settings;`
- `tauri-app/src-tauri/src/lib.rs` — Register `commands::settings::get_settings` and `commands::settings::update_setting`
- `tauri-app/src-tauri/src/models/mod.rs` — Add `AppSetting` struct `{ key: String, value: String }`
- `tauri-app/src/views/Settings.vue` — Replace the local `ref` and no-op `saveSettings()` with calls to the settings store
- `tauri-app/src/types/index.ts` — Add `AppSettings` interface

### Implementation steps
1. Create `commands/settings.rs` with two commands:
   - `get_settings()` — `SELECT key, value FROM app_settings` returning `Vec<AppSetting>` or a `HashMap<String, String>`
   - `update_setting(key: String, value: String)` — `INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)`
2. Register both commands in `lib.rs`
3. Create `stores/settings.ts` Pinia store with reactive `settings` map, `fetchSettings()` and `updateSetting(key, value)` actions
4. In `Settings.vue`, replace local state: on mount call `settingsStore.fetchSettings()`, bind inputs to store values, on save call `settingsStore.updateSetting()` for each changed key
5. Extend Settings.vue with additional settings rows as needed (editor command for Feature 7, stale PR threshold for Feature 14, etc.)

### Dependencies
- None

### Estimated scope
**Small** (< 1 hour)

---

## Feature 6: PR Comment Thread Viewer

**Status**: ✅ Implemented
**Implementation notes**: `commands/comments.rs` with `fetch_pr_comments`, `CommentThread.vue` component, `useComments.ts` composable, threaded conversation rendering in `PullRequestDetail.vue`.

### Summary
Fetch PR comments and review threads via `gh pr view --json comments,reviews` and display them as a threaded conversation in the PR detail view.

### Files to create
- `tauri-app/src-tauri/src/commands/comments.rs` — Command `fetch_pr_comments`
- `tauri-app/src/components/CommentThread.vue` — Renders a threaded conversation view
- `tauri-app/src/composables/useComments.ts` — Composable for fetching and managing comment state

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod comments;`
- `tauri-app/src-tauri/src/lib.rs` — Register `commands::comments::fetch_pr_comments`
- `tauri-app/src-tauri/src/github/mod.rs` — Add `fetch_comments(full_name: &str, pr_number: i64)` function
- `tauri-app/src-tauri/src/models/mod.rs` — Add `GhComment`, `GhReviewThread` structs for deserialising gh output
- `tauri-app/src/views/PullRequestDetail.vue` — Add a "Comments" section that uses `CommentThread`
- `tauri-app/src/types/index.ts` — Add `PrComment`, `ReviewThread` interfaces

### Implementation steps
1. In `github/mod.rs`, add `fetch_comments()` that runs `gh pr view <number> --repo <name> --json comments,reviews,reviewThreads` and parses the JSON
2. Add Rust models for the gh CLI JSON output: `GhComment { author: GhAuthor, body: String, createdAt: String }`, `GhReview { author: GhAuthor, body: String, state: String, comments: Vec<GhComment> }`
3. In `commands/comments.rs`, implement `fetch_pr_comments(pr_id, state)` that looks up the repo/number from the DB, calls `fetch_comments()`, and returns the parsed data
4. On the frontend, add TypeScript interfaces matching the response
5. Create `useComments.ts` composable with `fetchComments(prId)` wrapping the invoke call
6. Build `CommentThread.vue` that renders comments chronologically, grouping review comments under their review header, rendering markdown bodies
7. Add the component to `PullRequestDetail.vue` in a new section below the existing content

### Dependencies
- None

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 7: One-Click Open in Editor

**Status**: ✅ Implemented
**Implementation notes**: `commands/editor.rs` with `open_in_editor`, editor command setting in `app_settings`, "Open in Editor" button in `WorktreePanel.vue`, configurable editor command in `Settings.vue`.

### Summary
Replace the clipboard-copy workflow with a button that spawns a configurable editor command (e.g. `code`, `cursor`, `zed`) at the worktree path.

### Files to create
- `tauri-app/src-tauri/src/commands/editor.rs` — Command `open_in_editor`

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod editor;`
- `tauri-app/src-tauri/src/lib.rs` — Register `commands::editor::open_in_editor`
- `tauri-app/src/components/WorktreePanel.vue` — Add "Open in Editor" button that calls the new command
- `tauri-app/src/views/Settings.vue` — Add a settings row for configuring the editor command (stored in `app_settings` as `editor_command`)
- `tauri-app/src-tauri/src/db/migrations.rs` — Add `INSERT OR IGNORE INTO app_settings(key, value) VALUES ('editor_command', 'code');` to seed the default

### Implementation steps
1. Add the seed value for `editor_command` in migrations
2. Create `commands/editor.rs` with `open_in_editor(path: String, state: State<DbState>)` that reads `editor_command` from `app_settings`, then spawns `Command::new(editor).arg(path).spawn()` (detached, fire-and-forget)
3. Register the command in `lib.rs`
4. In `WorktreePanel.vue`, add a new button "Open in Editor" alongside "Copy Review Command" that calls `invoke('open_in_editor', { path: branchWorktree.path })`
5. In `Settings.vue`, add a row with a text input for the editor command, persisted via the settings store (Feature 5)

### Dependencies
- Feature 5 (Settings Backend Connection) for reading/writing the `editor_command` setting

### Estimated scope
**Small** (< 1 hour)

---

## Feature 8: Batch Approve/Merge

**Status**: ✅ Implemented
**Implementation notes**: `commands/batch.rs` with `batch_approve`/`batch_merge`, `BatchActionBar.vue` floating action bar, checkbox column in `PRTable.vue`, selection state in `PullRequests.vue`, `BatchResult` model.

### Summary
Multi-select PRs in the PR table, then bulk approve or merge them with progress feedback.

### Files to create
- `tauri-app/src-tauri/src/commands/batch.rs` — Commands `batch_approve`, `batch_merge`
- `tauri-app/src/components/BatchActionBar.vue` — Floating action bar shown when PRs are selected

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod batch;`
- `tauri-app/src-tauri/src/lib.rs` — Register `commands::batch::batch_approve`, `commands::batch::batch_merge`
- `tauri-app/src-tauri/src/models/mod.rs` — Add `BatchResult { pr_id: i64, success: bool, message: String }` struct
- `tauri-app/src/components/PRTable.vue` — Add checkbox column, selection state, emit selected IDs
- `tauri-app/src/views/PullRequests.vue` — Manage selection state, show `BatchActionBar` when items selected
- `tauri-app/src/stores/pullRequests.ts` — Add `batchApprove(prIds)` and `batchMerge(prIds, method)` actions
- `tauri-app/src/types/index.ts` — Add `BatchResult` interface

### Implementation steps
1. In `commands/batch.rs`, implement `batch_approve(pr_ids: Vec<i64>, body: Option<String>, state)` that iterates over IDs, calls `get_pr_context` + `github::approve_pr` for each, collects results. Similarly for `batch_merge` with forbidden-target enforcement per PR
2. Return `Vec<BatchResult>` so the frontend can display per-PR success/failure
3. Register commands in `lib.rs`
4. In `PRTable.vue`, add a checkbox `<td>` as the first column, with a "select all" checkbox in the header. Maintain a `Set<number>` of selected PR IDs via `defineModel` or emit
5. In `PullRequests.vue`, track selection state. When selection is non-empty, render `BatchActionBar.vue`
6. `BatchActionBar.vue` shows selected count, "Approve All" and "Merge All" buttons, and a progress/results area
7. Wire buttons to `prStore.batchApprove()` / `prStore.batchMerge()`, display results inline with success/error per PR
8. After batch completes, refresh the PR list

### Dependencies
- None strictly (uses existing approve/merge backend logic)

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 9: PR Age Heatmap & Velocity Dashboard

**Status**: ✅ Implemented
**Implementation notes**: `commands/analytics.rs` with `get_age_distribution`/`get_review_velocity`, `AgeHeatmap.vue` and `VelocityChart.vue` components, charting integrated into `Dashboard.vue`.

### Summary
Aggregate SQL queries for review throughput and PR aging, displayed as charts on a new dashboard section.

### Files to create
- `tauri-app/src-tauri/src/commands/analytics.rs` — Commands `get_age_distribution`, `get_review_velocity`
- `tauri-app/src/components/AgeHeatmap.vue` — Visual heatmap/bar chart of PR ages
- `tauri-app/src/components/VelocityChart.vue` — Line/bar chart of review throughput over time

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod analytics;`
- `tauri-app/src-tauri/src/lib.rs` — Register new analytics commands
- `tauri-app/src-tauri/src/models/mod.rs` — Add `AgeBucket { label: String, count: i64 }`, `VelocityPoint { date: String, reviewed: i64, merged: i64 }` structs
- `tauri-app/src/views/Dashboard.vue` — Add age heatmap and velocity chart sections below the existing stats grid
- `tauri-app/src/stores/pullRequests.ts` — Add `fetchAgeDistribution()` and `fetchReviewVelocity()` actions
- `tauri-app/src/types/index.ts` — Add corresponding TypeScript interfaces
- `tauri-app/package.json` — Add a lightweight charting library (e.g. `chart.js` + `vue-chartjs`, or `unovis`)

### Implementation steps
1. In `commands/analytics.rs`, implement `get_age_distribution(state)` with SQL: bucket open PRs by age (0-1d, 1-3d, 3-7d, 7-14d, 14+d) using `julianday('now') - julianday(created_at)`
2. Implement `get_review_velocity(days: Option<i64>, state)` with SQL: group `pr_reviews` by `DATE(reviewed_at)` for the last N days, counting reviews completed per day; also count merges from `pull_requests` by `DATE(merged_at)`
3. Register commands
4. Add frontend types and store actions
5. Install charting library, build `AgeHeatmap.vue` rendering a colour-coded bar chart
6. Build `VelocityChart.vue` rendering a time-series chart
7. Add both to `Dashboard.vue` below the "Needs Attention" section

### Dependencies
- None

### Estimated scope
**Large** (3+ hours) — SQL aggregation design + charting library integration + two chart components

---

## Feature 10: Author Performance Insights

**Status**: ✅ Implemented
**Implementation notes**: `commands/authors.rs` with `get_author_stats`, `AuthorStatsTable.vue` component, `Authors.vue` view at `/authors` route, sidebar nav entry.

### Summary
Per-author metrics (PRs opened, average time to merge, average size, review request rate) derived from existing PR data, displayed in a new view or dashboard section.

### Files to create
- `tauri-app/src-tauri/src/commands/authors.rs` — Command `get_author_stats`
- `tauri-app/src/components/AuthorStatsTable.vue` — Table displaying per-author metrics
- `tauri-app/src/views/Authors.vue` — (Optional) Dedicated view for author insights

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod authors;`
- `tauri-app/src-tauri/src/lib.rs` — Register `commands::authors::get_author_stats`
- `tauri-app/src-tauri/src/models/mod.rs` — Add `AuthorStats` struct
- `tauri-app/src/types/index.ts` — Add `AuthorStats` interface
- `tauri-app/src/stores/pullRequests.ts` — Add `fetchAuthorStats()` action
- `tauri-app/src/views/Dashboard.vue` — Add a compact "Top Authors" section
- `tauri-app/src/router/index.ts` — Optionally add `/authors` route
- `tauri-app/src/components/layout/AppSidebar.vue` — Optionally add "Authors" nav item

### Implementation steps
1. In `commands/authors.rs`, implement `get_author_stats(state)` with SQL grouping by author
2. Calculate `review_rate` as proportion of author's PRs that have a review
3. Register the command
4. Add frontend types, store action
5. Build `AuthorStatsTable.vue` as a sortable table
6. Integrate into Dashboard or a new route

### Dependencies
- None

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 11: Search & Keyboard Shortcuts

**Status**: ✅ Implemented
**Implementation notes**: `useKeyboardShortcuts.ts` composable with global keydown handlers, `SearchBar.vue` in `AppHeader.vue`, search filtering in `PullRequests.vue`, `focusedIndex` prop on `PRTable.vue`.

### Summary
A global search bar for filtering PRs by title/author/number, plus keyboard shortcuts for navigation.

### Files to create
- `tauri-app/src/composables/useKeyboardShortcuts.ts` — Global keyboard event handler composable
- `tauri-app/src/components/SearchBar.vue` — Floating/integrated search input

### Files to modify
- `tauri-app/src/App.vue` — Register `useKeyboardShortcuts()` composable at root level
- `tauri-app/src/components/layout/AppHeader.vue` — Integrate `SearchBar` into the header
- `tauri-app/src/views/PullRequests.vue` — Accept search query, filter `filteredPrs` by it
- `tauri-app/src/components/PRTable.vue` — Accept `focusedIndex` prop for keyboard-highlighted row
- `tauri-app/src/stores/pullRequests.ts` — Add `searchQuery` reactive ref and a `searchResults` computed

### Implementation steps
1. Create `useKeyboardShortcuts.ts` with global `keydown` listener: `/` focuses search, `j`/`k` navigates list, `Enter` opens detail, `g+d`/`g+p`/`g+r`/`g+s` for page navigation, `Escape` clears/blurs
2. Create `SearchBar.vue` with text input emitting `update:query`
3. Integrate into `AppHeader.vue`
4. In `PullRequests.vue`, filter `filteredPrs` by search query (case-insensitive match on title, author, #number, branch)
5. In `PRTable.vue`, add `focusedIndex` prop with highlight styling
6. Register composable in `App.vue`

### Dependencies
- None

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 12: AI Prompt Pack Builder

**Status**: ✅ Implemented
**Implementation notes**: `AiPromptBuilder.vue` component, `usePromptBuilder.ts` and `useResponseParser.ts` composables, integrated into `PullRequestDetail.vue` as collapsible AI review section.

### Summary
Structure PR diff + review rules + PR metadata into AI-ready prompts, with clipboard export and a response parser for structured feedback.

### Files to create
- `tauri-app/src/components/AiPromptBuilder.vue` — UI for building, previewing, and copying AI prompts
- `tauri-app/src/composables/usePromptBuilder.ts` — Composable for assembling prompts from PR data, diff, and rules
- `tauri-app/src/composables/useResponseParser.ts` — Parses structured AI responses

### Files to modify
- `tauri-app/src/views/PullRequestDetail.vue` — Add "AI Review" section/button
- `tauri-app/src/types/index.ts` — Add `PromptTemplate`, `ParsedAiResponse` interfaces

### Implementation steps
1. Create `usePromptBuilder.ts` that takes a `PullRequest`, optional diff, and `ReviewRule[]` and produces a structured markdown prompt
2. Create `useResponseParser.ts` that parses pasted AI responses looking for checklist patterns
3. Build `AiPromptBuilder.vue` with generate, copy, paste-back, and parse functionality
4. Add to `PullRequestDetail.vue` as a collapsible section

### Dependencies
- Feature 3 (PR Diff Viewer) — needs diff data; can degrade gracefully
- Feature 4 (Review Rules UI) — needs rules loaded; can work with empty rules

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 13: CI/CD Status Integration

**Status**: ✅ Implemented
**Implementation notes**: `commands/checks.rs` with `fetch_pr_checks`, `CiStatusBadge.vue` and `CiChecksPanel.vue` components, CI status column in `PRTable.vue`, integrated into `PullRequestDetail.vue`.

### Summary
Fetch CI check status via `gh pr checks --json` and display pass/fail/pending indicators on the PR detail and in the PR table.

### Files to create
- `tauri-app/src-tauri/src/commands/checks.rs` — Command `fetch_pr_checks`
- `tauri-app/src/components/CiStatusBadge.vue` — Compact badge showing overall CI status
- `tauri-app/src/components/CiChecksPanel.vue` — Expanded panel listing all individual checks

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod checks;`
- `tauri-app/src-tauri/src/lib.rs` — Register `commands::checks::fetch_pr_checks`
- `tauri-app/src-tauri/src/github/mod.rs` — Add `fetch_checks()` function
- `tauri-app/src-tauri/src/models/mod.rs` — Add `CiCheck` struct
- `tauri-app/src/views/PullRequestDetail.vue` — Add CI status section
- `tauri-app/src/components/PRTable.vue` — Optionally add CI status column
- `tauri-app/src/types/index.ts` — Add `CiCheck` interface

### Implementation steps
1. In `github/mod.rs`, add `fetch_checks()` that runs `gh pr checks <number> --repo <name> --json name,state,conclusion,detailsUrl`
2. In `commands/checks.rs`, implement `fetch_pr_checks(pr_id, state)`
3. Register the command
4. Add frontend types
5. Build `CiStatusBadge.vue` — green tick/red cross/yellow spinner
6. Build `CiChecksPanel.vue` — list of each check with status icon and link
7. Add to `PullRequestDetail.vue`

### Dependencies
- None

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 14: Stale PR Alerts

**Status**: ✅ Implemented
**Implementation notes**: `commands/stale.rs` with `get_stale_prs`/`close_pull_request`, `StalePrSection.vue` on Dashboard, `stale_threshold_days` setting seeded and configurable in `Settings.vue`.

### Summary
Configurable age threshold for stale PRs, a dashboard section highlighting them, and one-click close via `gh pr close`.

### Files to create
- `tauri-app/src-tauri/src/commands/stale.rs` — Commands `get_stale_prs`, `close_pull_request`
- `tauri-app/src/components/StalePrSection.vue` — Dashboard section listing stale PRs with close buttons

### Files to modify
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod stale;`
- `tauri-app/src-tauri/src/lib.rs` — Register new commands
- `tauri-app/src-tauri/src/github/mod.rs` — Add `close_pr()` function
- `tauri-app/src-tauri/src/db/migrations.rs` — Seed `stale_threshold_days` setting
- `tauri-app/src/views/Dashboard.vue` — Add `StalePrSection`
- `tauri-app/src/views/Settings.vue` — Add threshold config row
- `tauri-app/src/stores/pullRequests.ts` — Add `fetchStalePrs()` and `closePr()` actions

### Implementation steps
1. Seed the `stale_threshold_days` setting in migrations
2. In `github/mod.rs`, add `close_pr()` calling `gh pr close`
3. Implement `get_stale_prs()` and `close_pull_request()` commands
4. Register commands
5. Add store actions
6. Build `StalePrSection.vue`
7. Add to `Dashboard.vue` and `Settings.vue`

### Dependencies
- Feature 5 (Settings Backend Connection) for reading the threshold

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 15: PR Description/Body Viewer

**Status**: ✅ Implemented
**Implementation notes**: `body` field added to `GhPrJson`, `PullRequest` model, and `pull_requests` schema. `MarkdownRenderer.vue` component rendering with `marked`. Description section in `PullRequestDetail.vue`.

### Summary
Add the PR body (description markdown) field to the data model, fetch it during sync, and render it as formatted markdown in the PR detail view.

### Files to create
- `tauri-app/src/components/MarkdownRenderer.vue` — Reusable component for rendering markdown content

### Files to modify
- `tauri-app/src-tauri/src/github/mod.rs` — Add `body` to `GH_PR_FIELDS` and `GhPrJson`
- `tauri-app/src-tauri/src/models/mod.rs` — Add `body: Option<String>` to `PullRequest` and `GhPrJson`
- `tauri-app/src-tauri/src/db/migrations.rs` — Add `body TEXT` column migration
- `tauri-app/src-tauri/src/commands/pull_requests.rs` — Update `PR_SELECT` and `parse_pr_row` to include body
- `tauri-app/src-tauri/src/commands/sync.rs` — Include `body` in upsert
- `tauri-app/src/types/index.ts` — Add `body: string | null` to `PullRequest`
- `tauri-app/src/views/PullRequestDetail.vue` — Add "Description" section with `MarkdownRenderer`
- `tauri-app/package.json` — Add `marked` or `markdown-it`

### Implementation steps
1. Add `body` to `GH_PR_FIELDS` and `GhPrJson` struct
2. Add `body TEXT` column migration
3. Update `PullRequest` Rust model
4. Update `PR_SELECT` and `parse_pr_row` column indices
5. Update sync upsert to include `body`
6. Update frontend `PullRequest` type
7. Install `marked`, create `MarkdownRenderer.vue`
8. Add "Description" section to `PullRequestDetail.vue`

### Dependencies
- None

### Estimated scope
**Medium** (1-3 hours) — Column addition affects multiple files due to index-based row parsing

---

## Feature 16: Review Time Tracking

**Status**: ✅ Implemented
**Implementation notes**: `useReviewTimer.ts` composable with Page Visibility API, `review_duration_seconds` column on `pr_reviews`, `record_review_time` command, elapsed time shown in `PullRequestDetail.vue`, total review time in dashboard stats.

### Summary
Record time spent viewing the PR detail page, store it in `pr_reviews`, and surface total review time on the dashboard.

### Files to create
- `tauri-app/src/composables/useReviewTimer.ts` — Composable that tracks active time on the PR detail view

### Files to modify
- `tauri-app/src-tauri/src/db/migrations.rs` — Add `review_duration_seconds` column to `pr_reviews`
- `tauri-app/src-tauri/src/models/mod.rs` — Add field to `PrReview`
- `tauri-app/src-tauri/src/commands/pull_requests.rs` — Add `record_review_time` command
- `tauri-app/src-tauri/src/lib.rs` — Register command
- `tauri-app/src-tauri/src/commands/stats.rs` — Add total review time to dashboard stats
- `tauri-app/src/views/PullRequestDetail.vue` — Use timer composable, show elapsed time
- `tauri-app/src/types/index.ts` — Update types

### Implementation steps
1. Add migration for `review_duration_seconds` column
2. Update Rust model
3. Add `record_review_time` command
4. Register command
5. Create `useReviewTimer.ts` with start/stop/pause logic and Page Visibility API
6. Use in `PullRequestDetail.vue`, save on unmount
7. Update dashboard stats

### Dependencies
- None

### Estimated scope
**Medium** (1-3 hours)

---

## Feature 17: Repository Grouping & Tagging

**Status**: ✅ Implemented
**Implementation notes**: `commands/groups.rs` with CRUD commands, `repo_groups` and `repo_group_members` tables, `GroupManager.vue` and `GroupFilter.vue` components, `stores/groups.ts` Pinia store, integrated into `Repositories.vue`, `PullRequests.vue`, and `Dashboard.vue`.

### Summary
Allow users to create groups (e.g. "Backend", "Frontend"), assign repositories to groups, and filter by group.

### Files to create
- `tauri-app/src-tauri/src/commands/groups.rs` — CRUD commands for groups and membership
- `tauri-app/src/components/GroupManager.vue` — UI for creating/editing/deleting groups
- `tauri-app/src/components/GroupFilter.vue` — Dropdown/chip filter for groups
- `tauri-app/src/stores/groups.ts` — Pinia store for groups

### Files to modify
- `tauri-app/src-tauri/src/db/migrations.rs` — Add `repo_groups` and `repo_group_members` tables
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod groups;`
- `tauri-app/src-tauri/src/lib.rs` — Register group commands
- `tauri-app/src-tauri/src/models/mod.rs` — Add `RepoGroup`, `RepoGroupMember` structs
- `tauri-app/src/types/index.ts` — Add `RepoGroup` interface
- `tauri-app/src/views/Repositories.vue` — Add group management
- `tauri-app/src/views/PullRequests.vue` — Add group filter
- `tauri-app/src/views/Dashboard.vue` — Optionally filter by group

### Implementation steps
1. Add migrations for new tables
2. Implement 7+ CRUD commands
3. Register all commands
4. Create Pinia store with actions
5. Add types
6. Build `GroupManager.vue` and `GroupFilter.vue`
7. Integrate into views

### Dependencies
- None

### Estimated scope
**Large** (3+ hours)

---

## Feature 18: Quick Review Templates

**Status**: ✅ Implemented
**Implementation notes**: `commands/templates.rs` with `list_templates`/`set_templates`, `review_templates` table with seed data, `TemplateSelector.vue` and `TemplateManager.vue` components, integrated into `PullRequestDetail.vue` and `Settings.vue`.

### Summary
Stored comment templates that can be selected from a dropdown when approving a PR, pre-filling the review body.

### Files to create
- `tauri-app/src-tauri/src/commands/templates.rs` — CRUD commands for templates
- `tauri-app/src/components/TemplateSelector.vue` — Dropdown for selecting a template
- `tauri-app/src/components/TemplateManager.vue` — Settings-page UI for managing templates

### Files to modify
- `tauri-app/src-tauri/src/db/migrations.rs` — Add `review_templates` table with seed data
- `tauri-app/src-tauri/src/commands/mod.rs` — Add `pub mod templates;`
- `tauri-app/src-tauri/src/lib.rs` — Register template commands
- `tauri-app/src-tauri/src/models/mod.rs` — Add `ReviewTemplate` struct
- `tauri-app/src/types/index.ts` — Add `ReviewTemplate` interface
- `tauri-app/src/views/PullRequestDetail.vue` — Add `TemplateSelector` near approve button
- `tauri-app/src/views/Settings.vue` — Add `TemplateManager` section

### Implementation steps
1. Add migration for `review_templates` table with seed data
2. Implement `list_templates` and `set_templates` commands
3. Register commands
4. Add frontend types and store actions
5. Build `TemplateSelector.vue` and `TemplateManager.vue`
6. Integrate into detail and settings views

### Dependencies
- None

### Estimated scope
**Medium** (1-3 hours)

---

## Recommended Implementation Order

### Phase 1 — Foundation
1. Feature 5: Settings Backend Connection
2. Feature 15: PR Description/Body Viewer

### Phase 2 — Core Enhancements
3. Feature 1: Auto-Sync Polling
4. Feature 2: Native Notifications
5. Feature 13: CI/CD Status Integration
6. Feature 14: Stale PR Alerts
7. Feature 7: One-Click Open in Editor

### Phase 3 — Review Workflow
8. Feature 4: Review Rules UI
9. Feature 6: PR Comment Thread Viewer
10. Feature 3: PR Diff Viewer
11. Feature 18: Quick Review Templates
12. Feature 8: Batch Approve/Merge

### Phase 4 — Analytics & Advanced
13. Feature 16: Review Time Tracking
14. Feature 9: PR Age Heatmap & Velocity Dashboard
15. Feature 10: Author Performance Insights
16. Feature 12: AI Prompt Pack Builder

### Phase 5 — Organisation & UX Polish
17. Feature 17: Repository Grouping & Tagging
18. Feature 11: Search & Keyboard Shortcuts

---

## Migration Strategy Note

Several features add columns to `pull_requests` or new tables. The recommended approach is to add a `schema_version` row to `app_settings` and implement versioned migration functions in `migrations.rs` that run sequentially, preventing column-already-exists errors.

## Summary Table

| # | Feature | Impact | Complexity |
|---|---------|--------|------------|
| 1 | Auto-Sync Polling | High | Moderate |
| 2 | Native Notifications | High | Simple |
| 3 | PR Diff Viewer | High | Complex |
| 4 | Review Rules UI | Medium | Simple |
| 5 | Settings Backend Connection | Medium | Simple |
| 6 | PR Comment Thread Viewer | High | Moderate |
| 7 | One-Click Open in Editor | Medium | Simple |
| 8 | Batch Approve/Merge | Medium | Moderate |
| 9 | PR Age Heatmap / Velocity Dashboard | Medium | Moderate |
| 10 | Author Performance Insights | Medium | Moderate |
| 11 | Search and Keyboard Shortcuts | Medium | Simple |
| 12 | Prompt Pack Builder for AI Analysis | High | Complex |
| 13 | CI/CD Status Integration | High | Simple |
| 14 | Stale PR Alerts | Medium | Simple |
| 15 | PR Description/Body Viewer | High | Simple |
| 16 | Review Time Tracking | Low | Simple |
| 17 | Repository Grouping and Tagging | Medium | Moderate |
| 18 | Quick Review Templates | Medium | Simple |
