use std::collections::HashMap;

use tauri::State;

use crate::db::DbState;

use super::CommandError;

#[tauri::command]
pub fn get_settings(state: State<'_, DbState>) -> Result<HashMap<String, String>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare("SELECT key, value FROM app_settings")?;
    let settings = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<HashMap<String, String>, _>>()?;
    Ok(settings)
}

#[tauri::command]
pub fn update_setting(
    key: String,
    value: String,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}
