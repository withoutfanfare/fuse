# Two-Branch PR Tracking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let each repository declare a production branch (existing `default_branch`) and a new optional integration branch, and drive the "wrong target" PR warning from those per-repo values instead of the hardcoded `main`/`master` check.

**Architecture:** Add one nullable `integration_branch` column to `repositories`; thread it through the Rust repo commands and the TypeScript `Repository` type; centralise the warning rule (currently duplicated in three components) into one pure predicate; add an integration-branch editor to `RepositoryCard`.

**Tech Stack:** Rust (rusqlite 0.32, Tauri v2), Vue 3 Composition API + TypeScript. Rust tests via `cargo test` from `tauri-app/src-tauri/`. Frontend verified via `npm run build` (tsc + vite) from `tauri-app/` — the project has no frontend test runner, so the TS predicate is a pure function verified by the type-checker and a manual app check.

## Global Constraints

- All Rust commands run from `tauri-app/src-tauri/`; all npm commands from `tauri-app/`.
- British English in comments and user-facing copy.
- Conventional commits. Never mention AI/Claude in commit messages; no Co-Authored-By trailers.
- Work on a feature branch off `develop`: `git checkout -b feat/two-branch-pr-tracking develop` before Task 1.
- Minimum change per task — do not refactor untouched code.
- The warning rule (single source of truth): a PR targets production directly when `base_branch == repo.default_branch` AND `head_branch != repo.integration_branch` (case-insensitive). A null/empty integration branch means the head can never equal it, so any PR targeting production warns.
- `default_branch` is the **production** branch. `integration_branch` is nullable; an empty string from the UI is normalised to SQL NULL.

---

### Task 1: Add `integration_branch` column (migration)

**Files:**
- Modify: `src-tauri/src/db/migrations.rs` (add a guarded `ALTER` inside `run_migrations`, and a new `#[cfg(test)] mod tests` at the bottom)

**Interfaces:**
- Consumes: nothing.
- Produces: a `repositories.integration_branch TEXT` column (nullable).

- [ ] **Step 1: Write the failing test**

Append to the bottom of `src-tauri/src/db/migrations.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_idempotent_and_add_integration_branch() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Running twice must not error (idempotent).
        run_migrations(&conn).unwrap();

        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(repositories)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(
            cols.contains(&"integration_branch".to_string()),
            "integration_branch column should exist, got: {cols:?}"
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test migrations_are_idempotent`
Expected: FAIL — the assertion fails because `integration_branch` does not exist yet.

- [ ] **Step 3: Add the migration**

Inside `run_migrations`, after the existing `last_delta_sync_at` `ALTER` block, add:

```rust
    // Add integration_branch to repositories for two-branch PR tracking.
    // Nullable: repos without a two-tier flow leave it unset.
    match conn.execute_batch("ALTER TABLE repositories ADD COLUMN integration_branch TEXT") {
        Ok(()) => {}
        Err(e) => {
            let msg = e.to_string();
            if !msg.contains("duplicate column name") {
                return Err(e);
            }
        }
    }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test migrations_are_idempotent`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/migrations.rs
git commit -m "feat: add integration_branch column to repositories"
```

---

### Task 2: Thread `integration_branch` through the Rust repo commands

**Files:**
- Modify: `src-tauri/src/models/mod.rs` (`Repository` struct)
- Modify: `src-tauri/src/commands/repositories.rs` (all three SELECTs, `update_repository_branch` signature + UPDATE, new `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: the `integration_branch` column from Task 1.
- Produces:
  - `Repository { id, owner, name, default_branch, integration_branch: Option<String>, added_at }`
  - `update_repository_branch(id: i64, default_branch: String, integration_branch: Option<String>, state) -> Result<Repository, CommandError>` — the JS side passes `{ id, defaultBranch, integrationBranch }`.

- [ ] **Step 1: Write the failing test**

Append to the bottom of `src-tauri/src/commands/repositories.rs`:

