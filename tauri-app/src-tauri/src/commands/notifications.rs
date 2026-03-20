use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// A notification rule defining when to trigger alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRule {
    pub id: i64,
    pub rule_type: String,
    pub rule_config: serde_json::Value,
    pub enabled: bool,
    pub created_at: String,
}

#[tauri::command]
pub fn list_notification_rules(
    state: State<'_, DbState>,
) -> Result<Vec<NotificationRule>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, rule_type, rule_config, enabled, created_at FROM notification_rules ORDER BY id",
    )?;
    let rules = stmt
        .query_map([], |row| {
            let config_str: String = row.get(2)?;
            let config: serde_json::Value =
                serde_json::from_str(&config_str).unwrap_or(serde_json::Value::Object(Default::default()));
            let enabled_int: i64 = row.get(3)?;
            Ok(NotificationRule {
                id: row.get(0)?,
                rule_type: row.get(1)?,
                rule_config: config,
                enabled: enabled_int != 0,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rules)
}

#[tauri::command]
pub fn create_notification_rule(
    rule_type: String,
    rule_config: serde_json::Value,
    state: State<'_, DbState>,
) -> Result<NotificationRule, CommandError> {
    let db = state.writer.lock().unwrap();
    let config_str = serde_json::to_string(&rule_config)?;
    db.execute(
        "INSERT INTO notification_rules (rule_type, rule_config) VALUES (?1, ?2)",
        rusqlite::params![rule_type, config_str],
    )?;
    let id = db.last_insert_rowid();
    let created_at: String = db.query_row(
        "SELECT created_at FROM notification_rules WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(NotificationRule {
        id,
        rule_type,
        rule_config,
        enabled: true,
        created_at,
    })
}

#[tauri::command]
pub fn delete_notification_rule(
    id: i64,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute("DELETE FROM notification_rules WHERE id = ?1", [id])?;
    Ok(())
}

#[tauri::command]
pub fn toggle_notification_rule(
    id: i64,
    enabled: bool,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let enabled_int: i64 = if enabled { 1 } else { 0 };
    db.execute(
        "UPDATE notification_rules SET enabled = ?1 WHERE id = ?2",
        rusqlite::params![enabled_int, id],
    )?;
    Ok(())
}
