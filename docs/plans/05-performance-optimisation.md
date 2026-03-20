# Performance Optimisation Plan

> Generated from a full-stack analysis of the Fuse codebase covering Rust backend, Vue frontend, CSS/rendering, and build configuration.

## Status Key

- [x] Not started
- [~] In progress
- [x] Complete

---

## Tier 1 — Biggest Perceived Speed Gains

### 1. Parallelise Data Fetches

**Impact:** Cuts initial load time by 50–70%

Multiple locations use sequential `await` chains for independent data fetches. Converting these to `Promise.all()` eliminates waterfall delays.

#### Files to change:

- [x] **`src/App.vue:76-81`** — Three sequential awaits on mount:
  ```js
  // Before (sequential — ~1.5s total)
  await repoStore.fetchAll()
  await prStore.fetchAll()
  await prStore.fetchStats()

  // After (parallel — ~0.5s total)
  await Promise.all([
    repoStore.fetchAll(),
    prStore.fetchAll(),
    prStore.fetchStats(),
  ])
  ```

- [x] **`src/views/Dashboard.vue:29-37`** — Five sequential fetches; split into critical (stats + PRs) and non-critical (analytics):
  ```js
  // Critical — gate the skeleton
  await Promise.all([prStore.fetchStats(), prStore.fetchAll()])
  initialLoad.value = false

  // Non-critical — parallelise analytics
  const [buckets, velocity, counts] = await Promise.all([
    prStore.fetchAgeDistribution(),
    prStore.fetchReviewVelocity(),
    prStore.fetchDailyPrCounts(),
  ])
  ageBuckets.value = buckets
  velocityData.value = velocity
  dailyCounts.value = counts
  ```

- [x] **`src/stores/pullRequests.ts:49-50`** — `syncAll()` calls `fetchAll()` then `fetchStats()` sequentially:
  ```js
  await Promise.all([fetchAll(), fetchStats()])
  ```

- [x] **`src/composables/useAutoSync.ts:30-31`** — Background sync event handler sequential:
  ```js
  await Promise.all([prStore.fetchAll(), prStore.fetchStats()])
  ```

- [x] **`src/views/PullRequestDetail.vue:163-168`** — `fetchRules` and `fetchDiff` awaited sequentially:
  ```js
  await Promise.all([
    prStore.fetchRules(pr.value.repo_id),
    fetchDiff(pr.value.id),
  ])
  ```

---

### 2. Reduce `backdrop-filter` Usage

**Impact:** Major frame rate improvement — eliminates 15+ simultaneous GPU compositing layers

74 occurrences of `backdrop-filter: blur(24px) saturate(1.4)` across 44 files. Each creates a GPU compositing layer that blurs all pixels beneath it. On a typical dashboard view, 15+ blur layers are active simultaneously.

#### Strategy:

- [x] **Keep `backdrop-filter`** on chrome elements that genuinely overlay content:
  - `TitleBar.vue` — always visible, overlays app content
  - `AppSidebar.vue` — overlays ambient blobs
  - `AppHeader.vue` — overlays scrolling content
  - `CommandPalette.vue` — modal overlay
  - `ShortcutOverlay.vue` — modal overlay
  - `ConfirmDialog.vue` — modal overlay
  - `ToastContainer.vue` — floating overlay

- [x] **Remove `backdrop-filter`** from all content cards and panels — replace with solid `background` using existing `--color-surface-panel` token (already semi-transparent). These sit above an already-blurred background so the blur is redundant:
  - All `StatsCard`, `PRCard`, `PRTable`, `PriorityQueue`, `DependencyGraph`
  - `WorkloadDashboard`, `AgeHeatmap`, `VelocityChart`
  - `ReviewProgress`, `StalePrSection`, `EmptyState`
  - `CiChecksPanel`, `WorktreePanel`, `BookmarksList`
  - `BatchActionBar`, `ReviewStatus`, `ReviewRulesEditor`
  - `TemplateManager`, `TemplateSelector`, `LabelRulesManager`
  - `CommentThread`, `AiPromptBuilder`, `GroupManager`
  - `AuthorStatsTable`, `RepositoryCard`
  - `DiffViewer` — particularly important as it renders thousands of child nodes
  - All view-level panels in `Dashboard`, `PullRequests`, `PullRequestDetail`, `Settings`, `Repositories`, `DigestView`, `ReviewSession`

