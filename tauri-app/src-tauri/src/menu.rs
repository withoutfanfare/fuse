use chrono::{NaiveDateTime, Utc};
use rusqlite::Connection;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

use crate::db::DbState;

/// PR data for tray menu display.
struct MenuPr {
    id: i64,
    number: i64,
    title: String,
    author: String,
    additions: i64,
    deletions: i64,
    changed_files: i64,
    is_draft: bool,
    review_decision: Option<String>,
    created_at: String,
    repo_name: String,
}

/// Create the system tray icon with a PR menu.
pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;

    let icon = Image::from_path("icons/32x32.png")
        .or_else(|_| app.default_window_icon().cloned().ok_or("no icon"))
        .map_err(|e| format!("Failed to load tray icon: {}", e))?;

    TrayIconBuilder::with_id("fuse-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Fuse — PR Review Companion")
        .on_menu_event(|app, event| {
            handle_tray_menu_event(app, &event);
        })
        .build(app)?;

    Ok(())
}

/// Format a PR as a rich menu item label.
///
/// Example: "  #1302 Stripe wallet data in payments… — hjeewa · +604 −86 · 13 files · 1d"
fn format_pr_label(pr: &MenuPr) -> String {
    let title = truncate(&pr.title, 40);
    let draft = if pr.is_draft { " [draft]" } else { "" };
    let review = match pr.review_decision.as_deref() {
        Some("APPROVED") => " ✓",
        Some("CHANGES_REQUESTED") => " ✗",
        _ => "",
    };
    let age = format_age(&pr.created_at);
    let size = format_size(pr.additions, pr.deletions, pr.changed_files);

    format!(
        "#{}{}{} {}  —  {} · {} · {}",
        pr.number, draft, review, title, pr.author, size, age
    )
}

/// Format additions/deletions/files into a compact size string.
fn format_size(additions: i64, deletions: i64, files: i64) -> String {
    format!(
        "+{} −{} · {} file{}",
        additions,
        deletions,
        files,
        if files == 1 { "" } else { "s" }
    )
}

/// Format a created_at timestamp into a human-readable age.
fn format_age(created_at: &str) -> String {
    let created = created_at.replace('T', " ").replace('Z', "");

    let parsed = NaiveDateTime::parse_from_str(&created, "%Y-%m-%d %H:%M:%S");
    let dt = match parsed {
        Ok(dt) => dt,
        Err(_) => return "?".to_string(),
    };

    let now = Utc::now().naive_utc();
    let diff = now.signed_duration_since(dt);

    let hours = diff.num_hours();
    if hours < 1 {
        let mins = diff.num_minutes();
        return format!("{}m", mins.max(1));
    }
    if hours < 24 {
        return format!("{}h", hours);
    }
    let days = diff.num_days();
    if days < 30 {
        return format!("{}d", days);
    }
    format!("{}w", days / 7)
}

/// Build the tray menu content from the database.
fn build_tray_menu(
    app: &AppHandle,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let prs = load_open_prs(app);

    let mut builder = MenuBuilder::new(app);

    if prs.is_empty() {
        builder = builder.item(&MenuItem::with_id(
            app,
            "no_prs",
            "No open pull requests",
            false,
            None::<&str>,
        )?);
    } else {
        // Header with count
        let count_label = format!(
            "{} open pull request{}",
            prs.len(),
            if prs.len() == 1 { "" } else { "s" }
        );
        builder = builder.item(&MenuItem::with_id(
            app,
            "pr_header",
            &count_label,
            false,
            None::<&str>,
        )?);
        builder = builder.separator();

        // Group PRs by repo
        let mut grouped: std::collections::BTreeMap<String, Vec<&MenuPr>> =
            std::collections::BTreeMap::new();
        for pr in &prs {
            grouped.entry(pr.repo_name.clone()).or_default().push(pr);
        }

        if grouped.len() == 1 {
            // Single repo — flat list
            for pr in &prs {
                let label = format_pr_label(pr);
                builder = builder.item(&MenuItem::with_id(
                    app,
                    &format!("pr_{}", pr.id),
                    &label,
                    true,
                    None::<&str>,
                )?);
            }
        } else {
            // Multiple repos — submenus
            for (repo_name, repo_prs) in &grouped {
                let short_name = repo_name.rsplit('/').next().unwrap_or(repo_name);
                let sub_label = format!("{} ({})", short_name, repo_prs.len());
                let mut sub = SubmenuBuilder::new(app, &sub_label);

                for pr in repo_prs {
                    let label = format_pr_label(pr);
                    sub = sub.item(&MenuItem::with_id(
                        app,
                        &format!("pr_{}", pr.id),
                        &label,
                        true,
                        None::<&str>,
                    )?);
                }

                builder = builder.item(&sub.build()?);
            }
        }
    }

    // Recent Bookmarks submenu (Phase 5.7)
    let recent_bookmarks = load_recent_bookmarks(app);
    if !recent_bookmarks.is_empty() {
        builder = builder.separator();
        let mut bookmark_sub = SubmenuBuilder::new(app, "Recent Bookmarks");
        for bm in &recent_bookmarks {
            let label = format_bookmark_label(bm);
            bookmark_sub = bookmark_sub.item(&MenuItem::with_id(
                app,
                &format!("bookmark_{}", bm.pr_id),
                &label,
                true,
                None::<&str>,
            )?);
        }
        builder = builder.item(&bookmark_sub.build()?);
    }

    builder = builder.separator();
    builder = builder.item(&MenuItem::with_id(
        app,
        "tray_open",
        "Open Fuse",
        true,
        None::<&str>,
    )?);
    builder = builder.item(&MenuItem::with_id(
        app,
        "tray_sync",
        "Sync Now",
        true,
        None::<&str>,
    )?);
    builder = builder.separator();
    builder = builder.item(&MenuItem::with_id(
        app,
        "tray_quit",
        "Quit Fuse",
        true,
        None::<&str>,
    )?);

    Ok(builder.build()?)
}

