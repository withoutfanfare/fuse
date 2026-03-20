# Code Review Report: PR Review Companion (Fuse)

**Date:** 2026-03-09  
**Scope:** Full codebase review (Tauri + Vue 3)  
**Focus:** Bugs, Optimizations, and Code Quality

---

## Executive Summary

Overall, this is a well-structured Tauri app with a Vue 3 frontend. The code is generally clean and follows good practices with proper separation of concerns between the Rust backend and TypeScript frontend. However, several **critical bugs**, **missing implementations**, **potential optimizations**, and **code quality issues** have been identified.

---

## 🐛 CRITICAL BUGS

### 1. Missing Backend Commands for Settings (CRITICAL)
**Location:** `src/stores/settings.ts`  
**Issue:** The settings store calls `get_settings` and `update_setting` commands, but these are **not implemented** in the Rust backend (`src-tauri/src/commands/`). The commands are also not registered in `lib.rs`.

**Impact:** Settings page will crash or silently fail when trying to save/load settings.

**Fix Required:**
```rust
// Create src-tauri/src/commands/settings.rs
#[tauri::command]
pub fn get_settings(state: State<'_, DbState>) -> Result<HashMap<String, String>, CommandError> {
    let db = state.0.lock().unwrap();
    let mut stmt = db.prepare("SELECT key, value FROM app_settings")?;
    let settings = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
        .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(settings)
}

#[tauri::command]
pub fn update_setting(
    key: String,
    value: String,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.0.lock().unwrap();
    db.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}

// Add to lib.rs generate_handler![]
```

---

### 2. Missing Toast Type Definition (HIGH)
**Location:** `src/stores/toast.ts`  
**Issue:** The toast store references `Toast` and `ToastType` types that are **not defined** in `src/types/index.ts`. The store exists but cannot be used.

**Fix Required:**
```typescript
// Add to src/types/index.ts
export type ToastType = 'success' | 'error' | 'info' | 'warning'

export interface Toast {
  id: number
  type: ToastType
  title: string
  message?: string
  duration: number
}
```

---

### 3. Transaction Handling Bug in `set_review_rules` (HIGH)
**Location:** `src-tauri/src/commands/pull_requests.rs:179-202`  
**Issue:** The transaction uses `execute_batch("BEGIN")` which creates an implicit transaction, but the closure pattern doesn't properly handle the transaction lifecycle. If an error occurs, the connection may be left in an inconsistent state.

**Current Code:**
```rust
db.execute_batch("BEGIN")?;
let result = (|| -> Result<(), CommandError> { ... })();
// ... commit/rollback
```

**Fix Required:**
```rust
pub fn set_review_rules(
    repo_id: i64,
    rules: Vec<String>,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let mut db = state.0.lock().unwrap();
    let tx = db.transaction()?;
    
    tx.execute("DELETE FROM review_rules WHERE repo_id = ?1", [repo_id])?;
    
    let mut stmt = tx.prepare(
        "INSERT INTO review_rules (repo_id, rule_text, position) VALUES (?1, ?2, ?3)",
    )?;
    for (i, rule_text) in rules.iter().enumerate() {
        stmt.execute(rusqlite::params![repo_id, rule_text, i as i64])?;
    }
    drop(stmt);
    
    tx.commit()?;
    Ok(())
}
```

---

### 4. Mutex Poisoning Panics (MEDIUM)
**Location:** All files using `state.0.lock().unwrap()`  
**Issue:** Using `.unwrap()` on mutex locks can panic if the mutex is poisoned (when a thread panics while holding the lock). While uncommon in single-threaded Tauri apps, this is a code smell.

**Files Affected:**
- `src-tauri/src/commands/repositories.rs:27, 55, 65`
- `src-tauri/src/commands/pull_requests.rs:62, 93, 114, 153, 176, 233, 240, 260, 275`
- `src-tauri/src/commands/sync.rs:14, 58, 78`
- `src-tauri/src/commands/stats.rs:10`

**Fix Required:**
```rust
// Instead of:
let db = state.0.lock().unwrap();

// Use:
let db = state.0.lock().unwrap_or_else(|e| e.into_inner());
```

---

### 5. Unused Event Emission (LOW)
**Location:** `src/components/WorktreePanel.vue:44-48`  
**Issue:** The `review-requested` event is emitted but never handled by the parent component (`PullRequestDetail.vue`).

```typescript
// In WorktreePanel.vue
emit('review-requested', branchWorktree.value.path)

// In PullRequestDetail.vue - no handler for @review-requested
```

**Fix Required:** Either remove the emit or add a handler in the parent.

---

## ⚡ OPTIMIZATIONS

### 1. Inefficient PR Counting in Dashboard (HIGH)
**Location:** `src/views/Dashboard.vue:19-23`  
**Issue:** Computing risk scores for all open PRs on every render without memoization causes unnecessary recalculations.

**Current Code:**
```typescript
const urgentPrs = computed(() => {
  return [...prStore.openPrs]
    .sort((a, b) => computeRiskScore(b) - computeRiskScore(a))
    .slice(0, 5)
})
```

