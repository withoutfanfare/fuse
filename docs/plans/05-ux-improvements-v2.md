# UX Improvements Implementation Plan v2

## New improvements beyond the original 18 in 02-ux-improvements.md

---

## Improvement 1: Tabbed PR Detail Layout

**Status**: ✅ Implemented
**Implementation notes**: `PullRequestDetail.vue` restructured into 4 tabs (Overview, Code, Discussion, AI) using `TabBar.vue`, `v-if` guards per section, sidebar (worktree + checklist) remains fixed across tabs.

**Description**: Restructure `PullRequestDetail.vue` from a single long scroll into a tabbed layout using the existing but unused `TabBar.vue` component. Tabs: Overview (summary, branch, labels, timeline), Code (diff + file tree), Discussion (comments + reviews), AI (merged AI review panel + prompt builder, resolving duplication).

**Problem**: Users scroll extensively to reach the section they need. Two AI panels (`AiReviewPanel` and `AiPromptBuilder`) adjacent on the same page is confusing and redundant.

**Approach**: Modify `PullRequestDetail.vue` to import `TabBar`, add active tab ref, create four sections with `v-if` guards. Sidebar (worktree + checklist) stays fixed across tabs.

**Effort**: Medium

---

## Improvement 2: Persistent Review Checklist State

**Status**: ✅ Implemented
**Implementation notes**: `commands/checklist.rs` with `save_checklist_state`/`get_checklist_state` Tauri commands keyed by PR ID, `useChecklist.ts` composable with debounced writes replacing ephemeral `checkedRules` ref.

**Description**: Persist the per-PR review checklist checked state through the Rust backend instead of a local reactive object that resets on navigation.

**Problem**: Codebase analysis notes "Review checklist state is ephemeral (lost on navigation)." Users lose track of which items they verified.

**Approach**: New `save_checklist_state` / `get_checklist_state` Tauri commands keyed by PR ID. New `useChecklist` composable wrapping fetch/save with debounced writes. Replace local `checkedRules` ref.

**Effort**: Medium

---

## Improvement 3: Wired Sort Controls in PR Table Header

**Status**: ✅ Implemented
**Implementation notes**: Local sort refs removed from `PRTable.vue`, sort state accepted as props from `PullRequests.vue` sourced from `useFiltersStore`, sort change events emitted upward, text arrows replaced with SVG chevrons.

**Description**: The PR table has sort logic and clickable `<th>` elements but the `filters` store `sortBy`/`sortAsc` values are never synchronised with the table. Wire them together so sort preferences persist across navigation.

**Problem**: "Sort state tracked in store but NOT applied to the PR list." Users expect sort preferences to survive navigation.

**Approach**: Remove local sort refs from `PRTable.vue`, accept sort state as props from `PullRequests.vue` sourced from `useFiltersStore`. Emit sort change events upward. Replace text arrows with SVG chevrons.

**Effort**: Small

---

## Improvement 4: Hover Preview Cards for PR Rows

**Status**: ✅ Implemented
**Implementation notes**: `PRHoverPreview.vue` with fixed positioning, `useHoverPreview.ts` composable managing delay timer and mouse position, `@mouseenter`/`@mouseleave` handlers on table rows, `prefers-reduced-motion` check.

**Description**: When hovering a PR row for 400ms, display a floating preview card showing PR description excerpt, labels, CI status summary, and risk breakdown. Avoids navigating into detail just to recall what a PR is about.

**Problem**: Triaging a long list requires clicking into each PR to see description or CI status, creating disruptive back-and-forth navigation.

**Approach**: New `PRHoverPreview.vue` with fixed positioning. `useHoverPreview` composable managing delay timer and mouse position. `@mouseenter`/`@mouseleave` handlers on table rows. `prefers-reduced-motion` check.

**Effort**: Medium

---

## Improvement 5: Keyboard-Navigable PR List (j/k/Enter)

**Status**: ✅ Implemented
**Implementation notes**: `focusedIndex` ref with `j`/`k`/`Enter`/`x` handlers added to `useKeyboardShortcuts.ts`, `.pr-row--focused` CSS class with visible outline, `scrollIntoView({ block: 'nearest' })` for viewport tracking.

**Description**: The shortcut overlay advertises `j`/`k` navigation but it's not actually implemented -- no handlers exist in `useKeyboardShortcuts.ts`. Implement proper list navigation: `j` down, `k` up, `Enter` opens, `x` toggles selection. Focused row gets a visible focus ring.

