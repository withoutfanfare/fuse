use serde::Serialize;
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// A single review time entry for a PR.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewTimeEntry {
    pub pr_id: i64,
    pub started_at: String,
    pub duration_seconds: i64,
}

/// Personal velocity statistics for the review time dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewVelocityStats {
    pub avg_review_seconds: f64,
    pub total_reviews: i64,
    pub total_seconds: i64,
    pub by_risk_tier: Vec<RiskTierTime>,
    pub weekly_trend: Vec<WeeklyTimePoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskTierTime {
    pub tier: String,
    pub avg_seconds: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeeklyTimePoint {
    pub week_start: String,
    pub total_seconds: i64,
    pub review_count: i64,
}

#[tauri::command]
pub fn log_review_time(
    pr_id: i64,
    duration_seconds: i64,
    state: State<'_, DbState>,
) -> Result<(), CommandError> {
    let db = state.writer.lock().unwrap();
    db.execute(
        "INSERT INTO review_time_log (pr_id, duration_seconds) VALUES (?1, ?2)",
        rusqlite::params![pr_id, duration_seconds],
    )?;
    // Also update the pr_reviews aggregate
    db.execute(
        r#"INSERT INTO pr_reviews (pr_id, status, updated_at)
           VALUES (?1, 'pending', datetime('now'))
           ON CONFLICT(pr_id) DO NOTHING"#,
        rusqlite::params![pr_id],
    )?;
    db.execute(
        "UPDATE pr_reviews SET review_duration_seconds = COALESCE(review_duration_seconds, 0) + ?1 WHERE pr_id = ?2",
        rusqlite::params![duration_seconds, pr_id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_review_velocity_stats(
    state: State<'_, DbState>,
) -> Result<ReviewVelocityStats, CommandError> {
    let db = state.reader.lock().unwrap();

    // Overall averages
    let (total_seconds, total_reviews): (i64, i64) = db
        .query_row(
            "SELECT COALESCE(SUM(review_duration_seconds), 0), COUNT(*) FROM pr_reviews WHERE review_duration_seconds > 0",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

    let avg_review_seconds = if total_reviews > 0 {
        total_seconds as f64 / total_reviews as f64
    } else {
        0.0
    };

    // By risk tier — approximate using change metrics
    let mut by_risk_tier = Vec::new();
    {
        let mut stmt = db.prepare(
            r#"SELECT
                CASE
                    WHEN (p.changed_files + (p.additions + p.deletions) / 200) >= 7 THEN 'high'
                    WHEN (p.changed_files + (p.additions + p.deletions) / 200) >= 4 THEN 'medium'
                    ELSE 'low'
                END as tier,
                AVG(r.review_duration_seconds) as avg_secs,
                COUNT(*) as cnt
            FROM pr_reviews r
            JOIN pull_requests p ON p.id = r.pr_id
            WHERE r.review_duration_seconds > 0
            GROUP BY tier
            ORDER BY avg_secs DESC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(RiskTierTime {
                tier: row.get(0)?,
                avg_seconds: row.get(1)?,
                count: row.get(2)?,
            })
        })?;
        for row in rows {
            by_risk_tier.push(row?);
        }
    }

    // Weekly trend — last 8 weeks
    let mut weekly_trend = Vec::new();
    {
        let mut stmt = db.prepare(
            r#"SELECT
                date(r.reviewed_at, 'weekday 0', '-6 days') as week_start,
                COALESCE(SUM(r.review_duration_seconds), 0) as total_secs,
                COUNT(*) as cnt
            FROM pr_reviews r
            WHERE r.reviewed_at IS NOT NULL
              AND r.review_duration_seconds > 0
              AND r.reviewed_at >= date('now', '-56 days')
            GROUP BY week_start
            ORDER BY week_start"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(WeeklyTimePoint {
                week_start: row.get(0)?,
                total_seconds: row.get(1)?,
                review_count: row.get(2)?,
            })
        })?;
        for row in rows {
            weekly_trend.push(row?);
        }
    }

    Ok(ReviewVelocityStats {
        avg_review_seconds,
        total_reviews,
        total_seconds,
        by_risk_tier,
        weekly_trend,
    })
}
