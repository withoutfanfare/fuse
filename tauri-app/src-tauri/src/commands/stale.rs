use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::PullRequest;

use super::pull_requests::{parse_pr_row, PR_SELECT};
use super::CommandError;

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
