mod commands;
mod db;
mod github;
mod menu;
mod models;
mod polling;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use db::DbState;
use polling::PollState;
use tauri::{AppHandle, Manager};

#[tauri::command]
fn start_polling(poll_state: tauri::State<'_, Arc<PollState>>) {
    poll_state.enabled.store(true, Ordering::Relaxed);
}

#[tauri::command]
fn stop_polling(poll_state: tauri::State<'_, Arc<PollState>>) {
    poll_state.enabled.store(false, Ordering::Relaxed);
}

#[tauri::command]
fn update_poll_interval(seconds: u64, poll_state: tauri::State<'_, Arc<PollState>>) {
    poll_state
        .interval_seconds
        .store(seconds, Ordering::Relaxed);
}

#[tauri::command]
fn refresh_menu(app_handle: AppHandle) {
    menu::refresh_menu(&app_handle);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let (writer, reader) =
                db::initialise(&data_dir).expect("Failed to initialise database");

            // Read the configured poll interval from app_settings
            let poll_interval: u64 = reader
                .query_row(
                    "SELECT value FROM app_settings WHERE key = 'poll_interval_seconds'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300);

            app.manage(DbState {
                writer: Mutex::new(writer),
                reader: Mutex::new(reader),
            });
            app.manage(Arc::new(PollState::new(poll_interval)));

            // Create the system tray icon
            let handle = app.handle().clone();
            menu::create_tray(&handle).expect("Failed to create tray icon");

            // Start the background polling loop
            polling::start_poll_loop(handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::repositories::add_repository,
            commands::repositories::remove_repository,
            commands::repositories::list_repositories,
            commands::repositories::update_repository_branch,
            commands::checklist::get_checklist_state,
            commands::checklist::save_checklist_state,
            commands::pull_requests::get_pull_requests,
            commands::pull_requests::get_pull_request,
            commands::pull_requests::get_pull_request_body,
            commands::pull_requests::update_review_status,
            commands::pull_requests::get_review_rules,
            commands::pull_requests::set_review_rules,
            commands::sync::sync_pull_requests,
            commands::sync::sync_pull_requests_incremental,
            commands::grove::grove_list_worktrees,
            commands::grove::grove_add_worktree,
            commands::grove::grove_remove_worktree,
            commands::pull_requests::approve_pull_request,
            commands::pull_requests::merge_pull_request,
            commands::stats::get_dashboard_stats,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::editor::open_in_editor,
            commands::checks::fetch_pr_checks,
            commands::comments::fetch_pr_comments,
            commands::diff::fetch_pr_diff,
            commands::stale::get_stale_prs,
            commands::stale::close_pull_request,
            commands::batch::batch_approve,
            commands::batch::batch_merge,
            commands::templates::list_templates,
            commands::templates::set_templates,
            commands::analytics::get_age_distribution,
            commands::analytics::get_review_velocity,
            commands::analytics::get_daily_pr_counts,
            commands::authors::get_author_stats,
            commands::groups::list_groups,
            commands::groups::create_group,
            commands::groups::delete_group,
            commands::groups::add_repo_to_group,
            commands::groups::remove_repo_from_group,
            commands::pull_requests::record_review_time,
            commands::reviews::trigger_worktree_review,
            commands::reviews::list_pr_ai_reviews,
            commands::issues::get_linked_issues,
            commands::commits::get_pr_commits,
            commands::conflicts::check_merge_conflicts,
            commands::priority_queue::get_priority_queue,
            commands::label_rules::list_label_rules,
            commands::label_rules::create_label_rule,
            commands::label_rules::delete_label_rule,
            commands::label_rules::toggle_label_rule,
            commands::label_rules::evaluate_label_rules,
            commands::deployments::get_deployment_status,
            commands::workload::get_reviewer_workload,
            commands::bookmarks::create_bookmark,
            commands::bookmarks::list_bookmarks,
            commands::bookmarks::update_bookmark,
            commands::bookmarks::delete_bookmark,
            commands::bookmarks::list_all_bookmarks,
            commands::bookmarks::get_bookmark_count,
            commands::bookmarks::toggle_bookmark_resolved,
            commands::handoffs::create_handoff,
            commands::handoffs::list_handoffs,
            commands::handoffs::delete_handoff,
            commands::handoffs::export_handoff_to_github,
            commands::digest::get_review_digest,
            commands::dependencies::compute_dependencies,
            commands::dependencies::get_pr_dependencies,
            commands::sessions::create_review_session,
            commands::sessions::get_review_session,
            commands::sessions::get_session_for_pr,
            commands::sessions::update_session_files,
            commands::sessions::update_session_notes,
            commands::sessions::update_session_status,
            commands::sessions::list_review_sessions,
            commands::notifications::list_notification_rules,
            commands::notifications::create_notification_rule,
            commands::notifications::delete_notification_rule,
            commands::notifications::toggle_notification_rule,
            commands::time_tracking::log_review_time,
            commands::time_tracking::get_review_velocity_stats,
            commands::checklist_templates::list_checklist_templates,
            commands::checklist_templates::create_checklist_template,
            commands::checklist_templates::update_checklist_template,
            commands::checklist_templates::delete_checklist_template,
            commands::filter_presets::list_filter_presets,
            commands::filter_presets::create_filter_preset,
            commands::filter_presets::delete_filter_preset,
            commands::filter_presets::rename_filter_preset,
            commands::aggregate::get_aggregate_dashboard,
            commands::review_summary::post_review_summary,
            start_polling,
            stop_polling,
            update_poll_interval,
            refresh_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
