use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::CommitInfo;

use super::CommandError;

/// Fetch the commit history for a pull request from GitHub.
///
/// Looks up the repository full name and PR number from the database,
/// then queries the GitHub CLI for the list of commits in the PR.
#[tauri::command]
pub fn get_pr_commits(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<CommitInfo>, CommandError> {
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

    drop(db);

    github::fetch_commits(&full_name, number)
}
