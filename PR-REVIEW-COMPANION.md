# Fuse — PR Review Companion

**A Tauri v2 desktop application for intelligent pull request monitoring, triage, and AI-assisted code review.**

Fuse is a desktop-native tool built for developers who review code across multiple repositories. It syncs pull requests via the GitHub CLI, scores them by risk, and provides AI-powered review assistance through Claude CLI — all within a glassmorphic, keyboard-driven interface.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Features](#2-features)
3. [Architecture](#3-architecture)
4. [Getting Started](#4-getting-started)
5. [Keyboard Shortcuts](#5-keyboard-shortcuts)
6. [Configuration](#6-configuration)
7. [Routes and Views](#7-routes-and-views)
8. [External Tool Integration](#8-external-tool-integration)

---

## 1. Overview

Fuse sits between you and GitHub. Rather than context-switching to the browser to check on pull requests, Fuse maintains a local SQLite database of PRs synced via the `gh` CLI, assigns risk scores, tracks review status, and surfaces the information you need before you open a single diff.

The application is built with:

| Layer | Technology |
|---|---|
| Desktop shell | Tauri v2 (Rust, `macos-private-api` feature) |
| Frontend | Vue 3 (Composition API), Vue Router, Pinia |
| Styling | CSS Custom Properties design system with glassmorphic panels |
| Backend logic | Rust — `rusqlite`, `serde`, `chrono`, `tokio`, `thiserror`, `regex` |
| Local storage | SQLite (15+ tables, managed via idempotent migrations) |
| Icons | Lucide (via `lucide-vue-next`) |
| Markdown | `marked` + `dompurify` |
| GitHub integration | GitHub CLI (`gh`) via `tauri-plugin-shell` |
| Git worktrees | Grove CLI |
| AI reviews | Claude CLI |

---

## 2. Features

Fuse implements 95 features and improvements across six development plans. Below is a summary organised by category.

### 2.1 Core PR Management

- **GitHub PR sync** — Fetches open pull requests from tracked repositories using `gh pr list`. Stores all metadata (title, author, branch, additions, deletions, changed files, labels, review decision, draft status, mergeability) in SQLite.
- **Batch operations** — Select multiple PRs and perform bulk actions via the `BatchActionBar` component.
- **Auto-sync polling** — Configurable background polling interval keeps the local database current without manual refreshes.
- **Stale PR detection** — PRs older than a configurable threshold (default: 14 days) are flagged and surfaced in a dedicated `StalePrSection`.
- **Repository groups** — Organise tracked repositories into named, colour-coded groups for filtering.
- **Review templates** — Pre-defined review note templates (LGTM, Minor Comments, Needs Discussion) with a template manager for creating custom ones.

### 2.2 Risk Scoring and Analytics

- **Risk scoring** — Each PR receives a computed risk score displayed via `RiskBadge` and `RiskGauge` components. Risk level drives glow borders on PR cards and ambient blob animations.
- **Analytics dashboard** — The Dashboard view shows aggregate statistics with `StatsCard` components, `Sparkline` charts, `VelocityChart`, and `AgeHeatmap` visualisations.
- **Author statistics** — Dedicated Authors view with `AuthorStatsTable` for per-contributor metrics.
- **Number countup animations** — Stats cards animate values on load.

### 2.3 AI-Powered Reviews

- **Claude CLI integration** — Triggers AI reviews via the Claude CLI against a Grove-managed worktree of the PR branch.
- **AI review panel** — Displays structured AI analysis within the app (`AiReviewPanel`).
- **AI review comparison** — Diffs between multiple AI reviews of the same PR to track how analysis changes across revisions (`AiReviewComparison`).
- **Prompt builder** — `AiPromptBuilder` component and `usePromptBuilder` composable for constructing structured prompts with context.

### 2.4 Code Review Workflow

- **Tabbed PR detail layout** — PR detail view organised into tabs (Overview, Code, Discussion, AI) via `TabBar`.
- **Diff viewer** — In-app diff rendering with `DiffViewer` and `DiffFileTree` components.
- **PR comments** — View and interact with comment threads via `CommentThread`.
- **Commit history timeline** — `CommitTimeline` component shows the commit-by-commit history of a PR.
- **CI/CD status checks** — `CiChecksPanel` and `CiStatusBadge` display pipeline status with a caching layer (TTL-based).
- **File-level annotation bookmarks** — Bookmark specific files and line ranges with notes during review (`BookmarksList`).
- **Review session mode** — A focused 3-panel review interface at `/review-session/:prId` with a Pomodoro timer (`useReviewTimer`), file tracking, and session notes.
- **Persistent review checklist state** — Per-PR checklist ticks survive navigation and restarts, stored in the `checklist_state` table.
- **Review handoff notes** — Compose and export handoff notes to GitHub when passing a review to another team member (`HandoffComposer`).

### 2.5 Advanced Features (v2)

- **Linked issue context panel** — Surfaces linked GitHub issues alongside PR details (`useLinkedIssues`).
- **Cross-PR dependency graph** — SVG-rendered visualisation of PR-to-PR dependencies detected from body references and branch ancestry (`DependencyGraph`).
- **Smart review queue** — Priority-scored queue that ranks PRs by urgency, size, age, and risk (`PriorityQueue`, `usePriorityQueue`).
- **Deployment status tracker** — Tracks deployment state for PRs (`DeploymentStatus`, `useDeploymentStatus`).
- **Reviewer workload balancer** — Dashboard showing review load distribution across team members (`WorkloadDashboard`, `useWorkload`).
- **Label-based automation rules** — Define rules that trigger actions (set priority, add checklist, assign group) based on PR labels (`LabelRulesManager`).
- **Recurring review digest** — Aggregated digest view at `/digest` summarising PR activity over a period (`DigestView`, `useReviewDigest`).
- **Merge conflict early detection** — Proactive detection of merge conflicts before they block merging (`ConflictBadge`, `useConflictDetection`).

### 2.6 User Interface

- **Glassmorphic design system** — CSS custom property token system (`tokens.css`) with backdrop-blur panels, ambient background blobs that respond to risk state, and smooth transitions (`transitions.css`).
- **Custom frameless window** — Rounded-corner frameless window with a custom `TitleBar` component.
- **Light and dark themes** — System preference detection with manual override (`useTheme`).
- **Accent colour picker** — Seven preset accent colours.
- **Density toggle** — Comfortable and Compact layout modes (`useDensity`).
- **Typographic scale system** — Five-level type scale defined in CSS custom properties.
- **Code font with ligatures toggle** — Optional ligature support for code display.
- **Skeleton loading states** — Dedicated skeleton components for all data-loading states.
- **Toast notifications** — In-app toast system via the `toast` Pinia store and `ToastContainer`.
- **Empty states** — Contextual empty state illustrations and messaging (`EmptyState`).
- **Brand logo signature glow** — Animated glow effect on the application brand mark.
- **Merge celebration confetti** — Confetti animation on PR merge (`useConfetti`).

### 2.7 User Experience

- **Command palette** — `Cmd+K` to open a fuzzy-searchable command palette (`CommandPalette`, `useCommandPalette`).
- **Keyboard-navigable PR list** — `j`/`k`/`Enter`/`x` for Vim-style list navigation.
- **Search and filter** — Full-text search with filter persistence via the `filters` Pinia store.
- **Sort controls** — Wired sort controls with store persistence.
- **Hover preview cards** — `PRHoverPreview` shows a summary popover on PR row hover.
- **Drag-to-reorder** — Reorderable lists for groups, templates, and rules.
- **Resizable split panel** — `ResizableSplit` component on the PR detail view.
- **Recently visited PRs** — Sidebar section tracking recently viewed PRs (`useRecentPrs`).
- **Inline quick-status dropdown** — Change PR review status inline from the list view (`QuickStatusPopover`).
- **Contextual focus mode** — `Cmd+Shift+F` to toggle a distraction-free focus mode (`useFocusMode`).
- **Notification centre drawer** — Slide-out drawer for reviewing all notifications (`NotificationDrawer`, `notificationCentre` store).
- **Onboarding wizard** — First-run setup flow (`OnboardingWizard`, `useOnboarding`).
- **Reduced motion and high contrast support** — Respects `prefers-reduced-motion` and `prefers-contrast` media queries.
- **Focus trap and ARIA** — All modals implement focus trapping and ARIA attributes (`useFocusTrap`).

---

## 3. Architecture

### 3.1 Rust Backend

The Tauri backend is organised into command modules under `src-tauri/src/commands/`. Each module exposes `#[tauri::command]` functions that the frontend invokes over IPC.

```text
src-tauri/src/
├── commands/
│   ├── analytics.rs        — Dashboard statistics and aggregations
│   ├── authors.rs           — Per-author metrics
│   ├── batch.rs             — Bulk PR operations
│   ├── bookmarks.rs         — File-level annotation bookmarks
│   ├── checklist.rs         — Review checklist persistence
│   ├── checks.rs            — CI/CD status retrieval
│   ├── comments.rs          — PR comment fetching
│   ├── commits.rs           — Commit history
│   ├── conflicts.rs         — Merge conflict detection
│   ├── dependencies.rs      — Cross-PR dependency graph
│   ├── deployments.rs       — Deployment status tracking
│   ├── diff.rs              — Diff fetching
│   ├── digest.rs            — Review digest generation
│   ├── editor.rs            — External editor launch
│   ├── groups.rs            — Repository group management
│   ├── grove.rs             — Grove CLI worktree integration
│   ├── handoffs.rs          — Review handoff notes
│   ├── issues.rs            — Linked issue retrieval
│   ├── label_rules.rs       — Label automation rules
│   ├── priority_queue.rs    — Smart priority scoring
│   ├── pull_requests.rs     — Core PR CRUD
│   ├── repositories.rs      — Repository management
│   ├── reviews.rs           — Review status management
│   ├── sessions.rs          — Review session tracking
│   ├── settings.rs          — App settings
│   ├── stale.rs             — Stale PR detection
│   ├── stats.rs             — Statistical queries
│   ├── sync.rs              — GitHub sync via gh CLI
│   ├── templates.rs         — Review templates
│   └── workload.rs          — Reviewer workload balancing
├── db/
│   ├── migrations.rs        — SQLite schema and migrations
│   └── mod.rs               — Database connection management
├── github/                  — GitHub CLI wrapper utilities
├── models/
│   └── mod.rs               — Shared Rust data models
├── polling.rs               — Background auto-sync polling loop
├── lib.rs                   — Tauri plugin and command registration
└── main.rs                  — Application entry point
```

### 3.2 SQLite Database

The database schema is defined in `src-tauri/src/db/migrations.rs` and applied idempotently on startup. Key tables:

| Table | Purpose |
|---|---|
| `repositories` | Tracked GitHub repositories |
| `pull_requests` | Synced PR metadata (state, branches, additions/deletions, labels, mergeability) |
| `pr_reviews` | Local review status and notes per PR |
| `pr_ai_reviews` | AI-generated review text with branch and timestamp |
| `review_rules` | Per-repository plain-English review rules |
| `review_templates` | Reusable review note templates |
| `repo_groups` / `repo_group_members` | Repository grouping |
| `app_settings` | Key-value application settings |
| `sync_log` | Sync history and error tracking |
| `checklist_state` | Per-PR review checklist tick persistence |
| `label_automation_rules` | Label-triggered automation rules |
| `pr_requested_reviewers` | Reviewer assignments for workload tracking |
| `review_bookmarks` | File-level annotation bookmarks |
| `review_handoffs` | Review handoff notes |
| `pr_dependencies` | Cross-PR dependency relationships |
| `review_sessions` | Focused review session tracking |

### 3.3 Vue Frontend

The frontend follows Vue 3 Composition API conventions throughout.

**Views** (8 routes):

| Route | View | Description |
|---|---|---|
| `/dashboard` | `Dashboard.vue` | Overview with stats, sparklines, and velocity charts |
| `/prs` | `PullRequests.vue` | Filterable, sortable PR list with keyboard navigation |
| `/prs/:id` | `PullRequestDetail.vue` | Tabbed detail view (Overview, Code, Discussion, AI) |
| `/review-session/:prId` | `ReviewSession.vue` | Focused 3-panel review with Pomodoro timer |
| `/repositories` | `Repositories.vue` | Repository management and group organisation |
| `/authors` | `Authors.vue` | Per-author statistics |
| `/digest` | `DigestView.vue` | Recurring review digest |
| `/settings` | `Settings.vue` | Application configuration |

**Stores** (7 Pinia stores):

| Store | Responsibility |
|---|---|
| `pullRequests` | PR data, sync actions, selection state |
| `repositories` | Repository list and management |
| `settings` | Application settings with persistence |
| `filters` | Search query, sort order, filter criteria |
| `groups` | Repository group management |
| `toast` | Toast notification queue |
| `notificationCentre` | Notification history and drawer state |

**Composables** (38):

The `composables/` directory contains reusable logic units covering everything from `useAiReview` and `useRiskScore` to `useConfetti` and `useCountUp`. Each composable encapsulates a single concern and is consumed by the relevant components.

**Components** (55+):

Components are organised flat under `src/components/` with a `layout/` subdirectory for `AppHeader`, `AppSidebar`, and `TitleBar`. A `skeletons/` subdirectory contains loading-state components.

### 3.4 Design System

The visual design is driven by CSS custom properties defined in `src/styles/tokens.css`:

- **Colour tokens** — Background, surface, border, and text colours for both light and dark themes. Risk-level colours (green, amber, orange, red) are tokenised.
- **Accent colours** — Seven selectable presets applied via a root-level CSS class.
- **Typography scale** — Five levels of heading/body sizes.
- **Spacing and radius** — Consistent spacing scale and border-radius tokens.
- **Density modes** — Comfortable and Compact density applied via token overrides.
- **Transitions** — Shared transition definitions in `transitions.css`.
- **Glassmorphism** — Backdrop-blur surfaces with semi-transparent backgrounds.
- **Ambient blobs** — Background gradient blobs that shift colour based on overall risk state.
- **Reduced motion** — All animations respect `prefers-reduced-motion`.
- **High contrast** — Adjusted token values for `prefers-contrast: more`.

---

## 4. Getting Started

### Prerequisites

- **macOS** (primary target platform)
- **Rust** (latest stable) and Cargo
- **Node.js** (v18+) and npm
- **Tauri CLI v2** — `cargo install tauri-cli --version "^2"`
- **GitHub CLI** (`gh`) — authenticated with access to your repositories
- **Grove CLI** (optional) — for worktree-based AI reviews
- **Claude CLI** (optional) — for AI-powered review analysis

### Installation

```sh
cd tauri-app

# Install frontend dependencies
npm install

# Run in development mode (launches both Vite dev server and Tauri window)
npm run tauri dev
```

A convenience script is also available:

```sh
./dev.sh
```

### Building for Production

```sh
cd tauri-app
npm run tauri build
```

This produces a native macOS application bundle in `src-tauri/target/release/bundle/`.

### First Run

1. Launch Fuse. The onboarding wizard will guide you through initial setup.
2. Add repositories by navigating to the Repositories view and entering `owner/name` pairs.
3. Trigger a sync (press `r` or use the command palette) to fetch open PRs.
4. Configure your polling interval, stale threshold, theme, accent colour, and density in Settings.

---

## 5. Keyboard Shortcuts

### Global

| Shortcut | Action |
|---|---|
| `Cmd+K` | Open command palette |
| `Cmd+Shift+F` | Toggle focus mode |
| `1` | Navigate to Dashboard |
| `2` | Navigate to Pull Requests |
| `3` | Navigate to Repositories |
| `4` | Navigate to Settings |
| `r` | Sync all repositories |
| `/` | Focus search input |
| `?` | Toggle keyboard shortcut overlay |
| `Escape` | Close overlay / blur active input |

### PR List (on the Pull Requests view)

| Shortcut | Action |
|---|---|
| `j` | Move focus down one row |
| `k` | Move focus up one row |
| `Enter` | Open the focused PR detail |
| `x` | Toggle selection on the focused PR |

---

## 6. Configuration

Application settings are stored in the `app_settings` SQLite table and managed through the Settings view.

| Setting | Default | Description |
|---|---|---|
| `poll_interval_seconds` | `300` | Background sync interval in seconds |
| `theme` | `dark` | Colour theme (`dark`, `light`, `system`) |
| `editor_command` | `code` | External editor command for opening files |
| `stale_threshold_days` | `14` | Number of days after which a PR is considered stale |

Additional UI preferences (accent colour, density mode, code font ligatures, reduced motion) are managed via the `useTheme` and `useDensity` composables and persisted in local storage.

---

## 7. Routes and Views

| Path | Name | Description |
|---|---|---|
| `/` | — | Redirects to `/dashboard` |
| `/dashboard` | `dashboard` | Aggregate stats, sparklines, velocity charts, age heatmap |
| `/prs` | `pull-requests` | Searchable, filterable, keyboard-navigable PR list |
| `/prs/:id` | `pr-detail` | Tabbed PR detail (Overview, Code, Discussion, AI) |
| `/review-session/:prId` | `review-session` | Focused 3-panel review with Pomodoro timer |
| `/repositories` | `repositories` | Repository management, groups, default branch editing |
| `/authors` | `authors` | Per-author review statistics |
| `/digest` | `digest` | Recurring review activity digest |
| `/settings` | `settings` | Application configuration |

---

## 8. External Tool Integration

### GitHub CLI (`gh`)

Fuse delegates all GitHub communication to the `gh` CLI, invoked via `tauri-plugin-shell`. This avoids managing OAuth tokens or API keys directly — authentication is handled by `gh auth`.

Typical commands used:
- `gh pr list` — Fetch open PRs for a repository
- `gh pr view` — Retrieve detailed PR metadata, diff, comments, and checks
- `gh pr diff` — Fetch the raw diff for a PR

### Grove CLI (Worktrees)

Grove manages Git worktrees, allowing Fuse to check out a PR branch into an isolated worktree without disturbing your working copy. This is used by the AI review workflow — Claude CLI runs against the worktree directory.

### Claude CLI (AI Reviews)

When triggered, Fuse invokes the Claude CLI against a worktree of the PR branch. The response is parsed and stored in the `pr_ai_reviews` table. Multiple reviews of the same PR are retained, enabling the AI review comparison feature to diff analysis across revisions.

---

*Fuse — PR Review Companion. Built with Tauri v2, Vue 3, and Rust.*

*Document updated 9 March 2026.*
