use tauri::State;

use crate::db::DbState;
use crate::models::Repository;

use super::CommandError;

/// Validate that a GitHub owner or repository name contains only valid characters.
fn is_valid_github_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

#[tauri::command]
pub fn add_repository(
    owner: String,
    name: String,
    default_branch: Option<String>,
    state: State<'_, DbState>,
) -> Result<Repository, CommandError> {
    if !is_valid_github_name(&owner) || !is_valid_github_name(&name) {
        return Err(CommandError::Gh("Invalid repository owner/name".into()));
    }

    let db = state.writer.lock().unwrap();
    let branch = default_branch.unwrap_or_else(|| "main".to_string());

    db.execute(
        "INSERT INTO repositories (owner, name, default_branch) VALUES (?1, ?2, ?3)",
        rusqlite::params![owner, name, branch],
    )?;

    let id = db.last_insert_rowid();
    let repo = db.query_row(
        "SELECT id, owner, name, default_branch, added_at FROM repositories WHERE id = ?1",
        [id],
        |row| {
            Ok(Repository {
                id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                default_branch: row.get(3)?,
                added_at: row.get(4)?,
            })
        },
    )?;

    Ok(repo)
}

#[tauri::command]
pub fn remove_repository(id: i64, state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let affected = db.execute("DELETE FROM repositories WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!("Repository with id {id}")));
    }
    Ok(())
}

/// Update the default branch for an existing repository.
#[tauri::command]
pub fn update_repository_branch(
    id: i64,
    default_branch: String,
    state: State<'_, DbState>,
) -> Result<Repository, CommandError> {
    let db = state.writer.lock().unwrap();
    let affected = db.execute(
        "UPDATE repositories SET default_branch = ?1 WHERE id = ?2",
        rusqlite::params![default_branch, id],
    )?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!("Repository with id {id}")));
    }
    let repo = db.query_row(
        "SELECT id, owner, name, default_branch, added_at FROM repositories WHERE id = ?1",
        [id],
        |row| {
            Ok(Repository {
                id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                default_branch: row.get(3)?,
                added_at: row.get(4)?,
            })
        },
    )?;
    Ok(repo)
}

#[tauri::command]
pub fn list_repositories(state: State<'_, DbState>) -> Result<Vec<Repository>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, owner, name, default_branch, added_at FROM repositories ORDER BY added_at DESC",
    )?;
    let repos = stmt
        .query_map([], |row| {
            Ok(Repository {
                id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                default_branch: row.get(3)?,
                added_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(repos)
}
