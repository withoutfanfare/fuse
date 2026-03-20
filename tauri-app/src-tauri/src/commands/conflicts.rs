use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::ConflictStatus;

use super::CommandError;

/// Check whether a pull request has merge conflicts.
///
/// Looks up the repository full name and PR number from the database,
/// then queries the GitHub CLI for the current mergeable state.
/// Uses async subprocess execution to avoid blocking the thread pool.
#[tauri::command]
pub async fn check_merge_conflicts(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<ConflictStatus, CommandError> {
    // Read DB state first, then drop the guard before awaiting
    let (full_name, number) = {
        let db = state.reader.lock().unwrap();

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

        (full_name, number)
    };

    github::fetch_conflict_status_async(&full_name, number).await
}
