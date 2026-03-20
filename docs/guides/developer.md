# Developer Guide

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          Fuse Desktop App                              │
│                                                                        │
│  ┌──────────────────────────────┐   IPC (invoke)   ┌────────────────┐  │
│  │     Vue 3 Frontend           │◄───────────────►│  Tauri v2       │  │
│  │                              │                  │  Rust Backend   │  │
│  │  ┌────────┐  ┌───────────┐  │  sync-completed  │                │  │
│  │  │ Pinia  │  │Composables│  │◄────── emit ─────│  ┌──────────┐  │  │
│  │  │ Stores │  │           │  │                  │  │ Commands │  │  │
│  │  └────────┘  └───────────┘  │                  │  └─────┬────┘  │  │
│  │                              │                  │        │       │  │
│  │  ┌────────┐  ┌───────────┐  │                  │  ┌─────▼────┐  │  │
│  │  │ Router │  │Components │  │                  │  │ SQLite   │  │  │
│  │  └────────┘  └───────────┘  │                  │  │ (WAL)    │  │  │
│  └──────────────────────────────┘                  │  └──────────┘  │  │
│                                                    │                │  │
│                                                    │  ┌──────────┐  │  │
│                                                    │  │ gh CLI   │  │  │
│                                                    │  └──────────┘  │  │
│                                                    │                │  │
│                                                    │  ┌──────────┐  │  │
│                                                    │  │ grove CLI│  │  │
│                                                    │  │(optional)│  │  │
│                                                    │  └──────────┘  │  │
│                                                    └────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Layer     | Technology                          | Version  |
|-----------|-------------------------------------|----------|
| Runtime   | Tauri                               | 2.x      |
| Backend   | Rust (edition 2021)                 | stable   |
| Database  | SQLite via `rusqlite` (bundled)      | 0.32     |
| Frontend  | Vue 3 (Composition API)             | 3.5+     |
| State     | Pinia                               | 3.x      |
| Routing   | vue-router                          | 4.x      |
| Build     | Vite                                | 6.x      |
| Icons     | lucide-vue-next                     | 0.577+   |
| Markdown  | marked + DOMPurify                  | 17.x / 3.x |
| CLI deps  | GitHub CLI (`gh`), `grove` (optional) | latest   |

### Directory Structure

