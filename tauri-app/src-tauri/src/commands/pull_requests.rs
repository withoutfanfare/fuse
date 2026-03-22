use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::{PullRequest, ReviewRule};

use super::CommandError;

/// Branches that PRs must never merge into.
const FORBIDDEN_TARGETS: &[&str] = &["main", "master"];

/// Helper to parse a PR row from a query that joins pull_requests with pr_reviews.
/// This parser works with `PR_SELECT` which omits `p.body` for efficiency.
pub fn parse_pr_row(row: &rusqlite::Row) -> Result<PullRequest, rusqlite::Error> {
    let labels_json: String = row.get(13)?;
    let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
    let is_draft_int: i64 = row.get(12)?;

    Ok(PullRequest {
        id: row.get(0)?,
        repo_id: row.get(1)?,
        number: row.get(2)?,
        title: row.get(3)?,
        author: row.get(4)?,
        state: row.get(5)?,
        head_branch: row.get(6)?,
        base_branch: row.get(7)?,
        additions: row.get(8)?,
        deletions: row.get(9)?,
        changed_files: row.get(10)?,
        review_decision: row.get(11)?,
        is_draft: is_draft_int != 0,
        url: row.get(14)?,
        labels,
        mergeable: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        merged_at: row.get(18)?,
        closed_at: row.get(19)?,
        body: None,
        last_synced_at: row.get(20)?,
        review_status: row.get(21)?,
        review_notes: row.get(22)?,
    })
}

/// Column list for the standard PR query with joined review status.
/// Omits `p.body` to reduce IPC payload on list queries — use
/// `PR_SELECT_WITH_BODY` or `get_pull_request_body` for detail views.
pub const PR_SELECT: &str = r#"
    p.id, p.repo_id, p.number, p.title, p.author,
    p.state, p.head_branch, p.base_branch,
    p.additions, p.deletions, p.changed_files,
    p.review_decision, p.is_draft, p.labels, p.url,
    p.mergeable, p.created_at, p.updated_at,
    p.merged_at, p.closed_at, p.last_synced_at,
    r.status, r.review_notes
"#;

/// Column list including `p.body` for detail / single-PR queries.
const PR_SELECT_WITH_BODY: &str = r#"
    p.id, p.repo_id, p.number, p.title, p.author,
    p.state, p.head_branch, p.base_branch,
    p.additions, p.deletions, p.changed_files,
    p.review_decision, p.is_draft, p.labels, p.url,
    p.mergeable, p.created_at, p.updated_at,
    p.merged_at, p.closed_at, p.body, p.last_synced_at,
    r.status, r.review_notes
"#;

/// Parse a PR row from a query that uses `PR_SELECT_WITH_BODY` (includes body).
fn parse_pr_row_with_body(row: &rusqlite::Row) -> Result<PullRequest, rusqlite::Error> {
    let labels_json: String = row.get(13)?;
    let labels: Vec<String> = serde_json::from_str(&labels_json).unwrap_or_default();
    let is_draft_int: i64 = row.get(12)?;

    Ok(PullRequest {
        id: row.get(0)?,
        repo_id: row.get(1)?,
        number: row.get(2)?,
        title: row.get(3)?,
        author: row.get(4)?,
        state: row.get(5)?,
        head_branch: row.get(6)?,
        base_branch: row.get(7)?,
        additions: row.get(8)?,
        deletions: row.get(9)?,
        changed_files: row.get(10)?,
        review_decision: row.get(11)?,
        is_draft: is_draft_int != 0,
        url: row.get(14)?,
        labels,
        mergeable: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        merged_at: row.get(18)?,
        closed_at: row.get(19)?,
        body: row.get(20)?,
        last_synced_at: row.get(21)?,
        review_status: row.get(22)?,
        review_notes: row.get(23)?,
    })
}

/// Look up the repo full name (owner/name), PR number, base branch, and draft status for a given PR id.
fn get_pr_context(
    db: &rusqlite::Connection,
    pr_id: i64,
) -> Result<(String, i64, String, bool), CommandError> {
    let (repo_id, number, base_branch, is_draft): (i64, i64, String, i64) = db
        .query_row(
            "SELECT repo_id, number, base_branch, is_draft FROM pull_requests WHERE id = ?1",
            [pr_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CommandError::NotFound(format!("Pull request with id {pr_id}"))
            }
            other => CommandError::Db(other),
        })?;

    let full_name: String = db.query_row(
        "SELECT owner || '/' || name FROM repositories WHERE id = ?1",
        [repo_id],
        |row| row.get(0),
    )?;

    Ok((full_name, number, base_branch, is_draft != 0))
}

#[tauri::command]
pub fn get_pull_requests(
    repo_id: Option<i64>,
    status_filter: Option<String>,
    state: State<'_, DbState>,
) -> Result<Vec<PullRequest>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut sql = format!(
        "SELECT {} FROM pull_requests p LEFT JOIN pr_reviews r ON r.pr_id = p.id WHERE 1=1",
        PR_SELECT
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(rid) = repo_id {
        sql.push_str(" AND p.repo_id = ?");
        params.push(Box::new(rid));
    }

    if let Some(ref filter) = status_filter {
        sql.push_str(" AND p.state = ?");
        params.push(Box::new(filter.to_uppercase()));
    }

    sql.push_str(" ORDER BY p.updated_at DESC");

    let mut stmt = db.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let prs = stmt
        .query_map(param_refs.as_slice(), parse_pr_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(prs)
}

#[tauri::command]
pub fn get_pull_request(id: i64, state: State<'_, DbState>) -> Result<PullRequest, CommandError> {
    let db = state.reader.lock().unwrap();
    let sql = format!(
        "SELECT {} FROM pull_requests p LEFT JOIN pr_reviews r ON r.pr_id = p.id WHERE p.id = ?1",
        PR_SELECT_WITH_BODY
    );
    let pr = db
        .query_row(&sql, [id], parse_pr_row_with_body)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CommandError::NotFound(format!("Pull request with id {id}"))
            }
            other => CommandError::Db(other),
        })?;
    Ok(pr)
}

