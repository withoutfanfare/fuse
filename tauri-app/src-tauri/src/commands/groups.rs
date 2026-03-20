use tauri::State;

use crate::db::DbState;
use crate::models::RepoGroup;

use super::CommandError;

#[tauri::command]
pub fn list_groups(state: State<'_, DbState>) -> Result<Vec<RepoGroup>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, name, colour, position FROM repo_groups ORDER BY position ASC, name ASC",
    )?;
    let groups = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut result = Vec::new();
    for (id, name, colour, position) in groups {
        let mut member_stmt =
            db.prepare("SELECT repo_id FROM repo_group_members WHERE group_id = ?1")?;
        let repo_ids = member_stmt
            .query_map([id], |row| row.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        result.push(RepoGroup {
            id,
            name,
            colour,
            position,
            repo_ids,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn create_group(
    name: String,
    colour: Option<String>,
    state: State<'_, DbState>,
) -> Result<RepoGroup, CommandError> {
    let db = state.writer.lock().unwrap();
    let colour = colour.unwrap_or_else(|| "#ff6b35".to_string());

    // Set position to max + 1
    let max_pos: i64 = db
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM repo_groups",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    db.execute(
        "INSERT INTO repo_groups (name, colour, position) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, colour, max_pos + 1],
    )?;

    let id = db.last_insert_rowid();
    Ok(RepoGroup {
        id,
        name,
        colour,
        position: max_pos + 1,
        repo_ids: vec![],
    })
}

#[tauri::command]
pub fn delete_group(id: i64, state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let affected = db.execute("DELETE FROM repo_groups WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!("Group with id {id}")));
    }
    Ok(())
}

#[tauri::command]
pub fn add_repo_to_group(
    group_id: i64,
    repo_id: i64,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute(
        "INSERT OR IGNORE INTO repo_group_members (group_id, repo_id) VALUES (?1, ?2)",
        rusqlite::params![group_id, repo_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn remove_repo_from_group(
    group_id: i64,
    repo_id: i64,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute(
        "DELETE FROM repo_group_members WHERE group_id = ?1 AND repo_id = ?2",
        rusqlite::params![group_id, repo_id],
    )?;
    Ok(())
}
