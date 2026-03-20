use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// A structured review handoff note for passing a partial review to another reviewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffNote {
    pub id: i64,
    pub pr_id: i64,
    pub reviewer_name: String,
    pub files_checked: Vec<String>,
    pub concerns: String,
    pub remaining_work: String,
    pub created_at: String,
}

/// Create a new handoff note for a pull request.
#[tauri::command]
pub fn create_handoff(
    pr_id: i64,
    reviewer_name: String,
    files_checked: Vec<String>,
    concerns: String,
    remaining_work: String,
    state: State<'_, DbState>,
) -> Result<HandoffNote, CommandError> {
    let db = state.writer.lock().unwrap();
    let files_json = serde_json::to_string(&files_checked)?;

    db.execute(
        "INSERT INTO review_handoffs (pr_id, reviewer_name, files_checked, concerns, remaining_work) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![pr_id, reviewer_name, files_json, concerns, remaining_work],
    )?;

    let id = db.last_insert_rowid();
    let created_at: String = db.query_row(
        "SELECT created_at FROM review_handoffs WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;

    Ok(HandoffNote {
        id,
        pr_id,
        reviewer_name,
        files_checked,
        concerns,
        remaining_work,
        created_at,
    })
}

/// List all handoff notes for a given pull request.
#[tauri::command]
pub fn list_handoffs(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<HandoffNote>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT id, pr_id, reviewer_name, files_checked, concerns, remaining_work, created_at \
         FROM review_handoffs \
         WHERE pr_id = ?1 \
         ORDER BY created_at DESC",
    )?;

    let notes = stmt
        .query_map([pr_id], |row| {
            let files_json: String = row.get(3)?;
            let files: Vec<String> = serde_json::from_str(&files_json).unwrap_or_default();
            Ok(HandoffNote {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                reviewer_name: row.get(2)?,
                files_checked: files,
                concerns: row.get(4)?,
                remaining_work: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(notes)
}

/// Delete a handoff note by ID.
#[tauri::command]
pub fn delete_handoff(id: i64, state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    let affected = db.execute("DELETE FROM review_handoffs WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(CommandError::NotFound(format!(
            "Handoff note with id {}",
            id
        )));
    }
    Ok(())
}

/// Format a handoff note as a markdown comment for GitHub.
fn format_handoff_markdown(note: &HandoffNote) -> String {
    let mut md = String::new();
    md.push_str("## Review Handoff\n\n");
    md.push_str(&format!("**Reviewer:** {}\n\n", note.reviewer_name));

    if !note.files_checked.is_empty() {
        md.push_str("### Files Reviewed\n");
        for file in &note.files_checked {
            md.push_str(&format!("- [x] `{}`\n", file));
        }
        md.push('\n');
    }

    if !note.concerns.is_empty() {
        md.push_str("### Concerns\n");
        md.push_str(&note.concerns);
        md.push_str("\n\n");
    }

    if !note.remaining_work.is_empty() {
        md.push_str("### Remaining Work\n");
        md.push_str(&note.remaining_work);
        md.push_str("\n\n");
    }

    md.push_str(&format!("_Handoff created at {}_\n", note.created_at));

    md
}

/// Export a handoff note as a GitHub PR comment using `gh pr comment`.
#[tauri::command]
pub fn export_handoff_to_github(
    id: i64,
    state: State<'_, DbState>,
) -> Result<String, CommandError> {
    let db = state.reader.lock().unwrap();

    // Fetch the handoff note
    let note = db
        .query_row(
            "SELECT id, pr_id, reviewer_name, files_checked, concerns, remaining_work, created_at \
             FROM review_handoffs WHERE id = ?1",
            [id],
            |row| {
                let files_json: String = row.get(3)?;
                let files: Vec<String> = serde_json::from_str(&files_json).unwrap_or_default();
                Ok(HandoffNote {
                    id: row.get(0)?,
                    pr_id: row.get(1)?,
                    reviewer_name: row.get(2)?,
                    files_checked: files,
                    concerns: row.get(4)?,
                    remaining_work: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CommandError::NotFound(format!("Handoff note with id {}", id))
            }
            other => CommandError::Db(other),
        })?;

    // Get the PR number and repo full name
    let (repo_id, pr_number): (i64, i64) = db.query_row(
        "SELECT repo_id, number FROM pull_requests WHERE id = ?1",
        [note.pr_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let full_name: String = db.query_row(
        "SELECT owner || '/' || name FROM repositories WHERE id = ?1",
        [repo_id],
        |row| row.get(0),
    )?;

    // Release the lock before making CLI calls
    drop(db);

    let markdown = format_handoff_markdown(&note);

    let output = super::command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "comment",
            &pr_number.to_string(),
            "--repo",
            &full_name,
            "--body",
            &markdown,
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr comment failed: {}",
            stderr
        )));
    }

    Ok(markdown)
}