```rust
#[cfg(test)]
mod tests {
    // Command handlers need a Tauri State, which is impractical to build in a
    // unit test, so we exercise the exact SQL they run against an in-memory DB.
    #[test]
    fn integration_branch_round_trips_through_sql() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        conn.execute(
            "INSERT INTO repositories (owner, name, default_branch) VALUES ('o', 'r', 'main')",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();

        // Set an integration branch (mirrors update_repository_branch's UPDATE).
        conn.execute(
            "UPDATE repositories SET default_branch = ?1, integration_branch = ?2 WHERE id = ?3",
            rusqlite::params!["main", "staging", id],
        )
        .unwrap();
        let got: Option<String> = conn
            .query_row(
                "SELECT integration_branch FROM repositories WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(got, Some("staging".to_string()));

        // Clearing to NULL must round-trip as None.
        conn.execute(
            "UPDATE repositories SET integration_branch = NULL WHERE id = ?1",
            [id],
        )
        .unwrap();
        let cleared: Option<String> = conn
            .query_row(
                "SELECT integration_branch FROM repositories WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cleared, None);
    }
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test integration_branch_round_trips`
Expected: PASS once Task 1's migration is in place — this is a **schema-contract test** exercised through raw SQL (the command handlers need a Tauri `State`, so they cannot be unit-tested directly here). It pins the `integration_branch` set/clear-to-NULL behaviour that the typed model and commands in the following steps must honour; those steps are then guarded by `cargo build` (the `row.get` indices must line up with the struct) and the manual app check in Final Verification.

- [ ] **Step 3: Add the field to the `Repository` model**

In `src-tauri/src/models/mod.rs`, change the `Repository` struct to:

```rust
pub struct Repository {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub default_branch: String,
    pub integration_branch: Option<String>,
    pub added_at: String,
}
```

- [ ] **Step 4: Update all three SELECT mappings and the INSERT round-trip**

In `src-tauri/src/commands/repositories.rs`, every `SELECT id, owner, name, default_branch, added_at ...` becomes `SELECT id, owner, name, default_branch, integration_branch, added_at ...`, and every row closure gains the new field with shifted indices. The mapping closure in `add_repository`, `update_repository_branch`, and `list_repositories` becomes:

```rust
            Ok(Repository {
                id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                default_branch: row.get(3)?,
                integration_branch: row.get(4)?,
                added_at: row.get(5)?,
            })
```

Concretely, replace the three SELECT strings:
- `add_repository` (currently `WHERE id = ?1`): `"SELECT id, owner, name, default_branch, integration_branch, added_at FROM repositories WHERE id = ?1"`
- `update_repository_branch`: same string as above.
- `list_repositories`: `"SELECT id, owner, name, default_branch, integration_branch, added_at FROM repositories ORDER BY added_at DESC"`

`add_repository`'s INSERT is unchanged (new repos get NULL `integration_branch`).

- [ ] **Step 5: Extend `update_repository_branch` to accept and store the integration branch**

Replace the `update_repository_branch` signature and UPDATE:

```rust
/// Update the production (default) and integration branches for a repository.
#[tauri::command]
pub fn update_repository_branch(
    id: i64,
    default_branch: String,
    integration_branch: Option<String>,
    state: State<'_, DbState>,
) -> Result<Repository, CommandError> {
    let db = state.writer.lock().unwrap();
    // Normalise an empty/whitespace integration branch to NULL.
    let integration = integration_branch.filter(|s| !s.trim().is_empty());
    let affected = db.execute(
        "UPDATE repositories SET default_branch = ?1, integration_branch = ?2 WHERE id = ?3",
        rusqlite::params![default_branch, integration, id],
    )?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!("Repository with id {id}")));
    }
    // ... unchanged SELECT + Ok(repo) below (now selecting integration_branch) ...
```

Keep the existing `query_row` SELECT + `Ok(repo)` tail (updated in Step 4).

- [ ] **Step 6: Build and run all tests**

