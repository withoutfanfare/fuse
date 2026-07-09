# Stability & Bug-Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the five defects found in the July 2026 code review — a data-corruption bug in PR sync, a panic that kills background polling, three reliability weaknesses — so Fuse is stable, reliable, and accurate.

**Architecture:** All changes are surgical fixes inside the existing Tauri v2 Rust backend (`tauri-app/src-tauri/src/`). No schema changes, no frontend changes, no new modules. Tests are added as `#[cfg(test)]` modules in the files they test, matching the existing convention (see `src/commands/issues.rs:106`).

**Tech Stack:** Rust (stable), rusqlite 0.32 with `bundled` feature (ships SQLite 3.46 — `RETURNING` is supported, it needs ≥ 3.35), Tauri v2. Tests run with `cargo test`.

## Global Constraints

- All work happens under `tauri-app/src-tauri/` — run every `cargo` command from that directory.
- British English in all comments and messages.
- Conventional commit format. **Never mention AI, Claude, or Claude Code in commit messages, and do not add Co-Authored-By trailers.**
- Work on a feature branch: `git checkout -b fix/stability-review-fixes develop` before Task 1. Never commit to `main`/`master`.
- Minimum change that solves each problem — do not refactor surrounding code, rename things, or touch formatting of untouched lines.
- Verification for every task: `cargo test` must pass and `cargo build` must succeed with no new warnings.

---

### Task 1: Fix UTF-8 panic in notification truncation (kills polling loop)

**The bug:** `truncate()` in `polling.rs` slices a string at a byte index (`&s[..max]`). Rust panics if a byte index falls inside a multi-byte character (emoji, accented letters). This function runs inside the background poll loop task (`polling.rs:59` → `send_change_notifications` → `format_notification`), so one PR title with an emoji near position 40/50 permanently kills background sync and notifications for the session.

**Files:**
- Modify: `tauri-app/src-tauri/src/polling.rs:169-176`
- Test: same file, new `#[cfg(test)] mod tests` at the bottom

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: `fn truncate(s: &str, max: usize) -> String` — same signature as today, now counts *characters* rather than bytes and never panics. No caller changes needed.

- [ ] **Step 1: Write the failing tests**

Append to the bottom of `tauri-app/src-tauri/src/polling.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("short title", 40), "short title");
    }

    #[test]
    fn truncate_long_ascii_appends_ellipsis() {
        let s = "a".repeat(50);
        assert_eq!(truncate(&s, 40), format!("{}…", "a".repeat(40)));
    }

    #[test]
    fn truncate_multibyte_does_not_panic() {
        // Each emoji is 4 bytes. The old code sliced at byte 40, which
        // lands mid-character and panics.
        let s = "🎉".repeat(60);
        assert_eq!(truncate(&s, 40), format!("{}…", "🎉".repeat(40)));
    }

    #[test]
    fn truncate_multibyte_at_exact_limit_unchanged() {
        let s = "é".repeat(40);
        assert_eq!(truncate(&s, 40), s);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run (from `tauri-app/src-tauri/`): `cargo test truncate`
Expected: `truncate_multibyte_does_not_panic` FAILS with a panic — `byte index 40 is not a char boundary`. The ASCII tests pass (the bug only bites multi-byte text).

- [ ] **Step 3: Fix the implementation**

Replace the existing function at `polling.rs:169-176`:

```rust
/// Truncate a string to a maximum length, appending ellipsis if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
```

with a character-boundary-safe version:

```rust
/// Truncate a string to a maximum number of characters, appending an
/// ellipsis if needed. Counts characters (not bytes) so multi-byte
/// text such as emoji never causes an out-of-boundary slice.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}…", truncated)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test truncate`
Expected: all 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tauri-app/src-tauri/src/polling.rs
git commit -m "fix: truncate notification titles on character boundaries

A byte-index slice panicked on multibyte titles (emoji, accents),
killing the background polling task until app restart."
```

---

### Task 2: Fix wrong PR id after upsert (corrupts reviewer assignments)

**The bug:** In `sync.rs`, after an `INSERT … ON CONFLICT DO UPDATE` upsert, the code reads `db.last_insert_rowid()` to get the PR's row id. SQLite does **not** set `last_insert_rowid()` when the upsert takes the UPDATE path — it keeps whatever value the last real INSERT left. In steady state the incremental sync (`fetch_and_upsert_prs_delta`) only processes already-known PRs, so the UPDATE path is taken every time and `pid` points at the wrong row. The following `DELETE`/`INSERT` on `pr_requested_reviewers` then deletes reviewers from one PR and attaches them to another. This corrupts the data behind `get_reviewer_workload` (`src/commands/workload.rs`). There is an inline comment at `sync.rs:575-576` claiming SQLite ≥ 3.35 updates the rowid on both paths — that comment is wrong and must be deleted.

**The fix:** Append `RETURNING id` to both upsert statements and read the id from the returned row via `query_row`. `RETURNING` works on both the INSERT and UPDATE paths and requires SQLite ≥ 3.35 (the bundled 3.46 qualifies).

**Files:**
- Modify: `tauri-app/src-tauri/src/commands/sync.rs` — two identical upsert sites:
  - `fetch_and_upsert_prs_delta`: statement at lines 223-284, `last_insert_rowid` at line 286
  - `fetch_and_upsert_prs`: statement at lines 512-573, comment at 575-576, `last_insert_rowid` at line 577
- Test: same file, new `#[cfg(test)] mod tests` at the bottom

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: no signature changes — `pid: i64` is still the variable the reviewer-junction code uses; only how it is obtained changes.

