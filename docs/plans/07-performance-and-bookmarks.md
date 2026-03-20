# Fuse — Performance Optimisation Plan

**Date:** 2026-03-10
**Status:** Planned
**Context:** Full-stack audit of rendering, backend, and CSS performance across three parallel investigations.

---

## Summary

The app suffers from three compounding performance issues:

1. **GPU overload** — persistent `backdrop-filter` and `blur(180px)` on always-visible elements cause constant GPU drain
2. **Backend thread blocking** — synchronous `gh` CLI calls with sequential loops and Mutex contention stall IPC threads
3. **Frontend render blocking** — eager imports, uncoordinated IPC floods, and setup-time side effects prevent the browser from painting loading states

The reader/writer `DbState` split has already been implemented (eliminating the worst Mutex contention). The items below are the remaining high-impact fixes.

---

## Phase 1 — CSS & Rendering (immediate, low risk)

### 1.1 Replace `backdrop-filter` on persistent chrome

**Severity:** Critical
**Files:** `TitleBar.vue`, `AppSidebar.vue`, `AppHeader.vue`, `base.css`

10 simultaneous `backdrop-filter: blur(24px) saturate(1.4)` instances run on every frame. The three persistent layout elements (title bar, sidebar, header) do not need real-time blur — they sit above the ambient blobs in the layout flow.

**Fix:**
- Replace `backdrop-filter` on `TitleBar.vue`, `AppSidebar.vue`, and `AppHeader.vue` with semi-opaque solid backgrounds (e.g. `rgba(18, 17, 16, 0.92)` dark / `rgba(255, 255, 255, 0.92)` light)
- Remove the corresponding `-webkit-backdrop-filter` declarations
- Keep `backdrop-filter` only on short-lived overlays: `CommandPalette`, `ConfirmDialog`, `ToastContainer`, `PRHoverPreview`
- Also remove it from `.card` and `.input-field` in `base.css` — these are rendered in quantity and the blur is imperceptible on small elements

### 1.2 Reduce ambient blob blur radius

**Severity:** High
**File:** `base.css:46`

`filter: blur(180px)` on three continuously-animating 450–600px elements is extremely GPU-intensive. The blobs are large enough that a smaller radius is visually indistinguishable.

**Fix:**
- Reduce `filter: blur(180px)` to `filter: blur(80px)` on `.ambient-blob`
- This halves the GPU sampling kernel area (proportional to radius²)

### 1.3 Replace layout-triggering transitions with `transform`

**Severity:** Medium
**Files:** `AppSidebar.vue:105`, `TabBar.vue:107`, `RouteProgressBar.vue:52`, `tokens.css:271-279`

Several transitions animate `width`, `height`, or `left` — all of which trigger full layout recalculation per frame.

**Fix:**
- `RouteProgressBar`: Animate `transform: scaleX()` with `transform-origin: left` instead of `width`
- `TabBar` indicator: Use `transform: translateX()` for position instead of `left`
- Focus mode header: Use `transform: translateY(-100%)` instead of `height: 0`; replace `transition: all` with explicit `transition: transform, opacity`
- Sidebar collapse: Lower priority — the flex layout interaction makes this non-trivial to refactor

### 1.4 Remove unused `brand-glow-shift` keyframe

**Severity:** Low
**File:** `tokens.css:175-179`

Declared but never referenced. Remove dead CSS.

---

## Phase 2 — Frontend Rendering (high impact, moderate risk)

### 2.1 Lazy-load tab-gated components in PullRequestDetail

**Severity:** High
**File:** `PullRequestDetail.vue:24-45`

20+ components are eagerly imported. Components only visible on non-default tabs should use `defineAsyncComponent`.

**Fix:**
Convert to `defineAsyncComponent`:
- `AiReviewPanel`, `AiPromptBuilder` — 'ai' tab only
- `DiffViewer`, `DiffFileTree`, `CommitTimeline` — 'code' tab only
- `CommentThread` — 'discussion' tab only
- `HandoffComposer`, `WorktreePanel` — sidebar, conditionally shown

