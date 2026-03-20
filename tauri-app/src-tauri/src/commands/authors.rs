use tauri::State;

use crate::db::DbState;
use crate::models::AuthorStats;

use super::CommandError;

#[tauri::command]
pub fn get_author_stats(state: State<'_, DbState>) -> Result<Vec<AuthorStats>, CommandError> {
    let db = state.reader.lock().unwrap();

    let sql = r#"
        SELECT
            p.author,
            COUNT(*) AS pr_count,
            AVG(CAST(p.additions AS REAL)) AS avg_additions,
            AVG(CAST(p.deletions AS REAL)) AS avg_deletions,
            SUM(CASE WHEN p.merged_at IS NOT NULL THEN 1 ELSE 0 END) AS merged_count,
            SUM(CASE WHEN r.status IN ('reviewed', 'approved', 'changes_requested') THEN 1 ELSE 0 END) AS reviewed_count
        FROM pull_requests p
        LEFT JOIN pr_reviews r ON r.pr_id = p.id
        GROUP BY p.author
        ORDER BY pr_count DESC
    "#;

    let mut stmt = db.prepare(sql)?;
    let authors = stmt
        .query_map([], |row| {
            Ok(AuthorStats {
                author: row.get(0)?,
                pr_count: row.get(1)?,
                avg_additions: row.get(2)?,
                avg_deletions: row.get(3)?,
                merged_count: row.get(4)?,
                reviewed_count: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(authors)
}
