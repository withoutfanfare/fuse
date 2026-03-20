# UX Improvements Implementation Plan

## Codebase Conventions

- **Component pattern**: Vue 3 `<script setup lang="ts">` with scoped styles
- **Stores**: Pinia composition API style (`defineStore` with `setup` function returning refs/computeds/functions)
- **Styling**: CSS custom properties from `tokens.css`, glassmorphic panels with `backdrop-filter: blur(24px) saturate(1.4)`, `var(--color-surface-panel)` backgrounds, `var(--shadow-card)` shadows, `var(--radius-lg)` borders
- **Backend calls**: `invoke()` from `@tauri-apps/api/core`
- **Routing**: Named routes with lazy-loaded views
- **No external UI library** — everything is hand-rolled with CSS custom properties
- **British English** in user-facing text

---

## Improvement 1: Command Palette (Cmd+K)

**Status**: ✅ Implemented
**Implementation notes**: `CommandPalette.vue` overlay with fuzzy search, `useCommandPalette.ts` composable, mounted in `App.vue`, z-index token added to `tokens.css`.

### Summary
A global overlay that opens with Cmd+K (or Ctrl+K on non-macOS), providing fuzzy search across PR titles, numbers, authors, and repos, plus quick navigation to pages and triggering actions like sync.

### Files to create
- `tauri-app/src/components/CommandPalette.vue` — The overlay component with search input, result list, keyboard navigation
- `tauri-app/src/composables/useCommandPalette.ts` — Composable managing open/close state, command registry, fuzzy matching logic

### Files to modify
- `tauri-app/src/App.vue` — Mount `<CommandPalette />` at root level, import the composable
- `tauri-app/src/styles/tokens.css` — Add `--z-command-palette: 100` z-index token

### Implementation steps
1. Create `useCommandPalette.ts` composable with `isOpen` ref, `toggle()`, `close()`, `open()` functions, a `commands` computed that aggregates: all PRs (from `usePullRequestsStore`), page navigation items, and actions (sync, refresh)
2. Implement simple fuzzy matching function (lowercase substring match on title, author, `#number`, repo name) that scores and sorts results
3. Create `CommandPalette.vue` with: full-screen overlay, centred glassmorphic panel (`max-width: 560px`), text input at top, scrollable results list below, keyboard navigation with arrow keys/Enter/Escape
4. Register global `keydown` listener in `App.vue` for Cmd+K
5. Each result item shows an icon, title, and optional subtitle
6. On selection: navigate to route (for pages/PRs) or invoke action (for sync)

### Dependencies
None (standalone), but benefits from Improvement 5 (toast) for action feedback.

### Estimated scope
**Medium** (2-3 hours)

---

## Improvement 2: Global Keyboard Shortcuts

**Status**: ✅ Implemented
**Implementation notes**: `useKeyboardShortcuts.ts` composable with page navigation, list navigation, and action shortcuts. `focusedIndex` prop on `PRTable.vue`. Invoked in `App.vue`.

### Summary
Page-level shortcuts (`1`-`4` for navigation), list shortcuts (`j`/`k` for up/down, `Enter` to open), `Escape` to go back, `r` to refresh/sync.

### Files to create
- `tauri-app/src/composables/useKeyboardShortcuts.ts` — Composable that registers/unregisters document-level keydown handlers, with conflict avoidance

### Files to modify
- `tauri-app/src/App.vue` — Invoke `useKeyboardShortcuts()` in setup
- `tauri-app/src/components/PRTable.vue` — Add `focusedIndex` ref, apply `.focused` CSS class to active row
- `tauri-app/src/views/PullRequests.vue` — Connect keyboard navigation to PRTable via template ref

### Implementation steps
1. Create `useKeyboardShortcuts.ts`. Accept `router`, `prStore` as arguments. On `onMounted`, add `keydown` listener
2. Guard against firing when an input element is focused
3. Map `1`-`4` to routes, `r` to sync, `Escape` to back
4. For `j`/`k`/`Enter`, expose a reactive `focusedIndex`
5. In `PRTable.vue`, accept optional `focusedIndex` prop, apply `.focused` CSS class

### Dependencies
Must coordinate with Improvement 1 (command palette) to avoid conflicts when palette is open.

### Estimated scope
**Medium** (2 hours)

---

