use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// A saved filter preset with a name and filter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    pub id: i64,
    pub name: String,
    pub is_builtin: bool,
    pub filter_config: serde_json::Value,
    pub created_at: String,
}

#[tauri::command]
pub fn list_filter_presets(
    state: State<'_, DbState>,
) -> Result<Vec<FilterPreset>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, name, is_builtin, filter_config, created_at FROM filter_presets ORDER BY is_builtin DESC, id",
    )?;
    let presets = stmt
        .query_map([], |row| {
            let config_str: String = row.get(3)?;
            let config: serde_json::Value =
                serde_json::from_str(&config_str).unwrap_or(serde_json::Value::Object(Default::default()));
            let builtin_int: i64 = row.get(2)?;
            Ok(FilterPreset {
                id: row.get(0)?,
                name: row.get(1)?,
                is_builtin: builtin_int != 0,
                filter_config: config,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(presets)
}

#[tauri::command]
pub fn create_filter_preset(
    name: String,
    filter_config: serde_json::Value,
    state: State<'_, DbState>,
) -> Result<FilterPreset, CommandError> {
    let db = state.writer.lock().unwrap();
    let config_str = serde_json::to_string(&filter_config)?;
    db.execute(
        "INSERT INTO filter_presets (name, is_builtin, filter_config) VALUES (?1, 0, ?2)",
        rusqlite::params![name, config_str],
    )?;
    let id = db.last_insert_rowid();
    let created_at: String = db.query_row(
        "SELECT created_at FROM filter_presets WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(FilterPreset {
        id,
        name,
        is_builtin: false,
        filter_config,
        created_at,
    })
}

#[tauri::command]
pub fn delete_filter_preset(
    id: i64,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    // Only allow deleting non-builtin presets
    db.execute(
        "DELETE FROM filter_presets WHERE id = ?1 AND is_builtin = 0",
        [id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn rename_filter_preset(
    id: i64,
    name: String,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute(
        "UPDATE filter_presets SET name = ?1 WHERE id = ?2 AND is_builtin = 0",
        rusqlite::params![name, id],
    )?;
    Ok(())
}
