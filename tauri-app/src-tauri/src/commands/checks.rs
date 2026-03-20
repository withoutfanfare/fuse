use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::CiCheck;

use super::CommandError;

/// Fetch CI/CD check statuses for a pull request from GitHub.
///
/// Looks up the repository full name and PR number from the database,
/// then queries the GitHub CLI for the latest check run results.
/// Uses async subprocess execution to avoid blocking the thread pool.
#[tauri::command]
pub async fn fetch_pr_checks(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<CiCheck>, CommandError> {
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

    github::fetch_checks_async(&full_name, number).await
}
