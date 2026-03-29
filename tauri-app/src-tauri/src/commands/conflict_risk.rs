use std::collections::HashMap;

use tauri::State;

use crate::db::DbState;
use crate::models::ConflictRiskEntry;

use super::CommandError;

/// Detect file-level overlap between open PRs targeting the same base branch.
///
/// For each repository, groups open PRs by base branch, parses their
/// `changed_file_paths` JSON, and identifies pairs with overlapping files.
/// Returns a list of conflict risk entries sorted by overlap count (highest first).
#[tauri::command]
pub fn detect_conflict_risks(
    state: State<'_, DbState>,
) -> Result<Vec<ConflictRiskEntry>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT id, repo_id, number, title, base_branch, changed_file_paths
         FROM pull_requests
         WHERE state = 'OPEN' AND merged_at IS NULL AND closed_at IS NULL",
    )?;

    struct PrRow {
        id: i64,
        repo_id: i64,
        number: i64,
        title: String,
        base_branch: String,
        file_paths: Vec<String>,
    }

    let rows: Vec<PrRow> = stmt
        .query_map([], |row| {
            let paths_json: String = row.get(5)?;
            let file_paths: Vec<String> =
                serde_json::from_str(&paths_json).unwrap_or_default();
            Ok(PrRow {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                number: row.get(2)?,
                title: row.get(3)?,
                base_branch: row.get(4)?,
                file_paths,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Group PRs by (repo_id, base_branch)
    let mut groups: HashMap<(i64, &str), Vec<&PrRow>> = HashMap::new();
    for pr in &rows {
        groups
            .entry((pr.repo_id, &pr.base_branch))
            .or_default()
            .push(pr);
    }

    let mut entries: Vec<ConflictRiskEntry> = Vec::new();

    for group in groups.values() {
        if group.len() < 2 {
            continue;
        }

        // Compare each pair of PRs in this group
        for i in 0..group.len() {
            for j in (i + 1)..group.len() {
                let a = group[i];
                let b = group[j];

                // Skip PRs with no file data (not yet synced with files field)
                if a.file_paths.is_empty() || b.file_paths.is_empty() {
                    continue;
                }

                // Find overlapping file paths
                let overlapping: Vec<String> = a
                    .file_paths
                    .iter()
                    .filter(|path| b.file_paths.contains(path))
                    .cloned()
                    .collect();

                if !overlapping.is_empty() {
                    let overlap_count = overlapping.len();

                    // Add entry for PR A referencing PR B
                    entries.push(ConflictRiskEntry {
                        pr_id: a.id,
                        pr_number: a.number,
                        pr_title: a.title.clone(),
                        other_pr_id: b.id,
                        other_pr_number: b.number,
                        other_pr_title: b.title.clone(),
                        overlapping_files: overlapping.clone(),
                        overlap_count,
                    });

                    // Add reciprocal entry for PR B referencing PR A
                    entries.push(ConflictRiskEntry {
                        pr_id: b.id,
                        pr_number: b.number,
                        pr_title: b.title.clone(),
                        other_pr_id: a.id,
                        other_pr_number: a.number,
                        other_pr_title: a.title.clone(),
                        overlapping_files: overlapping,
                        overlap_count,
                    });
                }
            }
        }
    }

    // Sort by overlap count descending
    entries.sort_by(|a, b| b.overlap_count.cmp(&a.overlap_count));

    Ok(entries)
}