Keep eager: `Breadcrumb`, `RiskGauge`, `ReviewStatusComponent`, `AuthorAvatar`, `TabBar`, `CiStatusBadge`, `ConflictBadge`, `DeploymentStatus` — all visible on the default 'overview' tab.

### 2.2 Defer `useChecklist` IPC call to `onMounted`

**Severity:** High
**Files:** `useChecklist.ts:51`, `PullRequestDetail.vue:76`

`useChecklist` calls `load()` unconditionally at setup time, firing an IPC round-trip during `<script setup>` before the DOM has rendered.

**Fix:**
- Remove the `load()` call from the composable body
- Export `load` from the composable
- Call `load()` inside `PullRequestDetail.vue`'s `onMounted`, after the initial paint yield

### 2.3 Stop blocking app shell render in `App.vue`

**Severity:** High
**File:** `App.vue:94-103`

`Promise.all([repoStore.fetchAll(), prStore.fetchAll(), prStore.fetchStats()])` blocks the entire app shell from rendering any route content.

**Fix:**
- Only `await repoStore.fetchAll()` (needed for sidebar)
- Fire `prStore.fetchAll()` and `prStore.fetchStats()` without awaiting — let individual views handle their own loading states with spinners/skeletons
- Each view already has loading guards; this change lets the shell (sidebar, header) render immediately

### 2.4 Coordinate Dashboard IPC calls

**Severity:** High
**Files:** `Dashboard.vue`, `DependencyGraph.vue`, `PriorityQueue.vue`, `WorkloadDashboard.vue`, `StalePrSection.vue`

7+ uncoordinated IPC calls fan out on every dashboard visit. Child components each self-fetch in their own `onMounted`.

**Fix:**
- Hoist all dashboard data fetching into `Dashboard.vue`'s `onMounted`
- Use a single `Promise.all` for all dashboard IPC calls
- Pass data down as props to child components
- `DependencyGraph`: Switch default mount to `fetchDependencies()` (read-only); only call `computeDependencies()` (write) on explicit user action

### 2.5 Build command palette PR list lazily

**Severity:** Medium
**File:** `useCommandPalette.ts:40-71`

`allCommands` computed rebuilds the full PR command list on every `prStore.prs` mutation, even when the palette is closed.

**Fix:**
- Only build the PR command list when `isOpen.value === true`
- Return static navigation commands when closed

### 2.6 Memoise `sortedPrs` risk scores

**Severity:** Medium
**File:** `PRTable.vue:97-121`

`sortedPrs` recomputes `computeRiskScore` for every PR on every `focusedIndex` change (every j/k keypress).

**Fix:**
- Separate risk score computation from the sort computed
- Cache scores in a `Map<prId, score>` that only rebuilds when the PR data changes, not when focus index changes

---

## Phase 3 — Rust Backend (high impact, moderate risk)

### 3.1 Parallelise deployment status fetches

**Severity:** Critical
**File:** `github/mod.rs:285-313`

`fetch_deployments` makes N+1 sequential blocking `Command::output()` calls — one to list deployments, then one per deployment for its status.

**Fix:**
- Collect all deployment IDs from the initial call
- Use `std::thread::scope` to fetch statuses in parallel
- Each thread spawns its own `gh api` process

### 3.2 Parallelise linked issue fetches

**Severity:** High
**File:** `commands/issues.rs:77-92`

`get_linked_issues` makes N sequential `gh issue view` calls, one per referenced issue.

**Fix:**
- Collect all issue numbers first
- Use `std::thread::scope` to fetch in parallel
- Typical N is 2-5, so parallelisation reduces latency from N×RTT to 1×RTT

### 3.3 Eliminate N+1 `SELECT id` in sync transaction

