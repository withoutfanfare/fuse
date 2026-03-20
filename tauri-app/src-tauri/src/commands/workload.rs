use tauri::State;

use crate::db::DbState;
use crate::models::ReviewerWorkloadStats;

use super::CommandError;

/// Aggregate reviewer workload statistics across all tracked repositories.
///
/// Computes assigned, completed, overdue counts and average response time
/// for each reviewer in a single aggregate query rather than per-reviewer
/// round trips.
#[tauri::command]
pub fn get_reviewer_workload(
    state: State<'_, DbState>,
) -> Result<Vec<ReviewerWorkloadStats>, CommandError> {
    let db = state.reader.lock().unwrap();

    let sql = r#"
        SELECT
            prr.reviewer,
            COUNT(CASE WHEN p.state = 'OPEN' THEN 1 END) AS assigned,
            COUNT(CASE WHEN rv.status IN ('reviewed','approved','changes_requested') THEN 1 END) AS completed,
            COUNT(CASE WHEN p.state = 'OPEN' AND p.updated_at < datetime('now', '-7 days') THEN 1 END) AS overdue,
            AVG(CASE WHEN rv.reviewed_at IS NOT NULL
                THEN (julianday(rv.reviewed_at) - julianday(p.created_at)) * 24.0 END) AS avg_hours
        FROM pr_requested_reviewers prr
        JOIN pull_requests p ON p.id = prr.pr_id
        LEFT JOIN pr_reviews rv ON rv.pr_id = prr.pr_id
        GROUP BY prr.reviewer
    "#;

    let mut stmt = db.prepare(sql)?;
    let mut results: Vec<ReviewerWorkloadStats> = stmt
        .query_map([], |row| {
            Ok(ReviewerWorkloadStats {
                reviewer: row.get(0)?,
                assigned_count: row.get(1)?,
                completed_count: row.get(2)?,
                overdue_count: row.get(3)?,
                avg_response_hours: row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Sort by total workload descending (assigned + overdue first) so the
    // busiest reviewers appear at the top.
    results.sort_by(|a, b| {
        let a_total = a.assigned_count + a.overdue_count;
        let b_total = b.assigned_count + b.overdue_count;
        b_total.cmp(&a_total)
    });

    Ok(results)
}
