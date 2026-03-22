use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub default_branch: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub repo_id: i64,
    pub number: i64,
    pub title: String,
    pub author: String,
    pub state: String,
    pub head_branch: String,
    pub base_branch: String,
    pub additions: i64,
    pub deletions: i64,
    pub changed_files: i64,
    pub review_decision: Option<String>,
    pub is_draft: bool,
    pub url: String,
    pub labels: Vec<String>,
    pub mergeable: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub body: Option<String>,
    pub last_synced_at: String,
    /// Joined from pr_reviews (optional)
    pub review_status: Option<String>,
    /// Joined from pr_reviews (optional)
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PrReview {
    pub id: i64,
    pub pr_id: i64,
    pub status: String,
    pub review_notes: Option<String>,
    pub review_file_path: Option<String>,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRule {
    pub id: i64,
    pub repo_id: i64,
    pub rule_text: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub repo_id: i64,
    pub repo_name: String,
    pub synced_at: String,
    pub pr_count: i64,
    pub error: Option<String>,
    /// PR change events detected during this sync cycle.
    #[serde(default)]
    pub changes: Vec<PrChangeEvent>,
}

/// Describes a single change detected during sync (new PR, merged, closed, updated).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrChangeEvent {
    pub repo_name: String,
    pub pr_number: i64,
    pub pr_title: String,
    pub author: String,
    pub change_type: String, // "new", "merged", "closed", "reopened", "updated"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_open_prs: i64,
    pub pending_reviews: i64,
    pub in_progress: i64,
    pub approved: i64,
    pub repos_count: i64,
    pub last_synced: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    pub branch: String,
    pub path: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub pr_id: i64,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTemplate {
    pub id: i64,
    pub name: String,
    pub body: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGroup {
    pub id: i64,
    pub name: String,
    pub colour: String,
    pub position: i64,
    pub repo_ids: Vec<i64>,
}

/// For deserialising gh CLI JSON output
#[derive(Debug, Deserialize)]
pub struct GhPrJson {
    pub number: i64,
    pub title: String,
    pub author: GhAuthor,
    pub state: String,
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
    #[serde(rename = "baseRefName")]
    pub base_ref_name: String,
    pub additions: i64,
    pub deletions: i64,
    #[serde(rename = "changedFiles")]
    pub changed_files: i64,
    #[serde(rename = "reviewDecision")]
    pub review_decision: Option<String>,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    pub url: String,
    pub labels: Vec<GhLabel>,
    pub mergeable: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "mergedAt")]
    pub merged_at: Option<String>,
    #[serde(rename = "closedAt")]
    pub closed_at: Option<String>,
    pub body: Option<String>,
    /// Requested reviewers for this PR (GitHub `reviewRequests` field).
    #[serde(rename = "reviewRequests", default)]
    pub review_requests: Vec<GhReviewRequest>,
}

/// A single review request entry from the `gh` CLI `reviewRequests` field.
#[derive(Debug, Clone, Deserialize)]
pub struct GhReviewRequest {
    pub login: Option<String>,
    pub name: Option<String>,
}

