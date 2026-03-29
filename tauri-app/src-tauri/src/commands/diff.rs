use tauri::State;

use crate::db::DbState;
use crate::github;

use super::CommandError;

/// Fetch the unified diff for a pull request from GitHub.
///
/// Looks up the repository full name and PR number from the database,
/// then queries the GitHub CLI for the raw diff text.
/// Uses async subprocess execution to avoid blocking the thread pool.
#[tauri::command]
pub async fn fetch_pr_diff(pr_id: i64, state: State<'_, DbState>) -> Result<String, CommandError> {
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

    github::fetch_diff_async(&full_name, number).await
}

/// Fetch the unified diff for a single commit from GitHub.
///
/// Uses the GitHub API to retrieve the patch for a specific commit SHA,
/// returning the raw unified diff text that can be parsed by the frontend.
#[tauri::command]
pub async fn fetch_commit_diff(
    pr_id: i64,
    commit_oid: String,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let full_name = {
        let db = state.reader.lock().unwrap();

        let repo_id: i64 = db
            .query_row(
                "SELECT repo_id FROM pull_requests WHERE id = ?1",
                [pr_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    CommandError::NotFound(format!("Pull request with id {pr_id}"))
                }
                other => CommandError::Db(other),
            })?;

        db.query_row(
            "SELECT owner || '/' || name FROM repositories WHERE id = ?1",
            [repo_id],
            |row| row.get::<_, String>(0),
        )?
    };

    github::fetch_commit_diff_async(&full_name, &commit_oid).await
}

/// Fetch the unified diff for a range of commits from GitHub.
///
/// Uses the GitHub compare API to retrieve the diff between two commit SHAs.
#[tauri::command]
pub async fn fetch_commit_range_diff(
    pr_id: i64,
    base_oid: String,
    head_oid: String,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let full_name = {
        let db = state.reader.lock().unwrap();

        let repo_id: i64 = db
            .query_row(
                "SELECT repo_id FROM pull_requests WHERE id = ?1",
                [pr_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    CommandError::NotFound(format!("Pull request with id {pr_id}"))
                }
                other => CommandError::Db(other),
            })?;

        db.query_row(
            "SELECT owner || '/' || name FROM repositories WHERE id = ?1",
            [repo_id],
            |row| row.get::<_, String>(0),
        )?
    };

    github::fetch_commit_range_diff_async(&full_name, &base_oid, &head_oid).await
}