**Problem**: Keyboard power users cannot navigate the PR list without a mouse despite the shortcut overlay advertising it.

**Approach**: Add `focusedIndex` ref. Add `j`, `k`, `Enter`, `x` handlers to `useKeyboardShortcuts.ts`. Apply `.pr-row--focused` CSS class with visible outline. `scrollIntoView({ block: 'nearest' })` to keep focused row visible.

**Effort**: Small

---

## Improvement 6: Inline Quick-Status Dropdown on PR Table

**Status**: ✅ Implemented
**Implementation notes**: `QuickStatusPopover.vue` using `<Teleport to="body">`, click handler on review badge in `PRTable.vue` with `@click.stop`, calls `prStore.updateReviewStatus()` directly.

**Description**: Small dropdown trigger on the Review column that allows changing review status directly from the list without navigating to detail. Clicking the review badge opens a popover with status options.

**Problem**: Updating review status requires navigating to detail, scrolling, changing, going back. Too many clicks for triaging many PRs.

**Approach**: New `QuickStatusPopover.vue` using `<Teleport to="body">`. On click of review badge in `PRTable.vue`, open popover. Calls `prStore.updateReviewStatus()` directly. `@click.stop` to prevent row navigation.

**Effort**: Medium

---

## Improvement 7: Recently Visited PRs in Sidebar

**Status**: ✅ Implemented
**Implementation notes**: `useRecentPrs.ts` composable with `localStorage`-backed list capped at 5 entries, pushed on `PullRequestDetail.vue` mount, rendered as sidebar section with PR numbers (collapsed mode shows numbers only).

**Description**: "Recent" section at the bottom of `AppSidebar.vue` showing last 5 PR detail pages visited, with PR number and truncated title. Quick navigation back without going through the list.

**Problem**: After navigating away, getting back to a PR requires going to the list, finding it, clicking. No navigation history concept.

**Approach**: `useRecentPrs` composable maintaining `ref` capped at 5 entries in `localStorage`. Push current PR on `PullRequestDetail.vue` mount. Render small list in sidebar. Collapsed mode shows just PR numbers.

**Effort**: Small

---

## Improvement 8: Focus Trap and ARIA for Modals

**Status**: ✅ Implemented
**Implementation notes**: `useFocusTrap.ts` composable trapping Tab/Shift+Tab, `role="dialog"`, `aria-modal="true"`, `aria-labelledby` added to all modal overlays (`ConfirmDialog`, `CommandPalette`, `ShortcutOverlay`), focus restored on close.

**Description**: `ConfirmDialog.vue`, `CommandPalette.vue`, and `ShortcutOverlay.vue` lack ARIA roles and focus trapping. Focus can escape to background elements.

**Problem**: Screen reader users cannot identify dialogs. Keyboard users Tab out of modals. WCAG 2.1 AA violation.

**Approach**: `useFocusTrap` composable trapping Tab/Shift+Tab. Add `role="dialog"`, `aria-modal="true"`, `aria-labelledby` to all overlays. Restore focus on close.

**Effort**: Small

---

## Improvement 9: Stale Threshold in Settings UI

**Status**: ✅ Implemented
**Implementation notes**: Number input added to `Settings.vue` for `stale_threshold_days` with descriptive text, persisted via existing settings store.

**Description**: Add a settings row for `stale_threshold_days` which exists in the DB but has no UI.

**Problem**: Users cannot control what counts as stale without digging into the backend.

**Approach**: Number input in `Settings.vue` for `stale_threshold_days` with descriptive text. Persist via existing settings store.

**Effort**: Small

---

## Improvement 10: Sparkline Data Wiring for Dashboard

**Status**: ✅ Implemented
**Implementation notes**: `get_daily_pr_counts` Tauri command for 7-day history, `StatsCard.vue` accepts `trend` prop, `Sparkline.vue` wired with velocity-derived trend data in `Dashboard.vue`.

**Description**: Wire the existing `Sparkline.vue` component (currently data-less) into `StatsCard.vue` with 7-day trends from velocity data. New lightweight endpoint for daily open/pending counts.

**Problem**: "Sparkline component exists but has no data wired to it." Dashboard stats are point-in-time with no trend context.

**Approach**: `StatsCard.vue` accepts optional `trend: number[]` prop. Derive trends from existing `velocityData`. New `get_daily_pr_counts` Tauri command for 7-day history.