## Improvement 3: Inline Quick Search

**Status**: ✅ Implemented
**Implementation notes**: `searchQuery` ref and search input in `PullRequests.vue` filters bar, `filteredPrs` computed extended with case-insensitive matching on title, author, branch, and number.

### Summary
A text input placed above the PR table that filters the displayed list in real-time by title, author, branch, and PR number.

### Files to create
None

### Files to modify
- `tauri-app/src/views/PullRequests.vue` — Add a `searchQuery` ref, an `<input>` in the filters bar, and extend the `filteredPrs` computed

### Implementation steps
1. Add `const searchQuery = ref('')`
2. Add an `<input>` element inside `.filters-bar` with placeholder "Search PRs..."
3. Extend `filteredPrs` computed: filter by `pr.title`, `pr.author`, `pr.head_branch`, and `String(pr.number)` against `searchQuery.value.toLowerCase()`
4. Add a clear button to reset the search

### Dependencies
None.

### Estimated scope
**Small** (30 minutes)

---

## Improvement 4: First-Run Onboarding

**Status**: ✅ Implemented
**Implementation notes**: `OnboardingWizard.vue` multi-step wizard, `useOnboarding.ts` composable with localStorage persistence, conditionally rendered in `App.vue`.

### Summary
A guided wizard that appears when no repositories are tracked, walking the user through checking GitHub CLI auth, adding their first repository, and triggering an initial sync.

### Files to create
- `tauri-app/src/components/OnboardingWizard.vue` — Multi-step wizard component
- `tauri-app/src/composables/useOnboarding.ts` — Composable tracking onboarding state

### Files to modify
- `tauri-app/src/App.vue` — Conditionally render `<OnboardingWizard />`
- `tauri-app/src/views/Dashboard.vue` — Optionally show "Getting started" prompt

### Implementation steps
1. Create `useOnboarding.ts` with: `showOnboarding` computed, `currentStep` ref, `dismissOnboarding()` function
2. Create `OnboardingWizard.vue` with three steps: Welcome, Add Repository, Syncing
3. Style as centred glassmorphic overlay
4. Each step has Back/Next/Finish buttons
5. On dismissal or completion, set localStorage flag

### Dependencies
None.

### Estimated scope
**Medium** (2-3 hours)

---

## Improvement 5: Toast Notification System

**Status**: ✅ Implemented
**Implementation notes**: `stores/toast.ts` Pinia store with auto-dismiss, `ToastContainer.vue` with glassmorphic cards and `TransitionGroup`, mounted in `App.vue`, retrofitted across action sites.

### Summary
A global toast/snackbar system for transient messages. Toasts appear in the bottom-right corner and auto-dismiss.

### Files to create
- `tauri-app/src/stores/toast.ts` — Pinia store managing a queue of toast messages
- `tauri-app/src/components/ToastContainer.vue` — Renders the toast stack with enter/leave transitions

### Files to modify
- `tauri-app/src/App.vue` — Mount `<ToastContainer />`
- `tauri-app/src/types/index.ts` — Add `Toast` interface
- `tauri-app/src/views/PullRequestDetail.vue` — Replace inline `actionMessage` with toast calls
- `tauri-app/src/components/WorktreePanel.vue` — Add toast for clipboard confirmation
- `tauri-app/src/stores/pullRequests.ts` — Optionally emit toasts on sync completion

### Implementation steps
1. Define `Toast` type in `types/index.ts`
2. Create `toast.ts` store with `addToast()` and auto-remove via `setTimeout`
3. Create `ToastContainer.vue` with fixed positioning, glassmorphic cards, coloured left borders, `<TransitionGroup>` animations
4. Mount in `App.vue`
5. Retrofit existing feedback locations

### Dependencies
None (foundational — many other improvements will use this).

### Estimated scope
**Medium** (1.5-2 hours)

---

## Improvement 6: Skeleton Loading States

**Status**: ✅ Implemented
**Implementation notes**: `skeleton.css` with shimmer keyframe, `SkeletonStatsCard.vue`, `SkeletonPRTableRow.vue`, and `SkeletonPRDetail.vue` components, `loading` prop on `PRTable.vue`, integrated into Dashboard, PullRequests, and PullRequestDetail views.

### Summary
Shimmer placeholder components matching content shapes, shown while data loads.

