use tauri::State;

use crate::db::DbState;
use crate::models::{PriorityFactor, PriorityQueueItem, PullRequest};

use super::pull_requests::{parse_pr_row, PR_SELECT};
use super::CommandError;

/// Configurable weights for priority score factors.
struct PriorityWeights {
    age_weight: f64,
    file_count_weight: f64,
    risk_weight: f64,
    review_status_weight: f64,
    author_workload_weight: f64,
}

impl Default for PriorityWeights {
    fn default() -> Self {
        Self {
            age_weight: 1.5,
            file_count_weight: 1.0,
            risk_weight: 2.0,
            review_status_weight: 1.5,
            author_workload_weight: 0.8,
        }
    }
}

/// Compute a basic risk score (mirrors the frontend computeRiskScore logic).
fn compute_risk_score(pr: &PullRequest) -> f64 {
    let mut score: f64 = 1.0;

    if pr.changed_files >= 12 {
        score += 2.0;
    } else if pr.changed_files >= 6 {
        score += 1.0;
    }

    let churn = pr.additions + pr.deletions;
    if churn >= 500 {
        score += 2.0;
    } else if churn >= 200 {
        score += 1.0;
    }

    if pr.review_decision.as_deref() == Some("CHANGES_REQUESTED") {
        score += 1.0;
    }

    // Age in hours since creation
    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&pr.created_at) {
        let age_hours =
            (chrono::Utc::now() - created.with_timezone(&chrono::Utc)).num_hours() as f64;
        if age_hours >= 72.0 {
            score += 2.0;
        } else if age_hours >= 24.0 {
            score += 1.0;
        }
    }

    if pr.is_draft {
        score -= 1.0;
    }

    score.clamp(1.0, 10.0)
}

/// Compute the priority score and factor breakdown for a single PR.
fn compute_priority(
    pr: &PullRequest,
    author_open_count: i64,
    weights: &PriorityWeights,
) -> (f64, Vec<PriorityFactor>) {
    let mut factors = Vec::new();
    let mut total = 0.0;

    // --- Age factor: older PRs need attention sooner ---
    let age_hours = chrono::DateTime::parse_from_rfc3339(&pr.created_at)
        .map(|created| {
            (chrono::Utc::now() - created.with_timezone(&chrono::Utc)).num_hours() as f64
        })
        .unwrap_or(0.0);

    let age_points = if age_hours >= 168.0 {
        3.0
    } else if age_hours >= 72.0 {
        2.0
    } else if age_hours >= 24.0 {
        1.0
    } else {
        0.0
    };
    if age_points > 0.0 {
        let weighted = age_points * weights.age_weight;
        factors.push(PriorityFactor {
            label: format!("Age ({:.0}h)", age_hours),
            points: weighted,
        });
        total += weighted;
    }

    // --- File count factor ---
    let file_points = if pr.changed_files >= 12 {
        2.0
    } else if pr.changed_files >= 6 {
        1.0
    } else {
        0.0
    };
    if file_points > 0.0 {
        let weighted = file_points * weights.file_count_weight;
        factors.push(PriorityFactor {
            label: format!("{} files changed", pr.changed_files),
            points: weighted,
        });
        total += weighted;
    }

    // --- Risk score factor (reuse existing algorithm) ---
    let risk = compute_risk_score(pr);
    let risk_points = risk / 3.0; // Normalise to roughly 0-3 range
    let weighted_risk = risk_points * weights.risk_weight;
    factors.push(PriorityFactor {
        label: format!("Risk score {:.1}/10", risk),
        points: weighted_risk,
    });
    total += weighted_risk;

    // --- Review status factor: unreviewed PRs score higher ---
    let review_points = match pr.review_status.as_deref() {
        None | Some("pending") => 2.0,
        Some("in_progress") => 1.0,
        Some("changes_requested") => 1.5,
        _ => 0.0, // already reviewed/approved
    };
    if review_points > 0.0 {
        let weighted = review_points * weights.review_status_weight;
        let status_label = pr
            .review_status
            .as_deref()
            .unwrap_or("pending")
            .replace('_', " ");
        factors.push(PriorityFactor {
            label: format!("Review status: {}", status_label),
            points: weighted,
        });
        total += weighted;
    }

    // --- Author workload factor: more open PRs from same author = higher priority ---
    if author_open_count > 2 {
        let workload_points = (author_open_count as f64 - 2.0).min(3.0);
        let weighted = workload_points * weights.author_workload_weight;
        factors.push(PriorityFactor {
            label: format!("Author has {} open PRs", author_open_count),
            points: weighted,
        });
        total += weighted;
    }

    // --- Draft penalty ---
    if pr.is_draft {
        let penalty = -3.0;
        factors.push(PriorityFactor {
            label: "Draft (deprioritised)".to_string(),
            points: penalty,
        });
        total += penalty;
    }

    (total.max(0.0), factors)
}

/// Return all open PRs sorted by priority score (highest first).
/// The "next to review" suggestion is simply the first item.
#[tauri::command]
pub fn get_priority_queue(
    state: State<'_, DbState>,
) -> Result<Vec<PriorityQueueItem>, CommandError> {
    let db = state.reader.lock().unwrap();

    // Fetch all open PRs with joined review status
    let sql = format!(
        "SELECT {} FROM pull_requests p \
         LEFT JOIN pr_reviews r ON r.pr_id = p.id \
         WHERE p.state = 'OPEN' \
         ORDER BY p.updated_at DESC",
        PR_SELECT
    );
    let mut stmt = db.prepare(&sql)?;
    let prs: Vec<PullRequest> = stmt
        .query_map([], parse_pr_row)?
        .collect::<Result<Vec<_>, _>>()?;

    // Pre-compute per-author open PR counts for the workload factor
    let mut author_counts: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    for pr in &prs {
        *author_counts.entry(pr.author.clone()).or_insert(0) += 1;
    }

    let weights = PriorityWeights::default();

    let mut queue: Vec<PriorityQueueItem> = prs
        .into_iter()
        .map(|pr| {
            let author_count = *author_counts.get(&pr.author).unwrap_or(&0);
            let (priority_score, factors) = compute_priority(&pr, author_count, &weights);
            PriorityQueueItem {
                pr,
                priority_score,
                factors,
            }
        })
        .collect();

    // Sort by priority score descending (highest priority first)
    queue.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(queue)
}