**Severity:** High
**File:** `commands/sync.rs:233-239`

Inside the upsert transaction, an extra `SELECT id FROM pull_requests WHERE repo_id = ? AND number = ?` runs per PR to get the internal ID for reviewer upserts.

**Fix:**
- Replace with `db.last_insert_rowid()` immediately after the `INSERT ... ON CONFLICT DO UPDATE` statement
- SQLite sets `last_insert_rowid()` to the rowid of the upserted row regardless of insert/update path (SQLite ≥ 3.35)
- Eliminates N unnecessary queries from inside the held writer lock

### 3.4 Parallelise batch operations

**Severity:** High
**File:** `commands/batch.rs:47-158`

`batch_approve` and `batch_merge` loop over PR IDs sequentially, making one blocking `gh` call per PR.

**Fix:**
- Use `std::thread::scope` to spawn one thread per PR for the network call
- Collect results, then write DB updates sequentially (fast)

### 3.5 Collapse reviewer workload queries

**Severity:** Medium
**File:** `commands/workload.rs:34-113`

4 separate queries per reviewer (4R+1 total) where a single `GROUP BY` query would suffice.

**Fix:**
Replace the loop with a single aggregate query:
```sql
SELECT
    prr.reviewer,
    COUNT(CASE WHEN p.state = 'OPEN' THEN 1 END) AS assigned,
    COUNT(CASE WHEN rv.status IN ('reviewed','approved','changes_requested') THEN 1 END) AS completed,
    COUNT(CASE WHEN p.state = 'OPEN' AND p.updated_at < datetime('now', '-7 days') THEN 1 END) AS overdue,
    AVG(CASE WHEN rv.reviewed_at IS NOT NULL
        THEN (julianday(rv.reviewed_at) - julianday(p.created_at)) * 24.0 END) AS avg_hours
FROM pr_requested_reviewers prr
JOIN pull_requests p ON p.id = prr.pr_id
LEFT JOIN pr_reviews rv ON rv.pr_id = prr.pr_id
GROUP BY prr.reviewer
```

### 3.6 Convert network-bound commands to `async fn`

**Severity:** Medium
**Files:** All commands that shell out to `gh`

Sync `fn` commands block Tauri's thread pool for the full subprocess duration. Converting to `async fn` with `tokio::process::Command` allows Tauri to schedule them on the async runtime.

**Priority targets:** `fetch_pr_checks`, `fetch_pr_comments`, `fetch_pr_diff`, `check_merge_conflicts`, `get_deployment_status`, `get_linked_issues`

**Lowest priority but highest impact:** `trigger_worktree_review` — can block a thread for 30-120s waiting for Claude CLI. Consider returning a job ID immediately and emitting a completion event.

---

## Phase 4 — Future Improvements (lower priority)

| Item | Severity | Notes |
|------|----------|-------|
| Virtual scrolling for PR list | Medium | Only needed if PR count exceeds ~200 |
| Virtual scrolling for DiffViewer | Medium | Large diffs mount all lines; consider collapsed hunks by default |
| Cache `ResizableSplit` container rect on drag start | Low | Currently calls `getBoundingClientRect()` on every `mousemove` |
| Debounce repo filter in `PullRequests.vue` | Low | Rapid filter changes cause racing IPC calls |
| Collapse `get_review_digest` scalar queries into CTEs | Low | 12 queries → 2; minimal real-world impact at current scale |
| Collapse `get_daily_pr_counts` correlated subqueries | Low | 14 table scans → 1 pass; minimal at 7-day window |
| Remove `transition: all` usage | Low | Replace with explicit property lists in `tokens.css`, `base.css`, `ConfirmDialog.vue` |
| Memoise `prCountForRepo` in `Repositories.vue` | Low | Replace plain function with `computed` returning a `Map` |
| `DependencyGraph` layout memoisation | Medium | BFS re-runs on every sync; add content-hash guard |