### Files to create
- `tauri-app/src/components/skeletons/SkeletonStatsCard.vue`
- `tauri-app/src/components/skeletons/SkeletonPRTableRow.vue`
- `tauri-app/src/components/skeletons/SkeletonPRDetail.vue`
- `tauri-app/src/styles/skeleton.css` — Shared shimmer keyframe animation

### Files to modify
- `tauri-app/src/views/Dashboard.vue` — Show skeleton cards while loading
- `tauri-app/src/components/PRTable.vue` — Show skeleton rows (accept `loading` prop)
- `tauri-app/src/views/PullRequestDetail.vue` — Show skeleton while loading
- `tauri-app/src/views/PullRequests.vue` — Pass `loading` prop to PRTable
- `tauri-app/src/styles/base.css` — Import `skeleton.css`

### Implementation steps
1. Create `skeleton.css` with `@keyframes shimmer` and `.skeleton` base class
2. Create each skeleton component matching real component dimensions
3. Add `loading` prop to `PRTable.vue`
4. Update views to conditionally render skeletons

### Dependencies
None.

### Estimated scope
**Medium** (2 hours)

---

## Improvement 7: Auto-Sync Background Polling

**Status**: ✅ Implemented
**Implementation notes**: `useAutoSync.ts` composable with `setInterval`, pulsing indicator in `AppHeader.vue`, interval configurable in `Settings.vue`, initialised in `App.vue`.

### Summary
A configurable timer that periodically triggers PR sync with a subtle header indicator.

### Files to create
- `tauri-app/src/composables/useAutoSync.ts` — Composable managing a `setInterval`

### Files to modify
- `tauri-app/src/App.vue` — Invoke `useAutoSync()` on mount
- `tauri-app/src/components/layout/AppHeader.vue` — Add pulsing indicator when auto-sync active
- `tauri-app/src/views/Settings.vue` — Connect poll interval setting

### Implementation steps
1. Create `useAutoSync.ts`: accept `intervalSeconds` ref, start/stop `setInterval`, watch for interval changes
2. Read interval from `localStorage` (default 300 seconds)
3. Initialise in `App.vue`
4. Add visual indicator in `AppHeader.vue`
5. Connect Settings.vue save to update interval

### Dependencies
None.

### Estimated scope
**Small** (1 hour)

---

## Improvement 8: Risk Score Tooltips

**Status**: ✅ Implemented
**Implementation notes**: `computeRiskBreakdown()` in `useRiskScore.ts`, `BaseTooltip.vue` reusable component, `RiskBadge.vue` updated with breakdown tooltip, wired in `PRTable`, `PRCard`, and `PullRequestDetail`.

### Summary
On hover over `RiskBadge`, show a tooltip breaking down individual scoring factors.

### Files to create
- `tauri-app/src/components/BaseTooltip.vue` — Reusable tooltip wrapper component

### Files to modify
- `tauri-app/src/composables/useRiskScore.ts` — Add `computeRiskBreakdown()` function
- `tauri-app/src/components/RiskBadge.vue` — Accept optional `pr` prop, wrap in `<BaseTooltip>`

### Implementation steps
1. Add `computeRiskBreakdown()` to `useRiskScore.ts` returning labelled factor list
2. Create `BaseTooltip.vue` with hover-triggered tooltip positioning
3. Update `RiskBadge.vue` to show breakdown on hover
4. Update call sites (`PRTable`, `PRCard`, `PullRequestDetail`) to pass `pr` object

### Dependencies
None.

### Estimated scope
**Medium** (1.5-2 hours)

---

## Improvement 9: Confirmation Dialogs

**Status**: ✅ Implemented
**Implementation notes**: `ConfirmDialog.vue` reusable modal with variant-styled confirm button, integrated into `PullRequestDetail.vue` (merge), `WorktreePanel.vue` (remove), and `Repositories.vue` (delete).

### Summary
Modal dialogs before destructive actions (merge, remove worktree, delete repository).

### Files to create
- `tauri-app/src/components/ConfirmDialog.vue` — Reusable modal with title, description, cancel/confirm buttons

### Files to modify
- `tauri-app/src/views/PullRequestDetail.vue` — Wrap `handleMerge()` with confirm dialog
- `tauri-app/src/components/WorktreePanel.vue` — Wrap `handleRemove()` with confirm dialog
- `tauri-app/src/views/Repositories.vue` — Wrap `removeRepo()` with confirm dialog

