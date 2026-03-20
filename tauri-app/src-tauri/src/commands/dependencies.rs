use regex::Regex;
use rusqlite::params;
use std::sync::OnceLock;
use tauri::State;

use crate::db::DbState;
use crate::models::PrDependency;

use super::CommandError;

/// Dependency type describing how one PR depends on another.
#[derive(Debug, Clone, Copy)]
enum DepType {
    /// Parsed from body text: "Depends on #N", "Blocked by #N", "After #N".
    BodyReference,
    /// PR base branch matches another PR's head branch (stacked PRs).
    BranchAncestry,
}

impl DepType {
    fn as_str(self) -> &'static str {
        match self {
            DepType::BodyReference => "body_reference",
            DepType::BranchAncestry => "branch_ancestry",
        }
    }
}

/// Parse dependency references from PR body text.
///
/// Recognises patterns such as "Depends on #41", "Blocked by #41", "After #41"
/// (case-insensitive). Returns a deduplicated list of PR numbers.
fn parse_dependency_references(body: Option<&str>) -> Vec<i64> {
    let Some(text) = body else {
        return Vec::new();
    };

    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        Regex::new(r"(?i)(?:depends\s+on|blocked\s+by|after)\s+#(\d+)").expect("Invalid regex")
    });

    let mut numbers: Vec<i64> = Vec::new();
    for caps in pattern.captures_iter(text) {
        if let Some(m) = caps.get(1) {
            if let Ok(n) = m.as_str().parse::<i64>() {
                if !numbers.contains(&n) {
                    numbers.push(n);
                }
            }
        }
    }
    numbers
}

/// Compute and persist dependency edges for all open PRs within the same repository.
///
/// Two detection strategies are used:
/// 1. **Body references**: parse "Depends on #N" / "Blocked by #N" / "After #N" from the PR body.
/// 2. **Branch ancestry**: if PR A's base_branch equals PR B's head_branch within the same repo,
///    then A depends on B (stacked PRs).
#[tauri::command]
pub fn compute_dependencies(state: State<'_, DbState>) -> Result<Vec<PrDependency>, CommandError> {
    let db = state.writer.lock().unwrap();

    // Clear existing computed dependencies — they are re-derived each time.
    db.execute("DELETE FROM pr_dependencies", [])?;

    // Fetch all open PRs with their body, head/base branch, number, and repo_id.
    let mut stmt = db.prepare(
        "SELECT id, repo_id, number, body, head_branch, base_branch
         FROM pull_requests
         WHERE state = 'OPEN' AND merged_at IS NULL AND closed_at IS NULL",
    )?;

    struct PrRow {
        id: i64,
        repo_id: i64,
        number: i64,
        body: Option<String>,
        head_branch: String,
        base_branch: String,
    }

    let rows: Vec<PrRow> = stmt
        .query_map([], |row| {
            Ok(PrRow {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                number: row.get(2)?,
                body: row.get(3)?,
                head_branch: row.get(4)?,
                base_branch: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut edges: Vec<(i64, i64, &str)> = Vec::new();

    for pr in &rows {
        // Strategy 1: body references
        let dep_numbers = parse_dependency_references(pr.body.as_deref());
        for dep_num in dep_numbers {
            // Find the target PR in the same repo by number
            if let Some(target) = rows
                .iter()
                .find(|r| r.repo_id == pr.repo_id && r.number == dep_num)
            {
                edges.push((pr.id, target.id, DepType::BodyReference.as_str()));
            }
        }

        // Strategy 2: branch ancestry (stacked PRs)
        // If this PR's base_branch is another open PR's head_branch in the same repo,
        // then this PR depends on that other PR.
        for other in &rows {
            if other.id == pr.id || other.repo_id != pr.repo_id {
                continue;
            }
            if pr.base_branch == other.head_branch {
                edges.push((pr.id, other.id, DepType::BranchAncestry.as_str()));
            }
        }
    }

    // Deduplicate edges (same pr_id + depends_on_pr_id)
    edges.sort_by_key(|e| (e.0, e.1));
    edges.dedup_by_key(|e| (e.0, e.1));

    // Persist edges
    let mut insert = db.prepare(
        "INSERT INTO pr_dependencies (pr_id, depends_on_pr_id, dependency_type)
         VALUES (?1, ?2, ?3)",
    )?;
    for (pr_id, dep_id, dep_type) in &edges {
        insert.execute(params![pr_id, dep_id, dep_type])?;
    }

    // Return all dependencies
    let mut query =
        db.prepare("SELECT id, pr_id, depends_on_pr_id, dependency_type FROM pr_dependencies")?;
    let deps = query
        .query_map([], |row| {
            Ok(PrDependency {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                depends_on_pr_id: row.get(2)?,
                dependency_type: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(deps)
}

/// Retrieve stored dependency edges. Call `compute_dependencies` first to populate.
#[tauri::command]
pub fn get_pr_dependencies(state: State<'_, DbState>) -> Result<Vec<PrDependency>, CommandError> {
    let db = state.reader.lock().unwrap();

    let mut stmt =
        db.prepare("SELECT id, pr_id, depends_on_pr_id, dependency_type FROM pr_dependencies")?;
    let deps = stmt
        .query_map([], |row| {
            Ok(PrDependency {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                depends_on_pr_id: row.get(2)?,
                dependency_type: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_depends_on() {
        let nums = parse_dependency_references(Some("Depends on #41 and blocked by #42"));
        assert_eq!(nums, vec![41, 42]);
    }

    #[test]
    fn test_parse_after() {
        let nums = parse_dependency_references(Some("After #10"));
        assert_eq!(nums, vec![10]);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let nums = parse_dependency_references(Some("BLOCKED BY #5"));
        assert_eq!(nums, vec![5]);
    }

    #[test]
    fn test_parse_deduplicates() {
        let nums = parse_dependency_references(Some("Depends on #3. Also depends on #3"));
        assert_eq!(nums, vec![3]);
    }

    #[test]
    fn test_parse_no_deps() {
        let nums = parse_dependency_references(Some("Just a regular PR body"));
        assert!(nums.is_empty());
    }

    #[test]
    fn test_parse_none_body() {
        let nums = parse_dependency_references(None);
        assert!(nums.is_empty());
    }
}
