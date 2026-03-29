use std::collections::{HashMap, HashSet};

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
    blocking_weight: f64,
    label_weight: f64,
    conflict_risk_weight: f64,
}

impl Default for PriorityWeights {
    fn default() -> Self {
        Self {
            age_weight: 1.5,
            file_count_weight: 1.0,
            risk_weight: 2.0,
            review_status_weight: 1.5,
            author_workload_weight: 0.8,
            blocking_weight: 1.5,
            label_weight: 0.5,
            conflict_risk_weight: 1.0,
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

/// Context data for priority scoring beyond the PR itself.
struct PriorityContext {
    /// Number of open PRs by this PR's author.
    author_open_count: i64,
    /// Number of other PRs that this PR is blocking.
    blocks_count: i64,
    /// Whether this PR is blocked by another open PR.
    is_blocked: bool,
    /// Number of overlapping files with other open PRs.
    conflict_overlap_count: usize,
    /// Whether any label matches a priority-boosting pattern.
    has_priority_label: bool,
}

/// Labels that indicate urgency and should boost priority score.
const PRIORITY_LABELS: &[&str] = &[
    "urgent",
    "critical",
    "hotfix",
    "priority",
    "p0",
    "p1",
    "high-priority",
    "blocker",
    "security",
];

/// Compute the priority score and factor breakdown for a single PR.
fn compute_priority(
    pr: &PullRequest,
    ctx: &PriorityContext,
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
    if ctx.author_open_count > 2 {
        let workload_points = (ctx.author_open_count as f64 - 2.0).min(3.0);
        let weighted = workload_points * weights.author_workload_weight;
        factors.push(PriorityFactor {
            label: format!("Author has {} open PRs", ctx.author_open_count),
            points: weighted,
        });
        total += weighted;
    }

    // --- Blocking factor: PRs that unblock others deserve higher priority ---
    if ctx.blocks_count > 0 {
        let blocking_points = (ctx.blocks_count as f64).min(3.0);
        let weighted = blocking_points * weights.blocking_weight;
        factors.push(PriorityFactor {
            label: format!("Blocks {} other PR(s)", ctx.blocks_count),
            points: weighted,
        });
        total += weighted;
    }

    // --- Blocked penalty: PRs blocked by others are lower priority ---
    if ctx.is_blocked {
        let penalty = -2.0;
        factors.push(PriorityFactor {
            label: "Blocked by another PR".to_string(),
            points: penalty,
        });
        total += penalty;
    }

    // --- Priority label boost ---
    if ctx.has_priority_label {
        let weighted = 2.0 * weights.label_weight;
        factors.push(PriorityFactor {
            label: "Priority label".to_string(),
            points: weighted,
        });
        total += weighted;
    }

    // --- Conflict risk factor: file overlap with other PRs ---
    if ctx.conflict_overlap_count > 0 {
        let conflict_points = (ctx.conflict_overlap_count as f64 / 3.0).min(3.0);
        let weighted = conflict_points * weights.conflict_risk_weight;
        factors.push(PriorityFactor {
            label: format!("{} files overlap with other PRs", ctx.conflict_overlap_count),
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
///
/// Combines multiple signals: age, file count, risk score, review status,
/// author workload, blocking/blocked-by relationships, priority labels,
/// and file-level conflict risk with other open PRs.
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
    let mut author_counts: HashMap<String, i64> = HashMap::new();
    for pr in &prs {
        *author_counts.entry(pr.author.clone()).or_insert(0) += 1;
    }

    // Pre-compute blocking relationships from pr_dependencies
    let mut blocks_count: HashMap<i64, i64> = HashMap::new();
    let mut blocked_prs: HashSet<i64> = HashSet::new();
    {
        let mut dep_stmt = db.prepare(
            "SELECT pr_id, depends_on_pr_id FROM pr_dependencies",
        )?;
        let deps: Vec<(i64, i64)> = dep_stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        let open_ids: HashSet<i64> = prs.iter().map(|p| p.id).collect();
        for (pr_id, depends_on_id) in &deps {
            if open_ids.contains(pr_id) && open_ids.contains(depends_on_id) {
                *blocks_count.entry(*depends_on_id).or_insert(0) += 1;
                blocked_prs.insert(*pr_id);
            }
        }
    }

    // Pre-compute file-level conflict risk (overlapping changed files)
    let mut conflict_overlap: HashMap<i64, usize> = HashMap::new();
    {
        // Group PRs by (repo_id, base_branch) for overlap detection
        struct FileInfo {
            id: i64,
            paths: Vec<String>,
        }
        let mut groups: HashMap<(i64, String), Vec<FileInfo>> = HashMap::new();
        {
            let mut file_stmt = db.prepare(
                "SELECT id, repo_id, base_branch, changed_file_paths
                 FROM pull_requests
                 WHERE state = 'OPEN' AND merged_at IS NULL AND closed_at IS NULL",
            )?;
            let rows: Vec<(i64, i64, String, String)> = file_stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
                .filter_map(|r| r.ok())
                .collect();

            for (id, repo_id, base_branch, paths_json) in rows {
                let paths: Vec<String> = serde_json::from_str(&paths_json).unwrap_or_default();
                if !paths.is_empty() {
                    groups
                        .entry((repo_id, base_branch))
                        .or_default()
                        .push(FileInfo { id, paths });
                }
            }
        }

        for group in groups.values() {
            for i in 0..group.len() {
                for j in (i + 1)..group.len() {
                    let overlap = group[i]
                        .paths
                        .iter()
                        .filter(|p| group[j].paths.contains(p))
                        .count();
                    if overlap > 0 {
                        *conflict_overlap.entry(group[i].id).or_insert(0) += overlap;
                        *conflict_overlap.entry(group[j].id).or_insert(0) += overlap;
                    }
                }
            }
        }
    }

    let weights = PriorityWeights::default();

    let mut queue: Vec<PriorityQueueItem> = prs
        .into_iter()
        .map(|pr| {
            let has_priority_label = pr.labels.iter().any(|l| {
                let lower = l.to_lowercase();
                PRIORITY_LABELS.iter().any(|p| lower.contains(p))
            });

            let ctx = PriorityContext {
                author_open_count: *author_counts.get(&pr.author).unwrap_or(&0),
                blocks_count: *blocks_count.get(&pr.id).unwrap_or(&0),
                is_blocked: blocked_prs.contains(&pr.id),
                conflict_overlap_count: *conflict_overlap.get(&pr.id).unwrap_or(&0),
                has_priority_label,
            };

            let (priority_score, factors) = compute_priority(&pr, &ctx, &weights);
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