/// Per-reviewer workload statistics aggregated across all tracked repositories.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewerWorkloadStats {
    pub reviewer: String,
    /// Number of open PRs currently assigned for review.
    pub assigned_count: i64,
    /// Number of PRs completed (reviewed/approved/changes_requested).
    pub completed_count: i64,
    /// Number of assigned PRs that are overdue (open > 3 days without a review).
    pub overdue_count: i64,
    /// Average response time in hours from PR creation to first review action.
    pub avg_response_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiCheck {
    pub name: String,
    pub state: String,
    pub conclusion: Option<String>,
    #[serde(rename = "detailsUrl")]
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgeBucket {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VelocityPoint {
    pub date: String,
    pub reviewed: i64,
    pub merged: i64,
}

/// Daily open-PR and pending-review counts for sparkline trends.
#[derive(Debug, Clone, Serialize)]
pub struct DailyPrCounts {
    pub open_counts: Vec<i64>,
    pub pending_counts: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthorStats {
    pub author: String,
    pub pr_count: i64,
    pub avg_additions: f64,
    pub avg_deletions: f64,
    pub merged_count: i64,
    pub reviewed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhAuthor {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct GhLabel {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrComment {
    pub author: GhAuthor,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiReview {
    pub id: i64,
    pub pr_id: i64,
    pub review_text: String,
    pub worktree_branch: String,
    pub created_at: String,
}

/// A GitHub issue linked to a pull request (e.g. via "Fixes #123" in the PR body).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedIssue {
    pub number: i64,
    pub title: String,
    pub state: String,
    pub url: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
}

/// For deserialising `gh issue view --json` output.
#[derive(Debug, Deserialize)]
pub struct GhIssueJson {
    pub number: i64,
    pub title: String,
    pub state: String,
    pub url: String,
    pub labels: Vec<GhLabel>,
    pub assignees: Vec<GhAuthor>,
}

/// A single commit within a pull request, as returned by `gh pr view --json commits`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub oid: String,
    #[serde(rename = "messageHeadline")]
    pub message_headline: String,
    #[serde(rename = "messageBody")]
    pub message_body: String,
    #[serde(rename = "committedDate")]
    pub committed_date: String,
    pub authors: Vec<CommitAuthor>,
}

/// Author information embedded within a commit object from `gh`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    /// The GitHub login, if available.
    pub login: Option<String>,
}

/// Wrapper for the top-level `commits` array in `gh pr view --json commits`.
#[derive(Debug, Deserialize)]
pub struct GhPrCommitsResponse {
    pub commits: Vec<CommitInfo>,
}

/// Merge conflict status for a pull request, fetched via the GitHub CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStatus {
    /// One of "MERGEABLE", "CONFLICTING", or "UNKNOWN".
    pub mergeable: String,
    /// GitHub's merge state status, e.g. "CLEAN", "DIRTY", "BLOCKED", "BEHIND", "UNKNOWN".
    pub merge_state_status: String,
    /// Whether the PR has merge conflicts.
    pub has_conflicts: bool,
}

/// Raw JSON shape returned by `gh pr view --json mergeable,mergeStateStatus`.
#[derive(Debug, Deserialize)]
pub struct GhConflictJson {
    pub mergeable: Option<String>,
    #[serde(rename = "mergeStateStatus")]
    pub merge_state_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrReviewComment {
    pub author: GhAuthor,
    pub body: String,
    pub state: String,
    pub comments: Option<Vec<PrComment>>,
    #[serde(rename = "submittedAt")]
    pub submitted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrCommentsResponse {
    pub comments: Vec<PrComment>,
    pub reviews: Vec<PrReviewComment>,
}

/// A pull request with a computed priority score for the smart review queue.
#[derive(Debug, Clone, Serialize)]
pub struct PriorityQueueItem {
    pub pr: PullRequest,
    pub priority_score: f64,
    pub factors: Vec<PriorityFactor>,
}

/// A single factor contributing to a PR's priority score.
#[derive(Debug, Clone, Serialize)]
pub struct PriorityFactor {
    pub label: String,
    pub points: f64,
}

/// An automation rule that triggers actions when a PR carries a matching label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelRule {
    pub id: i64,
    pub label_pattern: String,
    pub action_type: String,
    pub action_config: serde_json::Value,
    pub enabled: bool,
    pub created_at: String,
}

/// The result of evaluating label rules against a PR's labels.
#[derive(Debug, Clone, Serialize)]
pub struct LabelRuleMatch {
    pub rule: LabelRule,
    pub matched_label: String,
}

/// A deployment status for a branch, fetched from the GitHub Deployments API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub environment: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub url: Option<String>,
}

/// Raw JSON shape for a deployment from `gh api repos/{owner}/{repo}/deployments`.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GhDeploymentJson {
    pub id: u64,
    #[serde(rename = "ref")]
    pub deploy_ref: String,
    pub environment: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Raw JSON shape for a deployment status from `gh api`.
#[derive(Debug, Deserialize)]
pub struct GhDeploymentStatusJson {
    pub state: String,
    pub environment_url: Option<String>,
}

/// A directed dependency edge between two pull requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrDependency {
    pub id: i64,
    pub pr_id: i64,
    pub depends_on_pr_id: i64,
    pub dependency_type: String,
}

/// A focused review session for a single pull request with tracked progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub id: i64,
    pub pr_id: i64,
    pub started_at: String,
    pub files_reviewed: Vec<String>,
    pub session_notes: Option<String>,
    pub status: String,
}