---

## Phase 5 — Bookmark Improvements (feature enhancement)

Bookmarks are currently buried in the PR detail right sidebar. They require fully manual entry (file path typed by hand, line numbers guessed) and have no presence outside the PR detail view. This phase makes bookmarks useful, discoverable, and navigable.

### Current State

- **Data model:** `review_bookmarks` table with `pr_id`, `file_path`, `line_start`, `line_end`, `note`, `created_at`
- **UI:** `BookmarksList.vue` — expandable panel in `PullRequestDetail.vue` sidebar only
- **Creation:** Manual form with text input for file path, number inputs for lines, textarea for note
- **No auto-fill** except `pr_id` and timestamps
- **No navigation** — clicking a bookmark does nothing; it just shows the file path and note
- **Not visible** outside the PR detail view

### 5.1 Auto-fill bookmark details from diff context

**Severity:** High
**Files:** `BookmarksList.vue`, `DiffViewer.vue`, `useBookmarks.ts`

When a user is viewing a diff, they should be able to bookmark directly from the diff — not by manually typing a file path.

**Fix:**
- Add a "Bookmark" button/icon to the `DiffViewer` file header (next to each file name)
- Clicking it creates a bookmark pre-filled with:
  - `file_path` — from the diff file entry
  - `line_start` / `line_end` — from any selected line range in the diff (if the user has highlighted lines)
  - `note` — empty, with the cursor focused in the note field for immediate typing
- Also allow right-click → "Bookmark this line" on individual diff lines, auto-filling `file_path` and `line_start`
- Add a `repo_name` or `pr_number` display field to the bookmark for context when viewed globally

### 5.2 Navigate to diff location when bookmark is clicked

**Severity:** High
**Files:** `BookmarksList.vue`, `PullRequestDetail.vue`, `DiffViewer.vue`

Clicking a bookmark should take the user to the exact location in the diff.

**Fix:**
- When a bookmark is clicked in `BookmarksList`:
  1. Switch to the 'code' tab (if not already active)
  2. Expand the bookmarked file in `DiffViewer` (if collapsed)
  3. Scroll to the bookmarked line range and highlight it briefly (e.g. 2s yellow flash)
- Use `provide/inject` or an event bus to communicate between `BookmarksList` (sidebar) and `DiffViewer` (main panel)
- Add `data-line` attributes to diff lines for scroll targeting

### 5.3 Global bookmarks panel in the sidebar

**Severity:** High
**Files:** `AppSidebar.vue`, new `BookmarksView.vue` or sidebar section, `useBookmarks.ts`, `bookmarks.rs`

Bookmarks should be visible and accessible from anywhere in the app, not just buried in a single PR's detail sidebar.

**Fix:**
- Add a "Bookmarks" icon/link in `AppSidebar.vue` (below Pull Requests, above Settings)
- Create a global bookmarks view or sidebar drawer that shows all bookmarks across all PRs, grouped by PR
- Each bookmark entry shows:
  - PR title + number (e.g. `#1302 Stripe wallet data`)
  - Repository name
  - File path + line range
  - Note text (truncated)
  - Relative timestamp ("2h ago")
- Clicking a bookmark navigates to that PR's detail view and scrolls to the bookmarked diff location (per 5.2)
- Add a badge count to the sidebar icon showing total active bookmarks

**Backend changes:**
- Add a new command `list_all_bookmarks` that fetches bookmarks across all PRs with PR metadata joined:
  ```sql
  SELECT b.*, p.number, p.title, r.owner || '/' || r.name AS repo_name
  FROM review_bookmarks b
  JOIN pull_requests p ON p.id = b.pr_id
  JOIN repositories r ON r.id = p.repo_id
  WHERE p.state = 'OPEN'
  ORDER BY b.created_at DESC
  ```
- Add a `get_bookmark_count` command for the sidebar badge