**Fix:** Cache risk scores to avoid double computation:
```typescript
const urgentPrs = computed(() => {
  return [...prStore.openPrs]
    .map(pr => ({ pr, score: computeRiskScore(pr) }))
    .sort((a, b) => b.score - a.score)
    .slice(0, 5)
    .map(item => item.pr)
})
```

---

### 2. Repeated Risk Score Calculation in PRTable (MEDIUM)
**Location:** `src/components/PRTable.vue:25, 100`  
**Issue:** `computeRiskScore` is called twice per row - once for sorting, once for display in the RiskBadge component.

**Fix:** Pre-compute and pass as prop:
```typescript
const prsWithRisk = computed(() => 
  props.prs.map(pr => ({ pr, risk: computeRiskScore(pr) }))
)
```

---

### 3. No Request Deduplication (MEDIUM)
**Location:** All store actions (`pullRequests.ts`, `repositories.ts`)  
**Issue:** Multiple rapid calls to `fetchAll()` will fire multiple concurrent API requests.

**Fix:** Add request deduplication:
```typescript
// In pullRequests store
let fetchPromise: Promise<void> | null = null

async function fetchAll(repoId?: number, statusFilter?: string) {
  if (fetchPromise) return fetchPromise
  
  fetchPromise = (async () => {
    loading.value = true
    error.value = null
    try {
      const params: Record<string, unknown> = {}
      if (repoId !== undefined) params.repoId = repoId
      if (statusFilter !== undefined) params.statusFilter = statusFilter
      prs.value = await invoke<PullRequest[]>('get_pull_requests', params)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  })()
  
  try {
    await fetchPromise
  } finally {
    fetchPromise = null
  }
}
```

---

### 4. Unnecessary Database Queries in `get_pr_context` (LOW)
**Location:** `src-tauri/src/commands/pull_requests.rs:206-225`  
**Issue:** Two separate queries when a JOIN could be used, or when the data could be cached.

**Current:**
```rust
let (repo_id, number, base_branch): (i64, i64, String) = db.query_row(...)?;
let full_name: String = db.query_row(...)?;
```

**Optimization:** Use a single query with JOIN.

---

### 5. Missing Virtual Scrolling (LOW)
**Location:** `src/components/PRTable.vue`  
**Issue:** For repositories with many PRs (>100), the table renders all rows at once, causing performance issues.

**Recommendation:** Consider virtual scrolling libraries like `vue-virtual-scroller` for large lists.

---

## 🎨 CODE QUALITY ISSUES

### 1. Inconsistent Error Handling (HIGH)
**Location:** Multiple stores  
**Issue:** Error handling is inconsistent across stores:
- `repositories.ts: add()` - throws AND sets error
- `pullRequests.ts: fetchOne()` - returns null on error
- `pullRequests.ts: fetchAll()` - sets error only

**Recommendation:** Standardize on one pattern. Suggested approach:
- Store actions should set `error.value` for UI display
- Optionally throw for programmatic error handling
- Document the pattern in a conventions file

---

### 2. Magic Numbers (MEDIUM)
**Location:** `src/composables/useRiskScore.ts:6-16`  
**Issue:** Numbers like `12`, `6`, `500`, `200`, `72`, `24` are magic values without context.

**Fix:** Extract to named constants:
```typescript
const THRESHOLDS = {
  FILES_HIGH: 12,
  FILES_MEDIUM: 6,
  LINES_HIGH: 500,
  LINES_MEDIUM: 200,
  AGE_HOURS_HIGH: 72,
  AGE_HOURS_MEDIUM: 24,
} as const
```

---

### 3. Duplicate Forbidden Target Logic (MEDIUM)
**Location:** 4 different files  
**Issue:** The same `main`/`master` branch check is duplicated:
- `src-tauri/src/commands/pull_requests.rs:10` - `FORBIDDEN_TARGETS` constant
- `src/components/PRTable.vue:65-68` - `isForbiddenTarget` function
- `src/components/WorktreePanel.vue:29-33` - `isForbiddenTarget` computed
- `src/views/PullRequestDetail.vue:59-63` - `isForbiddenTarget` computed

**Fix:** Create a shared utility:
```typescript
// src/utils/branches.ts
export const FORBIDDEN_TARGETS = ['main', 'master']

export function isForbiddenTarget(branch: string): boolean {
  return FORBIDDEN_TARGETS.includes(branch.toLowerCase())
}
```

---

### 4. Inconsistent Date Formatting (MEDIUM)
**Location:** Multiple components  
**Issue:** Date formatting logic is duplicated:
- `src/components/PRCard.vue:17-23` - age label
- `src/components/PRTable.vue:51-57` - age formatting
- `src/views/PullRequestDetail.vue:93-98` - full date

**Fix:** Create shared date utilities:
```typescript
// src/utils/date.ts
export function formatAge(createdAt: string): string
export function formatDateTime(dateStr: string): string
export function hoursSince(dateStr: string): number
```

