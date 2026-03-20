use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::commands::sync::sync_pull_requests_with_db;
use crate::db::DbState;
use crate::models::{PrChangeEvent, SyncResult};

/// Shared state that controls the background polling loop.
pub struct PollState {
    pub enabled: AtomicBool,
    pub interval_seconds: AtomicU64,
}

impl PollState {
    pub fn new(interval: u64) -> Self {
        Self {
            enabled: AtomicBool::new(true),
            interval_seconds: AtomicU64::new(interval),
        }
    }
}

/// Spawn an async task that periodically syncs all repositories.
///
/// After each successful sync cycle the `sync-completed` event is emitted to
/// the frontend so the UI can refresh its data. Native notifications are sent
/// for meaningful PR changes (new, merged, closed, reopened).
pub fn start_poll_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let poll_state = app.state::<Arc<PollState>>();
            let interval = poll_state.interval_seconds.load(Ordering::Relaxed);

            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

            if !poll_state.enabled.load(Ordering::Relaxed) {
                continue;
            }

            // Emit sync-started so the frontend can show a loading indicator
            let _ = app.emit("sync-started", ());

            // Run the sync for all repositories
            let db_state = app.state::<DbState>();
            let results: Vec<SyncResult> = match sync_pull_requests_with_db(None, &db_state.writer)
            {
                Ok(r) => r,
                Err(e) => {
                    // Emit the error so the frontend can surface it
                    let _ = app.emit("sync-error", e.to_string());
                    Vec::new()
                }
            };

            // Send native notifications for important changes
            send_change_notifications(&app, &results);

            // Emit sync-completed for any successful sync cycle so the
            // frontend can refresh data and surface per-repo errors.
            if !results.is_empty() {
                let _ = app.emit("sync-completed", &results);
                // Refresh the menu bar PR list if anything changed
                let has_changes = results.iter().any(|r| !r.changes.is_empty());
                if has_changes {
                    crate::menu::refresh_menu(&app);
                }
            }
        }
    });
}

/// Send macOS native notifications for PR changes detected during sync.
///
/// Only notifies for new PRs (individual) and updated PRs (batched summary).
/// Merged, closed, and reopened changes are silently recorded without notification.
fn send_change_notifications(app: &AppHandle, results: &[SyncResult]) {
    let all_changes: Vec<&PrChangeEvent> = results.iter().flat_map(|r| r.changes.iter()).collect();

    if all_changes.is_empty() {
        return;
    }

    // New PRs get individual notifications
    for change in all_changes.iter().filter(|c| c.change_type == "new") {
        let (title, body) = format_notification(change);
        let _ = app
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show();
    }

    // Batch "updated" changes into a single notification
    let updated_count = all_changes
        .iter()
        .filter(|c| c.change_type == "updated")
        .count();

    if updated_count > 0 {
        let body = if updated_count == 1 {
            let change = all_changes
                .iter()
                .find(|c| c.change_type == "updated")
                .unwrap();
            format!(
                "#{} {} was updated",
                change.pr_number,
                truncate(&change.pr_title, 40)
            )
        } else {
            format!("{} pull requests were updated", updated_count)
        };

        let _ = app
            .notification()
            .builder()
            .title("PRs Updated")
            .body(&body)
            .show();
    }
}

/// Format a notification title and body for a single PR change.
fn format_notification(change: &PrChangeEvent) -> (String, String) {
    let short_title = truncate(&change.pr_title, 50);
    match change.change_type.as_str() {
        "new" => (
            format!("New PR #{}", change.pr_number),
            format!(
                "{} by {} in {}",
                short_title,
                change.author,
                short_repo(&change.repo_name)
            ),
        ),
        "merged" => (
            format!("PR #{} Merged", change.pr_number),
            format!("{} in {}", short_title, short_repo(&change.repo_name)),
        ),
        "closed" => (
            format!("PR #{} Closed", change.pr_number),
            format!("{} in {}", short_title, short_repo(&change.repo_name)),
        ),
        "reopened" => (
            format!("PR #{} Reopened", change.pr_number),
            format!(
                "{} by {} in {}",
                short_title,
                change.author,
                short_repo(&change.repo_name)
            ),
        ),
        _ => (
            format!("PR #{} Updated", change.pr_number),
            format!("{} in {}", short_title, short_repo(&change.repo_name)),
        ),
    }
}

/// Extract just the repo name from "owner/repo" format.
fn short_repo(full_name: &str) -> &str {
    full_name.rsplit('/').next().unwrap_or(full_name)
}

/// Truncate a string to a maximum length, appending ellipsis if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
