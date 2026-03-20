# Feature Implementation Plan v2

## New features beyond the original 18 features in 01-features.md

---

## Feature 1: Linked Issue Context Panel

**Status**: ✅ Implemented
**Implementation notes**: `commands/issues.rs` backend with `gh issue view` integration, `useLinkedIssues.ts` composable, `LinkedIssuePanel.vue` sidebar panel in PR detail view. Parses "Fixes #N" / "Closes #N" patterns from PR body.

**Description**: Fetch the GitHub issue linked to a PR (via `gh pr view --json closingIssuesReferences` or by parsing "Fixes #123" from the PR body) and display the issue title, body, labels, and assignees inline. Gives the reviewer the full "why" behind the change without leaving the app.

**Why it matters**: Reviewers see the diff (the "what") but must open GitHub separately to understand the requirement or bug that motivated the PR. This is the single biggest missing piece in the briefing layer vision.

**Technical approach**: New `gh` CLI call: `gh issue view <number> --repo <name> --json title,body,labels,assignees,state,url`. Parse "Fixes #N" / "Closes #N" patterns from `pr.body` in Rust. New `commands/issues.rs`, `LinkedIssue` model, `LinkedIssuePanel.vue` in the PR detail sidebar.

**Complexity**: Medium

---

## Feature 2: Cross-PR Dependency Graph

**Status**: ✅ Implemented
**Implementation notes**: `commands/dependencies.rs` with dependency detection (body text parsing, branch ancestry comparison), `pr_dependencies` SQLite table, `DependencyGraph.vue` SVG node-link diagram rendering directed graph of PR relationships.

**Description**: Detect and visualise dependencies between PRs -- when PR #42 depends on PR #41 (via branch ancestry, "Depends on #41" in the body, or shared file paths). Render a directed graph showing which PRs block others and highlight merge-order risks.

**Why it matters**: Agency teams managing 6-8 repos frequently encounter stacked PRs or PRs that conflict. Zero visibility into inter-PR relationships means a reviewer might approve PR #42 only to discover it cannot merge until #41 lands.

**Technical approach**: Parse body text for "Depends on", "Blocked by", "After #N" patterns. Compare `base_branch` values -- if PR #42's base is PR #41's head branch, that's an explicit dependency. Store edges in `pr_dependencies` SQLite table. Frontend renders SVG node-link diagram. Use already-synced `gh pr list --json baseRefName,headRefName` data.

**Complexity**: Large

---

## Feature 3: Smart Review Queue with Priority Scoring

**Status**: ✅ Implemented
**Implementation notes**: `commands/priority_queue.rs` with multi-factor priority scoring (risk, CI status, file-path sensitivity, author trust, labels), `sensitive_paths` SQLite table, `PriorityQueue.vue` component on Dashboard with factor breakdown tooltips.

**Description**: Replace the flat PR list with an intelligent priority queue combining risk score, age, author review history, file-path sensitivity rules, and CI status into a single "review priority" score. Auto-sorts so the most important PR is always at the top, with an explanation of why it ranked there.

**Why it matters**: Current risk score only considers size and age. A small PR touching `payments/` from a junior developer with failing CI is far more urgent than a large documentation PR. Engineering leads need triage that accounts for domain context.

**Technical approach**: Extend `useRiskScore.ts` into `usePriorityScore.ts`. Factors: risk score (existing), CI status, file-path sensitivity patterns (new `sensitive_paths` SQLite table with glob patterns), author trust level (from `author_stats`), review decision state, label-based urgency. Display factor breakdown in existing tooltip pattern.

**Complexity**: Medium

---

## Feature 4: Review Session Mode (Focus Mode)

**Status**: ✅ Implemented
**Implementation notes**: `commands/sessions.rs` with `review_sessions` SQLite table, `ReviewSession.vue` three-panel view (file tree left, diff centre, notes/checklist right) at `/review-session/:prId` route. Pomodoro-style timer, per-file checkboxes, session persistence for resume.

