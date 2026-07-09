use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::{GhFileChange, GhPrJson, GhStatusCheck, PrChangeEvent, SyncResult};

use super::CommandError;

#[tauri::command]
pub fn sync_pull_requests(
    repo_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<SyncResult>, CommandError> {
    sync_pull_requests_with_db(repo_id, &state.writer)
}

/// Core sync logic, callable from both the Tauri command and the background
/// polling loop. Accepts a `&Mutex<Connection>` so it does not depend on
/// `tauri::State`.
pub(crate) fn sync_pull_requests_with_db(
    repo_id: Option<i64>,
    db_mutex: &Mutex<Connection>,
) -> Result<Vec<SyncResult>, CommandError> {
    let db = db_mutex.lock().unwrap();

    // Collect the repositories to sync
    let repos: Vec<(i64, String, String)> = if let Some(id) = repo_id {
        let mut stmt = db.prepare("SELECT id, owner, name FROM repositories WHERE id = ?1")?;
        let result = stmt
            .query_map([id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        result
    } else {
        let mut stmt = db.prepare("SELECT id, owner, name FROM repositories")?;
        let result = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        result
    };

    // Release the lock before calling external processes
    drop(db);

    let mut results = Vec::new();

    for (id, owner, name) in &repos {
        let full_name = format!("{}/{}", owner, name);
        let result = fetch_and_upsert_prs(*id, &full_name, db_mutex);
        results.push(result);
    }

    Ok(results)
}

/// Incremental sync: only fetch PRs updated since the last sync timestamp.
/// Falls back to full sync if no previous timestamp exists.
#[tauri::command]
pub fn sync_pull_requests_incremental(
    repo_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<SyncResult>, CommandError> {
    let db = state.writer.lock().unwrap();

    let repos: Vec<(i64, String, String, Option<String>)> = if let Some(id) = repo_id {
        let mut stmt = db.prepare(
            "SELECT id, owner, name, last_delta_sync_at FROM repositories WHERE id = ?1",
        )?;
        let result = stmt
            .query_map([id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        result
    } else {
        let mut stmt = db.prepare(
            "SELECT id, owner, name, last_delta_sync_at FROM repositories",
        )?;
        let result = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        result
    };

    drop(db);

    let mut results = Vec::new();

    for (id, owner, name, last_sync) in &repos {
        let full_name = format!("{}/{}", owner, name);
        let result = if let Some(ref since) = last_sync {
            fetch_and_upsert_prs_delta(*id, &full_name, since, &state.writer)
        } else {
            fetch_and_upsert_prs(*id, &full_name, &state.writer)
        };

        // Update the last_delta_sync_at timestamp on success
        if result.error.is_none() {
            let db = state.writer.lock().unwrap();
            let _ = db.execute(
                "UPDATE repositories SET last_delta_sync_at = datetime('now') WHERE id = ?1",
                [*id],
            );
        }

        results.push(result);
    }

    Ok(results)
}

/// Fetch only recently updated PRs and upsert them (delta sync).
fn fetch_and_upsert_prs_delta(
    repo_id: i64,
    full_name: &str,
    since: &str,
    db_mutex: &Mutex<Connection>,
) -> SyncResult {
    let synced_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let repo_name = full_name.to_string();

    // Snapshot existing PRs for change detection
    let existing_map: HashMap<i64, PrSnapshot> = {
        let db = db_mutex.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT number, state, updated_at FROM pull_requests WHERE repo_id = ?1")
            .unwrap();
        stmt.query_map([repo_id], |row| {
            Ok(PrSnapshot {
                number: row.get(0)?,
                state: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .map(|s| (s.number, s))
        .collect()
    };

    // Fetch only PRs updated since last sync
    let prs = match github::fetch_prs_since(full_name, since) {
        Ok(prs) => prs,
        Err(err) => {
            let db = db_mutex.lock().unwrap();
            let error_msg = err.to_string();
            db.execute(
                "INSERT INTO sync_log (repo_id, synced_at, pr_count, error) VALUES (?1, ?2, 0, ?3)",
                rusqlite::params![repo_id, synced_at, error_msg],
            )
            .ok();
            return SyncResult {
                repo_id,
                repo_name,
                synced_at,
                pr_count: 0,
                error: Some(error_msg),
                changes: Vec::new(),
            };
        }
    };

    let pr_count = prs.len() as i64;

    // Detect changes
    let mut changes: Vec<PrChangeEvent> = Vec::new();
    for pr in &prs {
        let state_upper = pr.state.to_uppercase();
        if let Some(existing) = existing_map.get(&pr.number) {
            let old_state = existing.state.to_uppercase();
            if old_state != state_upper {
                let change_type = if state_upper == "MERGED" || pr.merged_at.is_some() {
                    "merged"
                } else if state_upper == "CLOSED" {
                    "closed"
                } else if state_upper == "OPEN" && old_state == "CLOSED" {
                    "reopened"
                } else {
                    "updated"
                };
                changes.push(PrChangeEvent {
                    repo_name: repo_name.clone(),
                    pr_number: pr.number,
                    pr_title: pr.title.clone(),
                    author: pr.author.login.clone(),
                    change_type: change_type.to_string(),
                });
            } else if pr.updated_at != existing.updated_at {
                changes.push(PrChangeEvent {
                    repo_name: repo_name.clone(),
                    pr_number: pr.number,
                    pr_title: pr.title.clone(),
                    author: pr.author.login.clone(),
                    change_type: "updated".to_string(),
                });
            }
        } else {
            changes.push(PrChangeEvent {
                repo_name: repo_name.clone(),
                pr_number: pr.number,
                pr_title: pr.title.clone(),
                author: pr.author.login.clone(),
                change_type: "new".to_string(),
            });
        }
    }

    // Upsert the delta PRs
    let db = db_mutex.lock().unwrap();
    let upsert_result = (|| -> Result<(), CommandError> {
        db.execute_batch("BEGIN")?;
        for pr in &prs {
            let labels_json =
                serde_json::to_string(&pr.labels.iter().map(|l| &l.name).collect::<Vec<_>>())
                    .unwrap_or_else(|_| "[]".to_string());
            let label_colours_json = build_label_colours_json(pr);
            let ci_status = compute_ci_status(&pr.status_check_rollup);
            let file_paths_json = build_file_paths_json(&pr.files);
            let is_draft_int: i64 = if pr.is_draft { 1 } else { 0 };
            let state_upper = pr.state.to_uppercase();

            let pid: i64 = db.query_row(
                r#"INSERT INTO pull_requests (
                    repo_id, number, title, author, state,
                    head_branch, base_branch, additions, deletions, changed_files,
                    review_decision, is_draft, url, labels, label_colours,
                    mergeable, created_at, updated_at, merged_at, closed_at,
                    body, ci_status, changed_file_paths, last_synced_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20,
                    ?21, ?22, ?23, datetime('now')
                )
                ON CONFLICT(repo_id, number) DO UPDATE SET
                    title = excluded.title,
                    author = excluded.author,
                    state = excluded.state,
                    head_branch = excluded.head_branch,
                    base_branch = excluded.base_branch,
                    additions = excluded.additions,
                    deletions = excluded.deletions,
                    changed_files = excluded.changed_files,
                    review_decision = excluded.review_decision,
                    is_draft = excluded.is_draft,
                    url = excluded.url,
                    labels = excluded.labels,
                    label_colours = excluded.label_colours,
                    mergeable = excluded.mergeable,
                    updated_at = excluded.updated_at,
                    merged_at = excluded.merged_at,
                    closed_at = excluded.closed_at,
                    body = excluded.body,
                    ci_status = excluded.ci_status,
                    changed_file_paths = excluded.changed_file_paths,
                    last_synced_at = datetime('now')
                RETURNING id"#,
                rusqlite::params![
                    repo_id,
                    pr.number,
                    pr.title,
                    pr.author.login,
                    state_upper,
                    pr.head_ref_name,
                    pr.base_ref_name,
                    pr.additions,
                    pr.deletions,
                    pr.changed_files,
                    pr.review_decision,
                    is_draft_int,
                    pr.url,
                    labels_json,
                    label_colours_json,
                    pr.mergeable,
                    pr.created_at,
                    pr.updated_at,
                    pr.merged_at,
                    pr.closed_at,
                    pr.body,
                    ci_status,
                    file_paths_json,
                ],
                |row| row.get(0),
            )?;

            db.execute(
                "DELETE FROM pr_requested_reviewers WHERE pr_id = ?1",
                rusqlite::params![pid],
            )?;
            for req in &pr.review_requests {
                let reviewer_name = req.login.as_deref().or(req.name.as_deref());
                if let Some(name) = reviewer_name {
                    db.execute(
                        "INSERT OR IGNORE INTO pr_requested_reviewers (pr_id, reviewer, assigned_at) VALUES (?1, ?2, ?3)",
                        rusqlite::params![pid, name, pr.created_at],
                    )?;
                }
            }
        }

        db.execute(
            "INSERT INTO sync_log (repo_id, synced_at, pr_count) VALUES (?1, ?2, ?3)",
            rusqlite::params![repo_id, synced_at, pr_count],
        )?;

        Ok(())
    })();

    match upsert_result {
        Ok(()) => {
            let _ = db.execute_batch("COMMIT");
            SyncResult {
                repo_id,
                repo_name,
                synced_at,
                pr_count,
                error: None,
                changes,
            }
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            SyncResult {
                repo_id,
                repo_name,
                synced_at,
                pr_count: 0,
                error: Some(e.to_string()),
                changes: Vec::new(),
            }
        }
    }
}

/// Snapshot of a PR's state before sync, used to detect changes.
struct PrSnapshot {
    number: i64,
    state: String,
    updated_at: String,
}

/// Build a JSON array of file paths from the gh CLI files field.
fn build_file_paths_json(files: &[GhFileChange]) -> String {
    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    serde_json::to_string(&paths).unwrap_or_else(|_| "[]".to_string())
}

/// Build a JSON map of label name → hex colour from the gh CLI labels.
fn build_label_colours_json(pr: &GhPrJson) -> String {
    let map: HashMap<String, String> = pr
        .labels
        .iter()
        .filter_map(|l| l.color.as_ref().map(|c| (l.name.clone(), c.clone())))
        .collect();
    serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
}

/// Compute a rollup CI status from the statusCheckRollup array.
/// Returns "passing", "failing", "pending", or None if no checks.
fn compute_ci_status(checks: &[GhStatusCheck]) -> Option<String> {
    if checks.is_empty() {
        return None;
    }

    let mut has_failure = false;
    let mut has_pending = false;

    for check in checks {
        // GitHub uses either `conclusion` (for completed checks) or `state`/`status`
        let conclusion = check.conclusion.as_deref().unwrap_or("");
        let state = check.state.as_deref().unwrap_or("");
        let status = check.status.as_deref().unwrap_or("");

        if conclusion == "FAILURE"
            || conclusion == "failure"
            || state == "FAILURE"
            || state == "ERROR"
        {
            has_failure = true;
        } else if conclusion.is_empty()
            && (state == "PENDING"
                || state == "pending"
                || status == "IN_PROGRESS"
                || status == "QUEUED"
                || status == "WAITING"
                || status == "in_progress"
                || status == "queued")
        {
            has_pending = true;
        }
    }

    if has_failure {
        Some("failing".to_string())
    } else if has_pending {
        Some("pending".to_string())
    } else {
        Some("passing".to_string())
    }
}

/// Fetch PRs from GitHub via the gh CLI and upsert them into the database.
fn fetch_and_upsert_prs(repo_id: i64, full_name: &str, db_mutex: &Mutex<Connection>) -> SyncResult {
    let synced_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let repo_name = full_name.to_string();

    // Snapshot existing PR states before sync so we can detect changes
    let existing_map: HashMap<i64, PrSnapshot> = {
        let db = db_mutex.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT number, state, updated_at FROM pull_requests WHERE repo_id = ?1")
            .unwrap();
        stmt.query_map([repo_id], |row| {
            Ok(PrSnapshot {
                number: row.get(0)?,
                state: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .map(|s| (s.number, s))
        .collect()
    };

    // Fetch PRs using the gh CLI (no DB lock held during network call)
    let prs = match github::fetch_prs(full_name) {
        Ok(prs) => prs,
        Err(err) => {
            let db = db_mutex.lock().unwrap();
            let error_msg = err.to_string();
            db.execute(
                "INSERT INTO sync_log (repo_id, synced_at, pr_count, error) VALUES (?1, ?2, 0, ?3)",
                rusqlite::params![repo_id, synced_at, error_msg],
            )
            .ok();
            return SyncResult {
                repo_id,
                repo_name,
                synced_at,
                pr_count: 0,
                error: Some(error_msg),
                changes: Vec::new(),
            };
        }
    };

    let pr_count = prs.len() as i64;

    // Detect changes by comparing fetched PRs against existing snapshots
    let mut changes: Vec<PrChangeEvent> = Vec::new();
    for pr in &prs {
        let state_upper = pr.state.to_uppercase();
        if let Some(existing) = existing_map.get(&pr.number) {
            // Existing PR — check for state changes
            let old_state = existing.state.to_uppercase();
            if old_state != state_upper {
                let change_type = if state_upper == "MERGED" || pr.merged_at.is_some() {
                    "merged"
                } else if state_upper == "CLOSED" {
                    "closed"
                } else if state_upper == "OPEN" && old_state == "CLOSED" {
                    "reopened"
                } else {
                    "updated"
                };
                changes.push(PrChangeEvent {
                    repo_name: repo_name.clone(),
                    pr_number: pr.number,
                    pr_title: pr.title.clone(),
                    author: pr.author.login.clone(),
                    change_type: change_type.to_string(),
                });
            } else if pr.updated_at != existing.updated_at {
                changes.push(PrChangeEvent {
                    repo_name: repo_name.clone(),
                    pr_number: pr.number,
                    pr_title: pr.title.clone(),
                    author: pr.author.login.clone(),
                    change_type: "updated".to_string(),
                });
            }
        } else {
            // New PR
            changes.push(PrChangeEvent {
                repo_name: repo_name.clone(),
                pr_number: pr.number,
                pr_title: pr.title.clone(),
                author: pr.author.login.clone(),
                change_type: "new".to_string(),
            });
        }
    }

    // Lock the DB, begin a transaction, upsert all PRs, write sync log, commit
    let db = db_mutex.lock().unwrap();

    let upsert_result = (|| -> Result<(), CommandError> {
        db.execute_batch("BEGIN")?;
        for pr in &prs {
            let labels_json =
                serde_json::to_string(&pr.labels.iter().map(|l| &l.name).collect::<Vec<_>>())
                    .unwrap_or_else(|_| "[]".to_string());
            let label_colours_json = build_label_colours_json(pr);
            let ci_status = compute_ci_status(&pr.status_check_rollup);
            let file_paths_json = build_file_paths_json(&pr.files);
            let is_draft_int: i64 = if pr.is_draft { 1 } else { 0 };
            // Normalise state to uppercase so stats queries work consistently
            let state_upper = pr.state.to_uppercase();

            let pid: i64 = db.query_row(
                r#"INSERT INTO pull_requests (
                    repo_id, number, title, author, state,
                    head_branch, base_branch, additions, deletions, changed_files,
                    review_decision, is_draft, url, labels, label_colours,
                    mergeable, created_at, updated_at, merged_at, closed_at,
                    body, ci_status, changed_file_paths, last_synced_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20,
                    ?21, ?22, ?23, datetime('now')
                )
                ON CONFLICT(repo_id, number) DO UPDATE SET
                    title = excluded.title,
                    author = excluded.author,
                    state = excluded.state,
                    head_branch = excluded.head_branch,
                    base_branch = excluded.base_branch,
                    additions = excluded.additions,
                    deletions = excluded.deletions,
                    changed_files = excluded.changed_files,
                    review_decision = excluded.review_decision,
                    is_draft = excluded.is_draft,
                    url = excluded.url,
                    labels = excluded.labels,
                    label_colours = excluded.label_colours,
                    mergeable = excluded.mergeable,
                    updated_at = excluded.updated_at,
                    merged_at = excluded.merged_at,
                    closed_at = excluded.closed_at,
                    body = excluded.body,
                    ci_status = excluded.ci_status,
                    changed_file_paths = excluded.changed_file_paths,
                    last_synced_at = datetime('now')
                RETURNING id"#,
                rusqlite::params![
                    repo_id,
                    pr.number,
                    pr.title,
                    pr.author.login,
                    state_upper,
                    pr.head_ref_name,
                    pr.base_ref_name,
                    pr.additions,
                    pr.deletions,
                    pr.changed_files,
                    pr.review_decision,
                    is_draft_int,
                    pr.url,
                    labels_json,
                    label_colours_json,
                    pr.mergeable,
                    pr.created_at,
                    pr.updated_at,
                    pr.merged_at,
                    pr.closed_at,
                    pr.body,
                    ci_status,
                    file_paths_json,
                ],
                |row| row.get(0),
            )?;

            {
                // Clear existing reviewer assignments for this PR, then re-insert
                db.execute(
                    "DELETE FROM pr_requested_reviewers WHERE pr_id = ?1",
                    rusqlite::params![pid],
                )?;
                for req in &pr.review_requests {
                    // Prefer `login` (user review request), fall back to `name` (team request)
                    let reviewer_name = req.login.as_deref().or(req.name.as_deref());
                    if let Some(name) = reviewer_name {
                        db.execute(
                            "INSERT OR IGNORE INTO pr_requested_reviewers (pr_id, reviewer, assigned_at) VALUES (?1, ?2, ?3)",
                            rusqlite::params![pid, name, pr.created_at],
                        )?;
                    }
                }
            }
        }

        // Record sync in the log
        db.execute(
            "INSERT INTO sync_log (repo_id, synced_at, pr_count) VALUES (?1, ?2, ?3)",
            rusqlite::params![repo_id, synced_at, pr_count],
        )?;

        // Prune old sync_log entries — keep only the last 100 per repo
        db.execute(
            "DELETE FROM sync_log WHERE repo_id = ?1 AND id NOT IN (
                SELECT id FROM sync_log WHERE repo_id = ?1 ORDER BY synced_at DESC LIMIT 100
            )",
            rusqlite::params![repo_id],
        )?;

        Ok(())
    })();

    match upsert_result {
        Ok(()) => {
            let _ = db.execute_batch("COMMIT");
            SyncResult {
                repo_id,
                repo_name,
                synced_at,
                pr_count,
                error: None,
                changes,
            }
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            SyncResult {
                repo_id,
                repo_name,
                synced_at,
                pr_count: 0,
                error: Some(e.to_string()),
                changes: Vec::new(),
            }
        }
    }
}

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