/// Rebuild the tray menu (called after sync events).
pub fn refresh_menu(app: &AppHandle) {
    let menu = match build_tray_menu(app) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to build tray menu: {}", e);
            return;
        }
    };

    if let Some(tray) = app.tray_by_id("fuse-tray") {
        let _ = tray.set_menu(Some(menu));

        // Update tooltip with PR count
        let prs = load_open_prs(app);
        let tooltip = if prs.is_empty() {
            "Fuse — No open PRs".to_string()
        } else {
            format!(
                "Fuse — {} open PR{}",
                prs.len(),
                if prs.len() == 1 { "" } else { "s" }
            )
        };
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

/// Handle tray menu item click events.
fn handle_tray_menu_event(app: &AppHandle, event: &tauri::menu::MenuEvent) {
    let id = event.id().0.as_str();

    if let Some(pr_id) = id.strip_prefix("pr_") {
        if let Ok(id_num) = pr_id.parse::<i64>() {
            show_main_window(app);
            let _ = app.emit("menu-navigate-pr", id_num);
        }
    } else if let Some(pr_id) = id.strip_prefix("bookmark_") {
        // Navigate to the PR that the bookmark belongs to
        if let Ok(id_num) = pr_id.parse::<i64>() {
            show_main_window(app);
            let _ = app.emit("menu-navigate-pr", id_num);
        }
    } else {
        match id {
            "tray_open" => {
                show_main_window(app);
            }
            "tray_sync" => {
                let _ = app.emit("menu-sync-requested", ());
            }
            "tray_quit" => {
                app.exit(0);
            }
            _ => {}
        }
    }
}

/// Bring the main window to the foreground.
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Load open PRs from the database for menu display.
fn load_open_prs(app: &AppHandle) -> Vec<MenuPr> {
    let db_state = match app.try_state::<DbState>() {
        Some(s) => s,
        None => return Vec::new(),
    };

    let conn = match db_state.reader.lock() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    query_open_prs(&conn)
}

fn query_open_prs(conn: &Connection) -> Vec<MenuPr> {
    let mut stmt = match conn.prepare(
        "SELECT p.id, p.number, p.title, p.author,
                p.additions, p.deletions, p.changed_files,
                p.is_draft, p.review_decision, p.created_at,
                r.owner || '/' || r.name
         FROM pull_requests p
         JOIN repositories r ON r.id = p.repo_id
         WHERE p.state = 'OPEN'
         ORDER BY r.name, p.updated_at DESC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([], |row| {
        Ok(MenuPr {
            id: row.get(0)?,
            number: row.get(1)?,
            title: row.get(2)?,
            author: row.get(3)?,
            additions: row.get(4)?,
            deletions: row.get(5)?,
            changed_files: row.get(6)?,
            is_draft: row.get::<_, i64>(7)? != 0,
            review_decision: row.get(8)?,
            created_at: row.get(9)?,
            repo_name: row.get(10)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

/// Truncate a string to a maximum length, appending ellipsis if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

/// Bookmark data for tray menu display.
struct MenuBookmark {
    pr_id: i64,
    pr_number: i64,
    file_path: String,
    note: String,
}

/// Format a bookmark as a menu item label.
fn format_bookmark_label(bm: &MenuBookmark) -> String {
    let note_preview = if bm.note.is_empty() {
        bm.file_path.clone()
    } else {
        truncate(&bm.note, 30)
    };
    format!("#{} — {}", bm.pr_number, note_preview)
}

/// Load the 5 most recent bookmarks from the database for menu display.
fn load_recent_bookmarks(app: &AppHandle) -> Vec<MenuBookmark> {
    let db_state = match app.try_state::<DbState>() {
        Some(s) => s,
        None => return Vec::new(),
    };

    let conn = match db_state.reader.lock() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        "SELECT b.pr_id, p.number, b.file_path, b.note \
         FROM review_bookmarks b \
         JOIN pull_requests p ON p.id = b.pr_id \
         WHERE p.state = 'OPEN' \
         ORDER BY b.created_at DESC \
         LIMIT 5",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([], |row| {
        Ok(MenuBookmark {
            pr_id: row.get(0)?,
            pr_number: row.get(1)?,
            file_path: row.get(2)?,
            note: row.get(3)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}
