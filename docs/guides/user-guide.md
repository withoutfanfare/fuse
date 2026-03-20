# PR Review Companion — User Guide

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Dashboard](#dashboard)
4. [Pull Requests](#pull-requests)
5. [Pull Request Detail](#pull-request-detail)
6. [Review Session Mode](#review-session-mode)
7. [Priority Queue](#priority-queue)
8. [Review Digest](#review-digest)
9. [Workload Dashboard](#workload-dashboard)
10. [Repositories](#repositories)
11. [Repository Groups](#repository-groups)
12. [Label Automation Rules](#label-automation-rules)
13. [Authors](#authors)
14. [Settings](#settings)
15. [Auto-Sync & Notifications](#auto-sync--notifications)
16. [Keyboard Shortcuts & Focus Mode](#keyboard-shortcuts--focus-mode)
17. [Command Palette](#command-palette)
18. [Risk Score Explained](#risk-score-explained)
19. [AI-Assisted Review](#ai-assisted-review)
20. [Tips & Workflows](#tips--workflows)

---

## Overview

PR Review Companion is a desktop application that brings all your GitHub pull requests into a single, keyboard-driven dashboard. It syncs PR data from every repository you track, scores each PR by risk, and gives you the tools to triage, review, approve, and merge without leaving the app. It also integrates AI-assisted code review through both a clipboard-based prompt workflow and direct integration with Claude CLI.

The app is built for developers and team leads who review pull requests regularly and want a faster, more structured way to stay on top of their review queue.

**Prerequisites:**

- GitHub CLI (`gh`) must be installed and authenticated on your machine. The app uses `gh` under the hood to fetch PR data, approve, merge, and manage worktrees.
- Grove CLI (optional) for enhanced git worktree management.
- Claude CLI (optional) for automated AI reviews.

## Getting Started

### First launch and onboarding wizard

When you open the app for the first time, an onboarding wizard walks you through three steps:

1. **Welcome** — a brief introduction. You can skip onboarding at any time.
2. **Add a Repository** — enter the owner and repository name (e.g. `facebook` / `react`) plus an optional default branch (defaults to `main`). You can add multiple repositories before continuing.
3. **Sync Pull Requests** — press **Sync Now** to fetch open PRs from all your tracked repositories. The wizard shows how many PRs were found once the sync finishes.

Press **Done** to close the wizard and land on the Dashboard.

### Understanding the interface layout

The app has three main regions:

- **Title bar** — a draggable bar at the top showing "PR Review Companion". On macOS the native traffic lights are used. On other platforms it includes standard controls.
- **Sidebar** — on the left, with links to Dashboard, Pull Requests, Repositories, Authors, and Settings. The Pull Requests link shows a badge with the number of PRs pending review. You can collapse the sidebar using the chevron button at the bottom. The sidebar also shows **Recent PRs** for quick access.
- **Header + content area** — the header shows the current page title, a subtitle, the sync status ("Synced 3m ago"), an auto-sync indicator, and a **Sync** button. The content area below displays the active page.

## Dashboard

The Dashboard gives you a bird's-eye view of your review queue.

### Stats overview

Four cards across the top show:

- **Open PRs** — total number of open pull requests across all tracked repositories.
- **Pending Review** — PRs that have not yet been reviewed.
- **In Progress** — PRs you have marked as "In Progress".
- **Approved** — PRs you have approved.

### Review progress donut chart

A small donut chart beneath the stats shows what proportion of open PRs have been reviewed or approved versus the total.

### Review Queue

The Review Queue panel ranks the current open pull requests by priority so you can decide what to look at next. Use the **Refresh** button in the panel header to recompute the queue without running a full repository sync.

### Needs Attention

This section lists the top five highest-risk open PRs, sorted by risk score (highest first). Each PR is shown as a card you can click to open its detail view. If there are no open PRs, you will see an "All caught up" message instead.

### PR Dependency Graph

The dependency graph highlights cross-PR relationships detected from branch ancestry and explicit references. Use it to spot review order dependencies before you merge or approve a stack.

### Reviewer Workload

This panel summarises reviewer assignment load across the currently synced repositories. It is useful for spotting overloaded reviewers and balancing review requests more evenly.

### PR Age Distribution heatmap

A heatmap showing how many PRs fall into different age buckets, so you can quickly spot whether PRs are piling up.

### Review Velocity chart

A chart tracking how quickly PRs are being reviewed over time, helping you spot trends in your team's review throughput.

### Stale PR Alerts

At the bottom of the Dashboard, the Stale Pull Requests section lists PRs that have not been updated for a long time (controlled by the stale threshold in Settings). Each stale card shows the PR number, how many days it has been stale, the title, author, and branch. You can click a stale PR to open its detail, or press **Close** to close it on GitHub (with a confirmation dialogue).

### Dashboard refresh behaviour

- The main **Sync** button in the app header refreshes repository data from GitHub.
- Panel-level **Refresh** buttons rerun that panel's local calculation only, which is faster when you want to update queue, dependency, or workload data without a full sync.

## Pull Requests

### Viewing the PR list

The Pull Requests page shows a table of all PRs across your tracked repositories. Each row displays the PR number, title, author, risk gauge, size bar, state, and other metadata. Click any row to open its detail view.

### Hover Preview

Hovering over a PR row in the table (or a PR card elsewhere) shows a **Hover Preview** popover with the PR description, recent activity, and quick stats without needing to navigate away.

### Filtering and Searching

- **Repository** — restrict the list to a single repository.
- **Group** — select a group to show only PRs from repositories in that group.
- **State** — filter by **Open**, **Merged**, **Closed**, or **All**.
- **Search** — searches by title, author, branch name, and PR number (e.g. `#42`). Use `/` to focus.

### Quick Status Popover

You can change a PR's review status (e.g., mark as "In Progress") directly from the list view by clicking the status badge, which opens a **Quick Status Popover**.

### Understanding the risk gauge and size bar

- **Risk Gauge** — a radial score from 1 to 10.
- **Size Bar** — a horizontal bar showing additions (green) and deletions (red).

### Column Sorting

Click column headers to sort by PR number, title, author, risk, size, state, or update time.

### Batch operations

Tick the checkbox on multiple PRs to select them. The **Batch Action Bar** slides up from the bottom:

- **Approve All** — submits an approval review for every selected PR.
- **Merge All** — squash-merges every selected PR.

## Pull Request Detail

Click any PR to open its full detail view, organised into tabs:

### Overview Tab

Shows PR metadata, description (rendered as Markdown), branch info, labels, and timeline. 
- **Conflict Badge** — appears prominently if a merge conflict is detected.
- **Linked Issues** — a panel showing issues linked to this PR on GitHub.
- **CI/CD Checks Panel** — listing every CI check run with its status.

### Code Tab (Diff Viewer)

An in-app syntax-highlighted diff viewer with a collapsible file tree.
- **File Bookmarks** — click the bookmark icon next to any file or line range to save a **Review Bookmark**. These are persisted locally and can be managed in the **Review Bookmarks** list.
- **Review Worktree** — use the panel in the right sidebar to create a git worktree for the PR via Grove. Once created, you can open the code in your configured external editor.

### Discussion Tab

Shows the full conversation thread, including comments and reviews.
- **Handoff Composer** — use this to write structured handoff notes when passing a review to another team member. These can be exported back to GitHub as a comment.

### AI Tab

Interface for AI-assisted review. 
- **AI Prompt Builder** — constructs a detailed prompt including diff context.
- **AI Review Comparison** — if you have multiple AI reviews for a PR, you can compare them side-by-side to see how the analysis has evolved.

### Sidebar Components (Detail View)

- **Review Status and Pipeline** — track and update the current review stage.
- **Review Checklist** — per-repository rules that you can tick off as you review.
- **Dependency Graph** — shows a small visualisation of how this PR relates to others (e.g., branch ancestry or body references).

## Review Session Mode

For deep, focused work, enter **Review Session Mode** from any PR detail page. This provides a distraction-free, 3-panel interface:
1. **File Tree / Checklist** — on the left.
2. **Diff Viewer / Editor** — in the centre.
3. **Notes / AI Analysis** — on the right.

A **Pomodoro Timer** helps you track your active review time, which is recorded in the local database to help improve workload analytics.

## Priority Queue

The **Priority Queue** view (accessible from the sidebar) automatically ranks all open PRs based on a weighted formula of:
- **Risk Score** (highest weight)
- **Age** (urgency)
- **Review Status** (getting it over the finish line)
- **Size** (quick wins vs deep reviews)

Use this when you have a large backlog and need the app to tell you where to start.

## Review Digest

The **Digest View** provides a recurring summary of review activity across all your repositories. It aggregates:
- PRs merged in the last period.
- New PRs that need attention.
- Stale PRs that should be closed.
- Team contribution highlights.

## Workload Dashboard

For team leads, the **Workload Dashboard** visualises how reviews are distributed across the team. 
- **Workload Balancer** — shows who is currently handling the most reviews.
- **Review Throughput** — charts showing how many reviews each team member completes over time.

## Repositories

### Adding and Managing

- Add repositories via `owner/name`.
- Configure the **Default Branch** for each.
- Edit **Review Rules** (checklists) that appear for every PR in that repo.

## Repository Groups

Groups let you organize repositories (e.g., by project or team).
- Create groups with custom names and colours.
- Assign repositories to one or more groups.
- Filter the main PR list and Dashboard by group.

## Label Automation Rules

Define rules that trigger actions based on PR labels. 
- **Label-based rules** can automatically:
    - Assign a PR to a specific group.
    - Set a priority level.
    - Add specific items to the review checklist.
    - Flag for immediate attention.

## Authors

The **Authors** page tracks performance metrics for every contributor:
- PRs authored vs merged.
- Average PR size and risk.
- "Reviewer Score" based on how many reviews they have provided to others.

## Settings

- **Sync Interval** — how often to poll GitHub (default: 5m).
- **Editor Command** — command used to open worktrees (e.g., `code`, `zed`).
- **Theme** — choose between **Dark**, **Light**, or **System** preference.
- **Accent Colour** — choose from 7 preset accent colours for the UI.
- **Density Toggle** — switch between **Comfortable** and **Compact** layout modes.
- **Review Templates** — create and manage reusable review comments.

## Auto-Sync & Notifications

### Background Polling

The app's Rust backend periodically fetches new data. A green pulsing indicator in the header shows when auto-sync is active.

### Desktop Notifications

Get native macOS/system notifications for:
- **New PRs** discovered during sync.
- **High-risk PRs** that need immediate attention.
- **Status changes** on PRs you are involved in.

## Keyboard Shortcuts & Focus Mode

### Focus Mode

Press **Cmd+Shift+F** to toggle **Focus Mode**, which hides the sidebar and header, leaving only the primary content area for distraction-free reading.

### Essential Shortcuts

| Key | Action |
|---------|------------------------------|
| `1` - `5`| Navigate between main views |
| `/` | Focus search |
| `r` | Manual sync all |
| `j` / `k`| Navigate list rows |
| `Enter` | Open PR detail |
| `Cmd+K` | Command Palette |
| `?` | Toggle shortcut overlay |
| `Esc` | Close / Blur |

## Command Palette

Press **Cmd+K** to open a fuzzy-searchable palette for:
- Jumping to any page.
- Searching and opening any PR by title or number.
- Triggering actions like "Sync All" or "Toggle Theme".

## Risk Score Explained

Risk is calculated (1-10) based on:
- **File Count** (6+ files = +1, 12+ files = +2)
- **Diff Size** (200+ lines = +1, 500+ lines = +2)
- **Review State** (Changes requested = +1)
- **Age** (24h+ = +1, 72h+ = +2)
- **Draft Status** (Draft = -1)

## AI-Assisted Review

### direct Claude CLI Integration

If configured, you can trigger a "Claude Review" which:
1. Creates a local worktree.
2. Runs the Claude CLI against the code.
3. Parses and displays the results in the **AI Tab**.

### Manual Prompt Workflow

1. **Generate Prompt** in the AI Tab.
2. **Copy to Clipboard**.
3. Paste into your preferred AI (Web or CLI).
4. **Paste Response** back into Fuse to parse into structured issues and approval status.

## Tips & Workflows

1. **Morning Triage** — check the **Priority Queue** first.
2. **Deep Review** — use **Review Session Mode** with the **Pomodoro Timer**.
3. **Team Sync** — use the **Workload Dashboard** to reassign stuck PRs.
4. **Consistency** — use **Review Rules** and **Label Automation** to ensure standards are met.