**Effort**: Medium

---

## Improvement 11: Reduced Motion and High Contrast Support

**Status**: ✅ Implemented
**Implementation notes**: `@media (prefers-reduced-motion: reduce)` block zeroing transition durations, `[data-high-contrast]` attribute overriding tokens for increased contrast/opacity, toggle in `Settings.vue`, 2px+ `focus-visible` outlines.

**Description**: Respect `prefers-reduced-motion` for all animations. Add high-contrast mode toggle that increases border contrast, removes transparency, and bumps muted text colours.

**Problem**: Users with vestibular disorders get forced animations. Low-vision users cannot distinguish subtle translucent surfaces.

**Approach**: `@media (prefers-reduced-motion: reduce)` block setting transitions to 0ms. `[data-high-contrast]` attribute overriding tokens. Toggle in Settings. Ensure 2px+ focus-visible outlines.

**Effort**: Medium

---

## Improvement 12: Inline Repository Default Branch Editing

**Status**: ✅ Implemented
**Implementation notes**: `update_repository_branch` Tauri command, click-to-edit `<input>` in `RepositoryCard.vue` with Enter/blur save and Escape cancel, pencil icon on hover.

**Description**: Click-to-edit affordance on the default branch field within the repository card.

**Problem**: "No way to edit a repository's default branch after adding." Must remove and re-add the repository.

**Approach**: New `update_repository_branch` Tauri command. Click handler on branch display switches to `<input>`. Enter/blur saves, Escape cancels. Small pencil icon on hover.

**Effort**: Small

---

## Improvement 13: CI Checks and Comments Caching Layer

**Status**: ✅ Implemented
**Implementation notes**: `useCache.ts` composable with `Map`-based TTL caching (default 60s), wrapping CI checks and comments invocations, module-level state surviving component remounts, refresh icon clearing cache entries.

**Description**: Frontend caching layer with configurable TTL (default 60s) so navigating away and back doesn't re-fetch data just loaded. "Last fetched Xs ago" indicator with manual refresh.

**Problem**: "No caching layer for CI checks or comments." Repeated visits trigger redundant API calls, adding latency and consuming rate limits.

**Approach**: `useCache` composable storing entries in a `Map` with TTL. Wrap CI checks and comments invocations. Module-level state survives component remounts. Refresh icon clears cache entry.

**Effort**: Medium

---

## Recommended Implementation Order

### Wave 1 -- Quick wins fixing documented pain points:
1. Improvement 3 (Sort controls)
2. Improvement 5 (j/k keyboard nav)
3. Improvement 9 (Stale threshold UI)
4. Improvement 12 (Default branch editing)

### Wave 2 -- Core UX depth:
5. Improvement 2 (Persistent checklist)
6. Improvement 1 (Tabbed PR detail)
7. Improvement 6 (Inline status dropdown)
8. Improvement 7 (Recent PRs sidebar)

### Wave 3 -- Polish and accessibility:
9. Improvement 8 (ARIA focus traps)
10. Improvement 11 (Reduced motion / high contrast)
11. Improvement 13 (Caching layer)

### Wave 4 -- Enhancement:
12. Improvement 4 (Hover previews)
13. Improvement 10 (Sparkline data wiring)

## Summary Table

| # | Improvement | Problem | Effort |
|---|-------------|---------|--------|
| 1 | Tabbed PR Detail Layout | Long scroll, duplicate AI panels | Medium |
| 2 | Persistent Review Checklist | Checklist lost on navigation | Medium |
| 3 | Wired Sort Controls | Sort state not applied | Small |
| 4 | Hover Preview Cards | Too many clicks to triage | Medium |
| 5 | Keyboard-Navigable PR List | j/k advertised but not implemented | Small |
| 6 | Inline Quick-Status Dropdown | Status change requires full navigation | Medium |
| 7 | Recently Visited PRs | No navigation history | Small |
| 8 | ARIA Focus Traps | Accessibility violations | Small |
| 9 | Stale Threshold in Settings | Missing UI for existing setting | Small |
| 10 | Sparkline Data Wiring | Component exists, no data | Medium |
| 11 | Reduced Motion / High Contrast | Accessibility gaps | Medium |
| 12 | Inline Default Branch Edit | Can't edit after creation | Small |
| 13 | CI/Comments Caching | Redundant API calls | Medium |
