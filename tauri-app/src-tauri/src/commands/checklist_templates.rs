use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// A review checklist template associated with a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplate {
    pub id: i64,
    pub repo_id: Option<i64>,
    pub name: String,
    pub items: Vec<ChecklistTemplateItem>,
    pub created_at: String,
}

/// A single item within a checklist template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplateItem {
    pub id: i64,
    pub template_id: i64,
    pub text: String,
    pub description: Option<String>,
    pub position: i64,
}

#[tauri::command]
pub fn list_checklist_templates(
    repo_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<ChecklistTemplate>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut sql = String::from(
        "SELECT id, repo_id, name, created_at FROM checklist_templates WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(rid) = repo_id {
        sql.push_str(" AND (repo_id = ? OR repo_id IS NULL)");
        params.push(Box::new(rid));
    }
    sql.push_str(" ORDER BY id");

    let mut stmt = db.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let templates: Vec<(i64, Option<i64>, String, String)> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut result = Vec::new();
    for (id, r_id, name, created_at) in templates {
        let mut item_stmt = db.prepare(
            "SELECT id, template_id, text, description, position FROM checklist_template_items WHERE template_id = ?1 ORDER BY position",
        )?;
        let items = item_stmt
            .query_map([id], |row| {
                Ok(ChecklistTemplateItem {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    text: row.get(2)?,
                    description: row.get(3)?,
                    position: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        result.push(ChecklistTemplate {
            id,
            repo_id: r_id,
            name,
            items,
            created_at,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn create_checklist_template(
    repo_id: Option<i64>,
    name: String,
    items: Vec<(String, Option<String>)>,
    state: State<'_, DbState>,
) -> Result<ChecklistTemplate, CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute_batch("BEGIN")?;

    let result = (|| -> Result<ChecklistTemplate, CommandError> {
        db.execute(
            "INSERT INTO checklist_templates (repo_id, name) VALUES (?1, ?2)",
            rusqlite::params![repo_id, name],
        )?;
        let template_id = db.last_insert_rowid();
        let created_at: String = db.query_row(
            "SELECT created_at FROM checklist_templates WHERE id = ?1",
            [template_id],
            |row| row.get(0),
        )?;

        let mut template_items = Vec::new();
        for (i, (text, description)) in items.iter().enumerate() {
            db.execute(
                "INSERT INTO checklist_template_items (template_id, text, description, position) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![template_id, text, description, i as i64],
            )?;
            let item_id = db.last_insert_rowid();
            template_items.push(ChecklistTemplateItem {
                id: item_id,
                template_id,
                text: text.clone(),
                description: description.clone(),
                position: i as i64,
            });
        }

        Ok(ChecklistTemplate {
            id: template_id,
            repo_id,
            name: name.clone(),
            items: template_items,
            created_at,
        })
    })();

    match result {
        Ok(template) => {
            db.execute_batch("COMMIT")?;
            Ok(template)
        }
        Err(e) => {
            let _ = db.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

#[tauri::command]
pub fn update_checklist_template(
    template_id: i64,
    name: String,
    items: Vec<(String, Option<String>)>,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute_batch("BEGIN")?;

    let result = (|| -> Result<(), CommandError> {
        db.execute(
            "UPDATE checklist_templates SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, template_id],
        )?;
        // Remove existing items and re-insert
        db.execute(
            "DELETE FROM checklist_template_items WHERE template_id = ?1",
            [template_id],
        )?;
        for (i, (text, description)) in items.iter().enumerate() {
            db.execute(
                "INSERT INTO checklist_template_items (template_id, text, description, position) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![template_id, text, description, i as i64],
            )?;
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

#[tauri::command]
pub fn delete_checklist_template(
    template_id: i64,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute(
        "DELETE FROM checklist_templates WHERE id = ?1",
        [template_id],
    )?;
    Ok(())
}
