# UI Design Implementation Plan

## Codebase Summary

The app is a Tauri v2 + Vue 3 + Pinia desktop application with a glassmorphic dark theme. Styling uses CSS custom properties defined in `tokens.css` and `base.css`. Components use Vue `<script setup lang="ts">` with scoped CSS. No third-party UI libraries, icon libraries, or charting libraries are currently installed. Unicode glyphs are used for icons. No existing transition system, toast system, or modal system exists.

---

## Improvement 1: Risk Score Radial Gauge

**Status**: ✅ Implemented
**Implementation notes**: `RiskGauge.vue` SVG component with animated arc and colour transitions, `riskColour()` in `useRiskScore.ts`, replaced `RiskBadge` in `PRCard`, `PRTable`, and `PullRequestDetail`.

### Summary
Replace the flat numeric `RiskBadge` pill with an SVG arc/radial gauge that fills proportionally (0-100% arc for scores 1-10) and transitions through green, yellow, orange, and red as the score increases.

### Files to create
- `src/components/RiskGauge.vue` — SVG-based radial gauge component

### Files to modify
- `src/components/PRCard.vue` — Replace `<RiskBadge>` with `<RiskGauge>` (36px diameter)
- `src/components/PRTable.vue` — Replace `<RiskBadge>` with `<RiskGauge>` (28px for table density)
- `src/views/PullRequestDetail.vue` — Replace `<RiskBadge>` with `<RiskGauge>` (48px)
- `src/composables/useRiskScore.ts` — Add `riskColour(score: number): string` function
- `src/styles/tokens.css` — Add `--color-risk-gauge-track: rgba(255, 255, 255, 0.08)`

### Implementation steps
1. Add track token to `tokens.css`
2. Add `riskColour()` to `useRiskScore.ts` mapping scores to risk colour tokens
3. Create `RiskGauge.vue` with inline SVG, `<circle>` track and value arc using `stroke-dasharray`/`stroke-dashoffset`, centred `<text>` for score
4. Apply `transition: stroke-dashoffset 0.6s ease, stroke 0.4s ease` for animation
5. Replace `<RiskBadge>` across all three locations

### Dependencies
None.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 2: Review Pipeline Progress Bar

**Status**: ✅ Implemented
**Implementation notes**: `ReviewPipeline.vue` with step circles, connecting lines, pulsing active step glow, and "Changes Requested" red badge. Pipeline tokens in `tokens.css`. Integrated into `ReviewStatus.vue`.

### Summary
Replace the flat row of status buttons in `ReviewStatus.vue` with a stepped horizontal pipeline visualisation (Pending > In Progress > Reviewed > Approved) with connected nodes and a progress line.

### Files to create
- `src/components/ReviewPipeline.vue` — Horizontal pipeline component with step circles connected by lines

### Files to modify
- `src/components/ReviewStatus.vue` — Replace `.status-buttons` with `<ReviewPipeline>`
- `src/styles/tokens.css` — Add `--color-pipeline-track` and `--color-pipeline-complete` tokens

### Implementation steps
1. Add pipeline tokens
2. Create `ReviewPipeline.vue` with ordered steps, computed completed/active/upcoming states, flex row layout with connecting lines
3. Active step pulses with glow animation; "Changes Requested" renders as red badge/fork
4. Update `ReviewStatus.vue` to use new component
5. Style with glassmorphic card patterns

### Dependencies
Benefits from Improvement 5 (icons) for step icons, but can use text labels initially.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 3: Dashboard Activity Sparklines

**Status**: ✅ Implemented
**Implementation notes**: `Sparkline.vue` pure SVG component with polyline and gradient fill, `useActivityHistory.ts` composable, `StatsCard.vue` accepts `history` prop, `Dashboard.vue` provides history data.

### Summary
Tiny inline SVG trend charts on each `StatsCard` showing 7-14 days of historical data.

### Files to create
- `src/components/Sparkline.vue` — Pure SVG sparkline with `<polyline>` and optional gradient fill
- `src/composables/useActivityHistory.ts` — Composable for historical metric data