- [x] **Remove `backdrop-filter` from skeleton components** — `SkeletonStatsCard.vue:11`, `SkeletonPRDetail.vue:149` — skeletons are opaque; there is nothing to blur beneath them. This is wasted GPU work during the most latency-sensitive moment.

- [x] **Add `will-change: transform, opacity`** to `.ambient-blob` in `base.css` to keep animated blobs pre-promoted.

---

### 3. Add Loading Indicators for Dashboard Sections

**Impact:** Eliminates blank screen perception during data loading

After `initialLoad` clears, several dashboard components render blank while independently fetching their own data.

- [x] **`PriorityQueue.vue`** — Add skeleton placeholder during `loading` state (currently shows nothing)
- [x] **`WorkloadDashboard.vue`** — Add skeleton placeholder during `loading` state (currently blank)
- [x] **`DependencyGraph.vue`** — Replace plain text "Analysing dependencies..." with animated skeleton
- [x] **`StalePrSection.vue`** — Add skeleton during its independent `onMounted` fetch
- [x] **`Dashboard.vue`** — Add per-section loading states for analytics cards (`AgeHeatmap`, `VelocityChart`) while their data fetches complete

---

### 4. Emit `sync-started` Event from Polling

**Impact:** Users can see when the app is actively syncing

- [x] **`src-tauri/src/polling.rs:49`** — Emit `sync-started` before the sync loop:
  ```rust
  let _ = app.emit("sync-started", ());
  // ... existing sync code ...
  let _ = app.emit("sync-completed", &results);
  ```

- [x] **`src/composables/useAutoSync.ts`** — Listen for `sync-started` and set a reactive `syncing` flag:
  ```ts
  const syncing = ref(false)
  await listen('sync-started', () => { syncing.value = true })
  await listen<SyncResult[]>('sync-completed', async (event) => {
    syncing.value = false
    // ... existing handler ...
  })
  ```

- [x] **Only emit `sync-completed` when changes occurred** — `polling.rs:55` currently emits on every poll tick regardless:
  ```rust
  let has_changes = results.iter().any(|r| !r.changes.is_empty());
  if has_changes {
      let _ = app.emit("sync-completed", &results);
  }
  ```

---

## Tier 2 — Important Performance Fixes

### 5. Fix Duplicate Sync Refresh

**Impact:** Eliminates redundant DB reads after every sync

- [x] **`src/stores/pullRequests.ts:49-50`** — `syncAll()` calls `fetchAll()` + `fetchStats()` after invoke
- [x] **`src/composables/useAutoSync.ts:30-31`** — Also calls `fetchAll()` + `fetchStats()` on the `sync-completed` event

Background syncs trigger both paths, causing duplicate reads. Fix: let the event handler be the single refresh path. Remove the store-level refresh from `syncAll()` and have it return the sync results directly, with the event-based refresh handling store updates.

---

### 6. Add Missing Database Indexes

**Impact:** Faster analytics and stats queries

- [x] **`src-tauri/src/db/migrations.rs`** — Add the following indexes:
  ```sql
  CREATE INDEX IF NOT EXISTS idx_pr_author         ON pull_requests(author);
  CREATE INDEX IF NOT EXISTS idx_pr_state_author   ON pull_requests(state, author);
  CREATE INDEX IF NOT EXISTS idx_pr_state_updated  ON pull_requests(state, updated_at);
  CREATE INDEX IF NOT EXISTS idx_pr_merged_at      ON pull_requests(merged_at) WHERE merged_at IS NOT NULL;
  CREATE INDEX IF NOT EXISTS idx_pr_created_at     ON pull_requests(created_at);
  CREATE INDEX IF NOT EXISTS idx_reviews_updated   ON pr_reviews(updated_at);
  CREATE INDEX IF NOT EXISTS idx_reviews_reviewed  ON pr_reviews(reviewed_at) WHERE reviewed_at IS NOT NULL;
  CREATE INDEX IF NOT EXISTS idx_synclog_synced_at ON sync_log(synced_at DESC);
  ```