**Description**: A dedicated review session view that locks the app into a focused, distraction-free mode for reviewing a single PR. Displays the diff, checklist, and comment drafting side by side with a Pomodoro-style timer. Tracks which files have been reviewed and saves progress to SQLite for resume.

**Why it matters**: The current PR detail view is a long scrolling page mixing metadata, actions, diff, comments, and AI review. For line-by-line review, reviewers need a focused layout. Also solves the "partial review" problem when interrupted mid-review.

**Technical approach**: New `ReviewSession.vue` view with three-panel layout (file tree left, diff centre, notes/checklist right). New `review_sessions` SQLite table storing `pr_id`, `started_at`, `files_reviewed` (JSON array), `session_notes`, `status`. File tree gets checkboxes per file. Route: `/review-session/:prId`.

**Complexity**: Large

---

## Feature 5: PR Commit History Viewer

**Status**: ✅ Implemented
**Implementation notes**: `commands/commits.rs` with `gh pr view --json commits` integration, `CommitTimeline.vue` component displaying commit messages, authors, timestamps, and per-commit diff viewing.

**Description**: Fetch and display individual commits within a PR using `gh pr view --json commits`, showing commit messages, authors, and timestamps. Allow viewing the diff per-commit rather than only the aggregate diff.

**Why it matters**: A PR with 15 commits tells a story. The aggregate diff hides that story. Stepping through commits sequentially helps reviewers understand the author's thought process, spot fixup commits that should have been squashed, and review incrementally.

**Technical approach**: New `gh` CLI call: `gh pr view <number> --repo <name> --json commits`. New `PrCommit` model. New `commands/commits.rs`. Frontend: `CommitList.vue` with "View diff for this commit" button using `gh api repos/{owner}/{repo}/commits/{sha}` or `git diff` within the grove worktree.

**Complexity**: Medium

---

## Feature 6: AI Review Comparison (Diff Between Reviews)

**Status**: ✅ Implemented
**Implementation notes**: `useAiReviewComparison.ts` composable parsing structured AI review output and comparing issue lists between reviews, `AiReviewComparison.vue` with side-by-side diff of reviews highlighting new, resolved, and persistent concerns.

**Description**: When a PR receives multiple AI reviews (code updated and re-reviewed), show a comparison highlighting what changed between reviews -- new issues, resolved issues, and persistent concerns. A "diff of the reviews" feature.

**Why it matters**: The app supports multiple AI reviews per PR via `pr_ai_reviews`. But when re-triggering AI after code changes, there's no way to see whether previous concerns were addressed. Painful for the "changes requested -> updated -> re-review" cycle.

**Technical approach**: Parse structured AI review output (already follows `### Summary / ### Issues / ### Verdict` format) into structured data. Compare issue lists between two reviews by matching on file path and description similarity. New `AiReviewComparison.vue` with side-by-side or inline diff of reviews.

**Complexity**: Medium

---

## Feature 7: Deployment Status Tracker

**Status**: ✅ Implemented
**Implementation notes**: `commands/deployments.rs` with GitHub Deployments API integration via `gh api`, `DeploymentStatus.vue` showing environment badges with links and timestamps alongside CI checks.

**Description**: Fetch deployment/environment status from GitHub using `gh api repos/{owner}/{repo}/deployments` and display which environment a PR's branch is deployed to (staging, preview, production). Show timestamps and status alongside CI checks.

**Why it matters**: For teams deploying to staging before review, knowing "this PR is live on staging" changes how a reviewer approaches it -- they can test the feature rather than just reading code. CI status and deployment state are different signals.

**Technical approach**: New `gh api` call to GitHub Deployments API: `gh api repos/{owner}/{repo}/deployments --jq '.[] | select(.ref == "<branch>")'` with statuses. New `Deployment` model, `commands/deployments.rs`. Frontend: `DeploymentStatus.vue` showing environment badges with links.

**Complexity**: Medium

---

## Feature 8: File-Level Annotation Bookmarks

