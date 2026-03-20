# UI Design Implementation Plan v2

## New visual/design improvements beyond the original 19 in 03-ui-design.md

---

## Improvement 1: Typographic Scale System

**Status**: ✅ Implemented
**Implementation notes**: 5-level token system (`--text-display` through `--text-caption`) in `tokens.css`, applied across `StatsCard`, `PRCard`, `PullRequestDetail` header, and section titles with negative letter-spacing at display sizes.

**Description**: Introduce a proper typographic scale with display weight for hero numbers (48px/800), heading (18-22px/700), subheading (15-16px/600), body (14px/400), caption (12px/500). Apply negative letter-spacing at display sizes.

**Problem**: Everything has roughly equal visual weight. Stats values, section titles, labels, and body text all compete at similar sizes. No clear scan hierarchy.

**Approach**: New `--text-display` through `--text-caption` tokens in `tokens.css`. Apply across `StatsCard`, `PRCard`, `PullRequestDetail` header, and section titles.

**Effort**: Small

---

## Improvement 2: Light Mode with System Preference Detection

**Status**: ✅ Implemented
**Implementation notes**: `[data-theme="light"]` token overrides in `tokens.css`, `useTheme()` composable with `matchMedia` listener for System detection, blob colour and shadow opacity adjustments, theme dropdown enabled in `Settings.vue`.

**Description**: Full light mode by inverting semantic surface tokens behind `[data-theme="light"]`. Third "System" option using `prefers-color-scheme`. Glassmorphic panels swap to white/grey with reduced saturation. Blobs shift to softer pastels.

**Problem**: App is unusable in bright environments. The disabled theme dropdown in Settings is an unfulfilled promise.

**Approach**: Duplicate semantic tokens in `tokens.css` under `[data-theme="light"]`. `useTheme()` composable with `matchMedia` listener for System option. Adjust blob colours and shadow opacities.

**Effort**: Large

---

## Improvement 3: Accent Colour Picker

**Status**: ✅ Implemented
**Implementation notes**: 7 accent presets (teal, violet, blue, amber, rose, emerald, orange) in `Settings.vue`, `useTheme()` sets `--color-accent` and derived tokens on `document.documentElement.style`, persisted via settings store.

**Description**: 6-8 predefined accent colour swatches (teal, violet, blue, amber, rose, emerald, orange) in Settings. Choice propagates through all `--color-accent` usages.

**Problem**: Personal preference matters in a daily-use tool. Also distinguishes multiple instances.

**Approach**: Accent presets as `{ name, hue, sat, light }` values. `useTheme()` sets `--color-accent` and derived tokens on `document.documentElement.style`. Store via settings store.

**Effort**: Medium

---

## Improvement 4: Density Toggle (Comfortable / Compact)

**Status**: ✅ Implemented
**Implementation notes**: `.density-compact` class overriding `--space-*` tokens with 40% reduction, `useDensity()` composable backed by settings store, toggle in `Settings.vue`, components inherit changes automatically via CSS custom properties.

**Description**: Two-state toggle switching between current spacing and compact mode (40% reduced padding, tighter rows, smaller gauges). Single CSS class on `#app` overrides spacing tokens.

**Problem**: Power users monitoring 30+ PRs need more rows without scrolling. Current layout wastes vertical space.

**Approach**: `.density-compact` class overriding `--space-*` tokens. `useDensity()` composable backed by settings store. Components inherit changes automatically via CSS custom properties.

**Effort**: Medium

---

## Improvement 5: Risk-Level Glow Borders on PR Cards

**Status**: ✅ Implemented
**Implementation notes**: Computed `--card-risk-color` CSS variable bound on `PRCard.vue`, `border-left: 3px solid` with hover `box-shadow` glow, colour-coded green/amber/red based on risk score.

**Description**: Gradient left-border on PR cards reflecting risk level. Low=green, medium=amber, high=red. Hover intensifies glow. Creates instant "traffic light" scan pattern.

**Problem**: Risk only communicated through small 36px gauge inside each card. Eye must locate each gauge individually when scanning 10+ cards.

**Approach**: Bind computed `--card-risk-color` CSS variable on `PRCard.vue`. Apply `border-left: 3px solid` and hover `box-shadow`. Mirrors toast left-border pattern.

**Effort**: Small

---

## Improvement 6: Number Countup Animation on Stats

