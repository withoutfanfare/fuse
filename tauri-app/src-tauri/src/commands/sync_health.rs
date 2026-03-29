use tauri::State;

use crate::db::DbState;
use crate::models::SyncHealthStatus;

use super::CommandError;

/// Return sync health status for each tracked repository.
///
/// For each repo, reports the last successful sync time, consecutive failure count,
/// the most recent error message, and how many minutes since the last success.
#[tauri::command]
pub fn get_sync_health(
    state: State<'_, DbState>,
) -> Result<Vec<SyncHealthStatus>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut stmt = db.prepare(
        r#"SELECT
            r.id,
            r.owner || '/' || r.name AS repo_name,
            last_ok.synced_at AS last_successful_sync,
            COALESCE(fail_run.cnt, 0) AS consecutive_failures,
            fail_run.last_error,
            CASE
                WHEN last_ok.synced_at IS NOT NULL
                THEN (julianday('now') - julianday(last_ok.synced_at)) * 1440
                ELSE NULL
            END AS minutes_since_success
        FROM repositories r
        LEFT JOIN (
            SELECT repo_id, MAX(synced_at) AS synced_at
            FROM sync_log
            WHERE error IS NULL
            GROUP BY repo_id
        ) last_ok ON last_ok.repo_id = r.id
        LEFT JOIN (
            SELECT
                s1.repo_id,
                COUNT(*) AS cnt,
                (SELECT s2.error FROM sync_log s2
                 WHERE s2.repo_id = s1.repo_id AND s2.error IS NOT NULL
                 ORDER BY s2.synced_at DESC LIMIT 1) AS last_error
            FROM sync_log s1
            WHERE s1.error IS NOT NULL
              AND s1.synced_at > COALESCE(
                  (SELECT MAX(s3.synced_at) FROM sync_log s3
                   WHERE s3.repo_id = s1.repo_id AND s3.error IS NULL),
                  '1970-01-01')
            GROUP BY s1.repo_id
        ) fail_run ON fail_run.repo_id = r.id
        ORDER BY consecutive_failures DESC, repo_name"#,
    )?;

    let results = stmt
        .query_map([], |row| {
            Ok(SyncHealthStatus {
                repo_id: row.get(0)?,
                repo_name: row.get(1)?,
                last_successful_sync: row.get(2)?,
                consecutive_failures: row.get(3)?,
                last_error: row.get(4)?,
                minutes_since_success: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}
