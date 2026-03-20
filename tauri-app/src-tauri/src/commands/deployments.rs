use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::Deployment;

use super::CommandError;

/// Fetch deployment statuses for a pull request's branch.
///
/// Looks up the repository full name and head branch from the database,
/// then queries the GitHub Deployments API filtering by the PR branch.
/// Uses async subprocess execution to avoid blocking the thread pool.
#[tauri::command]
pub async fn get_deployment_status(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<Deployment>, CommandError> {
    // Read DB state first, then drop the guard before awaiting
    let (full_name, head_branch) = {
        let db = state.reader.lock().unwrap();

        let (repo_id, head_branch): (i64, String) = db
            .query_row(
                "SELECT repo_id, head_branch FROM pull_requests WHERE id = ?1",
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

        (full_name, head_branch)
    };

    github::fetch_deployments_async(&full_name, &head_branch).await
}
