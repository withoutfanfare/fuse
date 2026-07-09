# Two-Branch PR Tracking — Design

- **Date:** 2026-07-09
- **Status:** Approved (design), pending implementation plan
- **Author:** Danny Harding

## Goal

Let each repository declare **two** meaningful branches — a **production** branch
(e.g. `main`/`master`) and an **integration** branch (e.g. `staging`/`develop`) —
and drive the "wrong target" PR warning from those per-repo values instead of the
current hardcoded `main`/`master` check. This makes the warning match each repo's
real workflow (feature → integration → production).

## Current State

- `repositories.default_branch TEXT NOT NULL DEFAULT 'main'` — a single branch per
  repo, set at add time, used for worktree base and display.
- The "forbidden target" warning is **hardcoded and duplicated** in three places,
  each with identical logic `base === 'main' || base === 'master'`:
  - `src/components/PRTable.vue` (`isForbiddenTarget(pr)`)
  - `src/views/PullRequestDetail.vue` (`isForbiddenTarget` computed)
  - `src/components/WorktreePanel.vue` (`isForbiddenTarget` computed)
- Sync already fetches PRs for **all** target branches (`gh pr list --state all`,
  no base filter), so no sync/data-fetch change is needed.

## Design

### 1. Data model (option A — reuse `default_branch` as production)

- Treat the existing `default_branch` as the **production** branch. No data change;
  relabel it in the UI.
- Add one nullable column: `integration_branch TEXT` (NULL when a repo has no
  two-tier flow).

Migration (idempotent, matching the existing `ADD COLUMN` + error-swallow pattern
in `db/migrations.rs`):

```sql
ALTER TABLE repositories ADD COLUMN integration_branch TEXT
```

Wrap in the same `match { Ok => {}, Err(duplicate column) => {} }` guard used for
`last_delta_sync_at` etc.

### 2. Backend (`commands/repositories.rs`)

- `Repository` model + TS `Repository` type gain `integration_branch: Option<String>`
  / `integration_branch: string | null`.
- `add_repository`: unchanged signature is fine (new repos start with
  `integration_branch = NULL`); include the column in the returned row.
- `update_repository_branch`: extend to also accept `integration_branch: Option<String>`
  and update both columns in one statement. (The command already returns the updated
  `Repository`.) Empty string from the UI is normalised to `NULL`.
- `list_repositories` / get: select and map the new column.

### 3. Centralised branch policy (removes the 3 duplicates)

New composable `src/composables/useBranchPolicy.ts` exposing a pure predicate:

```ts
// A PR targets production directly when its base is the repo's production branch
// and its head is NOT the repo's integration branch (the integration→production
// release PR is legitimate and must not warn).
function isDirectToProduction(pr, repo): boolean {
  if (!repo) return false
  const base = pr.base_branch.toLowerCase()
  const production = repo.default_branch.toLowerCase()
  if (base !== production) return false
  const integration = repo.integration_branch?.toLowerCase()
  return pr.head_branch.toLowerCase() !== integration
}
```

- `integration` NULL ⇒ the head can never equal it ⇒ any PR targeting production
  warns. This preserves today's behaviour for repos with `default_branch` of
  `main`/`master` and no integration branch configured.
- All three components call this helper. Where the repo object is not already in
  scope, look it up from the repositories store by `pr.repo_id` (same approach as
  the recent-PRs fix):
  - `PRTable.vue`: look up per row by `pr.repo_id`.
  - `PullRequestDetail.vue`: already resolves the repo via `repo_id` (used for
    `repoFullName`); reuse it.
  - `WorktreePanel.vue`: stays presentational. Its parent (`PullRequestDetail.vue`)
    already computes `isDirectToProduction` for the same PR, so pass that result in
    as a `directToProduction: boolean` prop instead of `WorktreePanel` re-deriving it
    (it has no head/integration context of its own).

### 4. UI (`RepositoryCard.vue`)

- Relabel the existing branch field as **"Production branch"**.
- Add an editable **"Integration branch"** field beside it (optional; blank ⇒ NULL),
  saved through the extended `update_repository_branch`.
- Update the warning copy (PRTable tooltip, PullRequestDetail banner, WorktreePanel
  message) to name the repo's actual branches, e.g. *"Targets production `main`
  directly — expected flow is via `staging`"*, falling back to a generic message
  when no integration branch is set.

### 5. Behaviour summary

| PR | Integration set? | Result |
|----|------------------|--------|
| feature → integration | yes | no warning |
| integration → production | yes | no warning (head is integration) |
| feature → production | yes | **warn** |
| feature → production | no | **warn** (preserves current main/master behaviour) |
| feature → some other branch | either | no warning |

## Testing

- **Rust:** migration idempotency (run twice, no error); add-repo then
  `update_repository_branch` round-trips `integration_branch` (set and clear to NULL);
  `list_repositories` returns the column.
- **TypeScript:** `useBranchPolicy` unit tests covering every row of the table above,
  including case-insensitivity and NULL integration.

## Out of Scope

- Visually **grouping** the PR list by target branch — deferred to a separate
  feature.
- Any change to sync/data fetching (all target branches are already fetched).
- Auto-detecting the integration branch from GitHub — it is configured manually.

## Files Touched

- `src-tauri/src/db/migrations.rs` — add `integration_branch` column
- `src-tauri/src/models/mod.rs` — `Repository.integration_branch`
- `src-tauri/src/commands/repositories.rs` — add/update/list handle the column
- `src/types/index.ts` — `Repository.integration_branch`
- `src/composables/useBranchPolicy.ts` — new shared predicate (+ test)
- `src/stores/repositories.ts` — pass integration branch through `updateBranch`
- `src/components/PRTable.vue`, `src/views/PullRequestDetail.vue`,
  `src/components/WorktreePanel.vue` — use the helper, updated copy
- `src/components/RepositoryCard.vue` — production + integration branch editing
