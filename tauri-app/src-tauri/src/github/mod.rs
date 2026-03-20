use crate::commands::{command_for, tokio_command_for, CommandError};
use crate::models::{
    CiCheck, CommitInfo, ConflictStatus, Deployment, GhConflictJson, GhDeploymentJson,
    GhDeploymentStatusJson, GhIssueJson, GhPrCommitsResponse, GhPrJson,
};

/// Fields to request from the GitHub CLI when listing pull requests.
const GH_PR_FIELDS: &str = "number,title,author,state,headRefName,baseRefName,additions,deletions,changedFiles,reviewDecision,isDraft,url,labels,mergeable,createdAt,updatedAt,mergedAt,closedAt,body,reviewRequests";

fn format_merge_error(stderr: &str) -> String {
    if stderr.contains("Pull Request is still a draft") {
        "Cannot merge a draft pull request. Mark it ready for review on GitHub first.".to_string()
    } else {
        format!("gh pr merge failed: {}", stderr)
    }
}

/// Fetch all pull requests for a repository using the `gh` CLI.
///
/// Runs `gh pr list` with `--state all` to retrieve open, closed, and merged PRs
/// and returns the parsed JSON output.
pub fn fetch_prs(full_name: &str) -> Result<Vec<GhPrJson>, CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "list",
            "--repo",
            full_name,
            "--state",
            "all",
            "--limit",
            "100",
            "--json",
            GH_PR_FIELDS,
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!("gh pr list failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let prs: Vec<GhPrJson> = serde_json::from_str(&stdout)?;
    Ok(prs)
}

/// Fetch only pull requests updated after a given timestamp (incremental sync).
///
/// Uses `gh pr list --search "updated:>TIMESTAMP"` to fetch only recently modified PRs,
/// dramatically reducing API calls for repositories with many open PRs.
pub fn fetch_prs_since(full_name: &str, since: &str) -> Result<Vec<GhPrJson>, CommandError> {
    let search_query = format!("updated:>{}", since);
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "list",
            "--repo",
            full_name,
            "--state",
            "all",
            "--limit",
            "100",
            "--search",
            &search_query,
            "--json",
            GH_PR_FIELDS,
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr list (delta) failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let prs: Vec<GhPrJson> = serde_json::from_str(&stdout)?;
    Ok(prs)
}

/// Approve a pull request on GitHub using the `gh` CLI.
pub fn approve_pr(
    full_name: &str,
    pr_number: i64,
    body: Option<&str>,
) -> Result<String, CommandError> {
    let mut args = vec![
        "pr".to_string(),
        "review".to_string(),
        pr_number.to_string(),
        "--repo".to_string(),
        full_name.to_string(),
        "--approve".to_string(),
    ];
    if let Some(b) = body {
        args.push("--body".to_string());
        args.push(b.to_string());
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args(&arg_refs)
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr review --approve failed: {}",
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Merge a pull request on GitHub using the `gh` CLI.
///
/// Uses squash merge by default. The caller must validate the target branch
/// before calling this function.
pub fn merge_pr(
    full_name: &str,
    pr_number: i64,
    method: Option<&str>,
) -> Result<String, CommandError> {
    let merge_method = match method.unwrap_or("squash") {
        "rebase" => "--rebase",
        "merge" => "--merge",
        _ => "--squash",
    };

    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "merge",
            &pr_number.to_string(),
            "--repo",
            full_name,
            merge_method,
            "--delete-branch",
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format_merge_error(&stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Fetch CI/CD check statuses for a pull request using the `gh` CLI.
#[allow(dead_code)]
pub fn fetch_checks(full_name: &str, pr_number: i64) -> Result<Vec<CiCheck>, CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "checks",
            &pr_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "name,state,conclusion,detailsUrl",
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!("gh pr checks failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let checks: Vec<CiCheck> = serde_json::from_str(&stdout)?;
    Ok(checks)
}

/// Fetch comments and reviews for a pull request using the `gh` CLI.
#[allow(dead_code)]
pub fn fetch_comments(full_name: &str, pr_number: i64) -> Result<serde_json::Value, CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "comments,reviews",
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr view comments failed: {}",
            stderr
        )));
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(value)
}

/// Fetch the unified diff for a pull request using the `gh` CLI.
pub fn fetch_diff(full_name: &str, pr_number: i64) -> Result<String, CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args(["pr", "diff", &pr_number.to_string(), "--repo", full_name])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!("gh pr diff failed: {}", stderr)));
    }

    Ok(String::from_utf8(output.stdout)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()))
}

/// Fetch details for a single GitHub issue by number using the `gh` CLI.
#[allow(dead_code)]
pub fn fetch_issue(full_name: &str, issue_number: i64) -> Result<GhIssueJson, CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "issue",
            "view",
            &issue_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "number,title,state,url,labels,assignees",
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh issue view failed: {}",
            stderr
        )));
    }

    let issue: GhIssueJson = serde_json::from_slice(&output.stdout)?;
    Ok(issue)
}

