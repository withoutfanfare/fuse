use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::{PrChangeEvent, SyncResult};

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
            let is_draft_int: i64 = if pr.is_draft { 1 } else { 0 };
            let state_upper = pr.state.to_uppercase();

            db.execute(
                r#"INSERT INTO pull_requests (
                    repo_id, number, title, author, state,
                    head_branch, base_branch, additions, deletions, changed_files,
                    review_decision, is_draft, url, labels, mergeable,
                    created_at, updated_at, merged_at, closed_at, body, last_synced_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20, datetime('now')
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
                    mergeable = excluded.mergeable,
                    updated_at = excluded.updated_at,
                    merged_at = excluded.merged_at,
                    closed_at = excluded.closed_at,
                    body = excluded.body,
                    last_synced_at = datetime('now')"#,
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
                    pr.mergeable,
                    pr.created_at,
                    pr.updated_at,
                    pr.merged_at,
                    pr.closed_at,
                    pr.body,
                ],
            )?;

            let pid = db.last_insert_rowid();
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
            let is_draft_int: i64 = if pr.is_draft { 1 } else { 0 };
            // Normalise state to uppercase so stats queries work consistently
            let state_upper = pr.state.to_uppercase();

            db.execute(
                r#"INSERT INTO pull_requests (
                    repo_id, number, title, author, state,
                    head_branch, base_branch, additions, deletions, changed_files,
                    review_decision, is_draft, url, labels, mergeable,
                    created_at, updated_at, merged_at, closed_at, body, last_synced_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20, datetime('now')
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
                    mergeable = excluded.mergeable,
                    updated_at = excluded.updated_at,
                    merged_at = excluded.merged_at,
                    closed_at = excluded.closed_at,
                    body = excluded.body,
                    last_synced_at = datetime('now')"#,
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
                    pr.mergeable,
                    pr.created_at,
                    pr.updated_at,
                    pr.merged_at,
                    pr.closed_at,
                    pr.body,
                ],
            )?;

            // Use last_insert_rowid() to get the internal PR id after upsert.
            // SQLite (≥ 3.35) sets this to the rowid regardless of insert/update path.
            let pid = db.last_insert_rowid();

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