### Files to modify
- `src/components/StatsCard.vue` — Add optional `history: number[]` prop, render `<Sparkline>` below label
- `src/views/Dashboard.vue` — Pass history data to each `StatsCard`
- `src/types/index.ts` — Add `ActivitySnapshot` interface if needed

### Implementation steps
1. Create `Sparkline.vue` — normalise data to viewBox, render polyline with gradient fill
2. Create `useActivityHistory.ts` — generate mock data initially, later wire to backend
3. Update `StatsCard.vue` to accept and render sparkline
4. Update `Dashboard.vue` to provide history data

### Dependencies
None (backend needed for real data).

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 4: Page Transition Animations

**Status**: ✅ Implemented
**Implementation notes**: `transitions.css` with page-fade and card-cascade classes, `<Transition>` wrapper on `<router-view>` in `App.vue`, `<TransitionGroup>` on Dashboard card grids.

### Summary
Vue `<Transition>` on `<router-view>` for crossfade/slide, plus cascading entrance animations for card grids.

### Files to create
- `src/styles/transitions.css` — Shared CSS transition classes for page-fade and card-cascade

### Files to modify
- `src/App.vue` — Wrap `<router-view>` with `<Transition name="page-fade" mode="out-in">`
- `src/styles/base.css` — Import `transitions.css`
- `src/views/Dashboard.vue` — Add `<TransitionGroup name="card-cascade">` around card grids

### Implementation steps
1. Create `transitions.css` with page-fade (opacity + translateY, 200ms) and card-cascade (staggered delays)
2. Import from `base.css`
3. Update `App.vue` to use `<Transition>` slot pattern
4. Update `Dashboard.vue` with `<TransitionGroup>` and cascade indices
5. Test all route transitions

### Dependencies
None.

### Estimated scope
**Small** (< 1 hour)

---

## Improvement 5: Proper SVG Icon System

**Status**: ✅ Implemented
**Implementation notes**: `lucide-vue-next` installed, Unicode glyphs replaced across `AppSidebar.vue`, `AppHeader.vue`, `RepositoryCard.vue`, `PullRequestDetail.vue`, `WorktreePanel.vue`, and `PRTable.vue` with Lucide components.

### Summary
Replace Unicode glyphs (`◉`, `⎇`, `◫`, `⚙`, `↻`, `✕`) with Lucide icons (`lucide-vue-next`).

### Files to create
None (library provides components directly).

### Files to modify
- `package.json` — Add `lucide-vue-next`
- `src/components/layout/AppSidebar.vue` — Replace string icons with Lucide components (`LayoutDashboard`, `GitPullRequest`, `FolderGit2`, `Settings`)
- `src/components/layout/AppHeader.vue` — Replace `↻` with `<RefreshCw>`
- `src/components/RepositoryCard.vue` — Replace `↻` and `✕` with `<RefreshCw>` and `<X>`
- `src/views/PullRequestDetail.vue` — Replace `←` with `<ArrowLeft>`
- `src/components/WorktreePanel.vue` — Replace `⚠` with `<AlertTriangle>`
- `src/components/PRTable.vue` — Replace `⚠` with `<AlertTriangle>`

### Implementation steps
1. Install `lucide-vue-next`
2. Identify every Unicode glyph in the codebase
3. Replace each with appropriate Lucide component, matching size with `:size` prop
4. Icons use `currentColor` so existing CSS colour rules continue to work

### Dependencies
None.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 6: Empty State Illustrations

**Status**: ✅ Implemented
**Implementation notes**: `EmptyState.vue` reusable component with illustration slot, SVG illustrations in `assets/illustrations/`, integrated into Dashboard, PRTable, WorktreePanel, and Repositories views.

### Summary
On-brand SVG illustrations for empty states using muted ember/grey tones and translucent shapes.

### Files to create
- `src/components/EmptyState.vue` — Reusable empty state wrapper with `#illustration` slot
- `src/assets/illustrations/empty-dashboard.svg`
- `src/assets/illustrations/empty-prs.svg`
- `src/assets/illustrations/empty-worktree.svg`
- `src/assets/illustrations/empty-repos.svg`