#[tauri::command]
pub fn get_pull_request_body(
    id: i64,
    state: State<'_, DbState>,
) -> Result<Option<String>, CommandError> {
    let db = state.reader.lock().unwrap();
    let body: Option<String> = db
        .query_row(
            "SELECT body FROM pull_requests WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CommandError::NotFound(format!("Pull request with id {id}"))
            }
            other => CommandError::Db(other),
        })?;
    Ok(body)
}

#[tauri::command]
pub fn update_review_status(
    pr_id: i64,
    status: String,
    notes: Option<String>,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();

    // Determine whether to set reviewed_at
    let sets_reviewed = matches!(
        status.as_str(),
        "reviewed" | "approved" | "changes_requested"
    );

    if sets_reviewed {
        db.execute(
            r#"INSERT INTO pr_reviews (pr_id, status, review_notes, reviewed_at, updated_at)
               VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))
               ON CONFLICT(pr_id) DO UPDATE SET
                   status = excluded.status,
                   review_notes = COALESCE(excluded.review_notes, pr_reviews.review_notes),
                   reviewed_at = datetime('now'),
                   updated_at = datetime('now')"#,
            rusqlite::params![pr_id, status, notes],
        )?;
    } else {
        db.execute(
            r#"INSERT INTO pr_reviews (pr_id, status, review_notes, updated_at)
               VALUES (?1, ?2, ?3, datetime('now'))
               ON CONFLICT(pr_id) DO UPDATE SET
                   status = excluded.status,
                   review_notes = COALESCE(excluded.review_notes, pr_reviews.review_notes),
                   updated_at = datetime('now')"#,
            rusqlite::params![pr_id, status, notes],
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_review_rules(
    repo_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<ReviewRule>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, repo_id, rule_text, position FROM review_rules WHERE repo_id = ?1 ORDER BY position",
    )?;
    let rules = stmt
        .query_map([repo_id], |row| {
            Ok(ReviewRule {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                rule_text: row.get(2)?,
                position: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rules)
}

#[tauri::command]
pub fn set_review_rules(
    repo_id: i64,
    rules: Vec<String>,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();

    // Wrap delete + inserts in a transaction to prevent partial writes
    db.execute_batch("BEGIN")?;

    let result = (|| -> Result<(), CommandError> {
        db.execute("DELETE FROM review_rules WHERE repo_id = ?1", [repo_id])?;

        let mut stmt = db.prepare(
            "INSERT INTO review_rules (repo_id, rule_text, position) VALUES (?1, ?2, ?3)",
        )?;
        for (i, rule_text) in rules.iter().enumerate() {
            stmt.execute(rusqlite::params![repo_id, rule_text, i as i64])?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => {
            db.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

#[tauri::command]
pub fn approve_pull_request(
    pr_id: i64,
    body: Option<String>,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let db = state.writer.lock().unwrap();
    let (full_name, number, _, _) = get_pr_context(&db, pr_id)?;
    drop(db);

    let result = github::approve_pr(&full_name, number, body.as_deref())?;

    // Update local review status to approved
    let db = state.writer.lock().unwrap();
    db.execute(
        r#"INSERT INTO pr_reviews (pr_id, status, reviewed_at, updated_at)
           VALUES (?1, 'approved', datetime('now'), datetime('now'))
           ON CONFLICT(pr_id) DO UPDATE SET
               status = 'approved',
               reviewed_at = datetime('now'),
               updated_at = datetime('now')"#,
        [pr_id],
    )?;

    Ok(result)
}

#[tauri::command]
pub fn merge_pull_request(
    pr_id: i64,
    merge_method: Option<String>,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let db = state.writer.lock().unwrap();
    let (full_name, number, base_branch, is_draft) = get_pr_context(&db, pr_id)?;
    drop(db);

    // Enforce merge target protection
    let base_branch_lower = base_branch.to_lowercase();
    if FORBIDDEN_TARGETS.contains(&base_branch_lower.as_str()) {
        return Err(CommandError::Gh(format!(
            "Refused to merge: PR #{} targets '{}'. PRs must only merge into staging, never main or master.",
            number, base_branch
        )));
    }

    if is_draft {
        return Err(CommandError::Gh(
            "Cannot merge a draft pull request. Mark it ready for review on GitHub first."
                .to_string(),
        ));
    }

    let result = github::merge_pr(&full_name, number, merge_method.as_deref())?;

    // Update local state to reflect the merge
    let db = state.writer.lock().unwrap();
    db.execute(
        "UPDATE pull_requests SET state = 'MERGED', merged_at = datetime('now') WHERE id = ?1",
        [pr_id],
    )?;

    Ok(result)
}

/// Delegates to `log_review_time` so both the granular log and the
/// aggregate `pr_reviews` row are updated consistently.
#[tauri::command]
pub fn record_review_time(
    pr_id: i64,
    seconds: i64,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    super::time_tracking::log_review_time(pr_id, seconds, state)
}
