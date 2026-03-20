use rusqlite::params;
use tauri::State;

use crate::db::DbState;
use crate::models::ReviewSession;

use super::CommandError;

/// Create a new review session for a pull request.
///
/// If an existing session is already in progress for this PR, returns it instead
/// of creating a duplicate.
#[tauri::command]
pub fn create_review_session(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<ReviewSession, CommandError> {
    let db = state.writer.lock().unwrap();

    // Check for an existing active session
    let existing: Option<ReviewSession> = db
        .query_row(
            "SELECT id, pr_id, started_at, files_reviewed, session_notes, status
             FROM review_sessions
             WHERE pr_id = ?1 AND status IN ('active', 'paused')
             ORDER BY started_at DESC LIMIT 1",
            [pr_id],
            |row| {
                let files_json: String = row.get(3)?;
                Ok(ReviewSession {
                    id: row.get(0)?,
                    pr_id: row.get(1)?,
                    started_at: row.get(2)?,
                    files_reviewed: serde_json::from_str(&files_json).unwrap_or_default(),
                    session_notes: row.get(4)?,
                    status: row.get(5)?,
                })
            },
        )
        .ok();

    if let Some(session) = existing {
        return Ok(session);
    }

    db.execute(
        "INSERT INTO review_sessions (pr_id, files_reviewed, status)
         VALUES (?1, '[]', 'active')",
        [pr_id],
    )?;

    let id = db.last_insert_rowid();

    let session = db.query_row(
        "SELECT id, pr_id, started_at, files_reviewed, session_notes, status
         FROM review_sessions WHERE id = ?1",
        [id],
        |row| {
            let files_json: String = row.get(3)?;
            Ok(ReviewSession {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                started_at: row.get(2)?,
                files_reviewed: serde_json::from_str(&files_json).unwrap_or_default(),
                session_notes: row.get(4)?,
                status: row.get(5)?,
            })
        },
    )?;

    Ok(session)
}

/// Retrieve an existing review session by ID.
#[tauri::command]
pub fn get_review_session(
    session_id: i64,
    state: State<'_, DbState>,
) -> Result<ReviewSession, CommandError> {
    let db = state.reader.lock().unwrap();

    db.query_row(
        "SELECT id, pr_id, started_at, files_reviewed, session_notes, status
         FROM review_sessions WHERE id = ?1",
        [session_id],
        |row| {
            let files_json: String = row.get(3)?;
            Ok(ReviewSession {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                started_at: row.get(2)?,
                files_reviewed: serde_json::from_str(&files_json).unwrap_or_default(),
                session_notes: row.get(4)?,
                status: row.get(5)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CommandError::NotFound(format!("Review session with id {session_id}"))
        }
        other => CommandError::Db(other),
    })
}

/// Get the most recent active or paused review session for a given PR.
#[tauri::command]
pub fn get_session_for_pr(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Option<ReviewSession>, CommandError> {
    let db = state.reader.lock().unwrap();

    let result = db
        .query_row(
            "SELECT id, pr_id, started_at, files_reviewed, session_notes, status
             FROM review_sessions
             WHERE pr_id = ?1 AND status IN ('active', 'paused')
             ORDER BY started_at DESC LIMIT 1",
            [pr_id],
            |row| {
                let files_json: String = row.get(3)?;
                Ok(ReviewSession {
                    id: row.get(0)?,
                    pr_id: row.get(1)?,
                    started_at: row.get(2)?,
                    files_reviewed: serde_json::from_str(&files_json).unwrap_or_default(),
                    session_notes: row.get(4)?,
                    status: row.get(5)?,
                })
            },
        )
        .ok();

    Ok(result)
}

/// Update the files reviewed in a session (JSON array of file paths).
#[tauri::command]
pub fn update_session_files(
    session_id: i64,
    files_reviewed: Vec<String>,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let json = serde_json::to_string(&files_reviewed)?;

    let changed = db.execute(
        "UPDATE review_sessions SET files_reviewed = ?1 WHERE id = ?2",
        params![json, session_id],
    )?;

    if changed == 0 {
        return Err(CommandError::NotFound(format!(
            "Review session with id {session_id}"
        )));
    }

    Ok(())
}

/// Update the session notes (free-form text).
#[tauri::command]
pub fn update_session_notes(
    session_id: i64,
    notes: String,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();

    let changed = db.execute(
        "UPDATE review_sessions SET session_notes = ?1 WHERE id = ?2",
        params![notes, session_id],
    )?;

    if changed == 0 {
        return Err(CommandError::NotFound(format!(
            "Review session with id {session_id}"
        )));
    }

    Ok(())
}

/// Update session status: 'active', 'paused', or 'completed'.
#[tauri::command]
pub fn update_session_status(
    session_id: i64,
    status: String,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();

    let changed = db.execute(
        "UPDATE review_sessions SET status = ?1 WHERE id = ?2",
        params![status, session_id],
    )?;

    if changed == 0 {
        return Err(CommandError::NotFound(format!(
            "Review session with id {session_id}"
        )));
    }

    Ok(())
}

/// List all review sessions for a given PR.
#[tauri::command]
pub fn list_review_sessions(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<ReviewSession>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT id, pr_id, started_at, files_reviewed, session_notes, status
         FROM review_sessions
         WHERE pr_id = ?1
         ORDER BY started_at DESC",
    )?;

    let sessions = stmt
        .query_map([pr_id], |row| {
            let files_json: String = row.get(3)?;
            Ok(ReviewSession {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                started_at: row.get(2)?,
                files_reviewed: serde_json::from_str(&files_json).unwrap_or_default(),
                session_notes: row.get(4)?,
                status: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sessions)
}