---

### 7. Optimise PR Table Rendering

**Impact:** Reduces unnecessary re-renders in the main PR list view

- [x] **`src/components/PRTable.vue:244`** — Remove inline `computeRiskScore(pr)` call from template. Pre-compute scores inside `sortedPrs` and attach to each row object.

- [x] **`src/components/PRTable.vue:221-296`** — Add `v-memo` to table rows:
  ```html
  <tr
    v-for="(pr, idx) in sortedPrs"
    :key="pr.id"
    v-memo="[pr, idx === focusedIndex, selectedIds?.has(pr.id)]"
  >
  ```

- [x] **`src/components/PRTable.vue:82`** — Make scroll listener passive:
  ```ts
  tableWrapper.value?.addEventListener('scroll', onScroll, { passive: true })
  ```

- [x] **`src/components/PRTable.vue:224`** — Move `scrollIntoView` from inline `:ref` callback to a `watch(focusedIndex)` handler to avoid running on every render.

---

### 8. Remove `UPPER()` from State Filter Query

**Impact:** Allows SQLite to use the `idx_pr_repo_state` index

- [x] **`src-tauri/src/commands/pull_requests.rs:77`** — State is already normalised to uppercase during sync (`sync.rs:172`), so `UPPER()` is unnecessary:
  ```rust
  // Before
  sql.push_str(" AND UPPER(p.state) = UPPER(?)");
  // After
  sql.push_str(" AND p.state = ?");
  ```
  The caller should also uppercase the filter value in Rust before binding.

---

### 9. Add SQLite Performance Pragmas

**Impact:** Faster writes and larger query cache

- [x] **`src-tauri/src/db/mod.rs`** — After enabling WAL mode, add:
  ```rust
  conn.execute_batch("PRAGMA synchronous = NORMAL")?;
  conn.execute_batch("PRAGMA cache_size = -32000")?;  // 32 MB
  ```

---

### 10. Debounce Filter Persistence

**Impact:** Eliminates main-thread blocking on every search keystroke

- [x] **`src/stores/filters.ts:28`** — Debounce the `sessionStorage` write:
  ```ts
  let persistTimer: ReturnType<typeof setTimeout> | null = null
  function persistDebounced() {
    if (persistTimer) clearTimeout(persistTimer)
    persistTimer = setTimeout(persist, 80)
  }
  watch([filterRepoId, filterState, searchQuery, sortBy, sortAsc], persistDebounced)
  ```

---

## Tier 3 — Build & Configuration

### 11. Add Cargo Release Profile

**Impact:** Smaller production binary (20-40% reduction)

- [x] **`src-tauri/Cargo.toml`** — Add:
  ```toml
  [profile.release]
  opt-level = "z"
  lto = true
  codegen-units = 1
  strip = true
  panic = "abort"
  ```

---

### 12. Disable Duplicate Tauri API Injection

**Impact:** Removes redundant global `window.__TAURI__` bundle

- [x] **`src-tauri/tauri.conf.json`** — Set `"withGlobalTauri": false`. The codebase uses ESM imports (`import { invoke } from '@tauri-apps/api/core'`) exclusively — zero references to `window.__TAURI__`.

---

### 13. Add Vite Manual Chunks

**Impact:** Enables vendor chunk caching; app code changes don't invalidate vendor bundle

- [x] **`vite.config.ts`** — Add build configuration:
  ```ts
  build: {
    target: 'safari16',
    chunkSizeWarningLimit: 800,
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-vue': ['vue', 'vue-router', 'pinia'],
          'vendor-tauri': ['@tauri-apps/api', '@tauri-apps/plugin-opener', '@tauri-apps/plugin-notification'],
          'vendor-markdown': ['marked', 'dompurify'],
        },
      },
    },
  },
  ```

