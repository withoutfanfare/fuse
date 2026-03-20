use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// Retrieve the persisted checklist state JSON for a given pull request.
#[tauri::command]
pub fn get_checklist_state(pr_id: i64, state: State<'_, DbState>) -> Result<String, CommandError> {
    let db = state.reader.lock().unwrap();
    let result = db.query_row(
        "SELECT state_json FROM checklist_state WHERE pr_id = ?1",
        [pr_id],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(json) => Ok(json),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok("{}".to_string()),
        Err(e) => Err(CommandError::Db(e)),
    }
}

/// Save the checklist state JSON for a given pull request (upsert).
#[tauri::command]
pub fn save_checklist_state(
    pr_id: i64,
    state_json: String,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute(
        "INSERT INTO checklist_state (pr_id, state_json, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(pr_id) DO UPDATE SET state_json = ?2, updated_at = datetime('now')",
        rusqlite::params![pr_id, state_json],
    )?;
    Ok(())
}
