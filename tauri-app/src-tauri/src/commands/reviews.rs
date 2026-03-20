use std::io::Write;
use std::process::Stdio;

use rusqlite::Connection;
use tauri::State;

use crate::db::DbState;
use crate::github;
use crate::models::AiReview;

use super::{command_for, CommandError};

struct ReviewContext {
    repo_name: String,
    full_name: String,
    pr_number: i64,
    head_branch: String,
    base_branch: String,
    title: String,
    author: String,
    body: Option<String>,
}

fn get_review_context(db: &Connection, pr_id: i64) -> Result<ReviewContext, CommandError> {
    let mut stmt = db.prepare(
        "SELECT r.name, r.owner || '/' || r.name, p.number, p.head_branch, p.base_branch,
                p.title, p.author, p.body
         FROM pull_requests p
         JOIN repositories r ON r.id = p.repo_id
         WHERE p.id = ?1",
    )?;

    let ctx = stmt.query_row(rusqlite::params![pr_id], |row| {
        Ok(ReviewContext {
            repo_name: row.get(0)?,
            full_name: row.get(1)?,
            pr_number: row.get(2)?,
            head_branch: row.get(3)?,
            base_branch: row.get(4)?,
            title: row.get(5)?,
            author: row.get(6)?,
            body: row.get(7)?,
        })
    })?;

    Ok(ctx)
}

fn grove_add(repo_name: &str, branch: &str) -> Result<(), CommandError> {
    let output = command_for("grove", "Grove CLI", CommandError::Grove)?
        .args(["add", repo_name, branch, "-f"])
        .output()
        .map_err(|e| CommandError::Grove(format!("Failed to run grove: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Grove(format!("grove add failed: {}", stderr)));
    }

    Ok(())
}

fn grove_remove(repo_name: &str, branch: &str) {
    if let Ok(mut command) = command_for("grove", "Grove CLI", CommandError::Grove) {
        let _ = command.args(["rm", repo_name, branch]).output();
    }
}

fn get_diff(full_name: &str, pr_number: i64) -> Result<String, CommandError> {
    github::fetch_diff(full_name, pr_number)
}

fn build_review_prompt(ctx: &ReviewContext, diff: &str) -> String {
    let body_section = match &ctx.body {
        Some(b) if !b.is_empty() => format!("\n## Description\n{}\n", b),
        _ => String::new(),
    };

    format!(
        r#"You are a senior engineer performing a code review. Review the following pull request and provide structured feedback.

## Pull Request
- Title: {}
- Author: {}
- Repository: {}
- Base branch: {}
- Head branch: {}
{}
## Diff
```diff
{}
```

## Instructions
Provide a thorough code review covering:
1. Correctness — logic errors, edge cases, off-by-one errors
2. Security — injection, authentication, authorisation issues
3. Performance — N+1 queries, unnecessary allocations, blocking calls
4. Maintainability — naming, complexity, test coverage gaps
5. Conventions — adherence to patterns visible in the diff context

Format your response as:
### Summary
A 2-3 sentence overview of the changes and overall quality.

### Issues
For each issue, use this format:
- **[CRITICAL|WARNING|SUGGESTION]** `path/to/file.ext` — Description of the issue.

### Verdict
Either "APPROVED" or "CHANGES REQUESTED", followed by one sentence justification."#,
        ctx.title, ctx.author, ctx.full_name, ctx.base_branch, ctx.head_branch, body_section, diff,
    )
}

fn run_claude(prompt: &str) -> Result<String, CommandError> {
    let mut child = command_for("claude", "Claude CLI", CommandError::Claude)?
        .args(["--print"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CommandError::Claude(
                    "claude CLI not found. Install it with: npm install -g @anthropic-ai/claude-code"
                        .to_string(),
                )
            } else {
                CommandError::Claude(format!("Failed to start claude: {}", e))
            }
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| CommandError::Claude(format!("Failed to write to claude stdin: {}", e)))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| CommandError::Claude(format!("Failed to read claude output: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Claude(format!(
            "claude exited with error: {}",
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn save_review(
    db: &Connection,
    pr_id: i64,
    review_text: &str,
    branch: &str,
) -> Result<AiReview, CommandError> {
    db.execute(
        "INSERT INTO pr_ai_reviews (pr_id, review_text, worktree_branch) VALUES (?1, ?2, ?3)",
        rusqlite::params![pr_id, review_text, branch],
    )?;

    let id = db.last_insert_rowid();
    let created_at: String = db.query_row(
        "SELECT created_at FROM pr_ai_reviews WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )?;

    Ok(AiReview {
        id,
        pr_id,
        review_text: review_text.to_string(),
        worktree_branch: branch.to_string(),
        created_at,
    })
}

#[tauri::command]
pub fn trigger_worktree_review(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<AiReview, CommandError> {
    // 1. Lock DB briefly to get context, then drop lock
    let ctx = {
        let db = state.writer.lock().unwrap();
        get_review_context(&db, pr_id)?
    };

    let branch = format!("pr-{}", ctx.pr_number);

    // 2. Create worktree
    grove_add(&ctx.repo_name, &branch)?;

    // 3. Get diff (on failure, clean up worktree)
    let diff = match get_diff(&ctx.full_name, ctx.pr_number) {
        Ok(d) => d,
        Err(e) => {
            grove_remove(&ctx.repo_name, &branch);
            return Err(e);
        }
    };

    // 4. Build prompt and run claude (on failure, clean up worktree)
    let prompt = build_review_prompt(&ctx, &diff);
    let review_text = match run_claude(&prompt) {
        Ok(text) => text,
        Err(e) => {
            grove_remove(&ctx.repo_name, &branch);
            return Err(e);
        }
    };

    // 5. Re-lock DB to save the review
    let review = {
        let db = state.writer.lock().unwrap();
        save_review(&db, pr_id, &review_text, &branch)?
    };

    // 6. Clean up worktree
    grove_remove(&ctx.repo_name, &branch);

    Ok(review)
}

#[tauri::command]
pub fn list_pr_ai_reviews(
    pr_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<AiReview>, CommandError> {
    let db = state.reader.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, pr_id, review_text, worktree_branch, created_at
         FROM pr_ai_reviews WHERE pr_id = ?1 ORDER BY created_at DESC LIMIT 20",
    )?;

    let reviews = stmt
        .query_map(rusqlite::params![pr_id], |row| {
            Ok(AiReview {
                id: row.get(0)?,
                pr_id: row.get(1)?,
                review_text: row.get(2)?,
                worktree_branch: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(reviews)
}
