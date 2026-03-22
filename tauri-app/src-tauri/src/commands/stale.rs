use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::PullRequest;

use super::pull_requests::{parse_pr_row, PR_SELECT};
use super::CommandError;

/// Parse an ISO 8601 timestamp string into a Unix timestamp (seconds).
fn parse_iso_timestamp(s: &str) -> i64 {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0)
}

/// A stale review-requested PR with escalation metadata.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaleReviewItem {
    pub pr: PullRequest,
    /// Hours since review was requested (based on PR created_at for review-requested PRs).
    pub hours_waiting: f64,
    /// Escalation level: 1 = at threshold, 2 = at 2x threshold, 3 = daily after that.
    pub escalation_level: u32,
    /// Whether the user has started reviewing (has checklist progress or time tracked).
    pub has_local_progress: bool,
}

/// Fetch all open pull requests that have not been updated within the
/// configured stale threshold (stored in `app_settings` as `stale_threshold_days`).
#[tauri::command]
pub fn get_stale_prs(state: State<'_, DbState>) -> Result<Vec<PullRequest>, CommandError> {
    let db = state.reader.lock().unwrap();

    let threshold_days: String = db
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'stale_threshold_days'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "14".to_string());

    let sql = format!(
        "SELECT {} FROM pull_requests p \
         LEFT JOIN pr_reviews r ON r.pr_id = p.id \
         WHERE p.state = 'OPEN' \
         AND p.updated_at <= datetime('now', '-' || ?1 || ' days') \
         ORDER BY p.updated_at ASC",
        PR_SELECT
    );

    let mut stmt = db.prepare(&sql)?;
    let prs = stmt
        .query_map([&threshold_days], parse_pr_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(prs)
}

/// Fetch open PRs where a review was requested but the user has not acted within
/// the configured threshold. Returns items sorted by waiting time (longest first)
/// with escalation levels and local progress detection.
///
/// Stale review threshold is read from `app_settings` key `stale_review_hours`
/// (default: 24). A PR is considered review-requested if `review_decision` is
/// 'REVIEW_REQUIRED' or it has entries in `pr_requested_reviewers`.
///
/// Escalation: level 1 at threshold, level 2 at 2× threshold, level 3+ daily after.
/// PRs with local checklist progress or review time logged are flagged but still
/// returned (frontend suppresses reminders for those).
#[tauri::command]
pub fn get_stale_review_requests(
    state: State<'_, DbState>,
) -> Result<Vec<StaleReviewItem>, CommandError> {
    let db = state.reader.lock().unwrap();

    let threshold_hours: f64 = db
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'stale_review_hours'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24.0);

    // Find open PRs that are review-requested (by review_decision or pr_requested_reviewers)
    let sql = format!(
        "SELECT {pr_select} FROM pull_requests p \
         LEFT JOIN pr_reviews r ON r.pr_id = p.id \
         WHERE p.state = 'OPEN' \
         AND p.merged_at IS NULL \
         AND p.closed_at IS NULL \
         AND (p.review_decision = 'REVIEW_REQUIRED' \
              OR EXISTS (SELECT 1 FROM pr_requested_reviewers WHERE pr_id = p.id)) \
         ORDER BY p.created_at ASC",
        pr_select = PR_SELECT
    );

    let mut stmt = db.prepare(&sql)?;
    let prs: Vec<PullRequest> = stmt
        .query_map([], parse_pr_row)?
        .collect::<Result<Vec<_>, _>>()?;

    let mut items = Vec::new();

    for pr in prs {
        let hours_waiting =
            (Utc::now().timestamp() - parse_iso_timestamp(&pr.created_at)) as f64
                / 3600.0;

        if hours_waiting < threshold_hours {
            continue;
        }

        let escalation_level = if hours_waiting < threshold_hours * 2.0 {
            1
        } else if hours_waiting < threshold_hours * 3.0 {
            2
        } else {
            3
        };

        // Check for local progress: checklist state or review time logged
        let has_checklist: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM checklist_state WHERE pr_id = ?1 AND state_json != '{}')",
                [pr.id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let has_time: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM review_time_log WHERE pr_id = ?1 AND duration_seconds > 0)",
                [pr.id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        items.push(StaleReviewItem {
            pr,
            hours_waiting,
            escalation_level,
            has_local_progress: has_checklist || has_time,
        });
    }

    Ok(items)
}

/// Save or update a review session snapshot for auto-save functionality.
/// Creates a new session if none exists, or updates the existing active session.
#[tauri::command]
pub fn save_session_snapshot(
    pr_id: i64,
    files_reviewed: Vec<String>,
    session_notes: Option<String>,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let files_json = serde_json::to_string(&files_reviewed)?;

    // Try to update existing active session
    let updated = db.execute(
        "UPDATE review_sessions SET files_reviewed = ?1, session_notes = ?2 \
         WHERE pr_id = ?3 AND status IN ('active', 'paused')",
        rusqlite::params![files_json, session_notes, pr_id],
    )?;

    if updated == 0 {
        // No active session — create one
        db.execute(
            "INSERT INTO review_sessions (pr_id, files_reviewed, session_notes, status) \
             VALUES (?1, ?2, ?3, 'active')",
            rusqlite::params![pr_id, files_json, session_notes],
        )?;
    }

    Ok(())
}

/// Clean up stale review sessions older than 7 days for closed/merged PRs.
#[tauri::command]
pub fn cleanup_stale_sessions(state: State<'_, DbState>) -> Result<u64, CommandError> {
    let db = state.writer.lock().unwrap();

    // Complete sessions for PRs that are no longer open
    let completed = db.execute(
        "UPDATE review_sessions SET status = 'completed' \
         WHERE status IN ('active', 'paused') \
         AND pr_id IN (SELECT id FROM pull_requests WHERE state != 'OPEN')",
        [],
    )?;

    // Delete sessions older than 7 days that are completed
    let deleted = db.execute(
        "DELETE FROM review_sessions \
         WHERE status = 'completed' \
         AND started_at <= datetime('now', '-7 days')",
        [],
    )?;

    Ok((completed + deleted) as u64)
}

/// Close a pull request on GitHub and update its local state.
#[tauri::command]
pub fn close_pull_request(pr_id: i64, state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();

    let (repo_id, number): (i64, i64) = db
        .query_row(
            "SELECT repo_id, number FROM pull_requests WHERE id = ?1",
            [pr_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
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

    drop(db);

    github::close_pr(&full_name, number)?;

    // Update local state to reflect closure
    let db = state.writer.lock().unwrap();
    db.execute(
        "UPDATE pull_requests SET state = 'CLOSED', closed_at = datetime('now') WHERE id = ?1",
        [pr_id],
    )?;

    Ok(())
}