**Status**: ✅ Implemented
**Implementation notes**: `useCountUp(targetValue, duration)` composable returning reactive `displayValue` using `requestAnimationFrame`, applied in `StatsCard.vue` with `watch` on value prop for smooth transitions.

**Description**: Stats cards animate values from 0 to actual over 600ms with easing. Value changes during sync smoothly transition old to new. Pure `requestAnimationFrame`, no library.

**Problem**: Numbers appear instantly. Animating creates arrival moment and draws eye to changes after sync. Reinforces "live data" feeling.

**Approach**: `useCountUp(targetValue, duration)` composable returning reactive `displayValue`. Apply in `StatsCard.vue` with `watch` on value prop.

**Effort**: Small

---

## Improvement 7: Resizable Split-Panel on PR Detail

**Status**: ✅ Implemented
**Implementation notes**: `ResizableSplit.vue` with mousedown/mousemove/mouseup drag events, `display: flex` layout with reactive `sidebarWidth` ref clamped 240-500px, `cursor: col-resize` divider, width persisted to localStorage.

**Description**: Replace fixed `grid-template-columns: 1fr 320px` with a resizable split-panel. Draggable divider between main content and sidebar (clamped 240-500px). Stored in localStorage.

**Problem**: Fixed 320px sidebar too narrow for long checklist rules, too wide when empty. Wide monitors want more diff space.

**Approach**: `ResizableSplit.vue` with mousedown/mousemove/mouseup events. `display: flex` with reactive `sidebarWidth` ref. `cursor: col-resize` divider.

**Effort**: Medium

---

## Improvement 8: Ambient Blob Response to Risk State

**Status**: ✅ Implemented
**Implementation notes**: Computed `queueRiskLevel` in `App.vue` from max risk score, blob opacity bound via CSS custom properties, risk-reactive magenta/teal opacity shifts with 2-3 second transitions.

**Description**: Background blobs subtly respond to overall queue risk. High risk: magenta blob increases opacity, teal dims. Low risk: teal brightens. Subliminal environmental cue.

**Problem**: Blobs are purely decorative, consuming resources but carrying no information. Making them responsive turns them into a data signal.

**Approach**: Computed `queueRiskLevel` in `App.vue` from max risk score. Bind blob opacity via CSS custom properties. 2-3 second transitions so shift is gradual.

**Effort**: Small

---

## Improvement 9: Code Font with Ligatures Toggle

**Status**: ✅ Implemented
**Implementation notes**: `--font-mono` updated to JetBrains Mono stack in `tokens.css`, `font-variant-ligatures` toggle in `Settings.vue`, diff line-height increased to 1.6, alternating `.diff-line-context:nth-child(even)` backgrounds.

**Description**: Default to ligature-capable font stack (`JetBrains Mono`, `Fira Code`, `Cascadia Code`). Toggle in Settings for ligatures on/off. Increase diff line-height to 1.6. Add subtle alternating-row background for context lines.

**Problem**: Diff viewer at 12px mono with 1.5 line-height is dense for extended reviews. Operators like `=>`, `!==` easier to parse with ligatures. Some developers prefer ligatures off.

**Approach**: Update `--font-mono` in `tokens.css`. Add ligatures toggle via `font-variant-ligatures`. Alternating backgrounds on `.diff-line-context:nth-child(even)`.

**Effort**: Small

---

## Improvement 10: Merge Celebration Animation

**Status**: ✅ Implemented
**Implementation notes**: `useConfetti()` composable spawning 40-60 absolutely positioned `<div>` particles with CSS animations, triggered from `handleMerge()` on success, positioned relative to button via `getBoundingClientRect()`, purple pulse badge on merged state.

**Description**: When a PR is successfully merged, trigger a brief confetti burst (1.5s) from the merge button position. Small rectangles in brand colours scatter with gravity physics. PR state badge pulses to purple "Merged".

**Problem**: Merging is the culmination of the review workflow but gets only a toast. Emotionally flat conclusion to a 30-minute review. Brief delight reinforces satisfaction.

**Approach**: `useConfetti()` composable spawning 40-60 absolutely positioned `<div>` particles with CSS animations. Trigger from `handleMerge()` on success. Position relative to button's `getBoundingClientRect()`. Auto-remove after animation.

**Effort**: Medium

---

## Improvement 11: Contextual Focus Mode

**Status**: ✅ Implemented
**Implementation notes**: `useFocusMode()` composable, `.focus-active` class on `.app-layout` setting `--sidebar-width: 0` and hiding header with transition, fixed "Exit Focus" pill at `z-index: 30`, Cmd+Shift+F keyboard shortcut.