### Files to modify
- `src/views/Dashboard.vue` — Replace `.empty-state` with `<EmptyState>`
- `src/components/PRTable.vue` — Replace `.empty-table` with `<EmptyState>`
- `src/components/WorktreePanel.vue` — Replace `.empty-state` with `<EmptyState>`
- `src/views/Repositories.vue` — Replace `.empty-state` with `<EmptyState>`

### Implementation steps
1. Create SVG illustrations using app colour tokens (under 5KB each)
2. Create `EmptyState.vue` with centred layout, slot for illustration, title, and description
3. Replace all four empty state locations

### Dependencies
None.

### Estimated scope
**Medium** (1-3 hours) — most time in SVG creation.

---

## Improvement 7: PR Size Visualisation Bar

**Status**: ✅ Implemented
**Implementation notes**: `SizeBar.vue` component with percentage-width green/red segments, integrated into `PRTable.vue` size column with hover tooltip for exact counts.

### Summary
A stacked horizontal bar in the Size column showing green (additions) and red (deletions) segments.

### Files to create
- `src/components/SizeBar.vue` — Component rendering a 4px-height stacked bar

### Files to modify
- `src/components/PRTable.vue` — Add `<SizeBar>` beneath existing size text

### Implementation steps
1. Create `SizeBar.vue` — compute percentage widths, render two coloured divs
2. Handle edge case where both values are zero
3. Import and place in PRTable size column
4. Add hover tooltip with exact counts

### Dependencies
None.

### Estimated scope
**Small** (< 1 hour)

---

## Improvement 8: Keyboard Shortcut Overlay

**Status**: ✅ Implemented
**Implementation notes**: `ShortcutOverlay.vue` glassmorphic modal with two-column grid, `useKeyboardShortcuts.ts` with `?` trigger and `g` prefix support, wired in `App.vue`.

### Summary
A glassmorphic modal listing all available keyboard shortcuts, triggered by pressing `?`.

### Files to create
- `src/components/ShortcutOverlay.vue` — Modal listing shortcuts in a two-column grid
- `src/composables/useKeyboardShortcuts.ts` — Global keydown listener with `g` prefix support

### Files to modify
- `src/App.vue` — Render `<ShortcutOverlay>` and initialise shortcuts composable

### Implementation steps
1. Create `useKeyboardShortcuts.ts` with `pendingPrefix` ref, `showOverlay` ref, and shortcuts array
2. Create `ShortcutOverlay.vue` with glassmorphic overlay and styled grid
3. Wire into `App.vue`
4. Ensure shortcuts skip when inputs are focused

### Dependencies
Benefits from Improvement 5 (icons) for key cap styling.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 9: Global Toast Notifications

**Status**: ✅ Implemented
**Implementation notes**: `useToast.ts` composable with reactive queue and auto-dismiss, `ToastItem.vue` with glassmorphic styling and coloured left borders, `ToastContainer.vue` mounted in `App.vue`, inline messages migrated to toasts.

### Summary
Slide-in notification system from top-right with auto-dismiss and type-based styling.

### Files to create
- `src/components/ToastContainer.vue` — Fixed-position container with `<TransitionGroup>`
- `src/components/ToastItem.vue` — Individual toast card
- `src/composables/useToast.ts` — Toast state management

### Files to modify
- `src/App.vue` — Mount `<ToastContainer>`
- `src/views/PullRequestDetail.vue` — Replace inline `actionMessage` with toast calls
- `src/views/Repositories.vue` — Add toast for add/remove feedback

### Implementation steps
1. Create `useToast.ts` with reactive queue and auto-dismiss
2. Create `ToastItem.vue` with glassmorphic styling and coloured left border
3. Create `ToastContainer.vue` as fixed container
4. Mount in `App.vue`
5. Migrate existing inline messages

### Dependencies
None.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 10: Collapsible Sidebar with Tooltips

**Status**: ✅ Implemented
**Implementation notes**: `useSidebarState.ts` composable with localStorage persistence, `--sidebar-width-collapsed` token, toggle button and tooltip labels in `AppSidebar.vue`, width transition animation.

### Summary
Icon-only rail mode (~56px) with tooltip labels on hover and localStorage persistence.

### Files to create
- `src/composables/useSidebarState.ts` — Collapsed state management with localStorage

