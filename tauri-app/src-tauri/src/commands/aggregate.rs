use serde::Serialize;
use tauri::State;

use crate::db::DbState;

use super::CommandError;

/// Cross-repository aggregate summary for the multi-repo dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct AggregateDashboard {
    pub total_open_prs: i64,
    pub review_requested_count: i64,
    pub high_risk_count: i64,
    pub stale_count: i64,
    pub repo_summaries: Vec<RepoSummary>,
    pub top_risk_prs: Vec<TopRiskPr>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoSummary {
    pub repo_id: i64,
    pub repo_name: String,
    pub open_pr_count: i64,
    pub oldest_pr_age_hours: f64,
    pub last_sync_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopRiskPr {
    pub pr_id: i64,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub repo_name: String,
    pub risk_score: f64,
    pub changed_files: i64,
    pub additions: i64,
    pub deletions: i64,
    pub created_at: String,
}

#[tauri::command]
pub fn get_aggregate_dashboard(
    state: State<'_, DbState>,
) -> Result<AggregateDashboard, CommandError> {
    let db = state.reader.lock().unwrap();

    // Total open PRs
    let total_open_prs: i64 = db.query_row(
        "SELECT COUNT(*) FROM pull_requests WHERE state = 'OPEN'",
        [],
        |row| row.get(0),
    )?;

    // Review requested count (PRs with pending review requests)
    let review_requested_count: i64 = db.query_row(
        r#"SELECT COUNT(DISTINCT pr.id) FROM pull_requests pr
           JOIN pr_requested_reviewers prr ON prr.pr_id = pr.id
           WHERE pr.state = 'OPEN'"#,
        [],
        |row| row.get(0),
    )?;

    // High-risk count (simple heuristic: changed_files >= 6 or total lines >= 500)
    let high_risk_count: i64 = db.query_row(
        r#"SELECT COUNT(*) FROM pull_requests
           WHERE state = 'OPEN'
           AND (changed_files >= 6 OR (additions + deletions) >= 500)"#,
        [],
        |row| row.get(0),
    )?;

    // Stale count (open PRs not updated in > 3 days)
    let stale_count: i64 = db.query_row(
        r#"SELECT COUNT(*) FROM pull_requests
           WHERE state = 'OPEN'
           AND updated_at < datetime('now', '-3 days')"#,
        [],
        |row| row.get(0),
    )?;

    // Per-repository summaries
    let mut repo_stmt = db.prepare(
        r#"SELECT
            r.id,
            r.owner || '/' || r.name as repo_name,
            COALESCE(pr_counts.cnt, 0) as open_count,
            COALESCE(pr_counts.oldest_age_hours, 0) as oldest_age,
            sl.synced_at
        FROM repositories r
        LEFT JOIN (
            SELECT repo_id,
                   COUNT(*) as cnt,
                   MAX((julianday('now') - julianday(created_at)) * 24) as oldest_age_hours
            FROM pull_requests
            WHERE state = 'OPEN'
            GROUP BY repo_id
        ) pr_counts ON pr_counts.repo_id = r.id
        LEFT JOIN (
            SELECT repo_id, MAX(synced_at) as synced_at
            FROM sync_log
            WHERE error IS NULL
            GROUP BY repo_id
        ) sl ON sl.repo_id = r.id
        ORDER BY open_count DESC"#,
    )?;
    let repo_summaries = repo_stmt
        .query_map([], |row| {
            Ok(RepoSummary {
                repo_id: row.get(0)?,
                repo_name: row.get(1)?,
                open_pr_count: row.get(2)?,
                oldest_pr_age_hours: row.get(3)?,
                last_sync_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Top 10 highest-risk PRs across all repos
    let mut risk_stmt = db.prepare(
        r#"SELECT
            p.id,
            p.number,
            p.title,
            p.author,
            r.owner || '/' || r.name as repo_name,
            (p.changed_files + (p.additions + p.deletions) / 200.0 +
             CASE WHEN (julianday('now') - julianday(p.created_at)) * 24 >= 72 THEN 2
                  WHEN (julianday('now') - julianday(p.created_at)) * 24 >= 24 THEN 1
                  ELSE 0 END) as risk_score,
            p.changed_files,
            p.additions,
            p.deletions,
            p.created_at
        FROM pull_requests p
        JOIN repositories r ON r.id = p.repo_id
        WHERE p.state = 'OPEN'
        ORDER BY risk_score DESC
        LIMIT 10"#,
    )?;
    let top_risk_prs = risk_stmt
        .query_map([], |row| {
            Ok(TopRiskPr {
                pr_id: row.get(0)?,
                number: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                repo_name: row.get(4)?,
                risk_score: row.get(5)?,
                changed_files: row.get(6)?,
                additions: row.get(7)?,
                deletions: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AggregateDashboard {
        total_open_prs,
        review_requested_count,
        high_risk_count,
        stale_count,
        repo_summaries,
        top_risk_prs,
    })
}