- [ ] **Step 1: Write the failing test**

This test proves the SQL pattern, and documents the `last_insert_rowid` trap that caused the bug. Append to the bottom of `tauri-app/src-tauri/src/commands/sync.rs`:

```rust
#[cfg(test)]
mod tests {
    /// `last_insert_rowid()` is NOT updated when an upsert takes the
    /// DO UPDATE path, so it must never be used to find the row an
    /// upsert touched. `RETURNING id` is correct on both paths.
    #[test]
    fn upsert_returning_yields_correct_id_on_update_path() {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE pull_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_id INTEGER NOT NULL,
                number INTEGER NOT NULL,
                title TEXT,
                UNIQUE(repo_id, number)
            );",
        )
        .unwrap();

        let upsert = "INSERT INTO pull_requests (repo_id, number, title)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(repo_id, number) DO UPDATE SET title = excluded.title
             RETURNING id";

        let id_a: i64 = db
            .query_row(upsert, rusqlite::params![1, 101, "PR A"], |r| r.get(0))
            .unwrap();
        let id_b: i64 = db
            .query_row(upsert, rusqlite::params![1, 102, "PR B"], |r| r.get(0))
            .unwrap();
        assert_ne!(id_a, id_b);

        // Re-upsert PR A (UPDATE path). RETURNING must give PR A's id back.
        let id_a_again: i64 = db
            .query_row(upsert, rusqlite::params![1, 101, "PR A v2"], |r| r.get(0))
            .unwrap();
        assert_eq!(id_a, id_a_again);

        // The trap this fix removes: last_insert_rowid() still points at
        // PR B after PR A's update.
        assert_eq!(db.last_insert_rowid(), id_b);
    }
}
```

- [ ] **Step 2: Run the test to verify it passes (it tests the new SQL pattern, which is valid already)**

Run: `cargo test upsert_returning`
Expected: PASS. (This test pins the SQL behaviour the fix relies on; the production-code fix follows.)

- [ ] **Step 3: Fix site 1 — `fetch_and_upsert_prs_delta` (lines 223-286)**

The current shape is:

```rust
            db.execute(
                r#"INSERT INTO pull_requests (
                    ...
                    last_synced_at = datetime('now')"#,
                rusqlite::params![ ... ],
            )?;

            let pid = db.last_insert_rowid();
```

Change it to run the same statement through `query_row` with `RETURNING id` appended. Concretely:

1. Change `db.execute(` to `let pid: i64 = db.query_row(`.
2. At the end of the SQL string, change the final line
   `last_synced_at = datetime('now')"#,`
   to
   `last_synced_at = datetime('now')
                RETURNING id"#,`
3. After the `rusqlite::params![ … ],` list, add a third argument `|row| row.get(0),` before the closing `)?;`.
4. Delete the line `let pid = db.last_insert_rowid();` (line 286).

The result (SQL body and params elided — they are unchanged):

```rust
            let pid: i64 = db.query_row(
                r#"INSERT INTO pull_requests (
                    ... unchanged columns, VALUES, ON CONFLICT clause ...
                    last_synced_at = datetime('now')
                RETURNING id"#,
                rusqlite::params![
                    // unchanged 23 parameters
                ],
                |row| row.get(0),
            )?;
```

The `DELETE FROM pr_requested_reviewers …` / `INSERT OR IGNORE …` block that follows keeps using `pid` unchanged.

- [ ] **Step 4: Fix site 2 — `fetch_and_upsert_prs` (lines 512-577)**

Apply exactly the same four edits to the second upsert. Additionally, **delete the incorrect comment** at lines 575-576:

```rust
            // Use last_insert_rowid() to get the internal PR id after upsert.
            // SQLite (≥ 3.35) sets this to the rowid regardless of insert/update path.
```

(Do not replace it — the `RETURNING id` in the SQL is self-explanatory.)

- [ ] **Step 5: Build and run all tests**

