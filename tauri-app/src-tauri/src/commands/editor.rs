use tauri::State;

use crate::db::DbState;

use super::CommandError;

#[tauri::command]
pub fn open_in_editor(path: String, state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = state.reader.lock().unwrap();
    let editor: String = db
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'editor_command'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "code".to_string());
    drop(db);

    std::process::Command::new(&editor)
        .arg(&path)
        .spawn()
        .map_err(|e| CommandError::Gh(format!("Failed to open editor '{}': {}", editor, e)))?;

    Ok(())
}