### Files to modify
- `src/components/layout/AppSidebar.vue` — Add collapsed mode, toggle button, tooltip spans
- `src/styles/tokens.css` — Add `--sidebar-width-collapsed: 56px`

### Implementation steps
1. Add token
2. Create composable with localStorage persistence
3. Update sidebar with toggle, conditional classes, width transition
4. Add tooltip labels for collapsed state

### Dependencies
Benefits from Improvement 5 (icons).

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 11: Author Avatars

**Status**: ✅ Implemented
**Implementation notes**: `AuthorAvatar.vue` with retina URL and `@error` fallback to coloured initial circle, CSP updated for GitHub domains, integrated into `PRCard`, `PRTable`, and `PullRequestDetail`.

### Summary
GitHub profile images (20-24px, rounded) next to author names throughout the app.

### Files to create
- `src/components/AuthorAvatar.vue` — Image component with fallback initials

### Files to modify
- `src/components/PRCard.vue` — Add avatar before author name
- `src/components/PRTable.vue` — Add avatar in author column
- `src/views/PullRequestDetail.vue` — Add avatar next to author name
- `tauri-app/src-tauri/tauri.conf.json` — Update CSP `img-src` for GitHub domains

### Implementation steps
1. Update CSP to allow `https://github.com` and `https://avatars.githubusercontent.com`
2. Create `AuthorAvatar.vue` with retina URL, `@error` fallback to coloured initial circle
3. Add avatar component across three locations
4. Style with `object-fit: cover` and border

### Dependencies
None.

### Estimated scope
**Small** (< 1 hour)

---

## Improvement 12: Sticky Table Header

**Status**: ✅ Implemented
**Implementation notes**: `position: sticky` on `<th>` elements with opaque background, `--shadow-scroll` token, `@scroll` listener on wrapper with conditional shadow pseudo-element.

### Summary
Fixed `<thead>` with scroll shadow appearing on scroll.

### Files to create
None.

### Files to modify
- `src/components/PRTable.vue` — Add `position: sticky` to `th`, scroll listener for shadow
- `src/styles/tokens.css` — Add `--shadow-scroll` token

### Implementation steps
1. Add scroll shadow token
2. Make `<th>` sticky with opaque background
3. Add `@scroll` listener to wrapper
4. Conditionally show shadow pseudo-element

### Dependencies
None.

### Estimated scope
**Small** (< 1 hour)

---

## Improvement 13: PR Age Colour Coding

**Status**: ✅ Implemented
**Implementation notes**: `ageColourClass()` function with threshold logic in `PRTable.vue`, CSS rules using `--color-status-*` tokens, applied to both `PRTable` and `PRCard` age displays.

### Summary
Dynamic text colour in the age column based on staleness thresholds.

### Files to create
None.

### Files to modify
- `src/components/PRTable.vue` — Add `ageColourClass()` function and CSS rules
- `src/components/PRCard.vue` — Apply same colour coding to `.pr-age`

### Implementation steps
1. Add `ageColourClass()` function with threshold logic
2. Bind class to age cells
3. Add CSS rules using existing `--color-status-*` tokens

### Dependencies
None.

### Estimated scope
**Small** (< 1 hour)

---

## Improvement 14: Confirmation Modals

**Status**: ✅ Implemented
**Implementation notes**: `ConfirmModal.vue` with glassmorphic overlay and variant-styled confirm button, `useConfirm.ts` async composable returning `Promise<boolean>`, mounted in `App.vue`, integrated into merge, delete, and remove actions.

### Summary
Glassmorphic confirmation dialogs for destructive actions.

### Files to create
- `src/components/ConfirmModal.vue` — Reusable modal with variant-styled confirm button
- `src/composables/useConfirm.ts` — Async `confirm()` function returning Promise<boolean>

### Files to modify
- `src/App.vue` — Mount `<ConfirmModal>` at root
- `src/views/PullRequestDetail.vue` — Wrap merge in confirm
- `src/views/Repositories.vue` — Wrap delete in confirm
- `src/components/WorktreePanel.vue` — Wrap remove in confirm