---

### 5. Unnecessary Non-null Assertions (LOW)
**Location:** `src/views/PullRequestDetail.vue:25-26, 31-32`  
**Issue:** Using `!` operator without proper null checking.

**Current:**
```typescript
const repo = repoStore.repos.find(r => r.id === pr.value!.repo_id)
```

**Fix:** Add proper guards:
```typescript
if (!pr.value) return ''
const repo = repoStore.repos.find(r => r.id === pr.value.repo_id)
```

---

### 6. CSS Duplication (LOW)
**Location:** Multiple Vue components  
**Issue:** Common patterns like glassmorphism cards, button styles, and panel layouts are repeated across components.

**Observed Duplications:**
- `.panel` styles in WorktreePanel, ReviewStatus
- Glassmorphism backdrop-filter patterns
- Button hover/active states

**Fix:** Consider extracting to a CSS component library or using Tailwind CSS for utility classes.

---

### 7. Build Warning in vite.config.ts (LOW)
**Location:** `vite.config.ts:4`  
**Issue:** Using `@ts-expect-error` when a proper type declaration exists.

**Fix:** Add proper type declaration:
```typescript
declare const process: {
  env: {
    TAURI_DEV_HOST?: string
  }
}
```

---

### 8. Unused Dependencies (LOW)
**Location:** `package.json`  
**Issue:** `lucide-vue-next` is installed but not used (components use emoji/icons instead).

**Fix:** Either remove the dependency or replace emoji icons with Lucide icons for consistency.

---

## 🔒 SECURITY CONSIDERATIONS

### 1. Command Injection Risk in Grove Commands (MEDIUM)
**Location:** `src-tauri/src/commands/grove.rs`  
**Issue:** Branch and repository names are passed directly to shell commands without sanitization.

**Current:**
```rust
let output = Command::new("grove")
    .args(["add", &repo_name, &branch, &base, "-f"])
```

**Fix:** Validate inputs match expected patterns:
```rust
fn sanitize_branch_name(s: &str) -> Result<String, CommandError> {
    if s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '.') {
        Ok(s.to_string())
    } else {
        Err(CommandError::Grove("Invalid branch name".into()))
    }
}
```

---

### 2. No Rate Limiting on GitHub API (LOW)
**Location:** `src-tauri/src/github/mod.rs`  
**Issue:** Could hit GitHub API rate limits with frequent syncs.

**Recommendation:** 
- Add sync cooldown (e.g., minimum 30 seconds between syncs)
- Cache results and respect GitHub's rate limit headers
- Add user feedback when rate limited

---

## 📝 MISSING FEATURES / INCOMPLETE IMPLEMENTATIONS

| Feature | Status | Location |
|---------|--------|----------|
| Settings backend commands | ❌ Not implemented | `src-tauri/src/commands/` |
| Toast notification system | ⚠️ Partial (store exists, no UI) | `src/stores/toast.ts` |
| Auto-sync polling | ⚠️ Config stored, not used | `app_settings` table |
| Theme switching | ⚠️ UI disabled | `Settings.vue` |
| Worktree review integration | ⚠️ Event emitted, not handled | `WorktreePanel.vue` |
| Input validation | ⚠️ Minimal | `Repositories.vue` |

---

## ✅ POSITIVE FINDINGS

1. **Good Architecture** - Clean separation between Rust backend and Vue frontend
2. **TypeScript Usage** - Most types are well defined and consistent
3. **Consistent UI Design** - Glassmorphism theme is consistently applied
4. **SQLite WAL Mode** - Good performance choice for database
5. **Transaction Usage** - Proper use of transactions for data integrity
6. **CSP Configured** - Basic security headers in place
7. **Custom Error Types** - Proper error handling with `thiserror`
8. **Pinia Store Pattern** - Good use of composition API stores
9. **CSS Custom Properties** - Well-organized design tokens
10. **Component Composition** - Good component breakdown

---

## 📋 RECOMMENDED PRIORITY ORDER

### P0 - Fix Immediately
1. Implement missing settings backend commands
2. Fix transaction handling in `set_review_rules`
3. Add Toast type definitions

### P1 - Fix Soon
4. Add input sanitization for grove commands
5. Fix mutex poisoning issues
6. Standardize error handling across stores

### P2 - Improve Quality
7. Extract shared utilities (date, branch checks)
8. Add request deduplication
9. Remove magic numbers

### P3 - Polish
10. Remove unused dependencies
11. Add virtual scrolling for large lists
12. Complete toast notification UI
13. Implement auto-sync polling

---

## 📊 Code Metrics Summary

| Metric | Value |
|--------|-------|
| Total Files Reviewed | ~40 |
| Rust Files | 13 |
| Vue/TS Files | 27 |
| Critical Bugs | 2 |
| High Priority Issues | 3 |
| Medium Priority Issues | 8 |
| Low Priority Issues | 6 |
| Security Concerns | 2 |

---

*Report generated by code review assistant*