Run: `cargo build && cargo test`
Expected: clean build (no new warnings beyond the pre-existing `models/mod.rs` dead-code warning), all tests PASS including `integration_branch_round_trips_through_sql` and `migrations_are_idempotent`.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/models/mod.rs src-tauri/src/commands/repositories.rs
git commit -m "feat: read and update integration_branch in repo commands"
```

---

### Task 3: Frontend types and store wiring

**Files:**
- Modify: `src/types/index.ts` (`Repository` interface)
- Modify: `src/stores/repositories.ts` (`updateBranch`)

**Interfaces:**
- Consumes: the Rust `update_repository_branch` command shape (`{ id, defaultBranch, integrationBranch }`).
- Produces:
  - `Repository.integration_branch: string | null`
  - `updateBranch(id: number, defaultBranch: string, integrationBranch: string | null): Promise<void>`

- [ ] **Step 1: Add the field to the `Repository` type**

In `src/types/index.ts`, the `Repository` interface becomes:

```ts
export interface Repository {
  id: number
  owner: string
  name: string
  default_branch: string
  integration_branch: string | null
  added_at: string
}
```

- [ ] **Step 2: Extend the store `updateBranch`**

In `src/stores/repositories.ts`, replace `updateBranch`:

```ts
  async function updateBranch(
    id: number,
    defaultBranch: string,
    integrationBranch: string | null,
  ) {
    error.value = null
    try {
      await invoke<Repository>('update_repository_branch', {
        id,
        defaultBranch,
        integrationBranch,
      })
      await fetchAll()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }
```

- [ ] **Step 3: Type-check**

Run: `npm run build`
Expected: FAILS in `Repositories.vue` (its `updateBranch(id, branch)` call now misses an argument) — this is expected and fixed in Task 6. If you are executing tasks strictly in order, instead run `npx vue-tsc --noEmit 2>&1 | grep -c "src/stores/repositories.ts"` and confirm the store file itself reports no errors; the `Repositories.vue` error is resolved in Task 6.

- [ ] **Step 4: Commit**

```bash
git add src/types/index.ts src/stores/repositories.ts
git commit -m "feat: add integration_branch to Repository type and store"
```

---

### Task 4: `useBranchPolicy` — the single warning predicate

**Files:**
- Create: `src/composables/useBranchPolicy.ts`

**Interfaces:**
- Consumes: `PullRequest` (`base_branch`, `head_branch`) and `Repository` (`default_branch`, `integration_branch`) types.
- Produces: `isDirectToProduction(pr, repo): boolean` (exported directly and via `useBranchPolicy()`).

- [ ] **Step 1: Create the predicate**

Create `src/composables/useBranchPolicy.ts`:

```ts
import type { PullRequest, Repository } from '../types'

/**
 * A PR targets production directly when its base branch is the repository's
 * production branch (`default_branch`) and its head is NOT the integration
 * branch — the integration → production release PR is legitimate and must not
 * warn. A null/empty integration branch means the head can never match it, so
 * any PR targeting production is flagged. Matching is case-insensitive.
 */
export function isDirectToProduction(
  pr: Pick<PullRequest, 'base_branch' | 'head_branch'>,
  repo: Pick<Repository, 'default_branch' | 'integration_branch'> | null | undefined,
): boolean {
  if (!repo) return false
  const base = pr.base_branch.toLowerCase()
  const production = repo.default_branch.toLowerCase()
  if (base !== production) return false
  const integration = repo.integration_branch?.toLowerCase() ?? null
  return pr.head_branch.toLowerCase() !== integration
}

export function useBranchPolicy() {
  return { isDirectToProduction }
}
```

- [ ] **Step 2: Type-check**

Run: `npx vue-tsc --noEmit 2>&1 | grep "useBranchPolicy" || echo "no errors in useBranchPolicy"`
Expected: `no errors in useBranchPolicy`.

- [ ] **Step 3: Manual logic check (no test runner in this project)**

Confirm by inspection against the spec's behaviour table:
- base=`main`, head=`feature`, production=`main`, integration=`staging` → `true` (warn).
- base=`main`, head=`staging`, integration=`staging` → `false` (release PR).
- base=`staging`, integration=`staging` → `false`.
- base=`main`, integration=`null` → `true`.
- base=`dev`, production=`main` → `false`.

- [ ] **Step 4: Commit**

```bash
git add src/composables/useBranchPolicy.ts
git commit -m "feat: add useBranchPolicy predicate for direct-to-production PRs"
```

---

### Task 5: Use the predicate in the three components (removes duplication)

**Files:**
- Modify: `src/components/PRTable.vue`
- Modify: `src/views/PullRequestDetail.vue`
- Modify: `src/components/WorktreePanel.vue`

**Interfaces:**
- Consumes: `isDirectToProduction` from Task 4; `Repository` from the repositories store.
- Produces: `WorktreePanel` gains a `directToProduction: boolean` prop.

- [ ] **Step 1: PRTable — replace hardcoded `isForbiddenTarget`**

In `src/components/PRTable.vue` script, add imports and the repo store (if not already present):

```ts
import { useRepositoriesStore } from '../stores/repositories'
import { isDirectToProduction } from '../composables/useBranchPolicy'
```

Add near the other store setup:

```ts
const repoStore = useRepositoriesStore()
```

Replace the existing function:

```ts
function isForbiddenTarget(pr: PullRequest): boolean {
  const base = pr.base_branch.toLowerCase()
  return base === 'main' || base === 'master'
}
```

with:

```ts
function isForbiddenTarget(pr: PullRequest): boolean {
  const repo = repoStore.repos.find(r => r.id === pr.repo_id)
  return isDirectToProduction(pr, repo)
}
```

(The template at `PRTable.vue:517-518` keeps calling `isForbiddenTarget(pr)`.) Update the tooltip on the `AlertTriangle` from `title="PRs should target staging, not main/master"` to `:title="'Targets ' + pr.base_branch + ' directly — this is your production branch'"`.

- [ ] **Step 2: PullRequestDetail — use the resolved repo**

In `src/views/PullRequestDetail.vue`, import the predicate:

```ts
import { isDirectToProduction } from '../composables/useBranchPolicy'
```

The view already resolves the repo for `repoFullName` via `repoStore.repos.find(r => r.id === pr.value!.repo_id)`. Add a repo computed if not present:

```ts
const currentRepo = computed(() =>
  pr.value ? repoStore.repos.find(r => r.id === pr.value!.repo_id) : undefined,
)
```

Replace the `isForbiddenTarget` computed body:

```ts
const isForbiddenTarget = computed(() => {
  if (!pr.value) return false
  return isDirectToProduction(pr.value, currentRepo.value)
})
```

Update the banner copy at `PullRequestDetail.vue:479` and the inline warning at `:608` to name the branch, e.g. `Targets <code>{{ pr.base_branch }}</code> directly — review carefully before merging to your production branch`.

- [ ] **Step 3: WorktreePanel — accept the flag as a prop**

In `src/components/WorktreePanel.vue`, add `directToProduction?: boolean` to the props, delete its own `isForbiddenTarget` computed, and use the prop in the template. Change the props block to include:

```ts
  directToProduction?: boolean
```

Replace the `v-if="isForbiddenTarget"` at `WorktreePanel.vue:92` with `v-if="directToProduction"`, and update the message text at `:96` to: `<p>This PR targets <code>{{ baseBranch }}</code> directly — expected flow is via your integration branch first.</p>`

- [ ] **Step 4: Pass the flag from PullRequestDetail to WorktreePanel**

At `PullRequestDetail.vue:460` where `<WorktreePanel :base-branch="pr.base_branch" ... />` is rendered, add `:direct-to-production="isForbiddenTarget"`.

- [ ] **Step 5: Type-check + build**

Run: `npm run build`
Expected: PASS (tsc clean, vite build succeeds). `Repositories.vue` still errors until Task 6 — if executing strictly in order, run `npx vue-tsc --noEmit 2>&1 | grep -vE "Repositories.vue" | grep -c "error TS"` and confirm `0` errors outside `Repositories.vue`.

- [ ] **Step 6: Commit**

```bash
git add src/components/PRTable.vue src/views/PullRequestDetail.vue src/components/WorktreePanel.vue
git commit -m "feat: drive direct-to-production warning from per-repo branch policy"
```

---

### Task 6: Edit the integration branch on `RepositoryCard`

**Files:**
- Modify: `src/components/RepositoryCard.vue`
- Modify: `src/views/Repositories.vue`

**Interfaces:**
- Consumes: `updateBranch(id, defaultBranch, integrationBranch)` from Task 3.
- Produces: `RepositoryCard` emits `update-branches: [id: number, production: string, integration: string]`.

- [ ] **Step 1: RepositoryCard — relabel production and add an integration editor**

In `src/components/RepositoryCard.vue`, change the emit type:

```ts
const emit = defineEmits<{
  remove: [id: number]
  sync: [id: number]
  'update-branches': [id: number, production: string, integration: string]
}>()
```

Add integration-branch editing state alongside the existing branch state:

```ts
const editingIntegration = ref(false)
const editIntegrationValue = ref('')
const integrationInput = ref<InstanceType<typeof SInput> | null>(null)

function startEditIntegration() {
  editIntegrationValue.value = props.repo.integration_branch ?? ''
  editingIntegration.value = true
  nextTick(() => {
    const input = integrationInput.value?.$el?.querySelector('input') as HTMLInputElement | null
    input?.focus()
    input?.select()
  })
}

function cancelEditIntegration() {
  editingIntegration.value = false
}

function saveIntegration() {
  const trimmed = editIntegrationValue.value.trim()
  if (trimmed !== (props.repo.integration_branch ?? '')) {
    emit('update-branches', props.repo.id, props.repo.default_branch, trimmed)
  }
  editingIntegration.value = false
}

function onIntegrationKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') saveIntegration()
  else if (event.key === 'Escape') cancelEditIntegration()
}
```

Update the existing `saveBranch` to emit the new event (production changed, integration unchanged):

```ts
function saveBranch() {
  const trimmed = editBranchValue.value.trim()
  if (trimmed && trimmed !== props.repo.default_branch) {
    emit('update-branches', props.repo.id, trimmed, props.repo.integration_branch ?? '')
  }
  editingBranch.value = false
}
```

- [ ] **Step 2: RepositoryCard — template for the integration branch**

Change the existing `Branch:` label to `Production:` and add an integration row after the branch `<span class="meta-item branch-meta">…</span>` block:

```html
      <span class="meta-item branch-meta">
        Integration:
        <template v-if="editingIntegration">
          <SInput
            ref="integrationInput"
            v-model="editIntegrationValue"
            size="sm"
            class="branch-edit-input"
            placeholder="none"
            @keydown="onIntegrationKeydown"
            @blur="saveIntegration"
          />
        </template>
        <template v-else>
          <code class="branch-display" @click="startEditIntegration">
            {{ repo.integration_branch || '—' }}
            <Pencil :size="11" class="branch-edit-icon" />
          </code>
        </template>
      </span>