### Implementation steps
1. Create `useConfirm.ts` with global reactive state and Promise-based API
2. Create `ConfirmModal.vue` with glassmorphic overlay
3. Mount in `App.vue`
4. Integrate into three destructive action sites

### Dependencies
None.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 15: Review Notes Markdown Preview

**Status**: ✅ Implemented
**Implementation notes**: `MarkdownPreview.vue` component, `useMarkdown.ts` wrapper with `marked` and `dompurify`, edit/preview toggle in `ReviewStatus.vue`.

### Summary
Toggle between edit textarea and rendered markdown preview in the ReviewStatus component.

### Files to create
- `src/components/MarkdownPreview.vue` — Markdown rendering component
- `src/composables/useMarkdown.ts` — Wrapper around markdown parser with sanitisation

### Files to modify
- `package.json` — Add `marked` and `dompurify`
- `src/components/ReviewStatus.vue` — Add edit/preview toggle

### Implementation steps
1. Install `marked` and `dompurify`
2. Create `useMarkdown.ts` with `renderMarkdown()` function
3. Create `MarkdownPreview.vue` with styled HTML output
4. Add toggle to `ReviewStatus.vue`

### Dependencies
None.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 16: Review Velocity Chart

**Status**: ✅ Implemented
**Implementation notes**: `VelocityChart.vue` SVG area chart with gradient fill and polyline stroke, `useReviewVelocity.ts` data composable, "Review Velocity" section in `Dashboard.vue`.

### Summary
SVG area chart of PRs reviewed per day over 30 days.

### Files to create
- `src/components/VelocityChart.vue` — SVG-based area chart
- `src/composables/useReviewVelocity.ts` — Data composable (mock data initially)

### Files to modify
- `src/views/Dashboard.vue` — Add "Review Velocity" section
- `src/types/index.ts` — Add `VelocityDataPoint` interface

### Implementation steps
1. Add type
2. Create data composable with 30 days of mock data
3. Create chart component — SVG path for area fill with gradient, polyline for stroke, axis labels
4. Add to Dashboard in a glassmorphic card section

### Dependencies
None (pure SVG approach).

### Estimated scope
**Large** (3+ hours)

---

## Improvement 17: Custom Window Chrome

**Status**: ✅ Implemented
**Implementation notes**: `TitleBar.vue` with `data-tauri-drag-region`, `"decorations": false` in Tauri config, macOS traffic light spacing, glassmorphic background, 38px height, mounted in `App.vue`.

### Summary
Custom draggable title bar matching the glassmorphic design, replacing native window decorations.

### Files to create
- `src/components/layout/TitleBar.vue` — Custom title bar with `data-tauri-drag-region`

### Files to modify
- `tauri-app/src-tauri/tauri.conf.json` — Set `"decorations": false`
- `src/App.vue` — Add `<TitleBar />` at top of layout
- `src/components/layout/AppHeader.vue` — Coordinate with title bar
- `src/styles/base.css` — Account for title bar height

### Implementation steps
1. Set `"decorations": false` in Tauri config
2. Create `TitleBar.vue` with `data-tauri-drag-region`, macOS traffic light spacing (78px left padding), Windows/Linux custom close/minimise/maximise buttons
3. Style with glassmorphic background, border-bottom, 38px height
4. Mount in `App.vue` and adjust layout
5. Test drag behaviour and window controls

### Dependencies
None, but implement carefully as it changes fundamental window behaviour.

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 18: PR Detail Tabs

**Status**: ✅ Implemented
**Implementation notes**: `TabBar.vue` with animated underline indicator, `PullRequestDetail.vue` restructured into Overview/Review/Actions tabs with `activeTab` ref and `<Transition>` content switching.

### Summary
Tabbed interface (Overview, Review, Actions) replacing the single scrolling column in PullRequestDetail.

### Files to create
- `src/components/TabBar.vue` — Reusable horizontal tab bar with animated underline indicator

### Files to modify
- `src/views/PullRequestDetail.vue` — Major restructure into tabbed sections

### Implementation steps
1. Create `TabBar.vue` with animated underline indicator
2. Restructure PullRequestDetail:
   - **Overview**: Change Summary, Branch, Labels, Timeline
   - **Review**: ReviewStatus component
   - **Actions**: GitHub Actions, WorktreePanel
