use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::DiffFileSummary;

use super::CommandError;

/// Fetch just the file list (path + stats) for a PR without full diff content.
///
/// Uses `gh pr view --json files` which is much faster than fetching the full
/// unified diff. Useful for rendering the file tree immediately while deferring
/// full diff content loading to on-demand per-file expansion.
#[tauri::command]
pub async fn get_pr_file_list(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<DiffFileSummary>, CommandError> {
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

    github::fetch_pr_file_list_async(&full_name, number).await
}
