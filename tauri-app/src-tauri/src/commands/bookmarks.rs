use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// A file-level annotation bookmark within a PR diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub pr_id: i64,
    pub file_path: String,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub note: String,
    pub category: String,
    pub resolved: bool,
    pub created_at: String,
}

/// A bookmark with joined PR and repository metadata for global views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkWithContext {
    pub id: i64,
    pub pr_id: i64,
    pub file_path: String,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub note: String,
    pub category: String,
    pub resolved: bool,
    pub created_at: String,
    pub pr_number: i64,
    pub pr_title: String,
    pub repo_name: String,
}

/// Create a new bookmark for a file (or line range) within a PR.
#[tauri::command]
pub fn create_bookmark(
    pr_id: i64,
    file_path: String,
    line_start: Option<i64>,
    line_end: Option<i64>,
    note: String,
    category: Option<String>,
    state: State<'_, DbState>,
) -> Result<Bookmark, CommandError> {
    let db = state.writer.lock().unwrap();
    let cat = category.unwrap_or_else(|| "note".to_string());

    db.execute(
        "INSERT INTO review_bookmarks (pr_id, file_path, line_start, line_end, note, category) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![pr_id, file_path, line_start, line_end, note, cat],
    )?;

    let id = db.last_insert_rowid();
    let created_at: String = db.query_row(
        "SELECT created_at FROM review_bookmarks WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;

    Ok(Bookmark {
        id,
        pr_id,
        file_path,
        line_start,
        line_end,
        note,
        category: cat,
        resolved: false,
        created_at,
    })
}

/// List all bookmarks for a given pull request.
#[tauri::command]
pub fn list_bookmarks(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<Bookmark>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT id, pr_id, file_path, line_start, line_end, note, category, resolved, created_at \
         FROM review_bookmarks \
         WHERE pr_id = ?1 \
         ORDER BY created_at ASC",
    )?;

    let bookmarks = stmt
        .query_map([pr_id], |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                file_path: row.get(2)?,
                line_start: row.get(3)?,
                line_end: row.get(4)?,
                note: row.get(5)?,
                category: row.get(6)?,
                resolved: row.get::<_, i64>(7)? != 0,
                created_at: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(bookmarks)
}

/// Update an existing bookmark's note, line range, category, and resolved status.
#[tauri::command]
pub fn update_bookmark(
    id: i64,
    note: String,
    line_start: Option<i64>,
    line_end: Option<i64>,
    category: Option<String>,
    resolved: Option<bool>,
    state: State<'_, DbState>,
) -> Result<Bookmark, CommandError> {
    let db = state.writer.lock().unwrap();

    let affected = db.execute(
        "UPDATE review_bookmarks SET note = ?1, line_start = ?2, line_end = ?3, \
         category = COALESCE(?4, category), resolved = COALESCE(?5, resolved) \
         WHERE id = ?6",
        rusqlite::params![
            note,
            line_start,
            line_end,
            category,
            resolved.map(|r| if r { 1i64 } else { 0i64 }),
            id
        ],
    )?;

    if affected == 0 {
        return Err(CommandError::NotFound(format!("Bookmark with id {}", id)));
    }

    let bookmark = db.query_row(
        "SELECT id, pr_id, file_path, line_start, line_end, note, category, resolved, created_at \
         FROM review_bookmarks WHERE id = ?1",
        [id],
        |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                file_path: row.get(2)?,
                line_start: row.get(3)?,
                line_end: row.get(4)?,
                note: row.get(5)?,
                category: row.get(6)?,
                resolved: row.get::<_, i64>(7)? != 0,
                created_at: row.get(8)?,
            })
        },
    )?;

    Ok(bookmark)
}

/// Delete a bookmark by ID.
#[tauri::command]
pub fn delete_bookmark(id: i64, state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let affected = db.execute("DELETE FROM review_bookmarks WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!("Bookmark with id {}", id)));
    }
    Ok(())
}

/// List all bookmarks across all open PRs with PR and repository context.
#[tauri::command]
pub fn list_all_bookmarks(
    state: State<'_, DbState>,
) -> Result<Vec<BookmarkWithContext>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT b.id, b.pr_id, b.file_path, b.line_start, b.line_end, b.note, \
                b.category, b.resolved, b.created_at, \
                p.number, p.title, r.owner || '/' || r.name AS repo_name \
         FROM review_bookmarks b \
         JOIN pull_requests p ON p.id = b.pr_id \
         JOIN repositories r ON r.id = p.repo_id \
         WHERE p.state = 'OPEN' \
         ORDER BY b.created_at DESC",
    )?;

    let bookmarks = stmt
        .query_map([], |row| {
            Ok(BookmarkWithContext {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                file_path: row.get(2)?,
                line_start: row.get(3)?,
                line_end: row.get(4)?,
                note: row.get(5)?,
                category: row.get(6)?,
                resolved: row.get::<_, i64>(7)? != 0,
                created_at: row.get(8)?,
                pr_number: row.get(9)?,
                pr_title: row.get(10)?,
                repo_name: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(bookmarks)
}

/// Get the total count of bookmarks across all open PRs.
#[tauri::command]
pub fn get_bookmark_count(state: State<'_, DbState>) -> Result<i64, CommandError> {
    let db = state.reader.lock().unwrap();

    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM review_bookmarks b \
         JOIN pull_requests p ON p.id = b.pr_id \
         WHERE p.state = 'OPEN'",
        [],
        |row| row.get(0),
    )?;

    Ok(count)
}

/// Toggle the resolved status of a bookmark.
#[tauri::command]
pub fn toggle_bookmark_resolved(
    id: i64,
    state: State<'_, DbState>,
) -> Result<Bookmark, CommandError> {
    let db = state.writer.lock().unwrap();

    let affected = db.execute(
        "UPDATE review_bookmarks SET resolved = CASE WHEN resolved = 0 THEN 1 ELSE 0 END WHERE id = ?1",
        [id],
    )?;

    if affected == 0 {
        return Err(CommandError::NotFound(format!("Bookmark with id {}", id)));
    }

    let bookmark = db.query_row(
        "SELECT id, pr_id, file_path, line_start, line_end, note, category, resolved, created_at \
         FROM review_bookmarks WHERE id = ?1",
        [id],
        |row| {
            Ok(Bookmark {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                file_path: row.get(2)?,
                line_start: row.get(3)?,
                line_end: row.get(4)?,
                note: row.get(5)?,
                category: row.get(6)?,
                resolved: row.get::<_, i64>(7)? != 0,
                created_at: row.get(8)?,
            })
        },
    )?;

    Ok(bookmark)
}