Run: `cargo build && cargo test`
Expected: clean build, all tests PASS (including Task 1's).

- [ ] **Step 6: Commit**

```bash
git add tauri-app/src-tauri/src/commands/sync.rs
git commit -m "fix: use RETURNING id instead of last_insert_rowid after PR upsert

last_insert_rowid() is not updated on the ON CONFLICT DO UPDATE path,
so incremental sync attached requested reviewers to the wrong PR,
corrupting reviewer workload data."
```

---

### Task 3: Stop a panicked worker thread aborting whole batch operations

**The bug:** Three places `join()` worker threads with `.expect(…)`, so a single panicking worker aborts the entire operation (and in a Tauri command, surfaces as an opaque error):
- `src/commands/batch.rs:112` (batch approve)
- `src/commands/batch.rs:218` (batch merge)
- `src/github/mod.rs:400` (deployment status fan-out)

**The fix:** Degrade per item. In `batch.rs`, a panicked worker becomes a failed `BatchResult` for that one PR (the handles are spawned from `contexts` in order, so zipping them back together is safe). In `github/mod.rs`, a panicked status worker is simply dropped from the deployment list.

**Files:**
- Modify: `tauri-app/src-tauri/src/commands/batch.rs:110-114` and `:216-220`
- Modify: `tauri-app/src-tauri/src/github/mod.rs:398-401`

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: no signature changes. Uses the existing `BatchResult { pr_id, success, message }` and `PrBatchContext { pr_id, … }` types already defined in `batch.rs`.

*No new unit test for this task:* exercising it would require injecting a panicking network call into `github::approve_pr`, which means refactoring for dependency injection — out of scope for a minimal fix. Verification is by compilation and the existing suite.

- [ ] **Step 1: Fix batch approve join (batch.rs:110-114)**

Replace:

```rust
        handles
            .into_iter()
            .map(|h| h.join().expect("Batch approve thread panicked"))
            .collect()
```

with:

```rust
        handles
            .into_iter()
            .zip(contexts.iter())
            .map(|(h, ctx)| {
                h.join().unwrap_or_else(|_| BatchResult {
                    pr_id: match ctx {
                        Ok(c) => c.pr_id,
                        Err(e) => e.pr_id,
                    },
                    success: false,
                    message: "Internal error: approve worker thread panicked".to_string(),
                })
            })
            .collect()
```

- [ ] **Step 2: Fix batch merge join (batch.rs:216-220)**

Replace:

```rust
        handles
            .into_iter()
            .map(|h| h.join().expect("Batch merge thread panicked"))
            .collect()
```

with:

```rust
        handles
            .into_iter()
            .zip(contexts.iter())
            .map(|(h, ctx)| {
                h.join().unwrap_or_else(|_| BatchResult {
                    pr_id: match ctx {
                        Ok(c) => c.pr_id,
                        Err(e) => e.pr_id,
                    },
                    success: false,
                    message: "Internal error: merge worker thread panicked".to_string(),
                })
            })
            .collect()
```

- [ ] **Step 3: Fix deployment status join (github/mod.rs:398-401)**

Replace:

```rust
        handles
            .into_iter()
            .map(|h| h.join().expect("Deployment status thread panicked"))
            .collect()
```

with:

```rust
        handles
            .into_iter()
            .filter_map(|h| h.join().ok())
            .collect()
```

- [ ] **Step 4: Build and test**

Run: `cargo build && cargo test`
Expected: clean build (no new warnings), all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add tauri-app/src-tauri/src/commands/batch.rs tauri-app/src-tauri/src/github/mod.rs
git commit -m "fix: degrade gracefully when a batch worker thread panics

A panicked worker now yields a per-item failure result instead of
aborting the whole batch approve/merge or deployment status fetch."
```

---

### Task 4: Prevent stdin pipe deadlock when sending prompts to the Claude CLI

**The bug:** `run_claude` in `src/commands/reviews.rs:116-153` writes the whole prompt to the child's stdin *before* reading stdout. If the prompt is large (review prompts embed full diffs) and the child fills its stdout pipe while we are still blocked writing stdin, both processes deadlock. Additionally, if `child.stdin` is `None` the prompt is silently dropped.

**The fix:** Write stdin from a separate thread while the main thread drains output with `wait_with_output()`, and treat a missing stdin handle as an error.

**Files:**
- Modify: `tauri-app/src-tauri/src/commands/reviews.rs:134-138` (and the checks that follow)

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: `fn run_claude(prompt: &str) -> Result<String, CommandError>` — unchanged signature.

*No new unit test for this task:* it requires a live `claude` binary and a pipe-buffer-sized prompt; not reproducible in a unit test without a fake-binary harness. Verification is by compilation plus a manual smoke test if the `claude` CLI is installed.

- [ ] **Step 1: Replace the stdin write**

In `run_claude`, replace lines 134-138:

```rust
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| CommandError::Claude(format!("Failed to write to claude stdin: {}", e)))?;
    }