**Description**: "Focus" toggle in PR detail header dims/hides sidebar nav and header bar, expanding content to near-full-width. Ambient blobs shift to calmer state. Floating "Exit Focus" pill and breadcrumb remain for escape.

**Problem**: When deep in a complex PR, sidebar and header are visual noise. 224px sidebar wastes space the diff viewer needs.

**Approach**: `useFocusMode()` composable. `.focus-active` class on `.app-layout` setting `--sidebar-width: 0`, hiding header with transition. Fixed exit pill at `z-index: 30`. Keyboard shortcut `Cmd+Shift+F`.

**Effort**: Medium

---

## Improvement 12: Notification Centre Drawer

**Status**: ✅ Implemented
**Implementation notes**: `NotificationDrawer.vue` with `position: fixed; right: 0`, `useNotifications()` store capped at 100 entries, slide transition via `translateX`, bell icon with unread badge in header.

**Description**: Replace ephemeral bottom-right toasts with a slide-out notification drawer (320px, full height) from a bell icon in the header. Scrollable chronological list of all session notifications. Unread badge on bell.

**Problem**: Toasts are ephemeral -- if not looking, information is lost. After syncing 8 repos, user wants to review what changed. Persistent history reduces missed-event anxiety.

**Approach**: `NotificationDrawer.vue` with `position: fixed; right: 0`. `useNotifications()` store accumulating capped at 100. Slide transition via `translateX`. Existing toasts can still show briefly while pushing to drawer.

**Effort**: Medium

---

## Improvement 13: Brand Logo Signature Glow

**Status**: ✅ Implemented
**Implementation notes**: `.sidebar-brand` block in `AppSidebar.vue` with animated gradient text (`background-clip: text`, `-webkit-text-fill-color: transparent`, `background-position` animation on 10s loop), 14px version in `TitleBar.vue`.

**Description**: Fuse wordmark/glyph at top of collapsed sidebar and in title bar. Animated glow: teal-to-violet gradient text shifting hue on 10s loop. Expanded sidebar shows full "Fuse" wordmark with thin underline accent.

**Problem**: No visible brand mark in chrome. Title bar shows nothing identifying. Every premium tool (Linear, Raycast, Arc) has distinctive brand presence. Without it, Fuse feels generic.

**Approach**: `.sidebar-brand` block in `AppSidebar.vue` with SVG or styled text. `background: linear-gradient`, `background-clip: text`, `-webkit-text-fill-color: transparent`, animated `background-position`. 14px version in `TitleBar.vue`.

**Effort**: Small

---

## Recommended Implementation Order

### Phase 1 -- Quick visual wins:
1. Improvement 1 (Typography scale)
2. Improvement 5 (Risk glow borders)
3. Improvement 6 (Countup animation)
4. Improvement 9 (Code font + ligatures)
5. Improvement 13 (Brand logo)

### Phase 2 -- Layout and interaction:
6. Improvement 4 (Density toggle)
7. Improvement 7 (Resizable split panel)
8. Improvement 8 (Ambient blob response)
9. Improvement 11 (Focus mode)

### Phase 3 -- Theming and delight:
10. Improvement 3 (Accent colour picker)
11. Improvement 10 (Merge celebration)
12. Improvement 12 (Notification drawer)

### Phase 4 -- Major effort:
13. Improvement 2 (Light mode)

## Summary Table

| # | Improvement | Problem | Effort |
|---|-------------|---------|--------|
| 1 | Typographic Scale | Flat visual hierarchy | Small |
| 2 | Light Mode | Unusable in bright environments | Large |
| 3 | Accent Colour Picker | No personalisation | Medium |
| 4 | Density Toggle | Too spacious for power users | Medium |
| 5 | Risk Glow Borders | Risk not visible at glance | Small |
| 6 | Countup Animation | Numbers appear flat | Small |
| 7 | Resizable Split Panel | Fixed sidebar doesn't adapt | Medium |
| 8 | Ambient Blob Risk Response | Blobs are decorative only | Small |
| 9 | Code Font + Ligatures | Diff readability | Small |
| 10 | Merge Celebration | Emotionally flat merge | Medium |
| 11 | Focus Mode | Too much visual noise | Medium |
| 12 | Notification Drawer | Toasts are ephemeral | Medium |
| 13 | Brand Logo Glow | No brand presence | Small |
