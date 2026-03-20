use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// Post a comment on a pull request via the gh CLI.
#[tauri::command]
pub fn post_review_summary(
    pr_id: i64,
    body: String,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
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

    let output = super::command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "comment",
            &number.to_string(),
            "--repo",
            &full_name,
            "--body",
            &body,
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!("gh pr comment failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
