use tauri::State;

use crate::db::DbState;
use crate::models::{AgeBucket, DailyPrCounts, VelocityPoint};

use super::CommandError;

#[tauri::command]
pub fn get_age_distribution(state: State<'_, DbState>) -> Result<Vec<AgeBucket>, CommandError> {
    let db = state.reader.lock().unwrap();

    let sql = r#"
        SELECT
            CASE
                WHEN julianday('now') - julianday(created_at) < 1 THEN '< 1 day'
                WHEN julianday('now') - julianday(created_at) < 3 THEN '1-3 days'
                WHEN julianday('now') - julianday(created_at) < 7 THEN '3-7 days'
                WHEN julianday('now') - julianday(created_at) < 14 THEN '7-14 days'
                ELSE '14+ days'
            END AS bucket,
            COUNT(*) AS cnt
        FROM pull_requests
        WHERE state = 'OPEN'
        GROUP BY bucket
        ORDER BY
            CASE bucket
                WHEN '< 1 day' THEN 1
                WHEN '1-3 days' THEN 2
                WHEN '3-7 days' THEN 3
                WHEN '7-14 days' THEN 4
                ELSE 5
            END
    "#;

    let mut stmt = db.prepare(sql)?;
    let buckets = stmt
        .query_map([], |row| {
            Ok(AgeBucket {
                label: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Ensure all buckets are present even if count is 0
    let all_labels = vec!["< 1 day", "1-3 days", "3-7 days", "7-14 days", "14+ days"];
    let mut result: Vec<AgeBucket> = Vec::new();
    for label in all_labels {
        let found = buckets.iter().find(|b| b.label == label);
        result.push(AgeBucket {
            label: label.to_string(),
            count: found.map_or(0, |b| b.count),
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn get_review_velocity(
    days: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<VelocityPoint>, CommandError> {
    let db = state.reader.lock().unwrap();
    let num_days = days.unwrap_or(30);

    // Build a date series for the last N days, then left-join reviews and merges
    let sql = r#"
        WITH RECURSIVE dates(d) AS (
            SELECT date('now', ?1 || ' days')
            UNION ALL
            SELECT date(d, '+1 day') FROM dates WHERE d < date('now')
        ),
        review_counts AS (
            SELECT DATE(reviewed_at) AS review_date, COUNT(*) AS cnt
            FROM pr_reviews
            WHERE reviewed_at IS NOT NULL
              AND DATE(reviewed_at) >= date('now', ?1 || ' days')
            GROUP BY DATE(reviewed_at)
        ),
        merge_counts AS (
            SELECT DATE(merged_at) AS merge_date, COUNT(*) AS cnt
            FROM pull_requests
            WHERE merged_at IS NOT NULL
              AND DATE(merged_at) >= date('now', ?1 || ' days')
            GROUP BY DATE(merged_at)
        )
        SELECT
            dates.d AS date,
            COALESCE(rc.cnt, 0) AS reviewed,
            COALESCE(mc.cnt, 0) AS merged
        FROM dates
        LEFT JOIN review_counts rc ON rc.review_date = dates.d
        LEFT JOIN merge_counts mc ON mc.merge_date = dates.d
        ORDER BY dates.d
    "#;

    let offset = format!("-{}", num_days);
    let mut stmt = db.prepare(sql)?;
    let points = stmt
        .query_map(rusqlite::params![offset], |row| {
            Ok(VelocityPoint {
                date: row.get(0)?,
                reviewed: row.get(1)?,
                merged: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(points)
}

/// Returns daily open-PR and pending-review counts for the last 7 days,
/// used by the dashboard sparklines.
#[tauri::command]
pub fn get_daily_pr_counts(state: State<'_, DbState>) -> Result<DailyPrCounts, CommandError> {
    let db = state.reader.lock().unwrap();

    // Count open PRs that existed on each of the last 7 days
    // (created before or on that day, and not closed/merged before that day).
    let sql = r#"
        WITH RECURSIVE dates(d) AS (
            SELECT date('now', '-6 days')
            UNION ALL
            SELECT date(d, '+1 day') FROM dates WHERE d < date('now')
        )
        SELECT
            dates.d,
            (SELECT COUNT(*) FROM pull_requests
             WHERE DATE(created_at) <= dates.d
               AND (closed_at IS NULL OR DATE(closed_at) > dates.d)
               AND (merged_at IS NULL OR DATE(merged_at) > dates.d)
            ) AS open_count,
            (SELECT COUNT(*) FROM pull_requests p
             LEFT JOIN pr_reviews r ON r.pr_id = p.id
             WHERE DATE(p.created_at) <= dates.d
               AND (p.closed_at IS NULL OR DATE(p.closed_at) > dates.d)
               AND (p.merged_at IS NULL OR DATE(p.merged_at) > dates.d)
               AND (r.status IS NULL OR r.status = 'pending')
            ) AS pending_count
        FROM dates
        ORDER BY dates.d
    "#;

    let mut stmt = db.prepare(sql)?;
    let mut open_counts: Vec<i64> = Vec::new();
    let mut pending_counts: Vec<i64> = Vec::new();

    let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)))?;

    for row in rows {
        let (open, pending) = row?;
        open_counts.push(open);
        pending_counts.push(pending);
    }

    Ok(DailyPrCounts {
        open_counts,
        pending_counts,
    })
}