### Implementation steps
1. Create `ConfirmDialog.vue` with props: `visible`, `title`, `message`, `confirmLabel`, `confirmVariant`. Emits: `confirm`, `cancel`. Glassmorphic overlay with focus trapping
2. Style confirm button per variant (danger = red, warning = yellow, primary = accent)
3. Add confirm state refs and dialogs to each destructive action site

### Dependencies
None.

### Estimated scope
**Medium** (1.5-2 hours)

---

## Improvement 10: PR Body Display

**Status**: ✅ Implemented
**Implementation notes**: `MarkdownRenderer.vue` component using `marked`, `body` field added to `PullRequest` type, collapsible "Description" section in `PullRequestDetail.vue`.

### Summary
A collapsible section in PR detail that renders the PR description as formatted markdown.

### Files to create
- `tauri-app/src/components/MarkdownRenderer.vue` — Markdown-to-HTML rendering component

### Files to modify
- `tauri-app/src/types/index.ts` — Add `body: string | null` to `PullRequest`
- `tauri-app/src/views/PullRequestDetail.vue` — Add collapsible "Description" section
- `tauri-app/package.json` — Add `marked` dependency

### Implementation steps
1. Add `body` field to `PullRequest` type
2. Install `marked`
3. Create `MarkdownRenderer.vue` using `marked.parse()` with `v-html` and scoped styles
4. Add toggle section to `PullRequestDetail.vue`
5. Hide section if body is null/empty

### Dependencies
Requires backend to also fetch and return the `body` field (covered in Features plan).

### Estimated scope
**Medium** (1.5-2 hours)

---

## Improvement 11: Contextual Empty States

**Status**: ✅ Implemented
**Implementation notes**: `EmptyState.vue` reusable component with icon, title, description, and CTA slot. Integrated into Dashboard, Repositories, PRTable, and PullRequests views with contextual messaging.

### Summary
Replace plain-text empty states with rich, actionable components including icon, description, and CTA button.

### Files to create
- `tauri-app/src/components/EmptyState.vue` — Reusable empty state component with slots

### Files to modify
- `tauri-app/src/views/Dashboard.vue` — Replace `.empty-state` div
- `tauri-app/src/views/Repositories.vue` — Replace `.empty-state` div
- `tauri-app/src/components/PRTable.vue` — Replace `.empty-table` div
- `tauri-app/src/views/PullRequests.vue` — Pass filter context to PRTable

### Implementation steps
1. Create `EmptyState.vue` with props: `icon`, `title`, `description`, and default slot for CTA
2. Dashboard: "All caught up" with "View all PRs" button
3. Repositories: "No repositories tracked" with guidance text
4. PR Table: differentiate "no PRs at all" vs "no matches for current filters"

### Dependencies
None, pairs well with Improvement 3 (inline search) for filter-aware empty state.

### Estimated scope
**Small** (1 hour)

---

## Improvement 12: PR Age Warning Indicators

**Status**: ✅ Implemented
**Implementation notes**: `ageClass()` function in `PRTable.vue` with tiered colour coding using `--color-status-*` tokens, applied to age `<td>` and `PRCard.vue` `.pr-age` span.

### Summary
Colour-code the age text in the PR table based on staleness thresholds.

### Files to create
None

### Files to modify
- `tauri-app/src/components/PRTable.vue` — Add `ageClass()` function, apply to age `<td>`, add CSS rules

### Implementation steps
1. Add `ageClass(createdAt: string): string` — <24h = green, 24-72h = neutral, 72-168h = yellow, 168-336h = orange, >336h = red
2. Bind class to age `<td>`
3. Add CSS rules using existing `--color-status-*` tokens
4. Optionally apply to `PRCard.vue` `.pr-age` span

### Dependencies
None.

### Estimated scope
**Small** (20 minutes)

---

## Improvement 13: Batch Actions

**Status**: ✅ Implemented
**Implementation notes**: `BatchActionBar.vue` floating bar with slide-up transition, checkbox column with select-all in `PRTable.vue`, batch event handling in `PullRequests.vue`, toast feedback on completion.

### Summary
Checkboxes in PR table allowing multi-selection, with floating action bar for bulk operations.