**Status**: ✅ Implemented
**Implementation notes**: `commands/bookmarks.rs` with CRUD operations, `review_bookmarks` SQLite table, `BookmarksList.vue` sidebar panel, bookmark indicators in `DiffFileTree.vue`, `b` keyboard shortcut to toggle bookmark.

**Description**: Allow reviewers to bookmark specific files or line ranges within a PR diff with short notes, creating a personal annotation layer that persists in SQLite. Bookmarks appear as markers in the diff file tree and can be navigated via keyboard shortcuts.

**Why it matters**: During a complex review, a reviewer often spots something in file A to cross-reference with file D. No way to mark "come back to this". Posting a GitHub comment for every internal thought pollutes the PR thread. This is a private annotation layer.

**Technical approach**: New `review_bookmarks` SQLite table. New `commands/bookmarks.rs` with CRUD. Frontend: clickable gutter in `DiffViewer.vue`, `BookmarksList.vue` sidebar panel, bookmark indicators in `DiffFileTree.vue`. Keyboard shortcut `b` to toggle bookmark.

**Complexity**: Medium

---

## Feature 9: Reviewer Workload Balancer Dashboard

**Status**: ✅ Implemented
**Implementation notes**: `commands/workload.rs` aggregating reviews per person with response time and overdue counts, `requestedReviewers` added to `GH_PR_FIELDS`, `pr_requested_reviewers` junction table, `WorkloadDashboard.vue` with per-reviewer bar chart.

**Description**: Dashboard section showing review workload distribution across team members -- who is assigned the most reviews, who has the most overdue, average response time per reviewer. Surfaces data from GitHub's `requestedReviewers` field.

**Why it matters**: Engineering leads need visibility into whether review work is evenly distributed. The app tracks author stats but not reviewer stats. Imbalanced review load leads to bottlenecks and burnout.

**Technical approach**: Add `requestedReviewers` to `GH_PR_FIELDS`. Store in `pr_requested_reviewers` junction table. New `commands/workload.rs` aggregating reviews per person, average response time, overdue count. Frontend: `WorkloadDashboard.vue` with bar chart per reviewer.

**Complexity**: Large

---

## Feature 10: PR Label Automation Rules

**Status**: ✅ Implemented
**Implementation notes**: `commands/label_rules.rs` with rule evaluation on sync, `label_rules` SQLite table with `condition_type`/`condition_value`/`label_name`/`auto_apply`, `LabelRulesManager.vue` for rule CRUD, auto-apply via `gh pr edit --add-label`.

**Description**: Define rules that automatically apply or suggest GitHub labels based on PR characteristics -- file paths touched, size thresholds, author, or base branch. E.g.: "If any file in `app/Models/` is changed, add label `database`".

**Why it matters**: Consistent labelling is essential for filtering but tedious to maintain manually. The app displays labels but treats them as read-only. Automating labels closes the loop -- the app writes back useful metadata.

**Technical approach**: New `label_rules` SQLite table with `condition_type`, `condition_value`, `label_name`, `auto_apply`. On sync, evaluate rules. If `auto_apply`, call `gh pr edit --add-label`. If suggest-only, store suggestions and show a badge in the UI. New `commands/label_rules.rs`.

**Complexity**: Medium

---

## Feature 11: Review Handoff Notes

**Status**: ✅ Implemented
**Implementation notes**: `commands/handoffs.rs` with `review_handoffs` SQLite table, `HandoffComposer.vue` pre-populating from checked files and bookmark notes, markdown export via `gh pr comment`.

**Description**: When a reviewer partially reviews a PR and needs to hand it off, they can write structured handoff notes (what they checked, what remains, concerns) that persist in SQLite and can be exported as a GitHub comment.

**Why it matters**: In agency teams, review handoffs happen regularly. No structured way to communicate "I reviewed files A-D, found concern X, but files E-G need domain expertise". The review just stalls.

**Technical approach**: New `review_handoffs` SQLite table. New `commands/handoffs.rs`. Frontend: `HandoffComposer.vue` pre-populating from checked files and bookmark notes. Export button formats as markdown and posts via `gh pr comment`.

**Complexity**: Medium

