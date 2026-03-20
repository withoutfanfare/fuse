use tauri::State;

use crate::db::DbState;
use crate::models::ReviewTemplate;

use super::CommandError;

#[tauri::command]
pub fn list_templates(state: State<'_, DbState>) -> Result<Vec<ReviewTemplate>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt =
        db.prepare("SELECT id, name, body, position FROM review_templates ORDER BY position")?;
    let templates = stmt
        .query_map([], |row| {
            Ok(ReviewTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                body: row.get(2)?,
                position: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(templates)
}

#[tauri::command]
pub fn set_templates(
    templates: Vec<ReviewTemplate>,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();

    // Wrap delete + inserts in a transaction to prevent partial writes
    db.execute_batch("BEGIN")?;

    let result = (|| -> Result<(), CommandError> {
        db.execute("DELETE FROM review_templates", [])?;

        let mut stmt =
            db.prepare("INSERT INTO review_templates (name, body, position) VALUES (?1, ?2, ?3)")?;
        for (i, template) in templates.iter().enumerate() {
            stmt.execute(rusqlite::params![template.name, template.body, i as i64])?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => {
            db.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}