```text
tauri-app/
├── src/                          # Vue frontend
│   ├── assets/
│   ├── components/               # Vue SFCs
│   │   ├── layout/               #   TitleBar, AppHeader, AppSidebar
│   │   └── skeletons/            #   Loading skeleton components
│   ├── composables/              # Reusable logic hooks
│   ├── router/                   # vue-router configuration
│   ├── stores/                   # Pinia stores
│   ├── styles/
│   │   └── tokens.css            # Design tokens
│   ├── types/
│   │   └── index.ts              # All TypeScript interfaces
│   ├── views/                    # Route-level views
│   ├── App.vue
│   └── main.ts
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/             # Tauri command handlers
│   │   │   ├── mod.rs            #   CommandError enum
│   │   │   ├── analytics.rs
│   │   │   ├── authors.rs
│   │   │   ├── batch.rs
│   │   │   ├── bookmarks.rs      #   File-level annotation bookmarks
│   │   │   ├── checklist.rs      #   Checklist state persistence
│   │   │   ├── checks.rs         #   CI status checks
│   │   │   ├── comments.rs
│   │   │   ├── commits.rs        #   PR commit history
│   │   │   ├── conflicts.rs      #   Merge conflict detection
│   │   │   ├── dependencies.rs   #   Cross-PR dependency graph
│   │   │   ├── deployments.rs    #   Deployment status tracking
│   │   │   ├── diff.rs
│   │   │   ├── digest.rs         #   Review digest generation
│   │   │   ├── editor.rs
│   │   │   ├── groups.rs
│   │   │   ├── grove.rs
│   │   │   ├── handoffs.rs       #   Review handoff notes
│   │   │   ├── issues.rs         #   Linked issue retrieval
│   │   │   ├── label_rules.rs    #   Label automation rules
│   │   │   ├── priority_queue.rs #   Smart priority scoring
│   │   │   ├── pull_requests.rs
│   │   │   ├── repositories.rs
│   │   │   ├── reviews.rs        #   AI review management
│   │   │   ├── sessions.rs       #   Review session tracking
│   │   │   ├── settings.rs
│   │   │   ├── stale.rs
│   │   │   ├── stats.rs
│   │   │   ├── sync.rs
│   │   │   ├── templates.rs
│   │   │   └── workload.rs       #   Reviewer workload analytics
│   │   ├── db/
│   │   │   ├── mod.rs            #   DbState, initialise()
│   │   │   └── migrations.rs     #   Schema DDL + migration logic
│   │   ├── github/
│   │   │   └── mod.rs            #   gh CLI wrapper functions
│   │   ├── models/
│   │   │   └── mod.rs            #   Shared data models
│   │   ├── polling.rs            #   Background sync loop
│   │   └── lib.rs                #   Tauri registration and entry
│   ├── capabilities/
│   │   └── default.json
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

---

## Backend Architecture

### Database Schema

#### Core Tables

**`repositories`**, **`pull_requests`**, **`pr_reviews`**, **`review_rules`**, **`sync_log`**, **`app_settings`**, **`repo_groups`**, **`repo_group_members`**, **`review_templates`** as documented in initial schema.

#### Feature-Specific Tables (v2)

**`pr_ai_reviews`**
Stores AI-generated analysis per PR, linked to specific branch revisions.
`id`, `pr_id`, `review_text`, `worktree_branch`, `created_at`.

**`checklist_state`**
Persists per-PR checklist tick states as JSON.
`pr_id`, `state_json`, `updated_at`.

**`label_automation_rules`**
Defines label-triggered actions (e.g., set priority, assign group).
`id`, `label_pattern`, `action_type`, `action_config`, `enabled`, `created_at`.

**`pr_requested_reviewers`**
Junction table for tracking reviewer assignments and workload.
`pr_id`, `reviewer`, `assigned_at`.

**`review_bookmarks`**
File-level annotation bookmarks with line ranges and notes.
`id`, `pr_id`, `file_path`, `line_start`, `line_end`, `note`, `created_at`.

**`review_handoffs`**
Structured notes for passing a review to another team member.
`id`, `pr_id`, `reviewer_name`, `files_checked`, `concerns`, `remaining_work`, `created_at`.

**`pr_dependencies`**
Graph relationships between PRs (ancestry or body references).
`id`, `pr_id`, `depends_on_pr_id`, `dependency_type`.

**`review_sessions`**
Focused review session tracking (Pomodoro style).
`id`, `pr_id`, `started_at`, `files_reviewed`, `session_notes`, `status`.

---

### Tauri Commands

(Partial list of key commands. See `lib.rs` for the full registry.)

| Module | Command | Description |
|---|---|---|
| **Repositories** | `add_repository`, `remove_repository`, `list_repositories`, `update_repository_branch` | Core repo management. |
| **Pull Requests** | `get_pull_requests`, `get_pull_request`, `approve_pull_request`, `merge_pull_request`, `update_review_status`, `record_review_time` | Core PR actions. |
| **Sync** | `sync_pull_requests` | Triggers `gh` sync. |
| **Checklist** | `get_checklist_state`, `save_checklist_state` | Persistence for PR checklists. |
| **Analytics** | `get_age_distribution`, `get_review_velocity`, `get_author_stats`, `get_daily_pr_counts` | Dashboard data. |
| **Batch** | `batch_approve`, `batch_merge` | Bulk actions. |
| **AI Reviews** | `trigger_worktree_review`, `list_pr_ai_reviews` | Integration with Claude CLI. |
| **Priority Queue** | `get_priority_queue` | Ranked PR list. |
| **Sessions** | `create_review_session`, `get_review_session`, `update_session_notes`, `list_review_sessions` | Focus mode logic. |
| **Workload** | `get_reviewer_workload` | Multi-user load balancing data. |
| **Bookmarks** | `create_bookmark`, `list_bookmarks`, `update_bookmark`, `delete_bookmark` | File annotations. |
| **Handoffs** | `create_handoff`, `list_handoffs`, `export_handoff_to_github` | Collaboration tools. |
| **Dependencies** | `compute_dependencies`, `get_pr_dependencies` | Dependency graph logic. |
| **Digest** | `get_review_digest` | Periodic summary generation. |
| **Polling** | `start_polling`, `stop_polling`, `update_poll_interval` | Backend poll loop control. |

---

## Frontend Architecture

### Pinia Stores

- `pullRequests`: PR data, sync, metrics.
- `repositories`: Tracking and settings.
- `settings`: UI and app config.
- `filters`: Persisted list filtering/sorting.
- `groups`: Repo organisation.
- `toast`: Notification overlay queue.
- `notificationCentre`: Persistent history of app notifications.

### Composables (Full List)

The application utilizes 38 specialized composables for modular logic:

| Composable | Responsibility |
|---|---|
| `useAiReview` | Direct Claude CLI integration. |
| `useAiReviewComparison` | Side-by-side analysis of multiple AI reviews. |
| `useAutoSync` | Foreground event listener for sync updates. |
| `useBookmarks` | Management of file-level review annotations. |
| `useCache` | TTL-based caching for expensive network calls (e.g., CI). |
| `useChecklist` | Persistent checklist state logic. |
| `useCommandPalette` | `Cmd+K` fuzzy search and navigation. |
| `useComments` | Fetching and threading PR discussions. |
| `useCommitHistory` | PR-level git history. |
| `useConfetti` | Visual celebration on PR merge. |
| `useConfirm` | Generic promise-based confirmation dialog. |
| `useConflictDetection` | Early detection of merge conflicts. |
| `useCountUp` | Animated numeric stats. |
| `useDensity` | Compact/Comfortable layout state. |
| `useDependencyGraph` | SVG graph rendering logic. |
| `useDeploymentStatus` | CI deployment environment tracking. |
| `useDiff` | Unified diff parsing and file tree management. |
| `useFocusMode` | Distraction-free UI state. |
| `useFocusTrap` | Accessibility/Modal focus management. |
| `useGrove` | Git worktree management via Grove CLI. |
| `useHandoffNotes` | Collaborative handoff composition. |
| `useHoverPreview` | Popover summary on row hover. |
| `useKeyboardShortcuts` | Global shortcut registry. |
| `useLabelRules` | Automation based on PR labels. |
| `useLinkedIssues` | GitHub issue linking. |
| `useMarkdown` | Sanitised markdown rendering. |
| `useNotifications` | OS-level notification bridge. |
| `useOnboarding` | First-run wizard state. |
| `usePriorityQueue` | Smart ranking algorithm. |
| `usePromptBuilder` | Context-aware AI prompt generation. |
| `useRecentPrs` | Sidebar history tracking. |
| `useResponseParser` | AI text-to-JSON parsing. |
| `useReviewDigest` | Activity summary generation. |
| `useReviewSession` | 3-panel focus interface state. |
| `useReviewTimer` | Time tracking with visibility API pause. |
| `useRiskScore` | Weighted risk calculation. |
| `useSidebarState` | Collapsible navigation state. |
| `useTheme` | Dark/Light/System and Accent colour management. |
| `useWorkload` | Team workload balancing metrics. |

---

### Component Catalogue

#### Visualisation & Dashboard
`AgeHeatmap`, `VelocityChart`, `Sparkline`, `RiskGauge`, `RiskBadge`, `SizeBar`, `WorkloadDashboard`, `DependencyGraph`.

### Dashboard View Composition

The dashboard route is implemented in `tauri-app/src/views/Dashboard.vue` and acts as a thin orchestration layer over multiple analytics panels.

- Top row: `StatsCard` and `ReviewProgress` surface aggregate PR counts and completion.
- Review queue: `PriorityQueue` shows the current ranked worklist and supports local refresh without a full GitHub sync.
- Needs attention: `PRCard` renders the top five highest-risk open PRs.
- Dependency map: `DependencyGraph` visualises detected cross-PR relationships.
- Reviewer load: `WorkloadDashboard` summarises reviewer assignment pressure.
- Historical analytics: `AgeHeatmap`, `VelocityChart`, and `StalePrSection` cover queue age, throughput, and stale work.

The dashboard now uses the full available content width rather than a fixed 1100 px container, so child panels should be built to expand fluidly across large desktop windows.

#### PR Detail Components
`DiffViewer`, `DiffFileTree`, `CommentThread`, `CommitTimeline`, `CiChecksPanel`, `CiStatusBadge`, `ConflictBadge`, `DeploymentStatus`, `BookmarksList`, `HandoffComposer`, `ReviewPipeline`, `ReviewProgress`, `ReviewStatus`, `WorktreePanel`.

#### List & Navigation
`PRTable`, `PRCard`, `PRHoverPreview`, `PriorityQueue`, `BatchActionBar`, `GroupFilter`, `GroupManager`, `Breadcrumb`, `TabBar`, `CommandPalette`.

#### Setup & Settings
`OnboardingWizard`, `RepositoryCard`, `ReviewRulesEditor`, `TemplateManager`, `TemplateSelector`, `LabelRulesManager`.

#### Core UI & Feedback
`TitleBar`, `AppHeader`, `AppSidebar`, `ToastContainer`, `ConfirmDialog`, `ShortcutOverlay`, `EmptyState`, `ContentLoader`, `ResizableSplit`, `MarkdownRenderer`, `AuthorAvatar`, `AuthorStatsTable`.

---

## Tauri Configuration

### Window & Shell
Fuse uses a **frameless window** (`decorations: false`) with a custom title bar. It requires the following Tauri plugins:
- `tauri-plugin-opener` (for browser links)
- `tauri-plugin-shell` (for `gh`, `grove`, and `claude` commands)
- `tauri-plugin-notification` (for OS-level alerts)

---

## Development Workflow

### Adding a Feature
1. **Schema**: Define tables in `db/migrations.rs`.
2. **Backend**: Implement logic in `commands/*.rs` and register in `lib.rs`.
3. **Types**: Add TypeScript interfaces to `src/types/index.ts`.
4. **Logic**: Create/update a Pinia store or Composable.
5. **UI**: Build the SFCs in `components/` and `views/`.
6. **Documentation**: Ensure the User Guide reflects the new capability.