---

### 14. Lazy-Load Overlay Components in App.vue

**Impact:** Faster initial render — these components are only shown on user action

- [x] **`src/App.vue`** — Wrap in `defineAsyncComponent`:
  ```ts
  const CommandPalette = defineAsyncComponent(() => import('./components/CommandPalette.vue'))
  const ShortcutOverlay = defineAsyncComponent(() => import('./components/ShortcutOverlay.vue'))
  const OnboardingWizard = defineAsyncComponent(() => import('./components/OnboardingWizard.vue'))
  const NotificationDrawer = defineAsyncComponent(() => import('./components/NotificationDrawer.vue'))
  ```

---

### 15. Add Lazy Loading to Author Avatar Images

**Impact:** Fewer upfront network requests in PR list

- [x] **`src/components/AuthorAvatar.vue`** — Add `loading="lazy"` to the `<img>` element.

---

## Tier 4 — Correctness & Safety

### 16. Fix MarkdownRenderer XSS Vulnerability

**Impact:** Security fix — prevents script injection from untrusted PR descriptions

- [x] **`src/components/MarkdownRenderer.vue`** — Currently uses `marked.parse()` output directly in `v-html` without sanitisation. Replace with `useMarkdown`'s `renderMarkdown` function which applies DOMPurify.

---

### 17. Use HashMap for Sync Snapshot Lookup

**Impact:** O(n²) → O(n) during PR sync

- [x] **`src-tauri/src/commands/sync.rs:120`** — Replace `existing_prs.iter().find()` with a `HashMap<i64, PrSnapshot>` keyed by PR number:
  ```rust
  use std::collections::HashMap;
  let existing_map: HashMap<i64, PrSnapshot> = existing_prs
      .into_iter()
      .map(|s| (s.number, s))
      .collect();
  // Then: existing_map.get(&pr.number)
  ```

---

### 18. Compile Regexes Once with OnceLock

**Impact:** Avoids repeated regex compilation on every PR during dependency/issue parsing

- [x] **`src-tauri/src/commands/dependencies.rs:37-38`** and **`src-tauri/src/commands/issues.rs:16-17`** — Use `std::sync::OnceLock`:
  ```rust
  use std::sync::OnceLock;
  static DEP_RE: OnceLock<Regex> = OnceLock::new();
  fn get_dep_re() -> &'static Regex {
      DEP_RE.get_or_init(|| Regex::new(r"...").unwrap())
  }
  ```

---

## Additional Findings (Future Consideration)

These items were identified but are lower priority or require more architectural discussion:

- **N+1 queries in `get_reviewer_workload`** — 4 queries per reviewer; should be a single aggregating SQL query
- **`get_daily_pr_counts` correlated subqueries** — 14 full table scans; replace with pre-aggregated CTE
- **Single global DB mutex** — long-term, switch to a connection pool (`r2d2-sqlite`) for concurrent read access in WAL mode
- **`trigger_worktree_review` blocks thread indefinitely** — no timeout on `claude` subprocess; should use `spawn_blocking` with timeout and emit progress events
- **Polling loop sleeps before first sync** — first sync delayed by full interval (default 5 min); consider immediate sync on startup
- **No error backoff in polling** — consecutive failures retry at normal interval; add exponential backoff
- **Move `@types/dompurify` and `@vitejs/plugin-vue` to `devDependencies`** in `package.json`
- **PRTable virtual scrolling** — for 100+ PRs, consider `vue-virtual-scroller` or pagination
- **DiffViewer virtualisation** — large diffs render thousands of DOM nodes; add collapsed-by-default for files beyond a threshold
- **`useCountUp` missing `prefers-reduced-motion` check** — should skip animation when user prefers reduced motion
- **`useCache` TTL not invalidated on sync events** — stale CI check data shown for up to 60 seconds after sync
- **Duplicate diff CSS** in `DiffViewer.vue` and `ReviewSession.vue` — extract to shared stylesheet
