use crate::models::Worktree;

use super::{command_for, CommandError};

#[tauri::command]
pub fn grove_list_worktrees(repo_name: String) -> Result<Vec<Worktree>, CommandError> {
    let output = command_for("grove", "Grove CLI", CommandError::Grove)?
        .args(["ls", &repo_name])
        .output()
        .map_err(|e| CommandError::Grove(format!("Failed to run grove: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Grove(format!("grove ls failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let worktrees = parse_grove_output(&stdout);
    Ok(worktrees)
}

/// Parse the output of `grove ls` into a list of worktrees.
///
/// Expected format (one per line):
///   branch-name  /path/to/worktree
/// Lines may contain (current) or * markers and [locked] annotations.
/// Header lines (starting with "Worktrees" or "=") are skipped.
fn parse_grove_output(output: &str) -> Vec<Worktree> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.starts_with("Worktrees") && !line.starts_with('='))
        .filter_map(|line| {
            let trimmed = line.trim();
            let is_current = trimmed.contains("(current)") || trimmed.contains('*');
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let branch = parts[0].trim_start_matches('*').trim().to_string();
                let path = parts[1].to_string();
                Some(Worktree {
                    branch,
                    path,
                    is_current,
                })
            } else if !parts.is_empty() {
                Some(Worktree {
                    branch: parts[0].to_string(),
                    path: String::new(),
                    is_current,
                })
            } else {
                None
            }
        })
        .collect()
}

#[tauri::command]
pub fn grove_add_worktree(
    repo_name: String,
    branch: String,
    base_branch: Option<String>,
) -> Result<String, CommandError> {
    let base = base_branch.unwrap_or_else(|| "origin/develop".to_string());
    let output = command_for("grove", "Grove CLI", CommandError::Grove)?
        .args(["add", &repo_name, &branch, &base, "-f"])
        .output()
        .map_err(|e| CommandError::Grove(format!("Failed to run grove: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Grove(format!("grove add failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub fn grove_remove_worktree(repo_name: String, branch: String) -> Result<String, CommandError> {
    let output = command_for("grove", "Grove CLI", CommandError::Grove)?
        .args(["rm", &repo_name, &branch])
        .output()
        .map_err(|e| CommandError::Grove(format!("Failed to run grove: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Grove(format!("grove rm failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