/// Fetch commit history for a pull request using `gh pr view --json commits`.
pub fn fetch_commits(full_name: &str, pr_number: i64) -> Result<Vec<CommitInfo>, CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "commits",
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr view commits failed: {}",
            stderr
        )));
    }

    let response: GhPrCommitsResponse = serde_json::from_slice(&output.stdout)?;
    Ok(response.commits)
}

/// Fetch merge conflict status for a pull request using the `gh` CLI.
///
/// Queries `gh pr view --json mergeable,mergeStateStatus` and derives
/// whether the PR currently has merge conflicts.
#[allow(dead_code)]
pub fn fetch_conflict_status(
    full_name: &str,
    pr_number: i64,
) -> Result<ConflictStatus, CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "mergeable,mergeStateStatus",
        ])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr view mergeable failed: {}",
            stderr
        )));
    }

    let raw: GhConflictJson = serde_json::from_slice(&output.stdout)?;
    let mergeable = raw.mergeable.unwrap_or_else(|| "UNKNOWN".to_string());
    let merge_state_status = raw
        .merge_state_status
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let has_conflicts = mergeable == "CONFLICTING";

    Ok(ConflictStatus {
        mergeable,
        merge_state_status,
        has_conflicts,
    })
}

/// Fetch deployments for a specific branch from the GitHub Deployments API.
///
/// Queries `gh api repos/{owner}/{repo}/deployments?ref={branch}` to retrieve
/// deployments matching the PR branch. For each deployment, fetches the latest
/// status to determine the current state and environment URL.
#[allow(dead_code)]
pub fn fetch_deployments(full_name: &str, branch: &str) -> Result<Vec<Deployment>, CommandError> {
    let endpoint = format!("repos/{full_name}/deployments?ref={branch}");
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args(["api", &endpoint])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If the API returns 404, the repo likely has no deployments — return empty
        if stderr.contains("404") || stderr.contains("Not Found") {
            return Ok(vec![]);
        }
        return Err(CommandError::Gh(format!(
            "gh api deployments failed: {}",
            stderr
        )));
    }

    let all: Vec<GhDeploymentJson> = serde_json::from_slice(&output.stdout)?;

    // Fetch the latest status for each deployment in parallel
    let mut deployments: Vec<Deployment> = std::thread::scope(|s| {
        let handles: Vec<_> =
            all.iter()
                .map(|dep| {
                    let full_name = full_name;
                    s.spawn(move || {
                        let status_endpoint = format!(
                            "repos/{}/deployments/{}/statuses?per_page=1",
                            full_name, dep.id
                        );
                        let status_output = command_for("gh", "GitHub CLI", CommandError::Gh)
                            .and_then(|mut command| {
                                command
                                    .args(["api", &status_endpoint])
                                    .output()
                                    .map_err(|e| {
                                        CommandError::Gh(format!("Failed to run gh CLI: {}", e))
                                    })
                            });

                        let (status, env_url) = match status_output {
                            Ok(out) if out.status.success() => {
                                let statuses: Vec<GhDeploymentStatusJson> =
                                    serde_json::from_slice(&out.stdout).unwrap_or_default();
                                match statuses.first() {
                                    Some(s) => (s.state.clone(), s.environment_url.clone()),
                                    None => ("pending".to_string(), None),
                                }
                            }
                            _ => ("unknown".to_string(), None),
                        };

                        Deployment {
                            environment: dep.environment.clone(),
                            status,
                            created_at: dep.created_at.clone(),
                            updated_at: dep.updated_at.clone(),
                            url: env_url,
                        }
                    })
                })
                .collect();

        handles
            .into_iter()
            .map(|h| h.join().expect("Deployment status thread panicked"))
            .collect()
    });

    // Deduplicate by environment — keep only the most recent deployment per environment
    deployments.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let mut seen = std::collections::HashSet::new();
    deployments.retain(|d| seen.insert(d.environment.clone()));

    Ok(deployments)
}

// ---------------------------------------------------------------------------
// Async variants — non-blocking subprocess execution via tokio::process
// ---------------------------------------------------------------------------

/// Async variant of `fetch_checks` using `tokio::process::Command`.
pub async fn fetch_checks_async(
    full_name: &str,
    pr_number: i64,
) -> Result<Vec<CiCheck>, CommandError> {
    let output = tokio_command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "checks",
            &pr_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "name,state,conclusion,detailsUrl",
        ])
        .output()
        .await
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!("gh pr checks failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let checks: Vec<CiCheck> = serde_json::from_str(&stdout)?;
    Ok(checks)
}