### Files to create
- `tauri-app/src/components/BatchActionBar.vue` — Floating bar with selected count and action buttons

### Files to modify
- `tauri-app/src/components/PRTable.vue` — Add checkbox column, manage `selectedIds` set
- `tauri-app/src/views/PullRequests.vue` — Handle batch action events

### Implementation steps
1. Add `selectedIds` ref to `PRTable.vue`, checkbox column with select-all
2. Create `BatchActionBar.vue` — fixed bottom bar with "Approve All", "Mark Reviewed", "Open in GitHub" buttons
3. Show bar when selections exist, with `<Transition>` slide-up
4. Handle batch events in `PullRequests.vue`
5. Clear selection after batch completion, show toast feedback

### Dependencies
Improvement 5 (toast) for feedback.

### Estimated scope
**Medium** (2-3 hours)

---

## Improvement 14: Collapsible Sidebar

**Status**: ✅ Implemented
**Implementation notes**: `--sidebar-width-collapsed` token, `collapsed` ref persisted to localStorage in `AppSidebar.vue`, toggle button with chevron icon, Cmd+B shortcut, tooltip labels on hover in collapsed mode.

### Summary
Sidebar collapses to icon-only rail mode (~56px), with tooltip labels on hover. Toggle via button or Cmd+B.

### Files to create
None (can reuse `BaseTooltip.vue` from Improvement 8 if available)

### Files to modify
- `tauri-app/src/components/layout/AppSidebar.vue` — Add `collapsed` ref, conditional classes, toggle button, tooltip labels
- `tauri-app/src/styles/tokens.css` — Add `--sidebar-width-collapsed: 56px`
- `tauri-app/src/App.vue` — Layout adjusts automatically via flexbox

### Implementation steps
1. Add `--sidebar-width-collapsed` token
2. Add `collapsed` ref (persisted to localStorage)
3. Add toggle button (chevron icon)
4. When collapsed: hide labels/badges, centre icons, transition width
5. Add native `title` attributes or `BaseTooltip` for hover labels
6. Register Cmd+B keyboard shortcut

### Dependencies
Enhanced by Improvement 8 (BaseTooltip) but works without it.

### Estimated scope
**Medium** (1.5 hours)

---

## Improvement 15: Review Progress Indicator

**Status**: ✅ Implemented
**Implementation notes**: `ReviewProgress.vue` SVG donut chart with `stroke-dasharray`/`stroke-dashoffset`, percentage text, "X of Y PRs reviewed" label, integrated into `Dashboard.vue`.

### Summary
A progress bar or donut chart on the dashboard showing percentage of open PRs reviewed.

### Files to create
- `tauri-app/src/components/ReviewProgress.vue` — SVG donut chart with percentage text

### Files to modify
- `tauri-app/src/views/Dashboard.vue` — Add `<ReviewProgress>` below stats grid

### Implementation steps
1. Create `ReviewProgress.vue` with `reviewed` and `total` props
2. Render SVG donut using `stroke-dasharray`/`stroke-dashoffset`
3. Centre percentage text
4. Add label "X of Y PRs reviewed"
5. Compute reviewed count from `prStore.openPrs`
6. Handle edge case: `total === 0`

### Dependencies
None.

### Estimated scope
**Small** (1 hour)

---

## Improvement 16: Relative Time Auto-Refresh

**Status**: ✅ Implemented
**Implementation notes**: Reactive `now` ref with 30-second `setInterval` in `AppHeader.vue`, `lastSyncedFormatted` computed referencing `now.value`, interval cleaned up on unmount.

### Summary
The "Synced Xm ago" text automatically updates every 30 seconds.

### Files to create
None

### Files to modify
- `tauri-app/src/components/layout/AppHeader.vue` — Add reactive `now` ref with `setInterval`

### Implementation steps
1. Add `const now = ref(Date.now())`
2. On mount, start `setInterval(() => { now.value = Date.now() }, 30_000)`
3. On unmount, clear the interval
4. Update `lastSyncedFormatted` computed to reference `now.value`

### Dependencies
None.

### Estimated scope
**Small** (15 minutes)

---

## Improvement 17: Breadcrumb Navigation