---

## Feature 12: Recurring Review Digest

**Status**: ✅ Implemented
**Implementation notes**: `commands/digest.rs` aggregating from `pr_reviews`, `pull_requests`, and `sync_log`, `DigestView.vue` with comparison sparklines at `/digest` route, native notification delivery via `tauri-plugin-notification`.

**Description**: Configurable daily or weekly digest summarising review activity -- PRs reviewed, pending, average review time, stale count, trend comparisons vs previous period. Delivered as native notification with link to a digest view.

**Why it matters**: The app shows real-time state but has no periodic retrospective. Engineering leads want a weekly "state of reviews" summary without manually checking the dashboard.

**Technical approach**: Use existing polling infrastructure to schedule digest generation. New `commands/digest.rs` aggregating from `pr_reviews`, `pull_requests`, `sync_log`. Frontend: `DigestView.vue` with comparison sparklines. Native notification via `tauri-plugin-notification`.

**Complexity**: Medium

---

## Feature 13: Conflict Detection and Resolution Hints

**Status**: ✅ Implemented
**Implementation notes**: `commands/conflicts.rs` with `gh api` and `git merge-tree` conflict detection, `ConflictBadge.vue` indicator badge, `ConflictDetail.vue` panel showing per-file conflict severity.

**Description**: Detect merge conflicts before review and surface which files conflict. When a grove worktree exists, run `git merge-tree` to identify specific conflicting hunks.

**Why it matters**: `mergeable` is a binary signal. Knowing the conflict is in `package-lock.json` (trivial) vs `src/core/billing.rs` (serious) changes prioritisation. Prevents wasting 30 minutes reviewing a PR that cannot merge.

**Technical approach**: When `mergeable === "CONFLICTING"`, fetch details via `gh api repos/{owner}/{repo}/pulls/{number}`. In a grove worktree, run `git merge-tree`. New `commands/conflicts.rs`. Frontend: `ConflictIndicator.vue` badge and `ConflictDetail.vue` panel.

**Complexity**: Medium

---

## Recommended Implementation Order

### Phase 1 -- Highest reviewer impact, lowest effort:
1. Feature 1 (Linked Issue Context) -- Completes the briefing layer
2. Feature 5 (Commit History) -- Low effort, high review quality
3. Feature 3 (Smart Priority Queue) -- Extends existing risk score

### Phase 2 -- Review workflow depth:
4. Feature 8 (Annotation Bookmarks)
5. Feature 4 (Review Session Mode)
6. Feature 13 (Conflict Detection)

### Phase 3 -- Team coordination:
7. Feature 11 (Review Handoff Notes)
8. Feature 9 (Workload Balancer)
9. Feature 6 (AI Review Comparison)

### Phase 4 -- Automation and intelligence:
10. Feature 10 (Label Automation)
11. Feature 7 (Deployment Status)
12. Feature 12 (Review Digest)
13. Feature 2 (Dependency Graph)

## Summary Table

| # | Feature | Problem Solved | Complexity |
|---|---------|---------------|------------|
| 1 | Linked Issue Context Panel | Reviewer lacks "why" context | Medium |
| 2 | Cross-PR Dependency Graph | Hidden merge-order risks | Large |
| 3 | Smart Review Queue | Flat list doesn't reflect urgency | Medium |
| 4 | Review Session Mode | No focused review layout | Large |
| 5 | PR Commit History Viewer | Cannot review by commit | Medium |
| 6 | AI Review Comparison | Cannot track resolved concerns | Medium |
| 7 | Deployment Status Tracker | No staging visibility | Medium |
| 8 | File-Level Annotation Bookmarks | No private note-taking | Medium |
| 9 | Reviewer Workload Balancer | No review load visibility | Large |
| 10 | PR Label Automation Rules | Inconsistent manual labelling | Medium |
| 11 | Review Handoff Notes | No structured handoff | Medium |
| 12 | Recurring Review Digest | No periodic retrospective | Medium |
| 13 | Conflict Detection & Hints | Binary conflict signal | Medium |