```

- [ ] **Step 3: Repositories.vue — handle the new event**

In `src/views/Repositories.vue`, replace the `updateBranch` handler:

```ts
async function updateBranches(id: number, production: string, integration: string) {
  try {
    await repoStore.updateBranch(id, production, integration.trim() || null)
  } catch {
    // store surfaces the error
  }
}
```

And at the `<RepositoryCard>` usage (currently `@update-branch="updateBranch"`), change to `@update-branches="updateBranches"`. Remove the now-unused old `updateBranch` function if it is no longer referenced.

- [ ] **Step 4: Type-check + build**

Run: `npm run build`
Expected: PASS — tsc clean, vite build succeeds, no remaining `Repositories.vue` errors.

- [ ] **Step 5: Commit**

```bash
git add src/components/RepositoryCard.vue src/views/Repositories.vue
git commit -m "feat: edit production and integration branches on repository card"
```

---

## Final Verification (after all tasks)

- [ ] `cargo test` from `tauri-app/src-tauri/` — all tests pass (migration idempotency + integration_branch round-trip + the existing suite).
- [ ] `npm run build` from `tauri-app/` — tsc clean, vite build succeeds.
- [ ] Run the app (`npm run tauri dev`): on a repo, set Production = `main` and Integration = `staging`; confirm a PR into `main` from a feature branch shows the warning, a PR into `staging` does not, and a PR from `staging` into `main` does not. Clear the integration branch and confirm any PR into `main` warns.
- [ ] Merge `feat/two-branch-pr-tracking` into `develop`. Never merge to `main` (human-only).
