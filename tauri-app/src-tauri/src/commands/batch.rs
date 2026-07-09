use tauri::State;

use crate::branch_policy::{direct_to_production_error, is_direct_to_production};
use crate::db::DbState;
use crate::github;
use crate::models::BatchResult;

use super::CommandError;

/// Look up the repo full name, PR number, branch context, and draft status for a given PR id.
fn get_pr_context(
    db: &rusqlite::Connection,
    pr_id: i64,
) -> Result<(String, i64, String, String, String, Option<String>, bool), CommandError> {
    let (repo_id, number, head_branch, base_branch, is_draft): (i64, i64, String, String, i64) =
        db.query_row(
            "SELECT repo_id, number, head_branch, base_branch, is_draft FROM pull_requests WHERE id = ?1",
            [pr_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CommandError::NotFound(format!("Pull request with id {pr_id}"))
            }
            other => CommandError::Db(other),
        })?;

    let (full_name, default_branch, integration_branch): (String, String, Option<String>) =
        db.query_row(
        "SELECT owner || '/' || name, default_branch, integration_branch FROM repositories WHERE id = ?1",
        [repo_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    Ok((
        full_name,
        number,
        head_branch,
        base_branch,
        default_branch,
        integration_branch,
        is_draft != 0,
    ))
}

/// Context needed for a batch network call, gathered from the DB before parallelisation.
struct PrBatchContext {
    pr_id: i64,
    full_name: String,
    number: i64,
}

#[tauri::command]
pub fn batch_approve(
    pr_ids: Vec<i64>,
    body: Option<String>,
    state: State<'_, DbState>,
) -> Result<Vec<BatchResult>, CommandError> {
    // Gather all PR contexts from the DB first (sequential, fast)
    let mut contexts: Vec<Result<PrBatchContext, BatchResult>> = Vec::new();
    {
        let db = state.writer.lock().unwrap();
        for &pr_id in &pr_ids {
            match get_pr_context(&db, pr_id) {
                Ok((full_name, number, _, _, _, _, _)) => {
                    contexts.push(Ok(PrBatchContext {
                        pr_id,
                        full_name,
                        number,
                    }));
                }
                Err(e) => {
                    contexts.push(Err(BatchResult {
                        pr_id,
                        success: false,
                        message: format!("Failed to look up PR {}: {}", pr_id, e),
                    }));
                }
            }
        }
    }

    // Run network calls in parallel using thread::scope
    let network_results: Vec<BatchResult> = std::thread::scope(|s| {
        let handles: Vec<_> = contexts
            .iter()
            .map(|ctx| {
                let body = body.as_deref();
                s.spawn(move || match ctx {
                    Err(err_result) => BatchResult {
                        pr_id: err_result.pr_id,
                        success: false,
                        message: err_result.message.clone(),
                    },
                    Ok(c) => match github::approve_pr(&c.full_name, c.number, body) {
                        Ok(_) => BatchResult {
                            pr_id: c.pr_id,
                            success: true,
                            message: format!("PR #{} approved successfully", c.number),
                        },
                        Err(e) => BatchResult {
                            pr_id: c.pr_id,
                            success: false,
                            message: format!("PR #{} failed: {}", c.number, e),
                        },
                    },
                })
            })
            .collect();

        handles
            .into_iter()
            .zip(contexts.iter())
            .map(|(h, ctx)| {
                h.join().unwrap_or_else(|_| BatchResult {
                    pr_id: match ctx {
                        Ok(c) => c.pr_id,
                        Err(e) => e.pr_id,
                    },
                    success: false,
                    message: "Internal error: approve worker thread panicked".to_string(),
                })
            })
            .collect()
    });

    // Write DB updates sequentially for successful approvals
    let db = state.writer.lock().unwrap();
    for result in &network_results {
        if result.success {
            let _ = db.execute(
                r#"INSERT INTO pr_reviews (pr_id, status, reviewed_at, updated_at)
                   VALUES (?1, 'approved', datetime('now'), datetime('now'))
                   ON CONFLICT(pr_id) DO UPDATE SET
                       status = 'approved',
                       reviewed_at = datetime('now'),
                       updated_at = datetime('now')"#,
                [result.pr_id],
            );
        }
    }

    Ok(network_results)
}

#[tauri::command]
pub fn batch_merge(
    pr_ids: Vec<i64>,
    method: Option<String>,
    state: State<'_, DbState>,
) -> Result<Vec<BatchResult>, CommandError> {
    // Gather all PR contexts from the DB first (sequential, fast)
    let mut contexts: Vec<Result<PrBatchContext, BatchResult>> = Vec::new();
    {
        let db = state.writer.lock().unwrap();
        for &pr_id in &pr_ids {
            match get_pr_context(&db, pr_id) {
                Ok((
                    full_name,
                    number,
                    head_branch,
                    base_branch,
                    default_branch,
                    integration_branch,
                    is_draft,
                )) => {
                    // Enforce merge target protection
                    if is_direct_to_production(
                        &base_branch,
                        &head_branch,
                        &default_branch,
                        integration_branch.as_deref(),
                    ) {
                        contexts.push(Err(BatchResult {
                            pr_id,
                            success: false,
                            message: direct_to_production_error(
                                number,
                                &base_branch,
                                &default_branch,
                                integration_branch.as_deref(),
                            ),
                        }));
                    } else if is_draft {
                        contexts.push(Err(BatchResult {
                            pr_id,
                            success: false,
                            message: format!(
                                "PR #{} refused: draft pull requests must be marked ready for review before merging.",
                                number
                            ),
                        }));
                    } else {
                        contexts.push(Ok(PrBatchContext {
                            pr_id,
                            full_name,
                            number,
                        }));
                    }
                }
                Err(e) => {
                    contexts.push(Err(BatchResult {
                        pr_id,
                        success: false,
                        message: format!("Failed to look up PR {}: {}", pr_id, e),
                    }));
                }
            }
        }
    }

    // Run network calls in parallel using thread::scope
    let network_results: Vec<BatchResult> = std::thread::scope(|s| {
        let handles: Vec<_> = contexts
            .iter()
            .map(|ctx| {
                let method = method.as_deref();
                s.spawn(move || match ctx {
                    Err(err_result) => BatchResult {
                        pr_id: err_result.pr_id,
                        success: false,
                        message: err_result.message.clone(),
                    },
                    Ok(c) => match github::merge_pr(&c.full_name, c.number, method) {
                        Ok(_) => BatchResult {
                            pr_id: c.pr_id,
                            success: true,
                            message: format!("PR #{} merged successfully", c.number),
                        },
                        Err(e) => BatchResult {
                            pr_id: c.pr_id,
                            success: false,
                            message: format!("PR #{} failed: {}", c.number, e),
                        },
                    },
                })
            })
            .collect();

        handles
            .into_iter()
            .zip(contexts.iter())
            .map(|(h, ctx)| {
                h.join().unwrap_or_else(|_| BatchResult {
                    pr_id: match ctx {
                        Ok(c) => c.pr_id,
                        Err(e) => e.pr_id,
                    },
                    success: false,
                    message: "Internal error: merge worker thread panicked".to_string(),
                })
            })
            .collect()
    });

    // Write DB updates sequentially for successful merges
    let db = state.writer.lock().unwrap();
    for result in &network_results {
        if result.success {
            let _ = db.execute(
                "UPDATE pull_requests SET state = 'MERGED', merged_at = datetime('now') WHERE id = ?1",
                [result.pr_id],
            );
        }
    }

    Ok(network_results)
}