```

with:

```rust
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| CommandError::Claude("Failed to open stdin for claude".to_string()))?;
    let prompt_owned = prompt.to_string();
    // Write stdin on a separate thread so a full stdout pipe cannot
    // deadlock against a blocked stdin write on large prompts.
    let stdin_writer = std::thread::spawn(move || stdin.write_all(prompt_owned.as_bytes()));
```

(`stdin` is moved into the thread and dropped when the write finishes, which closes the pipe and lets `claude` see end-of-input.)

- [ ] **Step 2: Check the writer result after the exit-status check**

Immediately *after* the existing `if !output.status.success() { … }` block (currently lines 144-150) and *before* `Ok(String::from_utf8_lossy(&output.stdout).to_string())`, insert:

```rust
    match stdin_writer.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return Err(CommandError::Claude(format!(
                "Failed to write to claude stdin: {}",
                e
            )))
        }
        Err(_) => {
            return Err(CommandError::Claude(
                "Claude stdin writer thread panicked".to_string(),
            ))
        }
    }
```

(Checked after the status check deliberately: if `claude` exits with an error, its stderr message is more useful than the broken-pipe write error that follows from it.)

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test`
Expected: clean build, all tests PASS.

- [ ] **Step 4: Commit**

```bash
git add tauri-app/src-tauri/src/commands/reviews.rs
git commit -m "fix: write claude stdin on a separate thread to avoid pipe deadlock

Large review prompts could deadlock against a full stdout pipe; a
missing stdin handle is now an error instead of silently dropping
the prompt."
```

---

### Task 5 (OPTIONAL — skip unless explicitly requested): Remove mutex poisoning as a failure amplifier

**The weakness:** All 107 `lock().unwrap()` call sites use `std::sync::Mutex`, which *poisons* if a thread panics while holding the lock — every later lock attempt then panics too, cascading one crash into a dead app. Tasks 1-3 remove the known panic sources, so this is defence in depth, not an active bug. Every `Mutex` in the crate guards a `rusqlite::Connection` (verified: `db/mod.rs:11-12` plus `&Mutex<Connection>` parameters in `sync.rs`), so a crate-wide swap to `parking_lot::Mutex` (which does not poison) is mechanical and the compiler catches any site missed.

**Files:**
- Modify: `tauri-app/src-tauri/Cargo.toml` (add dependency)
- Modify: every file with `use std::sync::Mutex` and every `.lock().unwrap()` site under `tauri-app/src-tauri/src/`

- [ ] **Step 1: Add the dependency**

In `tauri-app/src-tauri/Cargo.toml` under `[dependencies]` add:

```toml
parking_lot = "0.12"
```

- [ ] **Step 2: Swap the imports**

Find every std Mutex import and change it:

```bash
grep -rln "use std::sync::Mutex" src/
```

In each listed file replace `use std::sync::Mutex;` with `use parking_lot::Mutex;`. Where the import is combined (e.g. `use std::sync::{Arc, Mutex};`), keep the other items on the std import and add a separate `use parking_lot::Mutex;` line.

- [ ] **Step 3: Drop the unwraps**

`parking_lot`'s `lock()` returns the guard directly (no `Result`), so:

```bash
grep -rl "lock().unwrap()" src/ | xargs sed -i '' 's/\.lock()\.unwrap()/.lock()/g'
```

- [ ] **Step 4: Build — the compiler is the safety net**

Run: `cargo build && cargo test`
Expected: clean build and passing tests. Any mutex that was *not* a DB mutex would now fail to compile (a std `lock()` without `unwrap()` returns a `Result`, which won't type-check as a guard) — if that happens, revert that site to `std::sync::Mutex` with `.lock().unwrap()`.

- [ ] **Step 5: Commit**

```bash
git add tauri-app/src-tauri/Cargo.toml tauri-app/src-tauri/Cargo.lock tauri-app/src-tauri/src/
git commit -m "refactor: replace std Mutex with parking_lot for DB connections

parking_lot mutexes do not poison, so a panic in one thread can no
longer cascade into panics on every later database access."
```

---

## Final Verification (after all tasks)

- [ ] Run the full suite: `cargo test` from `tauri-app/src-tauri/` — all tests pass.
- [ ] Frontend build sanity check: `npm run build` from `tauri-app/` — succeeds (no frontend files were touched; this guards against accidental edits).
- [ ] Run the app: `npm run tauri dev` from `tauri-app/`, trigger a sync, and confirm the Reviewer Workload view populates and background sync keeps running.
- [ ] Merge: follow the repo's normal flow — merge `fix/stability-review-fixes` into `develop`. Never merge to `main` (human-only, hook-enforced).
