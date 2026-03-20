use tauri::State;

use crate::db::DbState;
use crate::models::DashboardStats;

use super::CommandError;

#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, DbState>) -> Result<DashboardStats, CommandError> {
    let db = state.reader.lock().unwrap();

    // NOTE: These queries rely on state being normalised to uppercase during sync
    // (see sync.rs — pr.state.to_uppercase() before INSERT).
    let total_open_prs: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests WHERE state = 'OPEN'",
        [],
        |row| row.get(0),
    )?;

    let pending_reviews: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests p LEFT JOIN pr_reviews r ON r.pr_id = p.id WHERE p.state = 'OPEN' AND (r.status IS NULL OR r.status = 'pending')",
        [],
        |row| row.get(0),
    )?;

    let in_progress: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests p INNER JOIN pr_reviews r ON r.pr_id = p.id WHERE p.state = 'OPEN' AND r.status = 'in_progress'",
        [],
        |row| row.get(0),
    )?;

    let approved: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests p INNER JOIN pr_reviews r ON r.pr_id = p.id WHERE p.state = 'OPEN' AND r.status = 'approved'",
        [],
        |row| row.get(0),
    )?;

    let repos_count: i64 =
        db.query_row("SELECT COUNT(*) FROM repositories", [], |row| row.get(0))?;

    let last_synced: Option<String> = db
        .query_row("SELECT MAX(synced_at) FROM sync_log", [], |row| row.get(0))
        .ok();

    Ok(DashboardStats {
        total_open_prs,
        pending_reviews,
        in_progress,
        approved,
        repos_count,
        last_synced,
    })
}
