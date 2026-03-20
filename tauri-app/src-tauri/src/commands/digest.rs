use serde::Serialize;
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// Aggregated review digest data for a given period.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewDigest {
    /// Number of PRs reviewed (status = 'reviewed' or 'approved') during the period.
    pub reviewed_count: i64,
    /// Number of PRs still pending review.
    pub pending_count: i64,
    /// Number of PRs merged during the period.
    pub merged_count: i64,
    /// Average review duration in seconds (for PRs with recorded review times).
    pub avg_review_seconds: f64,
    /// Number of PRs that have been open longer than the stale threshold.
    pub stale_count: i64,
    /// Total open PRs across all tracked repositories.
    pub total_open: i64,
    /// The period start date (ISO 8601).
    pub period_start: String,
    /// The period end date (ISO 8601).
    pub period_end: String,
    /// Comparison with the previous period of the same length.
    pub previous: Option<DigestComparison>,
}

/// Comparison metrics against the previous period.
#[derive(Debug, Clone, Serialize)]
pub struct DigestComparison {
    pub reviewed_count: i64,
    pub pending_count: i64,
    pub merged_count: i64,
    pub avg_review_seconds: f64,
    pub stale_count: i64,
}

/// Compute a review digest for a specified number of days.
///
/// The `days` parameter defines the period length. The digest covers
/// the most recent `days` days and compares against the previous `days` days.
#[tauri::command]
pub fn get_review_digest(
    days: i64,
    state: State<'_, DbState>,
) -> Result<ReviewDigest, CommandError> {
    let db = state.reader.lock().unwrap();

    // Read the stale threshold from settings (default 14 days)
    let stale_days: i64 = db
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'stale_threshold_days'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(14);

    // Current period: last `days` days
    let current = compute_period_stats(&db, days, 0, stale_days)?;

    // Previous period: the `days` before the current period
    let previous_stats = compute_period_stats(&db, days, days, stale_days)?;

    let previous = Some(DigestComparison {
        reviewed_count: previous_stats.reviewed_count,
        pending_count: previous_stats.pending_count,
        merged_count: previous_stats.merged_count,
        avg_review_seconds: previous_stats.avg_review_seconds,
        stale_count: previous_stats.stale_count,
    });

    Ok(ReviewDigest {
        reviewed_count: current.reviewed_count,
        pending_count: current.pending_count,
        merged_count: current.merged_count,
        avg_review_seconds: current.avg_review_seconds,
        stale_count: current.stale_count,
        total_open: current.total_open,
        period_start: current.period_start,
        period_end: current.period_end,
        previous,
    })
}

/// Internal stats for a single period.
struct PeriodStats {
    reviewed_count: i64,
    pending_count: i64,
    merged_count: i64,
    avg_review_seconds: f64,
    stale_count: i64,
    total_open: i64,
    period_start: String,
    period_end: String,
}

/// Compute stats for a period of `length` days, offset by `offset` days from now.
fn compute_period_stats(
    db: &rusqlite::Connection,
    length: i64,
    offset: i64,
    stale_days: i64,
) -> Result<PeriodStats, rusqlite::Error> {
    // Period boundaries
    let period_end: String = db.query_row(
        "SELECT datetime('now', ?1)",
        [format!("-{} days", offset)],
        |row| row.get(0),
    )?;

    let period_start: String = db.query_row(
        "SELECT datetime('now', ?1)",
        [format!("-{} days", offset + length)],
        |row| row.get(0),
    )?;

    // Reviewed PRs: those with review status 'reviewed' or 'approved' whose
    // review was updated within the period.
    let reviewed_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM pr_reviews \
         WHERE status IN ('reviewed', 'approved') \
         AND updated_at >= ?1 AND updated_at < ?2",
        rusqlite::params![period_start, period_end],
        |row| row.get(0),
    )?;

    // Pending PRs (snapshot — not period-scoped, but useful for current state)
    let pending_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests p \
         LEFT JOIN pr_reviews r ON r.pr_id = p.id \
         WHERE p.state = 'OPEN' AND (r.status IS NULL OR r.status = 'pending')",
        [],
        |row| row.get(0),
    )?;

    // Merged PRs within the period
    let merged_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests \
         WHERE merged_at IS NOT NULL \
         AND merged_at >= ?1 AND merged_at < ?2",
        rusqlite::params![period_start, period_end],
        |row| row.get(0),
    )?;

    // Average review duration for reviewed PRs with recorded times
    let avg_review_seconds: f64 = db
        .query_row(
            "SELECT COALESCE(AVG(review_duration_seconds), 0.0) FROM pr_reviews \
             WHERE review_duration_seconds > 0 \
             AND updated_at >= ?1 AND updated_at < ?2",
            rusqlite::params![period_start, period_end],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    // Stale PRs — open PRs not updated in more than `stale_days`
    let stale_count: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests \
         WHERE state = 'OPEN' \
         AND updated_at < datetime('now', ?1)",
        [format!("-{} days", stale_days)],
        |row| row.get(0),
    )?;

    // Total currently open PRs
    let total_open: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests WHERE state = 'OPEN'",
        [],
        |row| row.get(0),
    )?;

    Ok(PeriodStats {
        reviewed_count,
        pending_count,
        merged_count,
        avg_review_seconds,
        stale_count,
        total_open,
        period_start,
        period_end,
    })
}