**Status**: ✅ Implemented
**Implementation notes**: `Breadcrumb.vue` reusable component with `router-link` items and `>` separators, integrated into `PullRequestDetail.vue` replacing back button, title truncation for long PR titles.

### Summary
A breadcrumb trail on the PR detail page: Pull Requests > owner/repo > #123 Title.

### Files to create
- `tauri-app/src/components/Breadcrumb.vue` — Reusable breadcrumb component

### Files to modify
- `tauri-app/src/views/PullRequestDetail.vue` — Replace back button with `<Breadcrumb>`

### Implementation steps
1. Create `Breadcrumb.vue` accepting `items: {label, to?}[]` prop
2. Render as horizontal list with `>` separators, `<router-link>` for items with `to`
3. Compute breadcrumb items from PR data
4. Truncate long PR titles

### Dependencies
Integrates well with Improvement 18 (persistent filter state).

### Estimated scope
**Small** (45 minutes)

---

## Improvement 18: Persistent Filter State

**Status**: ✅ Implemented
**Implementation notes**: `stores/filters.ts` Pinia store with `filterRepoId`, `filterState`, `searchQuery`, `sortBy`, `sortAsc` refs. `resetFilters()` action. `PullRequests.vue` wired to store, `sessionStorage` sync.

### Summary
PR list filters persist across navigation using Pinia state.

### Files to create
- `tauri-app/src/stores/filters.ts` — Dedicated Pinia store for filter state

### Files to modify
- `tauri-app/src/views/PullRequests.vue` — Replace local filter refs with store
- `tauri-app/src/components/PRTable.vue` — Optionally read/write sort state from store

### Implementation steps
1. Create `filters.ts` with `filterRepoId`, `filterState`, `searchQuery`, `sortBy`, `sortAsc` refs
2. Add `resetFilters()` function
3. Replace local state in `PullRequests.vue` with store
4. Optionally sync to `sessionStorage` for app restart persistence
5. Support URL query params for deep linking from breadcrumbs

### Dependencies
None. Integrates well with Improvements 3 (search) and 17 (breadcrumbs).

### Estimated scope
**Small** (1 hour)

---

## Recommended Implementation Order

### Wave 1 — Foundations (no dependencies, used by others)
1. **#5 Toast Notification System**
2. **#16 Relative Time Auto-Refresh**
3. **#12 PR Age Warning Indicators**
4. **#18 Persistent Filter State**

### Wave 2 — Core UX
5. **#3 Inline Quick Search**
6. **#6 Skeleton Loading States**
7. **#11 Contextual Empty States**
8. **#9 Confirmation Dialogs**
9. **#17 Breadcrumb Navigation**

### Wave 3 — Interactive Features
10. **#8 Risk Score Tooltips**
11. **#14 Collapsible Sidebar**
12. **#15 Review Progress Indicator**
13. **#7 Auto-Sync Background Polling**
14. **#10 PR Body Display**

### Wave 4 — Advanced Features
15. **#2 Global Keyboard Shortcuts**
16. **#1 Command Palette (Cmd+K)**
17. **#13 Batch Actions**
18. **#4 First-Run Onboarding**

## Summary Table

| # | Improvement | Impact | Effort |
|---|-------------|--------|--------|
| 1 | Command Palette (Cmd+K) | High | Medium |
| 2 | Global Keyboard Shortcuts | High | Medium |
| 3 | Inline Quick Search on PR List | High | Small |
| 4 | First-Run Onboarding Flow | High | Medium |
| 5 | Toast Notification System | Medium | Small |
| 6 | Skeleton Loading States | Medium | Small |
| 7 | Auto-Sync with Background Polling | High | Medium |
| 8 | Contextual Tooltips on Risk Scores | Medium | Small |
| 9 | Confirmation Dialog for Destructive Actions | High | Small |
| 10 | PR Body / Description Display | High | Small |
| 11 | Improved Empty States with Contextual CTAs | Medium | Small |
| 12 | PR Age Warning Indicators | Medium | Small |
| 13 | Batch Actions on PR List | Medium | Large |
| 14 | Collapsible Sidebar | Medium | Small |
| 15 | Review Progress Indicator on Dashboard | Medium | Small |
| 16 | Relative Time Refresh | Low | Small |
| 17 | Breadcrumb Navigation on PR Detail | Medium | Small |
| 18 | Persistent Filter State | Medium | Small |