/// Async variant of `fetch_comments` using `tokio::process::Command`.
pub async fn fetch_comments_async(
    full_name: &str,
    pr_number: i64,
) -> Result<serde_json::Value, CommandError> {
    let output = tokio_command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "comments,reviews",
        ])
        .output()
        .await
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr view comments failed: {}",
            stderr
        )));
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(value)
}

/// Async variant of `fetch_diff` using `tokio::process::Command`.
pub async fn fetch_diff_async(full_name: &str, pr_number: i64) -> Result<String, CommandError> {
    let output = tokio_command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args(["pr", "diff", &pr_number.to_string(), "--repo", full_name])
        .output()
        .await
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!("gh pr diff failed: {}", stderr)));
    }

    Ok(String::from_utf8(output.stdout)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()))
}

/// Async variant of `fetch_conflict_status` using `tokio::process::Command`.
pub async fn fetch_conflict_status_async(
    full_name: &str,
    pr_number: i64,
) -> Result<ConflictStatus, CommandError> {
    let output = tokio_command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "mergeable,mergeStateStatus",
        ])
        .output()
        .await
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh pr view mergeable failed: {}",
            stderr
        )));
    }

    let raw: GhConflictJson = serde_json::from_slice(&output.stdout)?;
    let mergeable = raw.mergeable.unwrap_or_else(|| "UNKNOWN".to_string());
    let merge_state_status = raw
        .merge_state_status
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let has_conflicts = mergeable == "CONFLICTING";

    Ok(ConflictStatus {
        mergeable,
        merge_state_status,
        has_conflicts,
    })
}

/// Async variant of `fetch_deployments` using `tokio::process::Command`.
pub async fn fetch_deployments_async(
    full_name: &str,
    branch: &str,
) -> Result<Vec<Deployment>, CommandError> {
    let endpoint = format!("repos/{full_name}/deployments?ref={branch}");
    let output = tokio_command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args(["api", &endpoint])
        .output()
        .await
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("404") || stderr.contains("Not Found") {
            return Ok(vec![]);
        }
        return Err(CommandError::Gh(format!(
            "gh api deployments failed: {}",
            stderr
        )));
    }

    let all: Vec<GhDeploymentJson> = serde_json::from_slice(&output.stdout)?;

    // Fetch status for each deployment concurrently using tokio tasks
    let mut handles = Vec::new();
    for dep in &all {
        let status_endpoint = format!(
            "repos/{}/deployments/{}/statuses?per_page=1",
            full_name, dep.id
        );
        let environment = dep.environment.clone();
        let created_at = dep.created_at.clone();
        let updated_at = dep.updated_at.clone();

        handles.push(tokio::spawn(async move {
            let status_output = match tokio_command_for("gh", "GitHub CLI", CommandError::Gh) {
                Ok(mut command) => command.args(["api", &status_endpoint]).output().await,
                Err(err) => return Err(err),
            };

            let (status, env_url) = match status_output {
                Ok(out) if out.status.success() => {
                    let statuses: Vec<GhDeploymentStatusJson> =
                        serde_json::from_slice(&out.stdout).unwrap_or_default();
                    match statuses.first() {
                        Some(s) => (s.state.clone(), s.environment_url.clone()),
                        None => ("pending".to_string(), None),
                    }
                }
                _ => ("unknown".to_string(), None),
            };

            Ok::<Deployment, CommandError>(Deployment {
                environment,
                status,
                created_at,
                updated_at,
                url: env_url,
            })
        }));
    }

    let mut deployments = Vec::new();
    for handle in handles {
        deployments.push(
            handle
                .await
                .map_err(|e| CommandError::Gh(format!("Deployment status task failed: {}", e)))??,
        );
    }

    // Deduplicate by environment — keep only the most recent deployment per environment
    deployments.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let mut seen = std::collections::HashSet::new();
    deployments.retain(|d| seen.insert(d.environment.clone()));

    Ok(deployments)
}

/// Async variant of `fetch_issue` using `tokio::process::Command`.
pub async fn fetch_issue_async(
    full_name: &str,
    issue_number: i64,
) -> Result<GhIssueJson, CommandError> {
    let output = tokio_command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args([
            "issue",
            "view",
            &issue_number.to_string(),
            "--repo",
            full_name,
            "--json",
            "number,title,state,url,labels,assignees",
        ])
        .output()
        .await
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!(
            "gh issue view failed: {}",
            stderr
        )));
    }

    let issue: GhIssueJson = serde_json::from_slice(&output.stdout)?;
    Ok(issue)
}

/// Close a pull request on GitHub using the `gh` CLI.
pub fn close_pr(full_name: &str, pr_number: i64) -> Result<(), CommandError> {
    let output = command_for("gh", "GitHub CLI", CommandError::Gh)?
        .args(["pr", "close", &pr_number.to_string(), "--repo", full_name])
        .output()
        .map_err(|e| CommandError::Gh(format!("Failed to run gh CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Gh(format!("gh pr close failed: {}", stderr)));
    }

    Ok(())
}
