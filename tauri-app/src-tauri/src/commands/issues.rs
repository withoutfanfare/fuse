use regex::Regex;
use std::sync::OnceLock;
use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::LinkedIssue;

use super::CommandError;

/// Parse issue references from PR title and body text.
///
/// Recognises patterns such as `#123`, `fixes #123`, `closes #456`,
/// `resolves #789` (case-insensitive). Returns a deduplicated list of
/// issue numbers.
fn parse_issue_references(title: &str, body: Option<&str>) -> Vec<i64> {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        Regex::new(r"(?i)(?:fix(?:es|ed)?|close[sd]?|resolve[sd]?)\s+#(\d+)|#(\d+)")
            .expect("Invalid regex")
    });

    let mut numbers: Vec<i64> = Vec::new();

    for text in std::iter::once(title).chain(body.into_iter()) {
        for caps in pattern.captures_iter(text) {
            // First capture group is from keyword patterns, second from bare #N
            let num_str = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();
            if let Ok(n) = num_str.parse::<i64>() {
                if !numbers.contains(&n) {
                    numbers.push(n);
                }
            }
        }
    }

    numbers
}

/// Fetch linked issues for a pull request.
///
/// Parses issue references from the PR title and body, then fetches each
/// issue's details from GitHub concurrently via async subprocess calls.
#[tauri::command]
pub async fn get_linked_issues(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<LinkedIssue>, CommandError> {
    // Read DB state first, then drop the guard before awaiting
    let (full_name, title, body) = {
        let db = state.reader.lock().unwrap();

        let (repo_id, title, body): (i64, String, Option<String>) = db
            .query_row(
                "SELECT repo_id, title, body FROM pull_requests WHERE id = ?1",
                [pr_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    CommandError::NotFound(format!("Pull request with id {pr_id}"))
                }
                other => CommandError::Db(other),
            })?;

        let full_name: String = db.query_row(
            "SELECT owner || '/' || name FROM repositories WHERE id = ?1",
            [repo_id],
            |row| row.get(0),
        )?;

        (full_name, title, body)
    };

    let issue_numbers = parse_issue_references(&title, body.as_deref());

    // Fetch all linked issues concurrently using async tasks
    let mut handles = Vec::new();
    for number in issue_numbers {
        let full_name = full_name.clone();
        handles.push(tokio::spawn(async move {
            github::fetch_issue_async(&full_name, number)
                .await
                .ok()
                .map(|issue| LinkedIssue {
                    number: issue.number,
                    title: issue.title,
                    state: issue.state,
                    url: issue.url,
                    labels: issue.labels.into_iter().map(|l| l.name).collect(),
                    assignees: issue.assignees.into_iter().map(|a| a.login).collect(),
                })
        }));
    }

    let mut linked_issues = Vec::new();
    for handle in handles {
        if let Ok(Some(issue)) = handle.await {
            linked_issues.push(issue);
        }
    }

    Ok(linked_issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixes_pattern() {
        let numbers = parse_issue_references("Fix login bug", Some("Fixes #123 and closes #456"));
        assert_eq!(numbers, vec![123, 456]);
    }

    #[test]
    fn test_parse_bare_hash() {
        let numbers = parse_issue_references("PR for #42", None);
        assert_eq!(numbers, vec![42]);
    }

    #[test]
    fn test_parse_deduplicates() {
        let numbers = parse_issue_references("Fixes #10", Some("Also closes #10"));
        assert_eq!(numbers, vec![10]);
    }

    #[test]
    fn test_parse_no_references() {
        let numbers = parse_issue_references("Refactor utils", Some("General cleanup"));
        assert!(numbers.is_empty());
    }

    #[test]
    fn test_parse_case_insensitive() {
        let numbers = parse_issue_references("RESOLVES #99", None);
        assert_eq!(numbers, vec![99]);
    }
}