### 5.4 Bookmark from the PR list (quick-add)

**Severity:** Medium
**Files:** `PRTable.vue`, `useBookmarks.ts`

Allow users to create a "general" bookmark on a PR from the PR list without opening it — essentially a "flag this PR for later" action.

**Fix:**
- Add a bookmark icon button to each PR row in `PRTable.vue`
- Clicking it creates a bookmark with `file_path: '(general)'`, no line range, and a note prompt (or empty)
- If the PR already has bookmarks, show the icon as filled/active
- This gives users a quick triage workflow: scan the PR list, bookmark interesting ones, review later via the global bookmarks panel

### 5.5 Add bookmark metadata for richer context

**Severity:** Medium
**Files:** `migrations.rs`, `bookmarks.rs`, `types/index.ts`, `BookmarksList.vue`

The current bookmark has no category or priority. Adding lightweight metadata makes bookmarks more useful for review workflows.

**Fix:**
- Add a `category` column to `review_bookmarks`: `TEXT NOT NULL DEFAULT 'note'`
  - Values: `'note'` (default), `'bug'`, `'question'`, `'suggestion'`, `'blocker'`
- Add a `resolved` column: `INTEGER NOT NULL DEFAULT 0`
  - Allows marking bookmarks as addressed without deleting them
- Display category as a coloured icon/tag in the bookmark card
- Add a "Mark as resolved" toggle (strikethrough styling when resolved)
- Filter options in the global bookmarks view: by category, by resolved status

### 5.6 Include bookmarks in the command palette

**Severity:** Medium
**Files:** `useCommandPalette.ts`, `useBookmarks.ts`

Users should be able to search for and jump to bookmarks from the command palette (Cmd+K).

**Fix:**
- Add a "Bookmarks" section to the command palette results
- Each bookmark appears as: `📌 file_path:L123 — note text (PR #1302)`
- Selecting a bookmark navigates to the PR detail view and scrolls to the bookmarked location
- Only build the bookmark command list when the palette is open (consistent with Phase 2.5)

### 5.7 Add bookmarks to the system tray menu

**Severity:** Low
**Files:** `menu.rs`, `bookmarks.rs`

Show a "Bookmarks" submenu in the tray icon menu for quick access to recent bookmarks.

**Fix:**
- Add a "Recent Bookmarks" submenu after the PR list in the tray menu
- Show the 5 most recent bookmarks with truncated note + PR number
- Clicking navigates to the PR detail view (reusing the existing `menu-navigate-pr` event pattern)

---

## Implementation Order

Recommended sequence for maximum impact with minimum risk:

1. **Phase 1.1 + 1.2** — CSS changes (no logic changes, immediate GPU relief)
2. **Phase 2.2 + 2.3** — Defer checklist + unblock app shell (small, safe changes)
3. **Phase 2.1** — Lazy-load PR detail components (mechanical refactor)
4. **Phase 3.3** — Eliminate sync N+1 (single line change, big lock reduction)
5. **Phase 3.1 + 3.2** — Parallelise deployment + issue fetches (moderate Rust changes)
6. **Phase 5.1 + 5.2** — Bookmark auto-fill from diff + click-to-navigate (core bookmark UX)
7. **Phase 5.3** — Global bookmarks sidebar panel (biggest visibility improvement)
8. **Phase 2.4** — Coordinate dashboard fetches (moderate Vue refactor)
9. **Phase 3.5** — Collapse workload query (single SQL rewrite)
10. **Phase 5.4 + 5.5** — Quick-add from PR list + bookmark metadata (workflow polish)
11. **Phase 2.5 + 2.6** — Command palette + PRTable memoisation (polish)
12. **Phase 5.6** — Bookmarks in command palette (discoverability)
13. **Phase 3.4 + 3.6** — Batch parallelisation + async commands (larger Rust changes)
14. **Phase 5.7** — Bookmarks in system tray (nice-to-have)