3. Add `activeTab` ref defaulting to 'overview'
4. Add `<Transition>` for tab content

### Dependencies
Benefits from Improvement 4 (transitions).

### Estimated scope
**Medium** (1-3 hours)

---

## Improvement 19: Skeleton Loading Screens

**Status**: ✅ Implemented
**Implementation notes**: `skeleton.css` with shimmer keyframe and `.skeleton-block` class, `SkeletonCard.vue`, `SkeletonTable.vue`, `SkeletonStats.vue`, `SkeletonDetail.vue` components, integrated into Dashboard, PullRequests, and PullRequestDetail views.

### Summary
Animated shimmer placeholders matching content shapes for all loading states.

### Files to create
- `src/components/skeletons/SkeletonCard.vue` — PRCard shape
- `src/components/skeletons/SkeletonTable.vue` — PRTable rows
- `src/components/skeletons/SkeletonStats.vue` — StatsCard shape
- `src/components/skeletons/SkeletonDetail.vue` — PullRequestDetail layout
- `src/styles/skeleton.css` — Shared shimmer keyframe and `.skeleton-block` class

### Files to modify
- `src/styles/base.css` — Import `skeleton.css`
- `src/views/Dashboard.vue` — Show skeletons while loading
- `src/views/PullRequests.vue` — Show skeleton table while loading
- `src/views/PullRequestDetail.vue` — Replace "Loading..." with skeleton

### Implementation steps
1. Create `skeleton.css` with `@keyframes shimmer` and `.skeleton-block` base class
2. Create each skeleton component matching real component dimensions
3. Integrate into views behind loading state checks

### Dependencies
None.

### Estimated scope
**Medium** (1-3 hours)

---

## Recommended Implementation Order

| Phase | Improvements | Rationale |
|-------|-------------|-----------|
| **Phase 1: Foundation** | 5 (Icons), 4 (Transitions), 19 (Skeletons) | Icon system, animation infrastructure, loading states |
| **Phase 2: Core Components** | 1 (Risk Gauge), 7 (Size Bar), 13 (Age Colours), 12 (Sticky Header) | Small/medium table and card enhancements |
| **Phase 3: Layout & Chrome** | 10 (Collapsible Sidebar), 17 (Custom Window Chrome), 18 (PR Detail Tabs) | Structural layout changes |
| **Phase 4: Interactive Systems** | 9 (Toast Notifications), 14 (Confirmation Modals), 8 (Keyboard Shortcuts) | Global interactive systems |
| **Phase 5: Content Enhancements** | 2 (Review Pipeline), 6 (Empty States), 11 (Author Avatars), 15 (Markdown Preview) | Component-level improvements |
| **Phase 6: Data Visualisation** | 3 (Sparklines), 16 (Velocity Chart) | Most standalone, require data infrastructure |

## Summary Table

| # | Improvement | Impact | Effort |
|---|-------------|--------|--------|
| 1 | Risk Score Radial Gauge | High | Medium |
| 2 | Review Pipeline Progress Bar | High | Medium |
| 3 | Dashboard Activity Sparklines | High | Large |
| 4 | Page Transition Animations | High | Small |
| 5 | Proper SVG Icon System | High | Small |
| 6 | Empty State Illustrations | Medium | Medium |
| 7 | PR Size Visualisation Bar | Medium | Small |
| 8 | Keyboard Shortcut Overlay | Medium | Medium |
| 9 | Global Toast Notifications | Medium | Medium |
| 10 | Collapsible Sidebar with Tooltips | Medium | Medium |
| 11 | Author Avatars | Medium | Small |
| 12 | Sticky Table Header | Medium | Small |
| 13 | PR Age Colour Coding | Medium | Small |
| 14 | Confirmation Modals | Low | Small |
| 15 | Review Notes Markdown Preview | Low | Medium |
| 16 | Review Velocity Chart | High | Large |
| 17 | Custom Window Chrome | High | Medium |
| 18 | PR Detail Tabs | Medium | Medium |
| 19 | Skeleton Loading Screens | Medium | Small |
